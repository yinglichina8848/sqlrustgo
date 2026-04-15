# MVCC Snapshot Isolation 实现设计

**版本**: v1.0  
**日期**: 2026-04-15  
**Issue**: #1389  
**状态**: 设计完成，待实现

---

## 1. 概述

### 1.1 目标

将现有 MVCC 元数据系统升级为存储层 MVCC，实现 Snapshot Isolation（快照隔离）。

### 1.2 范围

**Phase 1 包含**：
- VersionChain append（版本链追加）
- visibility filtering（可见性过滤）
- commit stamping（提交时间戳）
- rollback discard（回滚丢弃）
- snapshot-consistent read（快照一致性读取）

**Phase 1 不包含**：
- Vacuum / GC
- Index MVCC
- Predicate locking
- Serializable (SSI)

---

## 2. 架构设计

```
┌─────────────────────────────────────────────────────────────┐
│                      Executor                              │
│  storage.read(key, snapshot)  // 传入快照                  │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│              TransactionManager + MvccEngine                │
│  - tx.begin() → Snapshot                                  │
│  - tx.commit() → global_timestamp++                        │
│  - 维护 active_transactions 列表                           │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                    MVCCStorage Trait                        │
│  - read(key, snapshot) → visible value                    │
│  - write_version(key, value, tx_id)                        │
│  - delete_version(key, tx_id)                              │
│  - commit_versions(tx_id, commit_ts)                       │
│  - rollback_versions(tx_id)                                │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│              VersionChainMap (Append-only)                  │
│  HashMap<Key, Vec<RowVersion>>                             │
│                                                             │
│  "users:1" → [v1(ts=10), v2(ts=20), v3(ts=35)]            │
│                         newest-first                        │
└─────────────────────────────────────────────────────────────┘
```

---

## 3. 核心数据结构

### 3.1 RowVersion

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RowVersion {
    /// 行数据序列化
    pub value: Vec<u8>,

    /// 创建事务 ID
    pub created_by: TxId,

    /// 创建提交时间戳（None = 未提交，脏读防护）
    pub created_commit_ts: Option<u64>,

    /// 删除事务 ID（None = 未删除）
    pub deleted_by: Option<TxId>,

    /// 删除提交时间戳
    pub deleted_commit_ts: Option<u64>,
}

impl RowVersion {
    pub fn new(tx_id: TxId, value: Vec<u8>) -> Self {
        Self {
            value,
            created_by: tx_id,
            created_commit_ts: None,
            deleted_by: None,
            deleted_commit_ts: None,
        }
    }

    pub fn new_deleted(tx_id: TxId) -> Self {
        Self {
            value: Vec::new(),
            created_by: tx_id,
            created_commit_ts: None,
            deleted_by: Some(tx_id),
            deleted_commit_ts: None,
        }
    }

    /// 可见性判断 - 快照隔离核心逻辑
    pub fn is_visible(&self, snapshot: &Snapshot) -> bool {
        // 规则 1: 自己的创建版本总是可见的（即使未提交）
        if self.created_by == snapshot.tx_id {
            return true;
        }

        // 规则 2: 未提交的创建版本不可见
        let created_ts = match self.created_commit_ts {
            Some(ts) => ts,
            None => return false,
        };

        // 规则 3: 创建提交时间必须在快照时间之前
        if created_ts > snapshot.snapshot_timestamp {
            return false;
        }

        // 规则 4: 如果被删除，删除提交时间必须在快照时间之后才可见
        if let Some(deleted_ts) = self.deleted_commit_ts {
            if deleted_ts <= snapshot.snapshot_timestamp {
                return false;
            }
        }

        true
    }

    pub fn commit(&mut self, timestamp: u64) {
        self.created_commit_ts = Some(timestamp);
    }

    pub fn mark_deleted(&mut self, tx_id: TxId, timestamp: u64) {
        self.deleted_by = Some(tx_id);
        self.deleted_commit_ts = Some(timestamp);
    }
}
```

### 3.2 VersionChainMap

```rust
pub struct VersionChainMap {
    chains: RwLock<HashMap<Vec<u8>, Vec<RowVersion>>>,
}

impl VersionChainMap {
    pub fn new() -> Self {
        Self {
            chains: RwLock::new(HashMap::new()),
        }
    }

    /// newest-first 查找可见版本
    pub fn find_visible(&self, key: &Key, snapshot: &Snapshot) -> Option<Vec<u8>> {
        let chains = self.chains.read().unwrap();

        if let Some(versions) = chains.get(key) {
            // newest-first: 从最新版本开始查找
            for version in versions.iter().rev() {
                if version.is_visible(snapshot) {
                    return Some(version.value.clone());
                }
            }
        }
        None
    }

    /// 追加新版本
    pub fn append(&mut self, key: Key, version: RowVersion) {
        let mut chains = self.chains.write().unwrap();
        chains
            .entry(key)
            .or_insert_with(Vec::new)
            .push(version);
    }

    /// 获取版本链引用
    pub fn get_chain(&self, key: &Key) -> Option<Vec<RowVersion>> {
        let chains = self.chains.read().unwrap();
        chains.get(key).cloned()
    }

    /// 提交版本 - 批量更新 created_commit_ts
    pub fn commit_versions(&mut self, tx_id: TxId, commit_ts: u64) {
        let mut chains = self.chains.write().unwrap();
        for versions in chains.values_mut() {
            for version in versions.iter_mut() {
                if version.created_by == tx_id && version.created_commit_ts.is_none() {
                    version.created_commit_ts = Some(commit_ts);
                }
            }
        }
    }

