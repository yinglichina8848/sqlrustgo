# Observability Layer Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 实现可观测性层，包含 TRANSACTION_HISTORY、LOCK_WAIT_GRAPH、RECOVERY_HISTORY、WAL_STATS 四个系统表及对应 SHOW 命令

**Architecture:** 创建 `crates/observability/` crate，通过 Observable trait 统一收集事件，混合模式持久化（热数据内存，冷数据定期刷盘）

**Tech Stack:** Rust, WAL, Transaction Manager, Information Schema

---

## 前置准备

### Task 0: 创建 observability crate

**Files:**
- Create: `crates/observability/Cargo.toml`
- Create: `crates/observability/src/lib.rs`
- Create: `crates/observability/src/observer.rs`
- Create: `crates/observability/src/metrics.rs`
- Create: `crates/observability/src/tables/mod.rs`

**Step 1: 创建 Cargo.toml**

```toml
[package]
name = "sqlrustgo-observability"
version = "0.1.0"
edition = "2021"

[dependencies]
sqlrustgo-types = { path = "../types" }
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1", features = ["sync", "rt"] }
```

**Step 2: 创建基础框架**

```rust
// src/lib.rs
pub mod observer;
pub mod metrics;
pub mod tables;
```

**Step 3: Commit**

```bash
git add crates/observability/
git commit -m "feat(observability): initial crate scaffolding"
```

---

## Task 1: Observable Trait 和基础框架

**Files:**
- Modify: `crates/observability/src/observer.rs`

**Step 1: 定义 ObservableEvent 枚举和 Trait**

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObservableEvent {
    BeginTransaction { tx_id: u64, isolation: String },
    CommitTransaction { tx_id: u64, timestamp: u64 },
    AbortTransaction { tx_id: u64, timestamp: u64 },
    LockWait { waiter: u64, holder: u64, key: String, mode: String },
    LockAcquire { tx_id: u64, key: String, mode: String },
    LockRelease { tx_id: u64, key: String },
    WalWrite { bytes: u64, lsn: u64 },
    WalFlush { lsn: u64 },
    RecoveryStart { crash_timestamp: u64 },
    RecoveryComplete { transactions_replayed: u64, status: String },
}

pub trait Observable {
    fn record(&self, event: ObservableEvent);
}
```

**Step 2: 创建 InMemoryStore 基础**

```rust
use std::collections::VecDeque;

pub struct InMemoryStore<T> {
    buffer: VecDeque<T>,
    max_size: usize,
}

impl<T> InMemoryStore<T> {
    pub fn new(max_size: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(max_size),
            max_size,
        }
    }

    pub fn append(&mut self, item: T) -> Option<T> {  // returns evicted if any
        if self.buffer.len() >= self.max_size {
            self.buffer.pop_front()
        } else {
            None
        };
        self.buffer.push_back(item);
        None
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.buffer.iter()
    }
}
```

**Step 3: Commit**

```bash
git add crates/observability/src/observer.rs
git commit -m "feat(observability): add Observable trait and InMemoryStore"
```

---

## Task 2: TRANSACTION_HISTORY 系统表

**Files:**
- Create: `crates/observability/src/tables/transaction_history.rs`
- Modify: `crates/observability/src/tables/mod.rs`

**Step 1: 定义 TransactionHistoryEntry**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionHistoryEntry {
    pub tx_id: u64,
    pub tx_uuid: String,
    pub start_time: u64,
    pub commit_time: Option<u64>,
    pub abort_time: Option<u64>,
    pub isolation_level: String,
    pub status: TransactionStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TransactionStatus {
    Active,
    Committed,
    Aborted,
}

pub struct TransactionHistory {
    store: InMemoryStore<TransactionHistoryEntry>,
    persistent_file: Option<File>,
}

impl TransactionHistory {
    pub fn new(max_memory: usize) -> Self { ... }
    pub fn append(&mut self, entry: TransactionHistoryEntry) { ... }
    pub fn query(&self, limit: Option<u32>) -> Vec<&TransactionHistoryEntry> { ... }
}
```

**Step 2: 添加刷盘逻辑**

```rust
impl TransactionHistory {
    fn flush_to_disk(&mut self) -> std::io::Result<()> {
        // Append entries to binary file
    }
}
```

**Step 3: Commit**

