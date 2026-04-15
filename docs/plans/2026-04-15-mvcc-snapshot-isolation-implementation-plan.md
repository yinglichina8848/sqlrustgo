# MVCC Snapshot Isolation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 实现存储层 MVCC Snapshot Isolation，将现有 MvccEngine 元数据系统升级为真正的多版本并发控制。

**Architecture:** Append-only VersionChain 方案。每个 key 维护一个版本链（newest-first），写入时追加新版本而非覆盖，通过 commit_timestamp 控制可见性。

**Tech Stack:** Rust, sqlrustgo-transaction crate, TDD workflow

---

## 前置准备

### 1. 创建工作分支

```bash
# 从 develop/v2.5.0 创建功能分支
git checkout -b feature/mvcc-snapshot-isolation-1389
```

### 2. 阅读设计文档

- `docs/plans/2026-04-15-mvcc-snapshot-isolation-design.md`

### 3. 阅读现有代码

- `crates/transaction/src/mvcc.rs` - 现有 MvccEngine
- `crates/transaction/src/manager.rs` - TransactionManager
- `crates/transaction/src/lib.rs` - 模块导出

---

## 实现任务

### Task 1: 扩展 RowVersion 结构

**文件:**
- 修改: `crates/transaction/src/mvcc.rs:160-187`

**Step 1: 编写失败测试**

在 `crates/transaction/src/mvcc.rs` 的 tests 模块添加：

```rust
#[test]
fn test_row_version_commit_timestamp() {
    let mut version = RowVersion::new(TxId::new(1), vec![1, 2, 3]);
    
    // 创建时未提交
    assert!(version.created_commit_ts.is_none());
    
    // 提交后有时间戳
    version.commit(100);
    assert_eq!(version.created_commit_ts, Some(100));
}

#[test]
fn test_row_version_delete_timestamp() {
    let mut version = RowVersion::new(TxId::new(1), vec![1, 2, 3]);
    version.commit(50);
    
    // 标记删除
    version.mark_deleted(TxId::new(2), 75);
    assert_eq!(version.deleted_by, Some(TxId::new(2)));
    assert_eq!(version.deleted_commit_ts, Some(75));
}
```

**Step 2: 运行测试验证失败**

```bash
cargo test -p sqlrustgo-transaction test_row_version_commit_timestamp -- --nocapture
```

**Step 3: 实现增强版 RowVersion**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RowVersion {
    pub value: Vec<u8>,
    pub created_by: TxId,
    pub created_commit_ts: Option<u64>,
    pub deleted_by: Option<TxId>,
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

    pub fn commit(&mut self, timestamp: u64) {
        self.created_commit_ts = Some(timestamp);
    }

    pub fn mark_deleted(&mut self, tx_id: TxId, timestamp: u64) {
        self.deleted_by = Some(tx_id);
        self.deleted_commit_ts = Some(timestamp);
    }
}
```

**Step 4: 运行测试验证通过**

```bash
cargo test -p sqlrustgo-transaction test_row_version -- --nocapture
```

**Step 5: 提交**

```bash
git add crates/transaction/src/mvcc.rs
git commit -m "feat(transaction): extend RowVersion with commit/delete timestamps"
```

---

### Task 2: 实现 RowVersion 可见性判断

**文件:**
- 修改: `crates/transaction/src/mvcc.rs`

**Step 1: 编写失败测试**

```rust
#[test]
fn test_row_version_is_visible_own_uncommitted() {
    let version = RowVersion::new(TxId::new(1), vec![1, 2, 3]);
    let snapshot = Snapshot::new(TxId::new(1), 100, vec![TxId::new(1)]);
    
    // 自己创建的版本即使未提交也可见
    assert!(version.is_visible(&snapshot));
}

#[test]
fn test_row_version_is_visible_other_uncommitted() {
    let version = RowVersion::new(TxId::new(1), vec![1, 2, 3]);
    // 未提交
    let snapshot = Snapshot::new(TxId::new(2), 100, vec![TxId::new(1)]);
    
    // 其他事务未提交的版本不可见
    assert!(!version.is_visible(&snapshot));
}

