# MVCC 实现详解 (v3.1.0)

> **基于 GitNexus 分析** | 67,755 符号, 102,165 关系
> Multi-Version Concurrency Control: 快照隔离、版本链、可见性判断

## 1. MVCC 架构

### 1.1 核心数据结构

```rust
pub struct TxId(u64);

pub struct Transaction {
    pub id: TxId,
    pub status: TransactionStatus,
    pub start_timestamp: u64,
    pub first_read_timestamp: Option<u64>,
    pub commit_timestamp: Option<u64>,
}

pub struct Snapshot {
    pub tx_id: TxId,
    pub snapshot_timestamp: u64,
    pub active_transactions: Vec<TxId>,
}

pub struct RowVersion {
    pub value: Vec<u8>,
    pub created_by: TxId,
    pub created_commit_ts: Option<u64>,
    pub deleted_by: Option<TxId>,
    pub deleted_commit_ts: Option<u64>,
}

pub struct MvccEngine {
    transactions: HashMap<TxId, Transaction>,
    next_tx_id: u64,
    global_timestamp: u64,
}
```

### 1.2 隔离级别支持

| 隔离级别 | 实现 | 可见性算法 |
|----------|------|-----------|
| ReadUncommitted | ✅ | 读取最新版本（含未提交） |
| ReadCommitted | ✅ | `is_visible_read_committed()` |
| RepeatableRead | ✅ | `is_visible()` 快照隔离 |
| Serializable(SSI) | ⚠️ | SSI 目录不存在，未实现 |

### 1.3 关键文件

