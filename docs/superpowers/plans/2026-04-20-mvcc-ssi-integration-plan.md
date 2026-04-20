# MVCC SSI 完整集成实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将 MVCC SSI 完整集成到执行引擎，支持标准 SQL 事务语句 (BEGIN/COMMIT/ROLLBACK/SET TRANSACTION)

**Architecture:**
- Parser 层: 新增 `TransactionStatement` 枚举，支持 `BEGIN`, `COMMIT`, `ROLLBACK`, `SET TRANSACTION`
- ExecutionEngine 层: 新增事务管理字段，使用 `SsiDetector` 检测冲突
- Storage 层: MVCC 快照与执行引擎集成

**Tech Stack:** Rust, Tokio async runtime, MVCC, SSI (Serializable Snapshot Isolation)

---

## 文件结构

```
crates/parser/src/
  - parser.rs          # 修改: 新增 TransactionStatement 枚举, parse_transaction()
  - lib.rs             # 修改: 导出 TransactionStatement

crates/transaction/src/
  - lib.rs             # 修改: 导出 TransactionManager (新增)
  - transaction_manager.rs  # 新增: TransactionManager 结构体

src/
  - execution_engine.rs     # 修改: 集成 TransactionManager, execute Begin/Commit/Rollback

crates/transaction/tests/
  - ssi_integration.rs      # 修改: 增强 SSI 测试
  - mvcc_integration.rs     # 新增: MVCC 集成测试

tests/
  - mvcc_transaction_test.rs  # 新增: SQL 事务回归测试
```

---

## Task 1: Parser 层 - 新增 TransactionStatement

**Files:**
- Modify: `crates/parser/src/parser.rs:22-37` (Statement enum)
- Modify: `crates/parser/src/parser.rs:369-384` (parse_statement dispatch)
- Create: `crates/parser/src/transaction.rs` (TransactionStatement, IsolationLevel)
- Modify: `crates/parser/src/lib.rs` (导出)

- [ ] **Step 1: 创建 transaction.rs 文件**

```rust
// crates/parser/src/transaction.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IsolationLevel {
    ReadCommitted,
    ReadUncommitted,
    SnapshotIsolation,
    Serializable,
}

impl Default for IsolationLevel {
    fn default() -> Self {
        IsolationLevel::ReadCommitted
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TransactionStatement {
    Begin(Option<IsolationLevel>),
    Commit,
    Rollback,
    SetTransaction { isolation_level: IsolationLevel },
    StartTransaction(Option<IsolationLevel>),
}
```

- [ ] **Step 2: 在 Statement 枚举中添加 Transaction 变体**

在 `parser.rs` 的 `Statement` 枚举中添加:
```rust
pub enum Statement {
    // ... existing variants ...
    Transaction(TransactionStatement),
}
```

- [ ] **Step 3: 在 parse_statement 中添加事务语句分发**

```rust
fn parse_transaction(&mut self) -> Result<TransactionStatement, String> {
    match self.current() {
        Some(Token::Begin) => {
            self.next();
            let isolation = self.parse_isolation_level()?;
            Ok(TransactionStatement::Begin(isolation))
        }
        Some(Token::Commit) => {
            self.next();
            Ok(TransactionStatement::Commit)
        }
        Some(Token::Rollback) => {
            self.next();
            Ok(TransactionStatement::Rollback)
        }
        Some(Token::Set) => {
            self.next();
            self.parse_set_transaction()
        }
        Some(Token::Start) => {
            self.next();
            self.expect(Token::Transaction)?;
            let isolation = self.parse_isolation_level()?;
            Ok(TransactionStatement::StartTransaction(isolation))
        }
        _ => Err("Expected BEGIN, COMMIT, ROLLBACK, or SET".to_string()),
    }
}

fn parse_isolation_level(&mut self) -> Result<Option<IsolationLevel>, String> {
    if matches!(self.current(), Some(Token::Transaction)) {
        self.next();
        self.expect(Token::Isolation)?;
        self.expect(Token::Level)?;
    }
    match self.current() {
        Some(Token::Serializable) => {
            self.next();
            Ok(Some(IsolationLevel::Serializable))
        }
        Some(Token::Read) => {
            self.next();
            match self.current() {
                Some(Token::Committed) => {
                    self.next();
                    Ok(Some(IsolationLevel::ReadCommitted))
                }
                Some(Token::Uncommitted) => {
                    self.next();
                    Ok(Some(IsolationLevel::ReadUncommitted))
                }
                _ => Err("Expected COMMITTED or UNCOMMITTED".to_string()),
            }
        }
        Some(Token::Snapshot) => {
            self.next();
            self.expect(Token::Isolation)?;
            Ok(Some(IsolationLevel::SnapshotIsolation))
        }
        _ => Ok(None), // Default isolation level
    }
}

fn parse_set_transaction(&mut self) -> Result<TransactionStatement, String> {
    self.expect(Token::Transaction)?;
    self.expect(Token::Isolation)?;
    self.expect(Token::Level)?;
    let level = match self.current() {
        Some(Token::Serializable) => {
            self.next();
            IsolationLevel::Serializable
        }
        Some(Token::Read) => {
            self.next();
            match self.current() {
                Some(Token::Committed) => {
                    self.next();
                    IsolationLevel::ReadCommitted
                }
                _ => return Err("Expected COMMITTED".to_string()),
            }
        }
        _ => return Err("Expected SERIALIZABLE or READ COMMITTED".to_string()),
    };
    Ok(TransactionStatement::SetTransaction { isolation_level: level })
}
```