#[test]
fn test_row_version_is_visible_committed_before_snapshot() {
    let mut version = RowVersion::new(TxId::new(1), vec![1, 2, 3]);
    version.commit(50); // 在 snapshot_ts 之前提交
    
    let snapshot = Snapshot::new(TxId::new(2), 100, vec![]);
    assert!(version.is_visible(&snapshot));
}

#[test]
fn test_row_version_is_visible_committed_after_snapshot() {
    let mut version = RowVersion::new(TxId::new(1), vec![1, 2, 3]);
    version.commit(150); // 在 snapshot_ts 之后提交
    
    let snapshot = Snapshot::new(TxId::new(2), 100, vec![]);
    assert!(!version.is_visible(&snapshot));
}

#[test]
fn test_row_version_is_visible_deleted_after_snapshot() {
    let mut version = RowVersion::new(TxId::new(1), vec![1, 2, 3]);
    version.commit(50);
    version.mark_deleted(TxId::new(2), 75); // 在 snapshot_ts 之前删除
    
    let snapshot = Snapshot::new(TxId::new(3), 100, vec![]);
    // 删除发生在 snapshot 之前，不可见
    assert!(!version.is_visible(&snapshot));
}

#[test]
fn test_row_version_is_visible_deleted_before_snapshot() {
    let mut version = RowVersion::new(TxId::new(1), vec![1, 2, 3]);
    version.commit(50);
    version.mark_deleted(TxId::new(2), 125); // 在 snapshot_ts 之后删除
    
    let snapshot = Snapshot::new(TxId::new(3), 100, vec![]);
    // 删除发生在 snapshot 之后，可见
    assert!(version.is_visible(&snapshot));
}
```

**Step 2: 运行测试验证失败**

```bash
cargo test -p sqlrustgo-transaction test_row_version_is_visible -- --nocapture
```

**Step 3: 实现 is_visible 方法**

在 RowVersion impl 块中添加：

```rust
impl RowVersion {
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
}
```

**Step 4: 运行测试验证通过**

```bash
cargo test -p sqlrustgo-transaction test_row_version_is_visible -- --nocapture
```

**Step 5: 提交**

```bash
git add crates/transaction/src/mvcc.rs
git commit -m "feat(transaction): implement RowVersion.is_visible() for snapshot isolation"
```

---

### Task 3: 实现 VersionChainMap

**文件:**
- 创建: `crates/transaction/src/version_chain.rs`
- 修改: `crates/transaction/src/lib.rs`

**Step 1: 编写失败测试**

```rust
// crates/transaction/tests/version_chain_test.rs

#[test]
fn test_version_chain_append_and_find_visible() {
    let mut chain = VersionChainMap::new();
    
    // 添加版本 v1
    chain.append(b"key1".to_vec(), RowVersion::new(TxId::new(1), b"v1".to_vec()));
    chain.commit_versions(TxId::new(1), 10);
    
    // 添加版本 v2
    chain.append(b"key1".to_vec(), RowVersion::new(TxId::new(2), b"v2".to_vec()));
    chain.commit_versions(TxId::new(2), 20);
    
    // 快照在 ts=15 时可见 v1
    let snapshot = Snapshot::new(TxId::new(3), 15, vec![]);
    assert_eq!(chain.find_visible(b"key1", &snapshot), Some(b"v1".to_vec()));
    
    // 快照在 ts=25 时可见 v2
    let snapshot = Snapshot::new(TxId::new(3), 25, vec![]);
    assert_eq!(chain.find_visible(b"key1", &snapshot), Some(b"v2".to_vec()));
}

#[test]
fn test_version_chain_rollback() {
    let mut chain = VersionChainMap::new();
    
    // 添加版本 v1
    chain.append(b"key1".to_vec(), RowVersion::new(TxId::new(1), b"v1".to_vec()));
    chain.commit_versions(TxId::new(1), 10);
    
    // 添加未提交版本 v2
    chain.append(b"key1".to_vec(), RowVersion::new(TxId::new(2), b"v2".to_vec()));
    // 不提交 tx2
    
    // 回滚 tx2
    chain.rollback_versions(TxId::new(2));
    
    // 只剩 v1
    let snapshot = Snapshot::new(TxId::new(3), 100, vec![]);
    assert_eq!(chain.find_visible(b"key1", &snapshot), Some(b"v1".to_vec()));
}
```

**Step 2: 运行测试验证失败**

```bash
cargo test -p sqlrustgo-transaction --test version_chain_test -- --nocapture
```

**Step 3: 实现 VersionChainMap**

```rust
// crates/transaction/src/version_chain.rs

