# WAL 性能优化实现计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 将 WAL 吞吐量从 13 MB/s 提升到 100-200 MB/s

**Architecture:** 
- 重构 `WalWriter` 使用 `Arc<Mutex<File>>` 复用文件句柄
- 添加预分配 Buffer 和 `serialized_size()` 方法
- 新增批量写入 API

**Tech Stack:** Rust, tempfile (测试), std::sync

---

## 任务 1: 重构 WalWriter 结构体

**Files:**
- Modify: `crates/storage/src/wal.rs:199-235`

**Step 1: 添加新字段和导入**

修改 `WalWriter` 结构体添加文件句柄复用：

```rust
use std::sync::{Arc, Mutex};

pub struct WalWriter {
    file: Arc<Mutex<File>>,
    writer: BufWriter<File>,
    lsn: u64,
}
```

**Step 2: 修改 `new()` 方法**

```rust
pub fn new(path: &PathBuf) -> std::io::Result<Self> {
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    let file = Arc::new(Mutex::new(file));
    let writer = BufWriter::new(file.lock().unwrap().try_clone()?);
    
    Ok(Self { file, writer, lsn: 0 })
}
```

**Step 3: 运行测试验证**

```bash
cargo test -p sqlrustgo-storage wal::tests::test_wal_write_read -- --nocapture
```

---

## 任务 2: 添加 serialized_size 方法

**Files:**
- Modify: `crates/storage/src/wal.rs:70-109`

**Step 1: 添加 `serialized_size()` 方法到 `WalEntry`**

在 `impl WalEntry` 中添加：

```rust
pub fn serialized_size(&self) -> usize {
    // 4 (length prefix) + 25 (fixed header) + key + data
    4 + 25 + self.key.as_ref().map(|k| k.len()).unwrap_or(0) 
        + self.data.as_ref().map(|d| d.len()).unwrap_or(0)
}

fn fixed_header_size() -> usize {
    8 + 8 + 8 + 1 + 8 + 4 + 4  // lsn + timestamp + tx_id + entry_type + table_id + key_len + data_len
}
```

**Step 2: 添加测试**

```rust
#[test]
fn test_wal_entry_serialized_size() {
    let entry = WalEntry {
        tx_id: 1,
        entry_type: WalEntryType::Insert,
        table_id: 100,
        key: Some(vec![1, 2, 3, 4]),
        data: Some(vec![10, 20, 30]),
        lsn: 0,
        timestamp: 1234567890,
    };
    
    let bytes = entry.to_bytes();
    assert_eq!(entry.serialized_size(), bytes.len() + 4);
}
```

**Step 3: 运行测试**

```bash
cargo test -p sqlrustgo-storage wal::tests::test_wal_entry_serialized_size -- --nocapture
```

---

## 任务 3: 重构 WalManager 持有模式

**Files:**
- Modify: `crates/storage/src/wal.rs:291-470`

**Step 1: 修改 WalManager 结构体**

```rust
pub struct WalManager {
    wal_path: PathBuf,
    writer: Option<Arc<Mutex<WalWriter>>>,
}

impl WalManager {
    pub fn new(wal_path: PathBuf) -> Self {
        Self { wal_path, writer: None }
    }
    
    fn get_writer(&self) -> std::io::Result<Arc<Mutex<WalWriter>>> {
        if let Some(ref writer) = self.writer {
            return Ok(writer.clone());
        }
        let writer = WalWriter::new(&self.wal_path)?;
        let writer = Arc::new(Mutex::new(writer));
        Ok(writer)
    }
}
```

**Step 2: 修改所有 log_* 方法使用共享 writer**

```rust
pub fn log_insert(&self, tx_id: u64, table_id: u64, key: Vec<u8>, data: Vec<u8>) -> std::io::Result<u64> {
    let writer = self.get_writer()?;
    let entry = WalEntry {
        tx_id,
        entry_type: WalEntryType::Insert,
        table_id,
        key: Some(key),
        data: Some(data),
        lsn: writer.lock().unwrap().current_lsn(),
        timestamp: current_timestamp(),
    };
    
    writer.lock().unwrap().append(&entry)
}
```

**Step 3: 运行测试**

```bash
cargo test -p sqlrustgo-storage wal::tests::test_wal_manager -- --nocapture
```

---

## 任务 4: 添加批量写入 API

**Files:**
- Modify: `crates/storage/src/wal.rs:470-520` (在 impl WalManager 中添加)

**Step 1: 添加 batch_insert 方法**

```rust
pub fn batch_insert(&self, entries: Vec<(u64, u64, Vec<u8>, Vec<u8>)>) -> std::io::Result<Vec<u64>> {
    let writer = self.get_writer()?;
    let mut guard = writer.lock().unwrap();
    let mut lsns = Vec::with_capacity(entries.len());
    
    for (tx_id, table_id, key, data) in entries {
        let entry = WalEntry {
            tx_id,
            entry_type: WalEntryType::Insert,
            table_id,
            key: Some(key),
            data: Some(data),
            lsn: guard.current_lsn(),
            timestamp: current_timestamp(),
        };
        let lsn = guard.append(&entry)?;
        lsns.push(lsn);
    }
    
    Ok(lsns)
}
```

**Step 2: 添加批量测试**