- [ ] **Step 4: 更新 parse_statement 分发**

```rust
Some(Token::Begin) | Some(Token::Start) | Some(Token::Commit) |
Some(Token::Rollback) | Some(Token::Set) => {
    let txn = self.parse_transaction()?;
    Ok(Statement::Transaction(txn))
}
```

- [ ] **Step 5: 单元测试**

```rust
#[test]
fn test_parse_begin() {
    let result = parse("BEGIN");
    assert!(matches!(result, Ok(Statement::Transaction(TransactionStatement::Begin(None)))));
}

#[test]
fn test_parse_begin_serializable() {
    let result = parse("BEGIN SERIALIZABLE");
    assert!(matches!(result, Ok(Statement::Transaction(TransactionStatement::Begin(Some(IsolationLevel::Serializable))))));
}

#[test]
fn test_parse_commit() {
    let result = parse("COMMIT");
    assert!(matches!(result, Ok(Statement::Transaction(TransactionStatement::Commit))));
}

#[test]
fn test_parse_rollback() {
    let result = parse("ROLLBACK");
    assert!(matches!(result, Ok(Statement::Transaction(TransactionStatement::Rollback))));
}
```

- [ ] **Step 6: 运行测试验证**

Run: `cargo test -p sqlrustgo-parser --lib`
Expected: All parser tests pass including new transaction tests

- [ ] **Step 7: Commit**

```bash
git add crates/parser/src/transaction.rs crates/parser/src/parser.rs crates/parser/src/lib.rs
git commit -m "feat(parser): add TransactionStatement and isolation level parsing"
```

---

## Task 2: TransactionManager 结构体

**Files:**
- Create: `crates/transaction/src/transaction_manager.rs`
- Modify: `crates/transaction/src/lib.rs` (导出)

- [ ] **Step 1: 创建 TransactionManager**