use crate::mvcc::{RowVersion, Snapshot, TxId};
use std::collections::HashMap;
use std::sync::RwLock;

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
    pub fn find_visible(&self, key: &[u8], snapshot: &Snapshot) -> Option<Vec<u8>> {
        let chains = self.chains.read().unwrap();

        if let Some(versions) = chains.get(key) {
            for version in versions.iter().rev() {
                if version.is_visible(snapshot) {
                    return Some(version.value.clone());
                }
            }
        }
        None
    }

    /// 追加新版本
    pub fn append(&mut self, key: Vec<u8>, version: RowVersion) {
        let mut chains = self.chains.write().unwrap();
        chains
            .entry(key)
            .or_insert_with(Vec::new)
            .push(version);
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
            versions.retain(|v| {
                v.created_by != tx_id || v.created_commit_ts.is_some()
            });
        }
    }
}

impl Default for VersionChainMap {
    fn default() -> Self {
        Self::new()
    }
}
```

**Step 4: 更新 lib.rs 导出**

```rust
// crates/transaction/src/lib.rs
pub mod version_chain;
pub use version_chain::VersionChainMap;
```

**Step 5: 运行测试验证通过**

```bash
cargo test -p sqlrustgo-transaction --test version_chain_test -- --nocapture
```

**Step 6: 提交**

```bash
git add crates/transaction/src/version_chain.rs crates/transaction/src/lib.rs
git commit -m "feat(transaction): implement VersionChainMap for append-only storage"
```

---

### Task 4: 实现 MVCCStorage Trait

**文件:**
- 创建: `crates/transaction/src/mvcc_storage.rs`
- 修改: `crates/transaction/src/lib.rs`

**Step 1: 编写失败测试**

```rust
// crates/transaction/tests/mvcc_storage_test.rs

#[test]
fn test_mvcc_storage_read_write() {
    let storage = MVCCStorageEngine::new();
    
    // 写入
    storage.write_version(b"key1".to_vec(), b"value1".to_vec(), TxId::new(1));
    
    // 未提交时其他事务不可见
    let snapshot = Snapshot::new(TxId::new(2), 100, vec![]);
    assert_eq!(storage.read(b"key1", &snapshot), None);
    
    // 提交
    storage.commit_versions(TxId::new(1), 10).unwrap();
    
    // 提交后可见
    assert_eq!(storage.read(b"key1", &snapshot), Some(b"value1".to_vec()));
}

#[test]
fn test_mvcc_storage_snapshot_isolation() {
    let storage = MVCCStorageEngine::new();
    
    // TX1 写入 v1
    storage.write_version(b"key1".to_vec(), b"v1".to_vec(), TxId::new(1));
    storage.commit_versions(TxId::new(1), 10).unwrap();
    
    // TX2 写入 v2
    storage.write_version(b"key1".to_vec(), b"v2".to_vec(), TxId::new(2));
    storage.commit_versions(TxId::new(2), 20).unwrap();
    
    // TX3 的快照在 ts=15，只能看到 v1
    let snapshot = Snapshot::new(TxId::new(3), 15, vec![]);
    assert_eq!(storage.read(b"key1", &snapshot), Some(b"v1".to_vec()));
    
    // TX4 的快照在 ts=25，只能看到 v2
    let snapshot = Snapshot::new(TxId::new(4), 25, vec![]);
    assert_eq!(storage.read(b"key1", &snapshot), Some(b"v2".to_vec()));
}
```

**Step 2: 运行测试验证失败**

```bash
cargo test -p sqlrustgo-transaction --test mvcc_storage_test -- --nocapture
```

**Step 3: 实现 MVCCStorage Trait 和实现**

```rust
// crates/transaction/src/mvcc_storage.rs