```rust
#[test]
fn test_wal_batch_insert() {
    let dir = tempfile::tempdir().unwrap();
    let wal_path = dir.path().join("test_batch.wal");
    
    let manager = WalManager::new(wal_path);
    let entries: Vec<_> = (0..1000)
        .map(|i| (1, 1, i.to_le_bytes().to_vec(), vec![0u8; 512]))
        .collect();
    
    let start = std::time::Instant::now();
    let lsns = manager.batch_insert(entries).unwrap();
    manager.log_commit(1).unwrap();
    let elapsed = start.elapsed();
    
    assert_eq!(lsns.len(), 1000);
    println!("Batch insert 1000 entries: {:?}", elapsed);
}
```

**Step 3: 运行测试**

```bash
cargo test -p sqlrustgo-storage wal::tests::test_wal_batch_insert -- --nocapture
```

---

## 任务 5: 优化 to_bytes 使用预分配

**Files:**
- Modify: `crates/storage/src/wal.rs:72-109`

**Step 1: 优化 to_bytes 使用 capacity**

```rust
pub fn to_bytes(&self) -> Vec<u8> {
    let capacity = self.serialized_size();
    let mut bytes = Vec::with_capacity(capacity);
    
    bytes.extend_from_slice(&self.lsn.to_le_bytes());
    bytes.extend_from_slice(&self.timestamp.to_le_bytes());
    bytes.extend_from_slice(&self.tx_id.to_le_bytes());
    bytes.push(self.entry_type as u8);
    bytes.extend_from_slice(&self.table_id.to_le_bytes());
    
    match &self.key {
        Some(k) => {
            bytes.extend_from_slice(&(k.len() as u32).to_le_bytes());
            bytes.extend_from_slice(k);
        }
        None => bytes.extend_from_slice(&0u32.to_le_bytes()),
    }
    
    match &self.data {
        Some(d) => {
            bytes.extend_from_slice(&(d.len() as u32).to_le_bytes());
            bytes.extend_from_slice(d);
        }
        None => bytes.extend_from_slice(&0u32.to_le_bytes()),
    }
    
    bytes
}
```

**Step 2: 运行所有测试**

```bash
cargo test -p sqlrustgo-storage wal -- --nocapture
```

---

## 任务 6: 更新性能测试目标

**Files:**
- Modify: `crates/storage/src/wal.rs:1207-1251`

**Step 1: 更新 test_wal_perf_throughput 阈值**

```rust
#[test]
fn test_wal_perf_throughput() {
    // ... existing code ...
    
    println!("WAL Throughput: {:.2} MB/s (target: >= 100 MB/s)", throughput_mbps);
    assert!(
        throughput_mbps >= 100.0,  // 从 5.0 更新到 100.0
        "WAL throughput too low: {:.2} MB/s",
        throughput_mbps
    );
}
```

**Step 2: 添加批量性能测试**

```rust
#[test]
fn test_wal_perf_batch_throughput() {
    let dir = tempfile::tempdir().unwrap();
    let wal_path = dir.path().join("bench_batch.wal");
    
    let manager = WalManager::new(wal_path);
    
    let start = std::time::Instant::now();
    
    // 10000 条记录
    let entries: Vec<_> = (0..10000)
        .map(|i| (1, 1, i.to_le_bytes().to_vec(), vec![0u8; 512]))
        .collect();
    
    manager.batch_insert(entries).unwrap();
    manager.log_commit(1).unwrap();
    
    let elapsed = start.elapsed();
    let total_bytes = 10000 * 516u64;
    let throughput_mbps = (total_bytes as f64 / 1_000_000.0) / elapsed.as_secs_f64();
    
    println!("WAL Batch Throughput: {:.2} MB/s", throughput_mbps);
    assert!(throughput_mbps >= 100.0, "Batch throughput too low");
}
```

**Step 3: 运行性能测试**

```bash
cargo test -p sqlrustgo-storage wal::tests::test_wal_perf -- --nocapture --ignored
```

---

## 任务 7: 完整回归测试

**Step 1: 运行所有存储测试**

```bash
cargo test -p sqlrustgo-storage -- --nocapture
```

**Step 2: 运行完整测试套件**

```bash
cargo test --all 2>&1 | grep -E "(test result|FAILED)"
```

**Step 3: 检查覆盖率**

```bash
cargo tarpaulin --out Html 2>&1 | grep -E "(storage/wal|coverage)"
```

---

## 任务 8: 提交代码

**Step 1: 提交更改**

```bash
git add -A && git commit -m "perf(wal): implement batched writes with file handle reuse

- Add Arc<Mutex<File>> to WalWriter for file handle reuse
- Add serialized_size() method for pre-allocation
- Add batch_insert() API for bulk writes
- WAL throughput target: >= 100 MB/s"
```

**Step 2: 验证提交**

```bash
git log -1 --stat
```

---

## 预期结果

| 测试 | 优化前 | 优化后 |
|------|--------|--------|
| test_wal_perf_throughput | 13 MB/s | >= 100 MB/s |
| test_wal_perf_batch_throughput | N/A | >= 150 MB/s |
| test_wal_perf_1000_insert | ~80ms | < 10ms |
| 所有功能测试 | 通过 | 通过 |

---

**Plan complete!** 

两个执行选项:

**1. Subagent-Driven (本会话)** - 我为每个任务派遣 subagent，任务间审查，快速迭代

**2. Parallel Session (新会话)** - 在新会话中使用 executing-plans，批量执行带检查点

选择哪个方式?