```rust
// crates/transaction/src/transaction_manager.rs
use crate::{Snapshot, TxId, INVALID_TX_ID, SsiDetector, SsiError};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionState {
    Active,
    Committed,
    Aborted,
}

#[derive(Debug)]
pub struct ActiveTransaction {
    pub tx_id: TxId,
    pub snapshot: Snapshot,
    pub state: TransactionState,
    pub read_keys: Vec<Vec<u8>>,
    pub write_keys: Vec<Vec<u8>>,
}

impl ActiveTransaction {
    pub fn new(tx_id: TxId, snapshot: Snapshot) -> Self {
        Self {
            tx_id,
            snapshot,
            state: TransactionState::Active,
            read_keys: Vec::new(),
            write_keys: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct TransactionManager {
    ssi_detector: SsiDetector,
    active_transactions: HashMap<TxId, ActiveTransaction>,
    next_tx_id: u64,
}

impl TransactionManager {
    pub fn new() -> Self {
        Self {
            ssi_detector: SsiDetector::new(Arc::new(crate::DistributedLockManager::new())),
            active_transactions: HashMap::new(),
            next_tx_id: 1,
        }
    }

    pub fn begin_transaction(&mut self, isolation: crate::IsolationLevel) -> Result<TxId, SsiError> {
        let tx_id = TxId::new(self.next_tx_id);
        self.next_tx_id += 1;

        let snapshot_timestamp = self.calculate_snapshot_timestamp();
        let snapshot = Snapshot::new(tx_id, snapshot_timestamp, vec![]);

        let active_tx = ActiveTransaction::new(tx_id, snapshot);
        self.active_transactions.insert(tx_id, active_tx);

        Ok(tx_id)
    }

    fn calculate_snapshot_timestamp(&self) -> u64 {
        self.active_transactions
            .values()
            .map(|tx| tx.snapshot.snapshot_timestamp)
            .max()
            .unwrap_or(0)
    }

    pub fn record_read(&mut self, tx_id: TxId, key: Vec<u8>) -> Result<(), SsiError> {
        if let Some(tx) = self.active_transactions.get_mut(&tx_id) {
            tx.read_keys.push(key.clone());
            self.ssi_detector.record_read(tx_id, key)?;
        }
        Ok(())
    }

    pub fn record_write(&mut self, tx_id: TxId, key: Vec<u8>) -> Result<(), SsiError> {
        if let Some(tx) = self.active_transactions.get_mut(&tx_id) {
            tx.write_keys.push(key.clone());
            self.ssi_detector.record_write(tx_id, key)?;
        }
        Ok(())
    }

    pub fn commit(&mut self, tx_id: TxId) -> Result<(), SsiError> {
        // Validate SSI before commit
        self.ssi_detector.validate_commit(tx_id)?;

        // Mark as committed
        if let Some(tx) = self.active_transactions.get_mut(&tx_id) {
            tx.state = TransactionState::Committed;
        }

        // Remove from active
        self.active_transactions.remove(&tx_id);

        Ok(())
    }

    pub fn rollback(&mut self, tx_id: TxId) -> Result<(), SsiError> {
        if let Some(tx) = self.active_transactions.get_mut(&tx_id) {
            tx.state = TransactionState::Aborted;
        }
        self.active_transactions.remove(&tx_id);
        Ok(())
    }

    pub fn get_snapshot(&self, tx_id: TxId) -> Option<Snapshot> {
        self.active_transactions.get(&tx_id).map(|tx| tx.snapshot.clone())
    }
}

impl Default for TransactionManager {
    fn default() -> Self {
        Self::new()
    }
}
```

- [ ] **Step 2: 更新 lib.rs 导出**

```rust
pub mod transaction_manager;
pub use transaction_manager::TransactionManager;
```

- [ ] **Step 3: 运行测试**

Run: `cargo build -p sqlrustgo-transaction`
Expected: Compiles successfully

- [ ] **Step 4: Commit**

```bash
git add crates/transaction/src/transaction_manager.rs crates/transaction/src/lib.rs
git commit -m "feat(transaction): add TransactionManager struct"
```

---

## Task 3: ExecutionEngine 集成

**Files:**
- Modify: `src/execution_engine.rs` (添加事务管理字段和事务执行逻辑)

- [ ] **Step 1: 添加事务管理字段**

在 `ExecutionEngine` 结构体中添加:
```rust
pub struct ExecutionEngine {
    storage: Arc<RwLock<dyn StorageEngine>>,
    stats: Arc<RwLock<HashMap<String, TableStats>>>,
    // 新增字段
    transaction_manager: TransactionManager,
    current_tx_id: Option<TxId>,
    default_isolation: IsolationLevel,
}
```

- [ ] **Step 2: 实现 execute_transaction**

在 `ExecutionEngine` 中添加:
```rust
fn execute_transaction(&mut self, txn: &TransactionStatement) -> SqlResult<ExecutorResult> {
    match txn {
        TransactionStatement::Begin(isolation) => {
            let isolation = isolation.clone().unwrap_or(self.default_isolation.clone());
            self.begin_transaction(isolation)
        }
        TransactionStatement::Commit => self.commit_transaction(),
        TransactionStatement::Rollback => self.rollback_transaction(),
        TransactionStatement::SetTransaction { isolation_level } => {
            self.default_isolation = isolation_level.clone();
            Ok(ExecutorResult::new(vec![], 0))
        }
        TransactionStatement::StartTransaction(isolation) => {
            let isolation = isolation.clone().unwrap_or(self.default_isolation.clone());
            self.begin_transaction(isolation)
        }
    }
}

fn begin_transaction(&mut self, isolation: IsolationLevel) -> SqlResult<ExecutorResult> {
    let tx_id = self.transaction_manager
        .begin_transaction(isolation)
        .map_err(|e| SqlError::ExecutionError(e.to_string()))?;
    self.current_tx_id = Some(tx_id);
    Ok(ExecutorResult::new(vec![vec![Value::Integer(tx_id.0 as i64)]], 1))
}

fn commit_transaction(&mut self) -> SqlResult<ExecutorResult> {
    let tx_id = self.current_tx_id
        .ok_or_else(|| SqlError::ExecutionError("No active transaction".to_string()))?;
    self.transaction_manager
        .commit(tx_id)
        .map_err(|e| SqlError::ExecutionError(e.to_string()))?;
    self.current_tx_id = None;
    Ok(ExecutorResult::new(vec![vec![Value::Text("COMMIT".to_string())]], 1))
}

fn rollback_transaction(&mut self) -> SqlResult<ExecutorResult> {
    let tx_id = self.current_tx_id
        .ok_or_else(|| SqlError::ExecutionError("No active transaction".to_string()))?;
    self.transaction_manager
        .rollback(tx_id)
        .map_err(|e| SqlError::ExecutionError(e.to_string()))?;
    self.current_tx_id = None;
    Ok(ExecutorResult::new(vec![vec![Value::Text("ROLLBACK".to_string())]], 1))
}
```

