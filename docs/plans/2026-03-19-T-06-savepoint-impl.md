# T-06 SAVEPOINT 实现计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 实现事务内保存点支持，允许部分回滚

**Architecture:** UndoLog + Savepoint 栈结构

**Tech Stack:** Rust, Vec, 事务模块

---

### Task 1: 创建 UndoRecord 和 Savepoint 数据结构

**Files:**
- Create: `crates/transaction/src/savepoint.rs`

**Step 1: 编写测试**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_undo_record_insert() {
        let record = UndoRecord::Insert { key: vec![1, 2, 3] };
        assert!(matches!(record, UndoRecord::Insert { .. }));
    }

    #[test]
    fn test_savepoint_new() {
        let sp = Savepoint::new("test".to_string(), 10);
        assert_eq!(sp.name, "test");
        assert_eq!(sp.undo_log_index, 10);
    }
}
```

**Step 2: 运行测试确认失败**

Run: `cargo test -p sqlrustgo-transaction savepoint::tests::test_undo_record_insert`
Expected: FAIL

**Step 3: 编写实现**

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UndoRecord {
    Insert { key: Vec<u8> },
    Delete { key: Vec<u8>, old_value: Vec<u8> },
    Update { key: Vec<u8>, old_value: Vec<u8> },
}

#[derive(Debug, Clone)]
pub struct Savepoint {
    pub name: String,
    pub undo_log_index: usize,
}

impl Savepoint {
    pub fn new(name: String, undo_log_index: usize) -> Self {
        Self { name, undo_log_index }
    }
}
```

**Step 4: 运行测试确认通过**

Run: `cargo test -p sqlrustgo-transaction savepoint::tests`
Expected: PASS

**Step 5: 提交**

```bash
git add crates/transaction/src/savepoint.rs
git commit -m "feat(transaction): T-06 添加 UndoRecord 和 Savepoint 数据结构"
```

---

### Task 2: 实现 SavepointManager

**Files:**
- Modify: `crates/transaction/src/savepoint.rs`

**Step 1: 编写测试**

```rust
#[test]
fn test_savepoint_create() {
    let mut manager = SavepointManager::new();
    manager.savepoint("sp1".to_string()).unwrap();
    assert_eq!(manager.savepoints.len(), 1);
}

#[test]
fn test_nested_savepoints() {
    let mut manager = SavepointManager::new();
    manager.savepoint("sp1".to_string()).unwrap();
    manager.savepoint("sp2".to_string()).unwrap();
    assert_eq!(manager.savepoints.len(), 2);
    
    manager.rollback_to("sp1".to_string()).unwrap();
    assert_eq!(manager.savepoints.len(), 1);
}
```

**Step 2: 运行测试确认失败**

Run: `cargo test -p sqlrustgo-transaction savepoint::tests::test_savepoint_create`
Expected: FAIL

**Step 3: 编写实现**

```rust
use crate::mvcc::TxId;

pub struct SavepointManager {
    savepoints: Vec<Savepoint>,
    undo_log: Vec<UndoRecord>,
}

impl SavepointManager {
    pub fn new() -> Self {
        Self {
            savepoints: Vec::new(),
            undo_log: Vec::new(),
        }
    }

    pub fn savepoint(&mut self, name: String) -> Result<(), SavepointError> {
        // 同名 savepoint，后者覆盖前者
        if let Some(idx) = self.savepoints.iter().rposition(|s| s.name == name) {
            self.savepoints[idx].undo_log_index = self.undo_log.len();
        } else {
            self.savepoints.push(Savepoint::new(name, self.undo_log.len()));
        }
        Ok(())
    }

    pub fn rollback_to(&mut self, name: &str) -> Result<(), SavepointError> {
        let idx = self.savepoints
            .iter()
            .rposition(|s| s.name == name)
            .ok_or(SavepointError::NotFound)?;

        let sp = &self.savepoints[idx];
        
        // 回滚 undo log
        while self.undo_log.len() > sp.undo_log_index {
            self.undo_log.pop();
        }
        
        // 删除该 savepoint 之后的所有 savepoints
        self.savepoints.truncate(idx + 1);
        
        Ok(())
    }

    pub fn release_savepoint(&mut self, name: &str) -> Result<(), SavepointError> {
        self.savepoints.retain(|s| s.name != name);
        Ok(())
    }

    pub fn add_undo(&mut self, record: UndoRecord) {
        self.undo_log.push(record);
    }

    pub fn get_savepoint_count(&self) -> usize {
        self.savepoints.len()
    }
}

impl Default for SavepointManager {
    fn default() -> Self {
        Self::new()
    }
}
```

**Step 4: 运行测试确认通过**

Run: `cargo test -p sqlrustgo-transaction savepoint::tests::test_savepoint_create`
Expected: PASS