```bash
git add crates/observability/src/tables/transaction_history.rs
git commit -m "feat(observability): add TransactionHistory system table"
```

---

## Task 3: LOCK_WAIT_GRAPH 系统表

**Files:**
- Create: `crates/observability/src/tables/lock_wait_graph.rs`
- Modify: `crates/observability/src/tables/mod.rs`

**Step 1: 定义 LockWaitEdge 和 LockWaitGraph**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockWaitEdge {
    pub waiter_tx_id: u64,
    pub holder_tx_id: u64,
    pub lock_key: String,
    pub lock_mode: String,
    pub wait_start_time: u64,
}

pub struct LockWaitGraph {
    edges: Vec<LockWaitEdge>,
    active_waits: HashMap<u64, LockWaitEdge>,  // tx_id -> edge
}

impl LockWaitGraph {
    pub fn new() -> Self { ... }
    pub fn add_wait(&mut self, edge: LockWaitEdge) { ... }
    pub fn remove_wait(&mut self, tx_id: u64) { ... }
    pub fn get_active_waits(&self) -> Vec<&LockWaitEdge> { ... }
    pub fn detect_deadlock(&self) -> Vec<Vec<u64>> { ... }  // returns cycles
}
```

**Step 2: Commit**

```bash
git add crates/observability/src/tables/lock_wait_graph.rs
git commit -m "feat(observability): add LockWaitGraph system table"
```

---

## Task 4: RECOVERY_HISTORY 系统表

**Files:**
- Create: `crates/observability/src/tables/recovery_history.rs`
- Modify: `crates/observability/src/tables/mod.rs`

**Step 1: 定义 RecoveryHistoryEntry**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryHistoryEntry {
    pub recovery_id: u64,
    pub crash_timestamp: u64,
    pub recovery_timestamp: u64,
    pub lsn_recovered: u64,
    pub transactions_replayed: u64,
    pub status: RecoveryStatus,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RecoveryStatus {
    Success,
    Failed,
}

pub struct RecoveryHistory {
    entries: Vec<RecoveryHistoryEntry>,
    file_path: PathBuf,
}

impl RecoveryHistory {
    pub fn new(data_dir: PathBuf) -> Self { ... }
    pub fn append(&mut self, entry: RecoveryHistoryEntry) -> std::io::Result<()> { ... }
    pub fn load(&mut self) -> std::io::Result<()> { ... }
    pub fn query(&self, limit: Option<u32>) -> Vec<&RecoveryHistoryEntry> { ... }
}
```

**Step 2: Commit**

```bash
git add crates/observability/src/tables/recovery_history.rs
git commit -m "feat(observability): add RecoveryHistory system table"
```

---

## Task 5: WAL_STATS 系统表

**Files:**
- Create: `crates/observability/src/tables/wal_stats.rs`
- Modify: `crates/observability/src/tables/mod.rs`

**Step 1: 定义 WalStats**

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WalStats {
    pub total_writes: u64,
    pub total_bytes: u64,
    pub flush_count: u64,
    pub replay_count: u64,
    pub replay_time_ms: u64,
    pub last_flush_lsn: u64,
    pub current_lsn: u64,
}

pub struct WalStatsCollector {
    stats: Arc<Mutex<WalStats>>,
}