- [ ] **Step 3: 在 execute 方法中处理 Transaction 变体**

在 `execute` 方法的 match 语句中添加:
```rust
Statement::Transaction(ref txn) => self.execute_transaction(txn),
```

- [ ] **Step 4: 初始化 ExecutionEngine 时初始化事务管理器**

在 `ExecutionEngine::new` 中:
```rust
pub fn new(storage: Arc<RwLock<dyn StorageEngine>>) -> Self {
    Self {
        storage,
        stats: Arc::new(RwLock::new(HashMap::new())),
        transaction_manager: TransactionManager::new(),
        current_tx_id: None,
        default_isolation: IsolationLevel::ReadCommitted,
    }
}
```

- [ ] **Step 5: 添加 IsolationLevel 导入**

```rust
use crate::transaction::{IsolationLevel, TransactionManager, TxId};
```

- [ ] **Step 6: 运行测试**

Run: `cargo build`
Expected: Compiles successfully

- [ ] **Step 7: Commit**

```bash
git add src/execution_engine.rs
git commit -m "feat(execution): integrate TransactionManager into ExecutionEngine"
```

---

## Task 4: 增强 SSI 集成测试

**Files:**
- Modify: `crates/transaction/tests/ssi_integration.rs`

- [ ] **Step 1: 添加 WriteSkew 测试**

```rust
#[tokio::test]
async fn test_ssi_write_skew_detection() {
    // Classic WriteSkew: T1 reads X, writes Y; T2 reads Y, writes X
    // Both should see a conflict when validating
    let locks = Arc::new(DistributedLockManager::new());
    let detector = SsiDetector::new(locks);

    // T1: R(X), W(Y)
    let tx1 = TxId::new(1);
    detector.record_read(tx1, b"X".to_vec()).await;
    detector.record_write(tx1, b"Y".to_vec()).await;

    // T2: R(Y), W(X)
    let tx2 = TxId::new(2);
    detector.record_read(tx2, b"Y".to_vec()).await;
    detector.record_write(tx2, b"X".to_vec()).await;

    // Both should have RW-WR conflicts
    let result1 = detector.validate_commit(tx1).await;
    let result2 = detector.validate_commit(tx2).await;

    // At least one should fail
    assert!(result1.is_err() || result2.is_err());
}
```

- [ ] **Step 2: 添加更多冲突场景测试**

```rust
#[tokio::test]
async fn test_ssi_direct_write_conflict() {
    // T1 and T2 both write to X - should conflict
    let locks = Arc::new(DistributedLockManager::new());
    let detector = SsiDetector::new(locks);

    let tx1 = TxId::new(1);
    detector.record_write(tx1, b"X".to_vec()).await;

    let tx2 = TxId::new(2);
    detector.record_write(tx2, b"X".to_vec()).await;

    let result1 = detector.validate_commit(tx1).await;
    let result2 = detector.validate_commit(tx2).await;

    // First commit should succeed, second should fail
    assert!(result1.is_ok() ^ result2.is_ok());
}

#[tokio::test]
async fn test_ssi_multiple_readers_no_conflict() {
    // Multiple readers on same key - no conflict
    let locks = Arc::new(DistributedLockManager::new());
    let detector = SsiDetector::new(locks);

    let tx1 = TxId::new(1);
    detector.record_read(tx1, b"X".to_vec()).await;

    let tx2 = TxId::new(2);
    detector.record_read(tx2, b"X".to_vec()).await;

    let tx3 = TxId::new(3);
    detector.record_read(tx3, b"X".to_vec()).await;

    assert!(detector.validate_commit(tx1).await.is_ok());
    assert!(detector.validate_commit(tx2).await.is_ok());
    assert!(detector.validate_commit(tx3).await.is_ok());
}
```

- [ ] **Step 3: 运行测试**

Run: `cargo test -p sqlrustgo-transaction --test ssi_integration`
Expected: All SSI tests pass

- [ ] **Step 4: Commit**