use crate::mvcc::{RowVersion, Snapshot, TxId};
use crate::version_chain::VersionChainMap;
use crate::manager::TransactionError;
use std::sync::RwLock;

pub trait MVCCStorage: Send + Sync {
    fn read(&self, key: &[u8], snapshot: &Snapshot) -> Option<Vec<u8>>;
    fn write_version(&mut self, key: Vec<u8>, value: Vec<u8>, tx_id: TxId);
    fn delete_version(&mut self, key: Vec<u8>, tx_id: TxId);
    fn commit_versions(&mut self, tx_id: TxId, commit_ts: u64) -> Result<(), TransactionError>;
    fn rollback_versions(&mut self, tx_id: TxId) -> Result<(), TransactionError>;
    fn create_snapshot(&self, tx_id: TxId) -> Snapshot;
}

pub struct MVCCStorageEngine {
    chains: RwLock<VersionChainMap>,
}

impl MVCCStorageEngine {
    pub fn new() -> Self {
        Self {
            chains: RwLock::new(VersionChainMap::new()),
        }
    }
}

impl MVCCStorage for MVCCStorageEngine {
    fn read(&self, key: &[u8], snapshot: &Snapshot) -> Option<Vec<u8>> {
        let chains = self.chains.read().unwrap();
        chains.find_visible(key, snapshot)
    }

    fn write_version(&mut self, key: Vec<u8>, value: Vec<u8>, tx_id: TxId) {
        let mut chains = self.chains.write().unwrap();
        chains.append(key, RowVersion::new(tx_id, value));
    }

    fn delete_version(&mut self, key: Vec<u8>, tx_id: TxId) {
        let mut chains = self.chains.write().unwrap();
        chains.append(key, RowVersion::new_deleted(tx_id));
    }

    fn commit_versions(&mut self, tx_id: TxId, commit_ts: u64) -> Result<(), TransactionError> {
        let mut chains = self.chains.write().unwrap();
        chains.commit_versions(tx_id, commit_ts);
        Ok(())
    }

    fn rollback_versions(&mut self, tx_id: TxId) -> Result<(), TransactionError> {
        let mut chains = self.chains.write().unwrap();
        chains.rollback_versions(tx_id);
        Ok(())
    }

    fn create_snapshot(&self, _tx_id: TxId) -> Snapshot {
        // 简化版本，实际应从 MvccEngine 获取
        Snapshot::new_read_committed(_tx_id, 0)
    }
}
```

**Step 4: 更新 lib.rs 导出**

```rust
// crates/transaction/src/lib.rs
pub mod mvcc_storage;
pub use mvcc_storage::{MVCCStorage, MVCCStorageEngine};
```

**Step 5: 运行测试验证通过**

```bash
cargo test -p sqlrustgo-transaction --test mvcc_storage_test -- --nocapture
```

**Step 6: 提交**

```bash
git add crates/transaction/src/mvcc_storage.rs crates/transaction/src/lib.rs
git commit -m "feat(transaction): implement MVCCStorage trait and engine"
```

---

### Task 5: 集成到 TransactionManager

**文件:**
- 修改: `crates/transaction/src/manager.rs`

**Step 1: 编写失败测试**

```rust
#[test]
fn test_transaction_manager_with_mvcc_storage() {
    use crate::mvcc_storage::{MVCCStorage, MVCCStorageEngine};
    
    let storage = MVCCStorageEngine::new();
    let mut manager = TransactionManager::new();
    
    let tx_id = manager.begin().unwrap();
    
    // 写入
    storage.write_version(b"key1".to_vec(), b"value1".to_vec(), tx_id);
    
    // 自己的写入可见
    let ctx = manager.get_transaction_context().unwrap();
    assert_eq!(storage.read(b"key1", &ctx.snapshot), Some(b"value1".to_vec()));
    
    // 提交
    let commit_ts = manager.commit().unwrap().unwrap();
    storage.commit_versions(tx_id, commit_ts).unwrap();
    
    // 提交后仍可见
    let ctx = manager.get_transaction_context_for_query().unwrap();
    assert_eq!(storage.read(b"key1", &ctx.snapshot), Some(b"value1".to_vec()));
}
```

**Step 2: 运行测试验证失败**

```bash
cargo test -p sqlrustgo-transaction test_transaction_manager_with_mvcc_storage -- --nocapture
```

**Step 3: 实现集成方法**

在 TransactionManager 添加：

```rust
impl TransactionManager {
    /// 使用 MVCC 存储读取
    pub fn mvcc_read<S: MVCCStorage>(
        &self,
        storage: &S,
        key: &[u8],
    ) -> Result<Option<Vec<u8>>, TransactionError> {
        let ctx = self.get_transaction_context_for_query()?;
        Ok(storage.read(key, &ctx.snapshot))
    }

