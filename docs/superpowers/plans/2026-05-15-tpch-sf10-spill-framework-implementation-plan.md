# TPC-H SF=10 Spill Framework Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现 TPC-H SF=10 无 OOM 溢出管理框架，确保 22/22 查询通过

**Architecture:** 设计统一的 SpillingIterator trait，为 Sort/HashJoin/Aggregate 实现专用 spill 逻辑，采用自适应分区策略，支持降级机制

**Tech Stack:** Rust (Edition 2021), tokio async runtime, mmap for spill files

---

## File Structure

```
crates/spill/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── trait.rs           # SpillingIterator trait
│   ├── memory_tracker.rs  # AdaptiveMemoryTracker
│   ├── partition_manager.rs
│   ├── fallback_manager.rs
│   ├── operators/
│   │   ├── mod.rs
│   │   ├── sort_spill.rs
│   │   ├── hash_join_spill.rs
│   │   └── aggregate_spill.rs
│   └── error.rs
└── tests/
    └── spill_test.rs

crates/executor/src/
├── session_config.rs  # 修改：添加 spill 配置
├── spill_integration.rs  # 新增：spill 集成
```

---

## Task 1: 创建 spill crate 基础结构

**Files:**
- Create: `crates/spill/Cargo.toml`
- Create: `crates/spill/src/lib.rs`
- Create: `crates/spill/src/trait.rs`
- Create: `crates/spill/src/error.rs`

- [ ] **Step 1: 创建 Cargo.toml**

```toml
[package]
name = "sqlrustgo-spill"
version = "0.1.0"
edition = "2021"

[dependencies]
thiserror = "1.0"
tracing = "0.1"
parking_lot = "0.12"
tempfile = "3.8"
```

- [ ] **Step 2: 创建 lib.rs**

```rust
pub mod error;
pub mod trait;
pub mod memory_tracker;
pub mod partition_manager;
pub mod fallback_manager;
pub mod operators;

pub use error::{SpillError, SpillResult};
pub use trait::SpillingIterator;
pub use memory_tracker::AdaptiveMemoryTracker;
pub use partition_manager::PartitionManager;
pub use fallback_manager::FallbackManager;
```