```bash
git add crates/transaction/tests/ssi_integration.rs
git commit -m "test(transaction): add more SSI conflict detection tests"
```

---

## Task 5: SQL 事务回归测试

**Files:**
- Create: `tests/mvcc_transaction_test.rs`

- [ ] **Step 1: 创建 MVCC 事务测试文件**

```rust
use sqlrustgo::{parse, ExecutionEngine, MemoryStorage};
use sqlrustgo_storage::StorageEngine;
use std::sync::{Arc, RwLock};

fn create_engine() -> ExecutionEngine {
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    ExecutionEngine::new(storage)
}

#[test]
fn test_begin_commit_transaction() {
    let mut engine = create_engine();

    // Create table
    engine.execute("CREATE TABLE t1 (id INTEGER, value TEXT)").unwrap();
    engine.execute("INSERT INTO t1 VALUES (1, 'a')").unwrap();

    // Begin transaction
    let result = engine.execute("BEGIN");
    assert!(result.is_ok());

    // Read in transaction
    let result = engine.execute("SELECT * FROM t1");
    assert!(result.is_ok());

    // Commit
    let result = engine.execute("COMMIT");
    assert!(result.is_ok());
}

#[test]
fn test_begin_rollback_transaction() {
    let mut engine = create_engine();

    // Create table
    engine.execute("CREATE TABLE t1 (id INTEGER, value TEXT)").unwrap();

    // Begin transaction
    engine.execute("BEGIN").unwrap();

    // Insert in transaction
    engine.execute("INSERT INTO t1 VALUES (1, 'a')").unwrap();

    // Rollback
    let result = engine.execute("ROLLBACK");
    assert!(result.is_ok());

    // Verify data is rolled back
    let result = engine.execute("SELECT COUNT(*) FROM t1").unwrap();
    assert_eq!(result.rows[0][0], sqlrustgo::Value::Integer(0));
}

#[test]
fn test_begin_serializable() {
    let mut engine = create_engine();

    // Create table
    engine.execute("CREATE TABLE t1 (id INTEGER, value INTEGER)").unwrap();
    engine.execute("INSERT INTO t1 VALUES (1, 100)").unwrap();

    // Begin with SERIALIZABLE
    let result = engine.execute("BEGIN SERIALIZABLE");
    assert!(result.is_ok());

    // Commit
    let result = engine.execute("COMMIT");
    assert!(result.is_ok());
}

#[test]
fn test_set_transaction_isolation() {
    let mut engine = create_engine();

    engine.execute("SET TRANSACTION ISOLATION LEVEL SERIALIZABLE").unwrap();

    let result = engine.execute("BEGIN");
    assert!(result.is_ok());
}
```

- [ ] **Step 2: 运行测试**

Run: `cargo test --test mvcc_transaction_test`
Expected: All tests pass

- [ ] **Step 3: Commit**

```bash
git add tests/mvcc_transaction_test.rs
git commit -m "test: add MVCC transaction regression tests"
```

---

## Task 6: 最终验证

- [ ] **Step 1: 运行 L0 冒烟测试**

Run:
```bash
cargo build --all-features
cargo fmt --check
cargo clippy --all-features -- -D warnings
```
Expected: All pass

- [ ] **Step 2: 运行 L1/L2 测试**

Run:
```bash
cargo test -p sqlrustgo-parser --lib
cargo test -p sqlrustgo-transaction --lib
cargo test -p sqlrustgo-transaction --test ssi_integration
cargo test --test mvcc_transaction_test
```
Expected: All pass

- [ ] **Step 3: 更新设计文档标记完成**

在 `docs/superpowers/specs/2026-04-20-mvcc-ssi-integration-design.md` 中:
- 更新状态: `Draft` → `Implemented`
- 勾选所有验收标准

- [ ] **Step 4: Commit**

```bash
git add docs/superpowers/specs/2026-04-20-mvcc-ssi-integration-design.md
git commit -m "docs: update MVCC SSI integration design as implemented"
```

---

## 验收标准检查

- [ ] `BEGIN SERIALIZABLE; ... ; COMMIT;` 可执行
- [ ] WriteSkew 冲突被正确检测
- [ ] SSI 集成测试 > 20 tests (当前 28 + 新增)
- [ ] 回归测试框架包含 MVCC 测试
- [ ] Clippy 和 fmt 通过

---

## 执行选项

**1. Subagent-Driven (recommended)** - I dispatch a fresh subagent per task, review between tasks, fast iteration

**2. Inline Execution** - Execute tasks in this session using executing-plans, batch execution with checkpoints

Which approach?