    /// 回滚版本 - 移除未提交版本
    pub fn rollback_versions(&mut self, tx_id: TxId) {
        let mut chains = self.chains.write().unwrap();
        for versions in chains.values_mut() {
            // 保留非该事务创建的版本，以及已提交的版本
            versions.retain(|v| {
                v.created_by != tx_id || v.created_commit_ts.is_some()
            });
        }
    }
}
```

### 3.3 MVCCStorage Trait

```rust
/// MVCC 存储引擎接口
pub trait MVCCStorage: Send + Sync {
    /// 使用快照读取 - 返回快照时间点可见的值
    fn read(&self, key: &Key, snapshot: &Snapshot) -> Option<Vec<u8>>;

    /// 写入新版本（未提交）
    fn write_version(&mut self, key: Key, value: Vec<u8>, tx_id: TxId);

    /// 标记删除（未提交）
    fn delete_version(&mut self, key: Key, tx_id: TxId);

    /// 提交事务 - 批量更新提交时间戳
    fn commit_versions(&mut self, tx_id: TxId, commit_ts: u64) -> Result<(), TransactionError>;

    /// 回滚事务 - 移除未提交版本
    fn rollback_versions(&mut self, tx_id: TxId) -> Result<(), TransactionError>;

    /// 创建快照
    fn create_snapshot(&self, tx_id: TxId) -> Snapshot;
}
```

---

## 4. 与 TransactionManager 集成

### 4.1 读取流程

```
1. Executor 调用 storage.read(key, snapshot)
2. VersionChainMap.find_visible(key, snapshot)
3. 从 newest 到 oldest 遍历版本链
4. 第一个 is_visible() == true 的即为返回值
5. 无可见版本返回 None
```

### 4.2 写入流程

```
1. Executor 调用 storage.write_version(key, value, tx_id)
2. RowVersion::new(tx_id, value) // created_commit_ts = None
3. VersionChainMap.append(key, version)
4. 数据已写入但对其他事务不可见
```

### 4.3 提交流程

```
1. TransactionManager.commit()
2. MvccEngine.commit_transaction(tx_id) → commit_ts
3. storage.commit_versions(tx_id, commit_ts)
4. 所有 created_commit_ts = None 的版本被 stamp
5. 其他事务现在可以看到这些版本
```

### 4.4 回滚流程

```
1. TransactionManager.rollback()
2. storage.rollback_versions(tx_id)
3. 移除所有 created_by == tx_id 且 created_commit_ts == None 的版本
4. 该事务的写入完全消失
```

---

## 5. 实现任务分解

### T13.1: MvccEngine 集成（1 周）

| 任务 | 验收标准 |
|------|----------|
| 扩展 RowVersion 结构 | 包含 created_commit_ts, deleted_by, deleted_commit_ts |
| 实现 VersionChainMap | append, find_visible, commit, rollback |
| 实现 MVCCStorage Trait | 内存版本 |
| 单元测试 | RowVersion 可见性边界测试 |

### T13.2: 可见性检查（2 周）

| 任务 | 验收标准 |
|------|----------|
| 实现 is_visible() | 通过 snapshot_isolation_test.rs |
| 集成到 TransactionManager | read/write 自动关联 snapshot |
| 脏读测试 | 未提交数据不可见 |
| 不可重复读测试 | 同一事务多次读取一致 |

### T13.3: 版本链管理（2-3 周）

| 任务 | 验收标准 |
|------|----------|
| 提交 stamp | 批量更新版本 |
| 回滚 discard | 正确移除未提交版本 |
| 并发测试 | 多线程安全性 |
| 性能测试 | 版本链长度对读取性能影响 |

---

## 6. 验收标准

### 6.1 功能验收

| 测试 | 说明 |
|------|------|
| `test_snapshot_read_consistency` | 事务内多次读取返回相同结果 |
| `test_dirty_read_prevention` | 未提交数据对其他事务不可见 |
| `test_commit_visibility` | 提交后数据对后续快照可见 |
| `test_rollback_discard` | 回滚后数据完全消失 |
| `test_concurrent_read_write` | 并发读写不冲突 |

### 6.2 集成验收

- TransactionManager.read() 自动传入 snapshot
- TransactionManager.write() 自动关联 tx_id
- commit/rollback 正确 stamp/discard 版本

---

## 7. 关键设计决定

| 决定 | 选择 | 理由 |
|------|------|------|
| 版本链方向 | newest-first | O(visible_depth)，通常 1-2 次命中 |
| 版本存储位置 | HashMap<Key, Vec<Version>> | 简单，符合 M1 阶段 |
| 提交时间戳 | commit 时批量 stamp | 保证原子性 |
| 回滚策略 | retain 保留已提交 | 简单实现 |

---

## 8. 风险与缓解

| 风险 | 影响 | 概率 | 缓解 |
|------|------|------|------|
| 版本链过长 | 读取性能下降 | 低 | Phase 2 添加 GC |
| 内存压力 | 版本占用过多内存 | 中 | 限制版本数量或刷盘 |
| 并发冲突 | write-write 冲突 | 低 | Phase 2 实现锁机制 |

---

## 9. 后续规划

### Phase 2（不在本 Issue 范围）
- Vacuum / GC 清理旧版本
- Index MVCC 集成

### Phase 3（Serializable）
- Predicate locking
- SSI (Serializable Snapshot Isolation)

---

**设计状态**: ✅ 完成  
**下一步**: 进入实现阶段