    /// 使用 MVCC 存储写入
    pub fn mvcc_write<S: MVCCStorage>(
        &mut self,
        storage: &mut S,
        key: Vec<u8>,
        value: Vec<u8>,
    ) -> Result<(), TransactionError> {
        let tx_id = self.current_tx.ok_or(TransactionError::NoTransaction)?;
        storage.write_version(key, value, tx_id);
        Ok(())
    }

    /// MVCC 提交流程
    pub fn mvcc_commit<S: MVCCStorage>(
        &mut self,
        storage: &mut S,
    ) -> Result<Option<u64>, TransactionError> {
        let tx_id = self.current_tx.take().ok_or(TransactionError::NoTransaction)?;

        let commit_ts = {
            let mut mvcc = self.mvcc.write().map_err(|_| TransactionError::LockError)?;
            mvcc.commit_transaction(tx_id)
        }.ok_or(TransactionError::InvalidTransaction)?;

        storage.commit_versions(tx_id, commit_ts)?;
        Ok(Some(commit_ts))
    }

    /// MVCC 回滚流程
    pub fn mvcc_rollback<S: MVCCStorage>(
        &mut self,
        storage: &mut S,
    ) -> Result<(), TransactionError> {
        let tx_id = self.current_tx.take().ok_or(TransactionError::NoTransaction)?;

        {
            let mut mvcc = self.mvcc.write().map_err(|_| TransactionError::LockError)?;
            if !mvcc.abort_transaction(tx_id) {
                return Err(TransactionError::InvalidTransaction);
            }
        }

        storage.rollback_versions(tx_id)?;
        Ok(())
    }
}
```

**Step 4: 运行测试验证通过**

```bash
cargo test -p sqlrustgo-transaction test_transaction_manager_with_mvcc_storage -- --nocapture
```

**Step 5: 提交**

```bash
git add crates/transaction/src/manager.rs
git commit -m "feat(transaction): integrate TransactionManager with MVCCStorage"
```

---

### Task 6: 集成测试 - Snapshot Isolation

**文件:**
- 创建: `tests/anomaly/mvcc_snapshot_isolation_integration_test.rs`

**Step 1: 编写测试**

```rust
//! MVCC Snapshot Isolation Integration Tests

#[cfg(test)]
mod tests {
    use sqlrustgo_transaction::mvcc_storage::{MVCCStorage, MVCCStorageEngine};
    use sqlrustgo_transaction::manager::TransactionManager;

    /// Test: Snapshot Isolation - TX1 读取不受 TX2 提交影响
    #[test]
    fn test_snapshot_isolation_read_consistency() {
        let storage = MVCCStorageEngine::new();
        let mut manager1 = TransactionManager::new();
        let mut manager2 = TransactionManager::new();

        // TX1 写入 v1
        let tx1 = manager1.begin().unwrap();
        storage.write_version(b"counter".to_vec(), b"1".to_vec(), tx1);
        let commit_ts = manager1.mvcc_commit(&mut storage).unwrap().unwrap();

        // TX2 在 TX1 提交后开始
        let tx2 = manager2.begin().unwrap();
        let ctx2 = manager2.get_transaction_context_for_query().unwrap();

        // TX2 应该看到 v1
        assert_eq!(storage.read(b"counter", &ctx2.snapshot), Some(b"1".to_vec()));

        // TX1 再写入 v2
        storage.write_version(b"counter".to_vec(), b"2".to_vec(), tx1);
        let _ = manager1.mvcc_commit(&mut storage);

        // TX2 的快照不应看到 v2
        assert_eq!(storage.read(b"counter", &ctx2.snapshot), Some(b"1".to_vec()));
    }