**Step 5: 提交**

```bash
git add crates/transaction/src/savepoint.rs
git commit -m "feat(transaction): T-06 实现 SavepointManager"
```

---

### Task 3: 添加 SavepointError

**Files:**
- Modify: `crates/transaction/src/savepoint.rs`

**Step 1: 编写测试**

```rust
#[test]
fn test_savepoint_not_found() {
    let mut manager = SavepointManager::new();
    let result = manager.rollback_to("nonexistent".to_string());
    assert!(matches!(result, Err(SavepointError::NotFound)));
}
```

**Step 2: 运行测试确认失败**

Run: `cargo test -p sqlrustgo-transaction savepoint::tests::test_savepoint_not_found`
Expected: FAIL

**Step 3: 编写实现**

```rust
#[derive(Debug, Clone)]
pub enum SavepointError {
    NotFound,
    InvalidOperation,
}

impl std::fmt::Display for SavepointError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SavepointError::NotFound => write!(f, "savepoint not found"),
            SavepointError::InvalidOperation => write!(f, "invalid savepoint operation"),
        }
    }
}

impl std::error::Error for SavepointError {}
```

**Step 4: 运行测试确认通过**

Run: `cargo test -p sqlrustgo-transaction savepoint::tests::test_savepoint_not_found`
Expected: PASS

**Step 5: 提交**

```bash
git add crates/transaction/src/savepoint.rs
git commit -m "feat(transaction): T-06 添加 SavepointError"
```

---

### Task 4: 集成到 TransactionManager

**Files:**
- Modify: `crates/transaction/src/manager.rs`

**Step 1: 编写测试**

```rust
#[test]
fn test_transaction_savepoint() {
    let mut manager = TransactionManager::new();
    manager.begin().unwrap();
    
    manager.savepoint("sp1".to_string()).unwrap();
    
    let ctx = manager.get_transaction_context().unwrap();
    assert!(ctx.savepoints.len() >= 1);
}
```

**Step 2: 运行测试确认失败**

Run: `cargo test -p sqlrustgo-transaction manager::tests::test_transaction_savepoint`
Expected: FAIL

**Step 3: 集成实现**

在 TransactionManager 中添加:
```rust
use crate::savepoint::SavepointManager;

pub struct TransactionManager {
    mvcc: Arc<RwLock<MvccEngine>>,
    current_tx: Option<TxId>,
    isolation_level: IsolationLevel,
    savepoint_manager: SavepointManager,
}

impl TransactionManager {
    pub fn savepoint(&mut self, name: String) -> Result<(), SavepointError> {
        self.savepoint_manager.savepoint(name)
    }

    pub fn rollback_to(&mut self, name: String) -> Result<(), SavepointError> {
        self.savepoint_manager.rollback_to(&name)
    }

    pub fn release_savepoint(&mut self, name: String) -> Result<(), SavepointError> {
        self.savepoint_manager.release_savepoint(&name)
    }
}
```

**Step 4: 运行测试确认通过**

Run: `cargo test -p sqlrustgo-transaction manager::tests::test_transaction_savepoint`
Expected: PASS

**Step 5: 提交**

```bash
git add crates/transaction/src/manager.rs
git commit -m "feat(transaction): T-06 集成 Savepoint 到 TransactionManager"
```

---

### Task 5: 更新 lib.rs 和最终测试

**Files:**
- Modify: `crates/transaction/src/lib.rs`

**Step 1: 添加导出**

```rust
pub mod savepoint;

pub use savepoint::{Savepoint, SavepointError, SavepointManager, UndoRecord};
```

**Step 2: 运行完整测试**

Run: `cargo test -p sqlrustgo-transaction`
Expected: ALL PASS

**Step 3: 运行 clippy**

Run: `cargo clippy -p sqlrustgo-transaction`
Expected: NO WARNINGS

**Step 4: 提交**

```bash
git add crates/transaction/src/lib.rs
git commit -m "feat(transaction): T-06 SAVEPOINT 支持完成

- UndoRecord 数据结构
- Savepoint 栈管理
- SAVEPOINT name
- ROLLBACK TO name
- RELEASE SAVEPOINT name
- 集成到 TransactionManager"
```

---

### Task 6: 创建 PR

```bash
gh pr create --base develop/v1.6.0 --title "feat(transaction): T-06 SAVEPOINT 支持" --body "..."
```

---

## 验收标准

- [ ] UndoRecord 和 Savepoint 数据结构
- [ ] SAVEPOINT 操作正常工作
- [ ] ROLLBACK TO 操作正常工作
- [ ] RELEASE SAVEPOINT 操作正常工作
- [ ] 嵌套 savepoint 支持
- [ ] 集成到 TransactionManager
- [ ] 所有测试通过
- [ ] Clippy 无警告
- [ ] PR 创建成功
