# MVCC (多版本并发控制) 设计文档

**版本**: v2.5.0
**最后更新**: 2026-04-16
**Issue**: #1389

---

## 概述

MVCC通过快照隔离为并发事务处理提供支持，允许读者不阻塞写者，反之亦然。

## 架构

### 核心组件

```
┌─────────────────────────────────────────────────────────────────┐
│                      TransactionManager                          │
│  - begin_tx()                                                   │
│  - commit_tx(tx_id)                                             │
│  - abort_tx(tx_id)                                             │
│  - get_snapshot(tx_id) -> Snapshot                               │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                       MVCCStorage                                │
│  - 实现 StorageEngine trait                                      │
│  - 多版本元组存储                                               │
│  - 版本链管理                                                   │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      VersionChainMap                             │
│  - 只追加版本存储                                               │
│  - tx_id -> VersionChain 映射                                   │
│  - GC/Vacuum支持                                                │
└─────────────────────────────────────────────────────────────────┘
```

### 关键类型

```rust
// 事务状态
pub struct Transaction {
    pub tx_id: u64,
    pub start_ts: u64,      // 快照时间戳
    pub status: TxStatus,   // Active/Committed/Aborted
}

// 版本链条目
pub struct RowVersion {
    pub tx_id: u64,              // 创建事务ID
    pub begin_ts: u64,           // 开始时间戳
    pub commit_ts: Option<u64>,  // 提交时间戳（未提交为None）
    pub delete_ts: Option<u64>,  // 删除时间戳
    pub data: Vec<u8>,           // 序列化的行数据
}

// 读取事务的快照
pub struct Snapshot {
    pub tx_id: u64,              // 自身事务ID
    pub active_txs: Vec<u64>,    // 活跃事务ID列表
}
```

## 实现细节

### 版本链

每行有一个由`VersionChainMap`管理的版本链：

```
行键 -> [版本1] -> [版本2] -> [版本3] -> NULL
           │            │            │
        commit_ts     commit_ts    commit_ts
          100           200          300
```

### 读取可见性规则

版本V对事务T可见当且仅当：
1. V.commit_ts <= T.snapshot_ts（在快照前提交）
2. V.tx_id != T.tx_id 且 V.tx_id 不在 T.active_txs 中
3. V.delete_ts > T.snapshot_ts（未被删除）

### 写操作

**插入**:
1. 创建新的RowVersion，tx_id = 当前事务
2. 设置begin_ts = 当前时间戳
3. 添加到版本链

**更新**:
1. 用delete_ts = 当前时间戳标记当前版本
2. 用更新后的数据创建新版本
3. 链中两个版本相连

**删除**:
1. 用delete_ts = 当前时间戳标记当前版本

### GC (垃圾回收)

后台线程定期：
1. 查找commit_ts < oldest_active_snapshot的版本
2. 检查是否有飞行中事务依赖它们
3. 从链中移除未引用的版本

## 事务管理器集成

```rust
impl TransactionManager {
    pub fn begin(&self) -> Result<TransactionId> {
        let tx_id = self.next_tx_id();
        let snapshot = Snapshot {
            tx_id,
            active_txs: self.active_txs.clone(),
        };
        self.snapshots.insert(tx_id, snapshot);
        Ok(tx_id)
    }

    pub fn commit(&self, tx_id: TransactionId) -> Result<()> {
        // 标记事务为已提交
        // 更新此事务创建的所有版本
        // 从活跃列表中移除
    }
}
```

## MVCC + WAL集成

PR: #1450

`MVCCStorage`包装`WalStorage`以提供：
1. 事务日志用于崩溃恢复
2. 多版本元组存储
3. 原子提交/中止

```rust
pub struct MVCCStorage {
    inner: WalStorage,
    mvcc: MVCCEngine,
}

impl StorageEngine for MVCCStorage {
    fn insert(&self, table_id: u64, row: &[Value]) -> Result<TxId> {
        // 1. 写入WAL条目
        // 2. 创建MVCC版本
        // 3. 返回事务ID
    }
}
```

## 性能考虑

1. **版本链长度**: 长事务为O(n)
2. **GC频率**: 基于工作负载可配置
3. **内存使用**: 与版本链长度成正比

## 测试覆盖

| 测试 | 位置 | 状态 |
|------|------|------|
| 快照隔离 | `mvcc_snapshot_isolation_test.rs` | ✅ |
| 并发事务 | `mvcc_concurrency_test.rs` | ✅ |
| GC/Vacuum | `mvcc_gc_test.rs` | ✅ |
| 崩溃恢复 | `crash_recovery_test.rs` | ✅ |

## 未来工作

### Phase 2 (v2.6.0)
- 可串行化快照隔离 (SSI)
- MVCC索引集成

### Phase 3 (v2.7.0)
- 分布式MVCC
- 跨节点事务可见性
