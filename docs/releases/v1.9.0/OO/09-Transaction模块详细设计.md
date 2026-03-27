# SQLRustGo Transaction 模块详细设计文档

> **版本**: v1.9.0
> **日期**: 2026-03-26
> **模块**: sqlrustgo-transaction

---

## 1. 模块概述

Transaction 模块负责事务管理和并发控制。

### 1.1 模块职责

- 事务生命周期管理
- MVCC 实现
- 锁管理
- 死锁检测
- Savepoint 支持

### 1.2 模块结构

```
crates/transaction/
├── src/
│   ├── lib.rs           # 模块入口
│   ├── manager.rs       # 事务管理器
│   ├── mvcc.rs          # MVCC 实现
│   ├── lock.rs          # 锁管理
│   ├── deadlock.rs      # 死锁检测
│   └── savepoint.rs     # Savepoint
└── Cargo.toml
```

---

## 2. 核心类设计

### 2.1 事务管理器

```uml
@startuml

class TransactionManager {
  -transactions: HashMap<TxId, Transaction>
  -version_store: VersionStore
  -lock_manager: LockManager
  -deadlock_detector: DeadlockDetector
  --
  +begin(isolation): TxId
  +commit(tx_id): Result<()>
  +rollback(tx_id): Result<()>
  +get_tx(tx_id): &Transaction
}

class Transaction {
  -tx_id: TxId
  -isolation_level: IsolationLevel
  -state: TxState
  -start_time: Timestamp
  -snapshot: Snapshot
  -read_set: Vec<RowKey>
  -write_set: Vec<RowKey>
}

enum TxState {
  Active
  Preparing
  Committed
  Aborted
}

enum IsolationLevel {
  ReadUncommitted
  ReadCommitted
  RepeatableRead
  Serializable
}

TransactionManager --> Transaction
Transaction --> TxState
Transaction --> IsolationLevel

@enduml
```

### 2.2 MVCC 设计

```uml
@startuml

class VersionStore {
  -versions: BTreeMap<RowKey, Vec<RowVersion>>
  --
  +write(key, value, tx_id): Result<()>
  +read(key, tx_id, snapshot): Option<Value>
  +gc(older_than): usize
}

class RowVersion {
  -tx_id: TxId
  -begin_ts: TxId
  -end_ts: Option<TxId>
  -data: Vec<u8>
  -is_deleted: bool
}

class Snapshot {
  -tx_id: TxId
  -active_txs: Vec<TxId>
  -min_visible: TxId
}

VersionStore --> RowVersion
Transaction --> Snapshot
Snapshot --> VersionStore

@enduml
```

### 2.3 锁管理

```uml
@startuml

class LockManager {
  -table_locks: HashMap<String, TableLock>
  -row_locks: HashMap<RowKey, RowLock>
  -wait_graph: WaitGraph
  --
  +acquire_shared(tx_id, key): Result<()>
  +acquire_exclusive(tx_id, key): Result<()>
  +release(tx_id, key): Result<()>
  +upgrade(tx_id, key): Result<()>
}

class TableLock {
  -table: String
  -mode: LockMode
  -holders: Vec<TxId>
  -waiters: Vec<TxId>
}

class RowLock {
  -key: RowKey
  -mode: LockMode
  -holders: Vec<TxId>
  -waiters: Vec<TxId>
}

enum LockMode {
  Shared
  Exclusive
  IntentionShared
  IntentionExclusive
}

LockManager --> TableLock
LockManager --> RowLock
TableLock --> LockMode

@enduml
```

---

## 3. 事务状态机

### 3.1 状态转换

```uml
@startuml

[*] --> Active: BEGIN

Active --> Preparing: PREPARE TRANSACTION

Preparing --> Committed: COMMIT

Preparing --> Aborted: ROLLBACK

Active --> Aborted: ROLLBACK

Active --> Aborted: DEADLOCK

Active --> Active: SQL Operations

Committed --> [*]

Aborted --> [*]

@enduml
```

### 3.2 隔离级别实现

| 隔离级别 | 读行为 | 写行为 | 实现 |
|----------|--------|--------|------|
| READ UNCOMMITTED | 读取最新版本 | 排他锁 | MVCC + 锁 |
| READ COMMITTED | 每次读取新快照 | 排他锁 | MVCC |
| REPEATABLE READ | 事务开始快照 | 排他锁 | MVCC + 快照 |
| SERIALIZABLE | 验证串行化 | 锁 + MVCC | 锁 + 验证 |

---

## 4. 死锁检测

### 4.1 等待图

```uml
@startuml

class WaitGraph {
  -edges: HashMap<TxId, Vec<TxId>>
  --
  +add_edge(from, to)
  +remove_edge(from, to)
  +detect_cycle(): Option<Vec<TxId>>
}

class DeadlockDetector {
  -interval_ms: u64
  -timeout_ms: u64
  -wait_graph: WaitGraph
  --
  +start_detection()
  +check_deadlock(): Option<Vec<TxId>>
  +resolve_deadlock(tx_id)
}

WaitGraph --> DeadlockDetector

@enduml
```

---

## 5. 与代码对应检查

### 5.1 模块文件对应

| 设计内容 | 代码文件 | 状态 |
|----------|----------|------|
| 事务管理器 | `manager.rs` | ✅ 对应 |
| MVCC | `mvcc.rs` | ✅ 对应 |
| 锁管理 | `lock.rs` | ✅ 对应 |
| 死锁检测 | `deadlock.rs` | ✅ 对应 |
| Savepoint | `savepoint.rs` | ✅ 对应 |

### 5.2 功能覆盖检查

| 功能 | 代码实现 | 状态 |
|------|----------|------|
| 事务开始/提交/回滚 | ✅ | ✅ |
| MVCC 读 | ✅ | ✅ |
| MVCC 写 | ✅ | ✅ |
| READ COMMITTED | ✅ | ✅ |
| REPEATABLE READ | ✅ | ✅ |
| 共享锁 | ✅ | ✅ |
| 排他锁 | ✅ | ✅ |
| 死锁检测 | ✅ | ✅ |
| Savepoint | ✅ | ✅ |

---

## 6. 测试设计

### 6.1 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_begin_commit() {
        let mgr = TransactionManager::new();
        let tx_id = mgr.begin(IsolationLevel::ReadCommitted).unwrap();
        
        let result = mgr.commit(tx_id);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_mvcc_read() {
        let mgr = TransactionManager::new();
        let tx_id = mgr.begin(IsolationLevel::ReadCommitted).unwrap();
        
        // 写入
        mgr.write(tx_id, "key1", "value1").unwrap();
        
        // 提交
        mgr.commit(tx_id).unwrap();
        
        // 新事务读取
        let tx2 = mgr.begin(IsolationLevel::ReadCommitted).unwrap();
        let value = mgr.read(tx2, "key1").unwrap();
        assert_eq!(value, Some("value1".to_string()));
    }
    
    #[test]
    fn test_deadlock_detection() {
        let detector = DeadlockDetector::new(100);
        
        // 创建死锁场景
        // ...
        
        let cycle = detector.check_deadlock();
        assert!(cycle.is_some());
    }
}
```

---

**文档版本历史**

| 版本 | 日期 | 作者 | 变更 |
|------|------|------|------|
| 1.0 | 2026-03-26 | OpenCode | 初始版本 |

**文档状态**: ✅ 已完成
