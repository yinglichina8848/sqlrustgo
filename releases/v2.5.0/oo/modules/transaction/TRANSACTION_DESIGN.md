# Transaction 模块设计

**版本**: v2.5.0
**模块**: Transaction (事务管理)

---

## 一、What (是什么)

Transaction 是 SQLRustGo 的事务管理模块，负责事务的开启、提交、回滚，以及事务隔离级别的实现。

## 二、Why (为什么)

- **ACID 保证**: 原子性、一致性、隔离性、持久性
- **并发控制**: MVCC 和锁管理
- **隔离级别**: 支持多种隔离级别
- **保存点**: 支持事务内保存点

## 三、核心数据结构

```rust
pub struct TransactionManager {
    active_txs: HashMap<TransactionId, Transaction>,
    snapshots: HashMap<TransactionId, Snapshot>,
    locks: LockManager,
}

pub struct Transaction {
    id: TransactionId,
    start_ts: u64,
    status: TxStatus,
    isolation_level: IsolationLevel,
    savepoints: Vec<Savepoint>,
}
```

## 四、隔离级别

| 隔离级别 | 脏读 | 不可重复读 | 幻读 |
|----------|------|------------|------|
| READ UNCOMMITTED | 可能 | 可能 | 可能 |
| READ COMMITTED | ❌ | 可能 | 可能 |
| REPEATABLE READ | ❌ | ❌ | 可能 |
| SNAPSHOT (默认) | ❌ | ❌ | ❌ |
| SERIALIZABLE | ❌ | ❌ | ❌ |

## 五、相关文档

- [ARCHITECTURE_V2.5.md](../architecture/ARCHITECTURE_V2.5.md)
- [MVCC_DESIGN.md](./mvcc/MVCC_DESIGN.md)

---

*Transaction 模块设计 v2.5.0*