impl WalStatsCollector {
    pub fn new() -> Self { ... }
    pub fn record_write(&self, bytes: u64, lsn: u64) { ... }
    pub fn record_flush(&self, lsn: u64) { ... }
    pub fn record_replay(&self, time_ms: u64) { ... }
    pub fn get_stats(&self) -> WalStats { ... }
}
```

**Step 2: Commit**

```bash
git add crates/observability/src/tables/wal_stats.rs
git commit -m "feat(observability): add WalStats system table"
```

---

## Task 6: Parser SHOW 命令扩展

**Files:**
- Modify: `crates/parser/src/parser.rs:579-607`

**Step 1: 扩展 ShowStatement 枚举**

```rust
// 在 ShowStatement 枚举中添加
pub enum ShowStatement {
    // ... existing variants ...
    TransactionHistory {
        limit: Option<u32>,
    },
    LockWaits,
    RecoveryHistory {
        limit: Option<u32>,
    },
    WalStats,
}
```

**Step 2: 添加 Parser 解析规则**

```rust
// 添加解析规则
"TRANSACTION" "HISTORY" => ShowStatement::TransactionHistory { limit: None }
"LIMIT" expr => { /* update limit */ }
"LOCK" "WAITS" => ShowStatement::LockWaits
"RECOVERY" "HISTORY" => ShowStatement::RecoveryHistory { limit: None }
"WAL" "STATS" => ShowStatement::WalStats
```

**Step 3: Commit**

```bash
git add crates/parser/src/parser.rs
git commit -m "feat(parser): add SHOW TRANSACTION HISTORY/LOCK WAITS/RECOVERY HISTORY/WAL STATS"
```

---

## Task 7: Execution Engine SHOW 命令实现

**Files:**
- Modify: `src/execution_engine.rs:3538-3564`

**Step 1: 扩展 execute_show 函数**

```rust
fn execute_show(&self, show: &ShowStatement) -> SqlResult<ExecutorResult> {
    match show {
        // ... existing ...
        ShowStatement::TransactionHistory { limit } => {
            self.execute_show_transaction_history(limit.as_ref())
        }
        ShowStatement::LockWaits => {
            self.execute_show_lock_waits()
        }
        ShowStatement::RecoveryHistory { limit } => {
            self.execute_show_recovery_history(limit.as_ref())
        }
        ShowStatement::WalStats => {
            self.execute_show_wal_stats()
        }
    }
}
```

**Step 2: 实现各个执行函数**

```rust
fn execute_show_transaction_history(&self, limit: Option<&u32>) -> SqlResult<ExecutorResult> {
    let history = self.observability.get_transaction_history(limit.map(|v| *v));
    // format as table
}

fn execute_show_lock_waits(&self) -> SqlResult<ExecutorResult> { ... }
fn execute_show_recovery_history(&self, limit: Option<&u32>) -> SqlResult<ExecutorResult> { ... }
fn execute_show_wal_stats(&self) -> SqlResult<ExecutorResult> { ... }
```

**Step 3: Commit**

```bash
git add src/execution_engine.rs
git commit -m "feat(executor): implement SHOW TRANSACTION HISTORY/LOCK WAITS/RECOVERY HISTORY/WAL STATS"
```

---

## Task 8: EXPLAIN ANALYZE TIMELINE

**Files:**
- Modify: `crates/parser/src/parser.rs:615-621`
- Modify: `src/execution_engine.rs`

**Step 1: 扩展 ExplainStatement**

```rust
pub struct ExplainStatement {
    pub analyze: bool,
    pub statement: Box<Statement>,
    pub format: Option<String>,
    pub timeline: bool,  // 新增
}
```

**Step 2: 实现 Timeline 格式**

```rust
fn execute_explain(&self, explain: &ExplainStatement) -> SqlResult<ExecutorResult> {
    if explain.timeline {
        // 返回时间线格式的查询执行分析
    }
}
```

**Step 3: Commit**

```bash
git commit -m "feat(executor): add EXPLAIN ANALYZE TIMELINE"
```

---

## Task 9: 集成到 Transaction 模块

**Files:**
- Modify: `crates/transaction/src/transaction_manager.rs`

**Step 1: 添加 Observable 集成**

```rust
pub struct TransactionManager {
    // ... existing fields ...
    observer: Option<Arc<dyn Observable>>,
}
```

**Step 2: 在事件点调用 observer.record()**

```rust
pub fn begin_transaction(&mut self, isolation: IsolationLevel) -> Result<TxId, SsiError> {
    // ... existing code ...
    if let Some(ref obs) = self.observer {
        obs.record(ObservableEvent::BeginTransaction {
            tx_id: tx_id.as_u64(),
            isolation: format!("{:?}", isolation),
        });
    }
}
```

**Step 3: Commit**

```bash
git add crates/transaction/src/transaction_manager.rs
git commit -m "feat(transaction): integrate observability event recording"
```

---

## Task 10: 集成到 WAL 模块

**Files:**
- Modify: `crates/storage/src/wal.rs`

**Step 1: 添加 WalStats 收集**

```rust
impl Wal {
    // ... existing fields ...
    stats_collector: Option<Arc<WalStatsCollector>>,
}
```

**Step 2: 在写入点记录事件**

```rust
pub fn append(&mut self, entry: WalEntry) -> Result<u64> {
    // ... existing code ...
    if let Some(ref stats) = self.stats_collector {
        stats.record_write(entry.size() as u64, lsn);
    }
}
```

**Step 3: Commit**

```bash
git add crates/storage/src/wal.rs
git commit -m "feat(wal): integrate WalStats collection"
```

---

## Task 11: INFORMATION_SCHEMA 集成

**Files:**
- Modify: `crates/information-schema/src/lib.rs`

**Step 1: 添加可观测性表查询方法**

```rust
impl InformationSchema<'_> {
    pub fn get_transaction_history(&self) -> Vec<TransactionHistoryRow> { ... }
    pub fn get_lock_waits(&self) -> Vec<LockWaitRow> { ... }
    pub fn get_recovery_history(&self) -> Vec<RecoveryHistoryRow> { ... }
    pub fn get_wal_stats(&self) -> Vec<WalStatsRow> { ... }
}
```

**Step 2: Commit**

```bash
git add crates/information-schema/src/lib.rs
git commit -m "feat(information-schema): add observability system tables"
```

---

## Task 12: 单元测试

**Files:**
- Create: `crates/observability/src/tables/transaction_history_tests.rs`
- Create: `crates/observability/src/tables/lock_wait_graph_tests.rs`

**Step 1: 写测试**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_history_append_and_query() {
        let mut history = TransactionHistory::new(100);
        history.append(TransactionHistoryEntry {
            tx_id: 1,
            // ...
        });
        let results = history.query(Some(10));
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_lock_wait_graph_detect_deadlock() {
        let mut graph = LockWaitGraph::new();
        // tx1 holds lock A, tx2 waits for A
        // tx2 holds lock B, tx1 waits for B
        // Should detect cycle
    }
}
```