- [ ] **Step 3: 创建 error.rs**

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SpillError {
    #[error("磁盘空间不足: available={available}, required={required}")]
    OutOfDiskSpace { available: u64, required: u64 },

    #[error("IO错误: {0}")]
    IoError(#[from] std::io::Error),

    #[error("内存限制达到，无法降级: {0}")]
    FallbackFailed(String),

    #[error("分区错误: {0}")]
    PartitionError(String),
}

pub type SpillResult<T> = Result<T, SpillError>;
```

- [ ] **Step 4: 创建 trait.rs**

```rust
use crate::error::SpillError;

/// 统一溢出迭代器抽象
pub trait SpillingIterator: Iterator {
    /// 开始溢出到磁盘
    fn start_spill(&mut self) -> SpillResult<()>;

    /// 检查是否正在溢出
    fn is_spilling(&self) -> bool;

    /// 获取当前分区数
    fn num_partitions(&self) -> usize;

    /// 标记完成并清理临时文件
    fn finish_spill(&mut self);
}

/// 溢出统计信息
#[derive(Debug, Clone, Default)]
pub struct SpillStats {
    pub spill_count: usize,
    pub bytes_spilled: u64,
    pub partitions_created: usize,
    pub fallback_attempts: usize,
}
```

- [ ] **Step 5: Commit**

```bash
git add crates/spill/
git commit -m "feat(spill): create spill crate base structure"
```

---

## Task 2: 实现 AdaptiveMemoryTracker

**Files:**
- Create: `crates/spill/src/memory_tracker.rs`

- [ ] **Step 1: 创建 memory_tracker.rs**

```rust
use parking_lot::atomic::AtomicU64;
use std::sync::atomic::{AtomicBool, Ordering};

pub struct AdaptiveMemoryTracker {
    current_bytes: AtomicU64,
    spill_threshold: usize,
    memory_limit: usize,
    fallback_mode: AtomicBool,
}

impl AdaptiveMemoryTracker {
    pub fn new(memory_limit: usize, spill_threshold: usize) -> Self {
        Self {
            current_bytes: AtomicU64::new(0),
            spill_threshold,
            memory_limit,
            fallback_mode: AtomicBool::new(false),
        }
    }

    pub fn allocate(&self, bytes: usize) -> bool {
        let prev = self.current_bytes.fetch_add(bytes as u64, Ordering::SeqCst);
        let new = prev + bytes as u64;
        new <= self.memory_limit as u64
    }

    pub fn deallocate(&self, bytes: usize) {
        self.current_bytes.fetch_sub(bytes as u64, Ordering::SeqCst);
    }

    pub fn should_spill(&self) -> bool {
        self.current_bytes.load(Ordering::SeqCst) >= self.spill_threshold as u64
    }

    pub fn is_memory_exceeded(&self) -> bool {
        self.current_bytes.load(Ordering::SeqCst) > self.memory_limit as u64
    }

    pub fn enable_fallback(&self) {
        self.fallback_mode.store(true, Ordering::SeqCst);
    }

    pub fn is_fallback_enabled(&self) -> bool {
        self.fallback_mode.load(Ordering::SeqCst)
    }

    pub fn current_usage(&self) -> usize {
        self.current_bytes.load(Ordering::SeqCst) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_tracker_allocate() {
        let tracker = AdaptiveMemoryTracker::new(1024, 512);
        assert!(tracker.allocate(100));
        assert_eq!(tracker.current_usage(), 100);
    }

    #[test]
    fn test_memory_tracker_should_spill() {
        let tracker = AdaptiveMemoryTracker::new(1024, 100);
        tracker.allocate(50);
        assert!(!tracker.should_spill());
        tracker.allocate(60);
        assert!(tracker.should_spill());
    }

    #[test]
    fn test_memory_tracker_deallocate() {
        let tracker = AdaptiveMemoryTracker::new(1024, 512);
        tracker.allocate(100);
        tracker.deallocate(50);
        assert_eq!(tracker.current_usage(), 50);
    }
}
```

- [ ] **Step 2: 运行测试**

Run: `cd crates/spill && cargo test --lib`
Expected: All tests pass

- [ ] **Step 3: Commit**

```bash
git add crates/spill/src/memory_tracker.rs
git commit -m "feat(spill): implement AdaptiveMemoryTracker"
```

---

## Task 3: 实现 PartitionManager

**Files:**
- Create: `crates/spill/src/partition_manager.rs`

- [ ] **Step 1: 创建 partition_manager.rs**

```rust
use crate::error::{SpillError, SpillResult};
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use tempfile::TempDir;

pub struct PartitionManager {
    spill_dir: TempDir,
    partitions: Vec<PartitionFile>,
}

struct PartitionFile {
    path: PathBuf,
    size_bytes: u64,
}

impl PartitionManager {
    pub fn new() -> SpillResult<Self> {
        let spill_dir = tempfile::tempdir()
            .map_err(|e| SpillError::IoError(e))?;
        Ok(Self {
            spill_dir,
            partitions: Vec::new(),
        })
    }

    pub fn with_dir(path: PathBuf) -> SpillResult<Self> {
        fs::create_dir_all(&path)
            .map_err(|e| SpillError::IoError(e))?;
        let spill_dir = TempDir::new_in(&path)
            .map_err(|e| SpillError::IoError(e))?;
        Ok(Self {
            spill_dir,
            partitions: Vec::new(),
        })
    }

    pub fn write_partition<T: serde::Serialize>(
        &mut self,
        data: &[T],
    ) -> SpillResult<usize> {
        let partition_id = self.partitions.len();
        let path = self.spill_dir.path().join(format!("partition_{}.bin", partition_id));

        let file = File::create(&path)
            .map_err(|e| SpillError::IoError(e))?;
        let mut writer = BufWriter::new(file);

        for item in data {
            let bytes = bincode::serialize(item)
                .map_err(|e| SpillError::PartitionError(e.to_string()))?;
            writer.write_all(&bytes)
                .map_err(|e| SpillError::IoError(e))?;
        }

        writer.flush()
            .map_err(|e| SpillError::IoError(e))?;

        let size_bytes = fs::metadata(&path)
            .map_err(|e| SpillError::IoError(e))?
            .len();

        let partition = PartitionFile {
            path,
            size_bytes,
        };
        self.partitions.push(partition);

        Ok(partition_id)
    }

    pub fn read_partition<T: serde::de::DeserializeOwned>(
        &self,
        partition_id: usize,
    ) -> SpillResult<Vec<T>> {
        let partition = self.partitions.get(partition_id)
            .ok_or_else(|| SpillError::PartitionError(format!("Invalid partition {}", partition_id)))?;

        let bytes = fs::read(&partition.path)
            .map_err(|e| SpillError::IoError(e))?;

        let items: Vec<T> = bincode::deserialize(&bytes)
            .map_err(|e| SpillError::PartitionError(e.to_string()))?;

        Ok(items)
    }

    pub fn num_partitions(&self) -> usize {
        self.partitions.len()
    }

    pub fn total_bytes_spilled(&self) -> u64 {
        self.partitions.iter().map(|p| p.size_bytes).sum()
    }

    pub fn cleanup(&mut self) {
        self.partitions.clear();
    }
}

impl Drop for PartitionManager {
    fn drop(&mut self) {
        let _ = self.spill_dir.close();
    }
}
```

- [ ] **Step 2: 添加 bincode 依赖到 Cargo.toml**

```toml
bincode = "1.3"
serde = { version = "1.0", features = ["derive"] }
```

- [ ] **Step 3: 运行测试**

Run: `cd crates/spill && cargo test --lib`
Expected: All tests pass

- [ ] **Step 4: Commit**

```bash
git add crates/spill/src/partition_manager.rs crates/spill/Cargo.toml
git commit -m "feat(spill): implement PartitionManager"
```

---

## Task 4: 实现 FallbackManager

**Files:**
- Create: `crates/spill/src/fallback_manager.rs`

- [ ] **Step 1: 创建 fallback_manager.rs**

```rust
use crate::error::SpillError;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct FallbackManager {
    attempts: AtomicUsize,
    max_attempts: usize,
    original_memory_limit: usize,
}

impl FallbackManager {
    pub fn new(max_attempts: usize, original_memory_limit: usize) -> Self {
        Self {
            attempts: AtomicUsize::new(0),
            max_attempts,
            original_memory_limit,
        }
    }

    pub fn can_fallback(&self) -> bool {
        self.attempts.load(Ordering::SeqCst) < self.max_attempts
    }

    pub fn try_fallback(&self) -> SpillResult<()> {
        if !self.can_fallback() {
            return Err(SpillError::FallbackFailed(
                "已达最大降级尝试次数".into(),
            ));
        }
        self.attempts.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    pub fn attempt_count(&self) -> usize {
        self.attempts.load(Ordering::SeqCst)
    }

    pub fn reset(&self) {
        self.attempts.store(0, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fallback_manager() {
        let manager = FallbackManager::new(3, 1024);
        assert!(manager.can_fallback());

        manager.try_fallback().unwrap();
        assert_eq!(manager.attempt_count(), 1);

        manager.try_fallback().unwrap();
        manager.try_fallback().unwrap();
        assert!(!manager.can_fallback());
    }
}
```

- [ ] **Step 2: 运行测试**

Run: `cd crates/spill && cargo test --lib`
Expected: All tests pass

- [ ] **Step 3: Commit**

```bash
git add crates/spill/src/fallback_manager.rs
git commit -m "feat(spill): implement FallbackManager"
```

---

## Task 5: 实现 SortSpillOperator

**Files:**
- Create: `crates/spill/src/operators/mod.rs`
- Create: `crates/spill/src/operators/sort_spill.rs`

- [ ] **Step 1: 创建 operators/mod.rs**

```rust
pub mod sort_spill;

pub use sort_spill::SortSpillOperator;
```

- [ ] **Step 2: 创建 sort_spill.rs**

```rust
use crate::error::SpillResult;
use crate::memory_tracker::AdaptiveMemoryTracker;
use crate::partition_manager::PartitionManager;
use crate::trait::SpillingIterator;
use std::cmp::Ordering;
use std::sync::Arc;

pub struct SortSpillOperator<T: Clone> {
    tracker: Arc<AdaptiveMemoryTracker>,
    partition_manager: PartitionManager,
    current_partition: Vec<T>,
    spilled_runs: Vec<usize>,
    comparator: fn(&T, &T) -> Ordering,
}

impl<T: Clone + serde::Serialize + serde::de::DeserializeOwned> SortSpillOperator<T> {
    pub fn new(tracker: Arc<AdaptiveMemoryTracker>, comparator: fn(&T, &T) -> Ordering) -> SpillResult<Self> {
        Ok(Self {
            tracker,
            partition_manager: PartitionManager::new()?,
            current_partition: Vec::new(),
            spilled_runs: Vec::new(),
            comparator,
        })
    }

    pub fn add(&mut self, item: T) -> SpillResult<()> {
        if self.tracker.should_spill() {
            self.start_spill()?;
        }

        let item_size = std::mem::size_of::<T>();
        if !self.tracker.allocate(item_size) {
            return self.start_spill().and_then(|_| {
                if !self.tracker.allocate(item_size) {
                    Err(crate::error::SpillError::OutOfDiskSpace {
                        available: 0,
                        required: item_size as u64,
                    })
                } else {
                    Ok(())
                }
            });
        }

        self.current_partition.push(item);
        Ok(())
    }

    pub fn calculate_partition_size(element_size: usize, available_memory: usize) -> usize {
        let rows_per_partition = available_memory / element_size;
        (rows_per_partition * 9) / 10
    }
}

impl<T: Clone + serde::Serialize + serde::de::DeserializeOwned> SpillingIterator for SortSpillOperator<T> {
    fn start_spill(&mut self) -> SpillResult<()> {
        if self.current_partition.is_empty() {
            return Ok(());
        }

        self.current_partition.sort_by(self.comparator);
        let partition_id = self.partition_manager.write_partition(&self.current_partition)?;
        self.spilled_runs.push(partition_id);
        self.current_partition.clear();

        for item in self.spilled_runs.iter() {
            let data = self.partition_manager.read_partition::<T>(*item)?;
            for elem in data {
                let size = std::mem::size_of::<T>();
                self.tracker.deallocate(size);
            }
        }

        Ok(())
    }

    fn is_spilling(&self) -> bool {
        !self.spilled_runs.is_empty()
    }

    fn num_partitions(&self) -> usize {
        self.spilled_runs.len()
    }

    fn finish_spill(&mut self) {
        self.current_partition.clear();
        self.spilled_runs.clear();
        self.partition_manager.cleanup();
    }
}
```

- [ ] **Step 3: 运行测试**

Run: `cd crates/spill && cargo test --lib`
Expected: All tests pass

- [ ] **Step 4: Commit**

```bash
git add crates/spill/src/operators/
git commit -m "feat(spill): implement SortSpillOperator"
```

---

## Task 6: 实现 HashJoinSpillOperator

**Files:**
- Create: `crates/spill/src/operators/hash_join_spill.rs`

- [ ] **Step 1: 创建 hash_join_spill.rs**

```rust
use crate::error::{SpillError, SpillResult};
use crate::memory_tracker::AdaptiveMemoryTracker;
use crate::partition_manager::PartitionManager;
use crate::trait::SpillingIterator;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;

pub struct HashJoinSpillOperator<K: Hash + Eq + Clone, V: Clone> {
    tracker: Arc<AdaptiveMemoryTracker>,
    partition_manager: PartitionManager,
    build_hash: HashMap<K, Vec<V>>,
    probe_buffer: Vec<(K, V)>,
    comparator: fn(&K, &K) -> bool,
}

impl<K: Hash + Eq + Clone + serde::Serialize + serde::de::DeserializeOwned,
     V: Clone + serde::Serialize + serde::de::DeserializeOwned>
    HashJoinSpillOperator<K, V>
{
    pub fn new(tracker: Arc<AdaptiveMemoryTracker>, comparator: fn(&K, &K) -> bool) -> SpillResult<Self> {
        Ok(Self {
            tracker,
            partition_manager: PartitionManager::new()?,
            build_hash: HashMap::new(),
            probe_buffer: Vec::new(),
            comparator,
        })
    }

    pub fn add_build(&mut self, key: K, value: V) -> SpillResult<()> {
        if self.tracker.should_spill() {
            self.spill_build_side()?;
        }

        let entry = self.build_hash.entry(key).or_insert_with(Vec::new);
        entry.push(value);

        let size = std::mem::size_of::<K>() + std::mem::size_of::<V>();
        let _ = self.tracker.allocate(size);
        Ok(())
    }

    pub fn add_probe(&mut self, key: K, value: V) {
        self.probe_buffer.push((key, value));
    }

    fn spill_build_side(&mut self) -> SpillResult<()> {
        let partition_id = self.partition_manager.write_partition(&self.build_hash)?;
        self.build_hash.clear();
        Ok(())
    }

    pub fn build_hash_map(&mut self) -> &HashMap<K, Vec<V>> {
        &self.build_hash
    }

    pub fn get_probe_buffer(&self) -> &[(K, V)] {
        &self.probe_buffer
    }
}

impl<K: Hash + Eq + Clone + serde::Serialize + serde::de::DeserializeOwned,
     V: Clone + serde::Serialize + serde::de::DeserializeOwned>
    SpillingIterator for HashJoinSpillOperator<K, V>
{
    fn start_spill(&mut self) -> SpillResult<()> {
        self.spill_build_side()
    }

    fn is_spilling(&self) -> bool {
        self.partition_manager.num_partitions() > 0
    }

    fn num_partitions(&self) -> usize {
        self.partition_manager.num_partitions()
    }

    fn finish_spill(&mut self) {
        self.build_hash.clear();
        self.probe_buffer.clear();
        self.partition_manager.cleanup();
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add crates/spill/src/operators/hash_join_spill.rs
git commit -m "feat(spill): implement HashJoinSpillOperator"
```

---

## Task 7: 实现 AggregateSpillOperator

**Files:**
- Create: `crates/spill/src/operators/aggregate_spill.rs`

- [ ] **Step 1: 创建 aggregate_spill.rs**

```rust
use crate::error::SpillResult;
use crate::memory_tracker::AdaptiveMemoryTracker;
use crate::partition_manager::PartitionManager;
use crate::trait::SpillingIterator;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct AggregatedState {
    pub count: usize,
    pub sum: f64,
}

pub struct AggregateSpillOperator<G: Clone + std::hash::Hash + Eq> {
    tracker: Arc<AdaptiveMemoryTracker>,
    partition_manager: PartitionManager,
    groups: HashMap<G, AggregatedState>,
    spilled_groups: Vec<(G, AggregatedState)>,
}

impl<G: Clone + std::hash::Hash + Eq + serde::Serialize + serde::de::DeserializeOwned>
    AggregateSpillOperator<G>
{
    pub fn new(tracker: Arc<AdaptiveMemoryTracker>) -> SpillResult<Self> {
        Ok(Self {
            tracker,
            partition_manager: PartitionManager::new()?,
            groups: HashMap::new(),
            spilled_groups: Vec::new(),
        })
    }

    pub fn aggregate(&mut self, group_key: G, value: f64) -> SpillResult<()> {
        if self.tracker.should_spill() {
            self.spill_groups()?;
        }

        let state = self.groups.entry(group_key).or_insert(AggregatedState {
            count: 0,
            sum: 0.0,
        });
        state.count += 1;
        state.sum += value;

        let _ = self.tracker.allocate(std::mem::size_of::<G>());
        Ok(())
    }

    fn spill_groups(&mut self) -> SpillResult<()> {
        for (key, state) in self.groups.drain() {
            self.spilled_groups.push((key, state));
        }
        let partition_id = self.partition_manager.write_partition(&self.spilled_groups)?;
        self.spilled_groups.clear();
        Ok(())
    }

    pub fn get_groups(&self) -> &HashMap<G, AggregatedState> {
        &self.groups
    }
}

impl<G: Clone + std::hash::Hash + Eq + serde::Serialize + serde::de::DeserializeOwned>
    SpillingIterator for AggregateSpillOperator<G>
{
    fn start_spill(&mut self) -> SpillResult<()> {
        self.spill_groups()
    }

    fn is_spilling(&self) -> bool {
        self.partition_manager.num_partitions() > 0
    }

    fn num_partitions(&self) -> usize {
        self.partition_manager.num_partitions()
    }

    fn finish_spill(&mut self) {
        self.groups.clear();
        self.spilled_groups.clear();
        self.partition_manager.cleanup();
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add crates/spill/src/operators/aggregate_spill.rs
git commit -m "feat(spill): implement AggregateSpillOperator"
```

---

## Task 8: 集成 SessionConfig

**Files:**
- Modify: `crates/executor/src/session_config.rs`

- [ ] **Step 1: 添加 spill 配置到 SessionConfig**

```rust
impl SessionConfig {
    pub fn with_spill_config(
        mut self,
        max_memory: usize,
        spill_threshold: usize,
        spill_dir: String,
    ) -> Self {
        self.max_memory_per_query = max_memory;
        self.spill_to_disk_threshold = spill_threshold;
        self.spill_dir = Some(spill_dir);
        self
    }
}
```

- [ ] **Step 2: 添加 spill_dir 字段**

```rust
pub struct SessionConfig {
    // ... existing fields ...
    pub spill_dir: Option<String>,
}
```

- [ ] **Step 3: Commit**

```bash
git add crates/executor/src/session_config.rs
git commit -m "feat(executor): add spill config to SessionConfig"
```

---

## Task 9: TPC-H Q1 PoC 测试

**Files:**
- Modify: `tests/tpch_gate_test.rs`

- [ ] **Step 1: 添加 SF=1 测试用例**

```rust
#[test]
#[ignore]
fn test_tpch_sf10_q1_no_oom() {
    let sf = 10.0;
    let dir = data_dir();
    let timeout = Duration::from_secs(600);

    // 使用 64GB 内存限制
    let storage = Arc::new(RwLock::new(MemoryStorage::new()));
    let mut engine = ExecutionEngine::new_with_config(
        storage,
        EngineConfig::default()
            .with_max_memory(64 * 1024 * 1024 * 1024)
            .with_spill_threshold(4 * 1024 * 1024 * 1024),
    );

    // ... 执行 Q1 ...
}
```

- [ ] **Step 2: 运行 PoC 测试**

Run: `TPCH_SF=1 TPCH_TIMEOUT_S=300 cargo test --test tpch_gate_test test_tpch_sf01_gate -- --nocapture`
Expected: Q1 通过

- [ ] **Step 3: Commit**

```bash
git add tests/tpch_gate_test.rs
git commit -m "test(tpch): add SF=10 Q1 spill PoC test"
```

---

## Task 10: TPC-H SF=1 完整测试

**Files:**
- Modify: `tests/tpch_gate_test.rs`

- [ ] **Step 1: 运行 SF=1 全部 22 个查询**

Run: `TPCH_SF=1 TPCH_TIMEOUT_S=300 cargo test --test tpch_gate_test test_tpch_sf01_gate -- --nocapture`
Expected: 22/22 通过

- [ ] **Step 2: 验证无 OOM**

- [ ] **Step 3: Commit**

```bash
git commit -m "test(tpch): verify SF=1 22/22 pass"
```

---

## Task 11: TPC-H SF=10 完整测试

**Files:**
- Modify: `tests/tpch_gate_test.rs`

- [ ] **Step 1: 运行 SF=10 全部 22 个查询**

Run: `TPCH_SF=10 TPCH_TIMEOUT_S=600 TPCH_DATA_DIR=/path/to/sf10/data cargo test --test tpch_gate_test test_tpch_sf01_gate -- --nocapture`
Expected: 22/22 通过，无 OOM

- [ ] **Step 2: 记录性能指标**

- [ ] **Step 3: Commit**

```bash
git commit -m "test(tpch): verify SF=10 22/22 pass without OOM"
```

---

## 验证检查清单

After each task, verify:

- [ ] `cargo check --all-features` passes
- [ ] `cargo clippy --all-features -- -D warnings` passes
- [ ] `cargo fmt --check --all` passes
- [ ] `cargo test --lib` in spill crate passes
- [ ] No placeholder comments in committed code

---

## 风险缓解

| 风险 | 缓解 |
|------|------|
| 溢出性能退化 | 使用 mmap 优化 IO，后续任务 |
| 分区不均衡 | 自适应分区策略 |
| 降级失败 | 保守阈值 + 预检查 |