| 文件 | 行数 | 作用 |
|------|------|------|
| [mvcc.rs](file:///Users/liying/workspace/dev/yinglichina163/sqlrustgo/crates/transaction/src/mvcc.rs) | ~800 | MVCC 核心引擎 |
| [version_chain.rs](file:///Users/liying/workspace/dev/yinglichina163/sqlrustgo/crates/transaction/src/version_chain.rs) | ~400 | 版本链管理 |
| [mvcc_storage.rs](file:///Users/liying/workspace/dev/yinglichina163/sqlrustgo/crates/transaction/src/mvcc_storage.rs) | 131 | MVCC 存储引擎 |
| [transaction_manager.rs](file:///Users/liying/workspace/dev/yinglichina163/sqlrustgo/crates/transaction/src/transaction_manager.rs) | 679 | 事务管理器 |

## 2. 读操作执行链路

### 2.1 快照读取时序图

```
SELECT * FROM orders WHERE id = 1
    │
    ▼
┌──────────────────────────────────────────────┐
│ 1. TransactionManager.begin_transaction()    │
│    ├── 分配 TxId                             │
│    ├── global_timestamp++                    │
│    └── 创建 Snapshot                         │
│        ├── snapshot_timestamp = 500          │
│        └── active_transactions: {101, 102}   │
└──────────────────┬───────────────────────────┘
                   │
                   ▼
┌──────────────────────────────────────────────┐
│ 2. MVCCStorage.read(key, snapshot)           │
│    ├── VersionChainMap.find_visible()        │
│    └── 从 newest → oldest 遍历版本链        │
└──────────────────┬───────────────────────────┘
                   │
                   ▼
┌──────────────────────────────────────────────┐
│ 3. 可见性判断 (is_visible)                   │
│    ├── Version 3: tx=102, cts=None           │
│    │   └── 未提交 → 不可见 ❌               │
│    ├── Version 2: tx=101, cts=450            │
│    │   └── 101 in active_txs → 不可见 ❌    │
│    └── Version 1: tx=50, cts=300             │
│        └── 300 ≤ 500 且 50 不活跃 → 可见 ✅ │
└──────────────────┬───────────────────────────┘
                   │
                   ▼
            返回 Version 1 数据
```

### 2.2 可见性算法

```rust
fn is_visible(&self, snapshot: &Snapshot) -> bool {
    if self.created_by == snapshot.tx_id { return true; }
    let created_ts = self.created_commit_ts?;
    if created_ts > snapshot.snapshot_timestamp { return false; }
    if let Some(deleted_ts) = self.deleted_commit_ts {
        if deleted_ts <= snapshot.snapshot_timestamp { return false; }
    }
    true
}
```

### 2.3 可见性判断状态图

```
              ┌──────────────────┐
              │  检查 RowVersion  │
              └────────┬─────────┘
                       │
              ┌────────▼─────────┐
              │ created_by ==    │
              │ snapshot.tx_id?  │
              └──┬──────────┬────┘
              YES│          │NO
                 ▼          │
          ┌──────────┐     │
          │ ✅ 可见   │     │
          └──────────┘     │
                           │
              ┌────────────▼───────┐
              │ commit_ts == None? │
              └──┬────────────┬────┘
              YES│            │NO
                 ▼            │
          ┌──────────┐       │
          │ ❌ 不可见 │       │
          │(未提交)   │       │
          └──────────┘       │
                              │
              ┌───────────────▼──────────┐
              │ commit_ts > snapshot_ts? │
              └──┬──────────────┬────────┘
              YES│              │NO
                 ▼              │
          ┌──────────┐         │
          │ ❌ 不可见 │         │
          │(快照后)   │         │
          └──────────┘         │
                               │
              ┌────────────────▼──────────┐
              │ deleted_ts ≤ snapshot_ts? │
              └──┬──────────────┬─────────┘
              YES│              │NO
                 ▼              ▼
          ┌──────────┐   ┌──────────┐
          │ ❌ 不可见 │   │ ✅ 可见   │
          │(已删除)   │   └──────────┘
          └──────────┘
```

## 3. 写操作执行链路

### 3.1 写入时序图

```
UPDATE orders SET total = 100 WHERE id = 1
    │
    ▼
┌──────────────────────────────────────────────┐
│ 1. MVCCStorage.write_version(key, value, tx) │
│    ├── RowVersion::new(tx_id, value)         │
│    │   └── commit_ts = None                  │
│    └── VersionChainMap.append(key, version)  │
└──────────────────┬───────────────────────────┘
                   │
                   ▼
┌──────────────────────────────────────────────┐
│ 2. 数据已写入但对其他事务不可见              │
│    Version Chain:                             │
│    v2(tx=103, cts=None) → v1(tx=50, cts=300) │
└──────────────────┬───────────────────────────┘
                   │
                   ▼
┌──────────────────────────────────────────────┐
│ 3. COMMIT                                    │
│    ├── commit_versions(tx_id, commit_ts)     │
│    │   └── stamp all versions: cts = new_ts  │
│    ├── WAL: log_commit(tx_id)                │
│    └── 其他事务现在可以看到 v2               │
└──────────────────────────────────────────────┘
```

### 3.2 写操作活动图

```
    ┌──────────────────────────────┐
    │         写入请求              │
    └──────────────┬───────────────┘
                   │
                   ▼
    ┌──────────────────────────────┐
    │  创建新 RowVersion            │
    │  - created_by = 当前 tx_id   │
    │  - commit_ts = None          │
    └──────────────┬───────────────┘
                   │
                   ▼
    ┌──────────────────────────────┐
    │  追加到版本链头部             │
    │  v_new → v_old → ...        │
    └──────────────┬───────────────┘
                   │
            ┌──────┴──────┐
            │ 事务提交?    │
            └──┬──────┬───┘
          COMMIT  ROLLBACK
               │      │
               ▼      ▼
    ┌──────────────┐ ┌──────────────┐
    │ stamp cts    │ │ 移除版本     │
    │ 对其他可见   │ │ (abort)      │
    └──────────────┘ └──────────────┘
```

## 4. 版本链管理

### 4.1 版本链结构

```
Key: "orders:1"
    │
    ▼
Version Chain (newest → oldest):
┌──────────────────────┐
│ v4: tx=105, cts=600  │ ← 最新已提交
│ value = [100, ...]   │
├──────────────────────┤
│ v3: tx=103, cts=500  │ ← 已提交
│ value = [80, ...]    │
├──────────────────────┤
│ v2: tx=101, cts=400  │ ← 已提交
│ value = [60, ...]    │
├──────────────────────┤
│ v1: tx=50, cts=300   │ ← 最早版本
│ value = [50, ...]    │
└──────────────────────┘
```

### 4.2 GC 策略

```
oldest_active_snapshot = 450

v1 (cts=300) → 300 < 450 → 可清理 ✅
v2 (cts=400) → 400 < 450 → 可清理 ✅
v3 (cts=500) → 500 ≥ 450 → 保留 ❌
v4 (cts=600) → 最新版本 → 保留 ❌
```

## 5. 算法复杂度与性能分析

### 5.1 操作复杂度

| 操作 | 复杂度 | 瓶颈 | 优化建议 |
|------|--------|------|---------|
| begin_transaction | O(1) | 无 | ✅ |
| commit_transaction | O(1) | 无 | ✅ |
| create_snapshot | O(A) | A=活跃事务数 | HashSet 替代 Vec |
| is_visible | O(A) | 线性扫描 active_txs | **HashSet O(1)** |
| write_version | O(1) | 无 | ✅ |
| commit_versions | O(V) | V=该事务版本数 | 批量 stamp |
| 版本链遍历 | O(V) | V=版本链长度 | **GC 清理旧版本** |

### 5.2 ⚠️ 已知问题

| 问题 | 严重性 | 影响 | 修复建议 |
|------|--------|------|---------|
| **SSI 未实现** | 🔴 严重 | 无法支持 SERIALIZABLE | 实现 SSI 冲突检测 |
| **版本链无 GC** | 🟡 中等 | 内存持续增长 | 实现后台 GC 线程 |
| **global_timestamp 非原子** | 🟡 中等 | 多线程需外部同步 | 使用 AtomicU64 |
| **active_transactions 用 Vec** | 🟡 中等 | is_visible O(A) | 改用 HashSet |

### 5.3 性能瓶颈分析

```
瓶颈1: is_visible 线性扫描
  当前: active_transactions: Vec<TxId>
  查找: O(A), A=活跃事务数
  影响: 高并发 (100+ 活跃事务) 时每次读操作都需线性扫描
  修复: 改用 HashSet<TxId> → O(1) 查找

瓶颈2: 版本链无清理
  当前: VersionChain 只追加不回收
  影响: 长期运行后历史版本膨胀，遍历变慢
  修复: 后台 GC 线程，清理 commit_ts < oldest_snapshot 的版本

瓶颈3: global_timestamp 非原子
  当前: &mut self 下递增
  影响: 多线程需外部 Mutex，增加锁竞争
  修复: 使用 AtomicU64 + fetch_add()
```

## 6. 与其他模块的依赖

```
MvccEngine
  ├── 依赖: serde (序列化)
  ├── 依赖: transaction::lock::LockManager (锁管理)
  ├── 被依赖: TransactionManager (事务管理)
  ├── 被依赖: MVCCStorage (存储层 MVCC)
  ├── 被依赖: WalTransactionalExecutor (WAL 集成)
  └── 被依赖: SsiDetector (SSI 检测, 未实现)
```

## 变更记录

| 日期 | 版本 | 说明 |
|------|------|------|
| 2026-05-15 | v2.0 | 基于 GitNexus 重新分析，补充可见性状态图、性能瓶颈 |