    /// Test: 防止脏读 - 未提交数据不可见
    #[test]
    fn test_no_dirty_read() {
        let storage = MVCCStorageEngine::new();
        let mut manager1 = TransactionManager::new();
        let mut manager2 = TransactionManager::new();

        // TX1 写入但未提交
        let tx1 = manager1.begin().unwrap();
        storage.write_version(b"data".to_vec(), b"secret".to_vec(), tx1);

        // TX2 开始
        let tx2 = manager2.begin().unwrap();
        let ctx2 = manager2.get_transaction_context_for_query().unwrap();

        // TX2 不应看到 TX1 未提交的写入
        assert_eq!(storage.read(b"data", &ctx2.snapshot), None);
    }

    /// Test: 回滚后数据消失
    #[test]
    fn test_rollback_discard() {
        let storage = MVCCStorageEngine::new();
        let mut manager = TransactionManager::new();

        let tx_id = manager.begin().unwrap();
        storage.write_version(b"data".to_vec(), b"temp".to_vec(), tx_id);

        // 回滚
        manager.mvcc_rollback(&mut storage).unwrap();

        // 数据应该消失
        let snapshot = sqlrustgo_transaction::mvcc::Snapshot::new_read_committed(
            sqlrustgo_transaction::mvcc::TxId::new(999),
            100
        );
        assert_eq!(storage.read(b"data", &snapshot), None);
    }
}
```

**Step 2: 运行集成测试**

```bash
cargo test --test mvcc_snapshot_isolation_integration_test -- --nocapture
```

**Step 3: 如有问题，修复并重新测试**

**Step 4: 提交**

```bash
git add tests/anomaly/mvcc_snapshot_isolation_integration_test.rs
git commit -m "test: add MVCC snapshot isolation integration tests"
```

---

### Task 7: 清理和验证

**Step 1: 运行所有 transaction 相关测试**

```bash
cargo test -p sqlrustgo-transaction -- --nocapture
```

**Step 2: 运行 clippy 检查**

```bash
cargo clippy -p sqlrustgo-transaction -- -D warnings
```

**Step 3: 格式化**

```bash
cargo fmt -p sqlrustgo-transaction
```

**Step 4: 提交所有更改**

```bash
git add -A
git commit -m "feat(transaction): complete MVCC snapshot isolation implementation"
```

---

## 验收标准

### Phase 1 验收检查清单

- [ ] `RowVersion` 包含 `created_commit_ts` 和 `deleted_commit_ts`
- [ ] `RowVersion::is_visible()` 实现正确
- [ ] `VersionChainMap::find_visible()` newest-first 查找
- [ ] `VersionChainMap::commit_versions()` 批量 stamp
- [ ] `VersionChainMap::rollback_versions()` 丢弃未提交版本
- [ ] `MVCCStorage` trait 定义完整
- [ ] `MVCCStorageEngine` 实现 trait
- [ ] `TransactionManager::mvcc_read/write/commit/rollback` 集成
- [ ] `test_snapshot_isolation_read_consistency` 通过
- [ ] `test_no_dirty_read` 通过
- [ ] `test_rollback_discard` 通过
- [ ] 所有 cargo test 通过
- [ ] 所有 clippy 检查通过

---

## 相关文件

### 需要创建的测试文件

- `crates/transaction/tests/version_chain_test.rs`
- `crates/transaction/tests/mvcc_storage_test.rs`
- `tests/anomaly/mvcc_snapshot_isolation_integration_test.rs`

### 需要修改的源文件

- `crates/transaction/src/mvcc.rs`
- `crates/transaction/src/lib.rs`
- `crates/transaction/src/manager.rs`

### 需要创建的新文件

- `crates/transaction/src/version_chain.rs`
- `crates/transaction/src/mvcc_storage.rs`

---

**Plan 完成时间**: 2026-04-15  
**预计实现时间**: 1-2 周 (Phase 1)