**Step 2: 运行测试**

```bash
cargo test -p sqlrustgo-observability
```

**Step 3: Commit**

```bash
git add crates/observability/
git commit -m "test(observability): add unit tests"
```

---

## Task 13: 集成测试

**Files:**
- Create: `tests/integration/test_observability.rs`

**Step 1: 写集成测试**

```rust
#[tokio::test]
async fn test_transaction_history_full_flow() {
    // 1. Begin transaction
    // 2. Execute some queries
    // 3. Commit
    // 4. Verify history contains entry
}

#[tokio::test]
async fn test_show_transaction_history_command() {
    // 1. Create observability instance
    // 2. Execute SHOW TRANSACTION HISTORY
    // 3. Verify output format
}
```

**Step 2: 运行测试**

```bash
cargo test --test test_observability
```

**Step 3: Commit**

```bash
git add tests/integration/test_observability.rs
git commit -m "test(observability): add integration tests"
```

---

## Task 14: Beta Gate 验证

**Step 1: 运行 Beta Gate**

```bash
bash scripts/gate/check_beta_v310.sh
```

**Step 2: 确认所有检查通过**

Expected: 18/18 PASS

**Step 3: 如有失败，修复并重新测试**

---

## 依赖关系图

```
Task 0 (crate)
    ↓
Task 1 (Observable trait) ← Task 2, 3, 4, 5
    ↓
Task 6 (Parser) ← Task 7 (Executor)
    ↓
Task 8 (Explain Timeline)
    ↓
Task 9 (Transaction集成) ← Task 2
Task 10 (WAL集成) ← Task 5
    ↓
Task 11 (InformationSchema)
    ↓
Task 12 (单元测试) ← Task 2-5
    ↓
Task 13 (集成测试) ← Task 7, 9, 10, 11
    ↓
Task 14 (Beta Gate)
```

---

## 验收条件检查清单

- [ ] `SHOW TRANSACTION HISTORY` 输出事务历史
- [ ] `SHOW LOCK WAITS` 输出当前锁等待
- [ ] `SHOW RECOVERY HISTORY` 输出恢复历史
- [ ] `SHOW WAL STATS` 输出 WAL 统计
- [ ] `EXPLAIN ANALYZE TIMELINE` 输出查询时间线
- [ ] SQL 表查询可用: `SELECT * FROM transaction_history`
- [ ] 内存缓冲区超过阈值自动刷盘
- [ ] 重启后 RECOVERY_HISTORY 持久化数据可恢复
- [ ] Beta Gate 仍通过 (18/18)
- [ ] 所有新代码通过 clippy 和 fmt

---

*Implementation Plan - 2026-05-14*
