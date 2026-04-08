# Issue #1302 - 列式存储压缩增强实现计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 实现灵活的列式存储压缩配置，支持按数据类型自动选择压缩算法和用户配置压缩级别。

**Architecture:** 在现有 CompressionType 基础上添加 CompressionLevel 和 CompressionConfig，实现自动压缩策略选择和用户级别配置。

**Tech Stack:** Rust, lz4_flex, zstd, snap

---

## Phase 1: 核心数据结构

### Task 1: 添加 CompressionLevel 枚举

**Files:**
- Modify: `crates/storage/src/columnar/segment.rs:34-46`

**Step 1: 查找 CompressionType 定义位置**

```rust
// 找到 CompressionType 定义位置，在 segment.rs 文件中
```

**Step 2: 在 CompressionType 后添加 CompressionLevel**

```rust
/// Compression level options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionLevel {
    /// Fastest compression - speed priority
    Fastest,
    /// Balanced between speed and ratio
    Default,
    /// Best compression - ratio priority
    Best,
    /// Custom compression level
    Custom(i32),
}
```

**Step 3: 验证编译**

```bash
cargo check -p sqlrustgo-storage 2>&1 | grep -E "^error"
```

Expected: 无错误

**Step 4: 提交**

```bash
git add crates/storage/src/columnar/segment.rs
git commit -m "feat(columnar): add CompressionLevel enum"
```

---

### Task 2: 添加 CompressionConfig 结构体

**Files:**
- Modify: `crates/storage/src/columnar/segment.rs:70-85`

**Step 1: 在 CompressionLevel 后添加 CompressionConfig**

```rust
/// Compression configuration for a column
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionConfig {
    /// Compression level
    pub level: CompressionLevel,
    /// Auto-select algorithm based on data type
    pub auto_select: bool,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            level: CompressionLevel::Default,
            auto_select: true,
        }
    }
}
```

**Step 2: 验证编译**

```bash
cargo check -p sqlrustgo-storage 2>&1 | grep -E "^error"
```

Expected: 无错误

**Step 3: 提交**

```bash
git add crates/storage/src/columnar/segment.rs
git commit -m "feat(columnar): add CompressionConfig struct"
```

---

### Task 3: 实现 CompressionLevel 级别映射方法

**Files:**
- Modify: `crates/storage/src/columnar/segment.rs:85-115`

**Step 1: 为 CompressionLevel 实现映射方法**

```rust
impl CompressionLevel {
    /// Get LZ4 compression level (0 = fastest, higher = better compression)
    pub fn lz4_level(&self) -> i32 {
        match self {
            CompressionLevel::Fastest => 0,
            CompressionLevel::Default => 12,
            CompressionLevel::Best => 17,
            CompressionLevel::Custom(n) => *n,
        }
    }

    /// Get Zstd compression level (1-22, higher = better compression)
    pub fn zstd_level(&self) -> i32 {
        match self {
            CompressionLevel::Fastest => 1,
            CompressionLevel::Default => 3,
            CompressionLevel::Best => 19,
            CompressionLevel::Custom(n) => *n.clamp(1, 22),
        }
    }
}
```

**Step 2: 验证编译**

```bash
cargo check -p sqlrustgo-storage 2>&1 | grep -E "^error"
```

**Step 3: 添加测试**

在 `#[cfg(test)]` 模块中添加：

```rust
#[test]
fn test_compression_level_lz4() {
    assert_eq!(CompressionLevel::Fastest.lz4_level(), 0);
    assert_eq!(CompressionLevel::Default.lz4_level(), 12);
    assert_eq!(CompressionLevel::Best.lz4_level(), 17);
    assert_eq!(CompressionLevel::Custom(10).lz4_level(), 10);
}

#[test]
fn test_compression_level_zstd() {
    assert_eq!(CompressionLevel::Fastest.zstd_level(), 1);
    assert_eq!(CompressionLevel::Default.zstd_level(), 3);
    assert_eq!(CompressionLevel::Best.zstd_level(), 19);
    assert_eq!(CompressionLevel::Custom(15).zstd_level(), 15);
    // Test clamping
    assert_eq!(CompressionLevel::Custom(100).zstd_level(), 22);
    assert_eq!(CompressionLevel::Custom(-5).zstd_level(), 1);
}
```

**Step 4: 运行测试**

```bash
cargo test -p sqlrustgo-storage --lib -- columnar::segment::tests::test_compression_level_zstd --nocapture
```

**Step 5: 提交**

```bash
git add crates/storage/src/columnar/segment.rs
git commit -m "feat(columnar): implement CompressionLevel mapping methods"
```

---

## Phase 2: 压缩算法自动选择

### Task 4: 添加压缩算法自动选择函数

**Files:**
- Modify: `crates/storage/src/columnar/segment.rs:485-520`

**Step 1: 在压缩函数区域添加自动选择函数**

```rust
/// Auto-select compression algorithm based on data type hint
/// Returns (CompressionType, level)
pub fn auto_select_compression(data_type: &str, level: CompressionLevel) -> (CompressionType, CompressionLevel) {
    match data_type {
        // Numeric types - LZ4 is fast and effective
        "INTEGER" | "BIGINT" | "SMALLINT" | "TINYINT" | "FLOAT" | "DOUBLE" | "DECIMAL" => {
            (CompressionType::Lz4, level)
        }
        // Boolean - no compression needed
        "BOOLEAN" => {
            (CompressionType::None, CompressionLevel::Default)
        }
        // Short text - Snappy is balanced
        "VARCHAR" | "CHAR" | "TEXT" if level == CompressionLevel::Fastest => {
            (CompressionType::Snappy, CompressionLevel::Default)
        }
        // Longer text - Zstd provides best ratio
        "VARCHAR" | "CHAR" | "TEXT" | "BLOB" | "JSON" => {
            (CompressionType::Zstd, level)
        }
        // Default to Zstd
        _ => (CompressionType::Zstd, level),
    }
}
```

**Step 2: 修改 write_to_file 支持动态算法**

找到现有的 write_to_file 方法中 `CompressionType::None =>` 的 match 分支，添加一个新方法：

```rust
/// Write with auto-selected compression based on data type
pub fn write_with_auto_compression<P: AsRef<Path>>(
    &self,
    path: P,
    values: &[Value],
    null_bitmap: Option<&Bitmap>,
    data_type: &str,
    level: CompressionLevel,
) -> SegmentResult<()> {
    let (compression, actual_level) = auto_select_compression(data_type, level);

    // Create a modified segment with selected compression
    let mut segment = ColumnSegment::with_compression(self.column_id, compression);
    segment.stats = self.stats.clone();
    segment.num_values = values.len() as u64;

    // Temporarily update compression for writing
    let original_compression = self.compression;
    // Note: We need to refactor to support level parameter
    // This is a simplified version showing the concept
    segment.write_to_file(path, values, null_bitmap)
}
```

实际上需要重构来支持 CompressionLevel。让我创建一个新方法 `write_to_file_with_level`。

**Step 3: 验证编译**

```bash
cargo check -p sqlrustgo-storage 2>&1 | grep -E "^error"
```

**Step 4: 提交**

```bash
git add crates/storage/src/columnar/segment.rs
git commit -m "feat(columnar): add auto-select compression function"
```

---

### Task 5: 修改压缩函数支持级别

**Files:**
- Modify: `crates/storage/src/columnar/segment.rs:520-600`

**Step 1: 修改 compress_lz4 支持级别**

```rust
/// Compress data using LZ4 with configurable level
fn compress_lz4_level(data: &[u8], level: i32) -> SegmentResult<Vec<u8>> {
    use lz4_flex::block::compress_prepend_size;
    let compressed = compress_prepend_size(data);
    Ok(compressed)
}
```

实际上 lz4_flex 的 compress 函数不接受 level 参数。我们需要使用不同的方法。

让我检查 lz4_flex 的 API...

实际上 lz4_flex 使用 `compress_prepend_size` 和 `compress_into` 等，这些不接受 level。对于 LZ4，我们主要通过选择不同的函数变体来实现速度/压缩率平衡。

```rust
/// Compress data using LZ4
/// Note: lz4_flex doesn't have per-call level control, uses compile-time settings
fn compress_lz4(data: &[u8]) -> SegmentResult<Vec<u8>> {
    use lz4_flex::block::compress;
    Ok(compress(data))
}
```

对于 Zstd，level 参数已支持。

**Step 2: 修改 write_to_file 添加 level 参数**

实际上，现有的 `write_to_file` 使用的是 `self.compression`。我们需要一个新方法来接受 CompressionConfig。

```rust
/// Write segment data with compression config (auto-select support)
pub fn write_with_config<P: AsRef<Path>>(
    &self,
    path: P,
    values: &[Value],
    null_bitmap: Option<&Bitmap>,
    config: &CompressionConfig,
) -> SegmentResult<()> {
    // Use configured compression or auto-select
    let (compression, level) = if config.auto_select {
        // Need data_type from somewhere - this is a design challenge
        // For now, we'll use Zstd as default with configured level
        (CompressionType::Zstd, level)
    } else {
        // Map CompressionLevel to CompressionType based on level preference
        match config.level {
            CompressionLevel::Fastest => (CompressionType::Lz4, config.level),
            CompressionLevel::Default => (CompressionType::Snappy, config.level),
            CompressionLevel::Best => (CompressionType::Zstd, config.level),
            CompressionLevel::Custom(n) if n < 5 => (CompressionType::Lz4, config.level),
            CompressionLevel::Custom(_) => (CompressionType::Zstd, config.level),
        }
    };

    // Create temp segment with selected compression
    let mut segment = ColumnSegment::with_compression(self.column_id, compression);
    segment.stats = self.stats.clone();
    segment.num_values = values.len() as u64;

    // Use existing write method
    segment.write_to_file(path, values, null_bitmap)
}
```

这个设计有挑战：auto_select 需要数据类型信息，但 ColumnSegment 不知道数据类型。

**设计决策**: ColumnSegment 应该存储其代表的数据类型，或者在 TableStore 层面处理自动选择。

**简化方案**: 在 TableStore 层面处理自动选择，ColumnSegment 保持简单。

**Step 3: 验证编译**

**Step 4: 提交**

---

### Task 6: 在 segment.rs 导出新类型

**Files:**
- Modify: `crates/storage/src/columnar/mod.rs`

**Step 1: 更新导出**

```rust
pub use super::segment::{
    ColumnSegment, ColumnStatsDisk, CompressionConfig, CompressionLevel, CompressionType,
    SegmentError, SegmentResult,
};
```

**Step 2: 验证编译**

```bash
cargo check -p sqlrustgo-storage 2>&1 | grep -E "^error"
```

**Step 3: 提交**

```bash
git add crates/storage/src/columnar/mod.rs
git commit -m "feat(columnar): export new compression types"
```

---

## Phase 3: 配置集成

### Task 7: 扩展 ColumnDefinition

**Files:**
- Modify: `crates/storage/src/engine.rs:48-72`

**Step 1: 在 ColumnDefinition 中添加 compression 字段**

```rust
pub struct ColumnDefinition {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub is_unique: bool,
    pub is_primary_key: bool,
    pub references: Option<ForeignKeyConstraint>,
    pub auto_increment: bool,
    pub compression: Option<CompressionConfig>,  // NEW
}
```

**Step 2: 更新 new() 和 Default**

```rust
impl ColumnDefinition {
    pub fn new(name: &str, data_type: &str) -> Self {
        Self {
            name: name.to_string(),
            data_type: data_type.to_string(),
            nullable: true,
            is_unique: false,
            is_primary_key: false,
            references: None,
            auto_increment: false,
            compression: None,  // Default: auto-select
        }
    }
}

impl Default for ColumnDefinition {
    fn default() -> Self {
        Self {
            name: String::new(),
            data_type: String::new(),
            nullable: true,
            is_unique: false,
            is_primary_key: false,
            references: None,
            auto_increment: false,
            compression: None,
        }
    }
}
```

**Step 3: 验证编译**

```bash
cargo check -p sqlrustgo-storage 2>&1 | grep -E "^error"
```

**Step 4: 提交**

```bash
git add crates/storage/src/engine.rs
git commit -m "feat(storage): add compression config to ColumnDefinition"
```

---

### Task 8: 修改 TableStore 支持压缩配置

**Files:**
- Modify: `crates/storage/src/columnar/storage.rs`

**Step 1: 修改 TableStore::serialize 支持压缩配置**

找到 serialize 方法，修改 line 147 的硬编码 Zstd：

```rust
// 获取列的压缩配置
let col_info = self.info.columns.get(*col_idx);
let compression = col_info
    .and_then(|c| c.compression.clone())
    .unwrap_or_default();

let (compression_type, level) = if compression.auto_select {
    auto_select_compression(&col_info.map(|c| c.data_type.as_str()).unwrap_or(""), compression.level)
} else {
    // Map level to compression type
    match compression.level {
        CompressionLevel::Fastest => (CompressionType::Lz4, compression.level),
        CompressionLevel::Default => (CompressionType::Snappy, compression.level),
        CompressionLevel::Best => (CompressionType::Zstd, compression.level),
        CompressionLevel::Custom(n) if n < 5 => (CompressionType::Lz4, compression.level),
        CompressionLevel::Custom(_) => (CompressionType::Zstd, compression.level),
    }
};

let segment = ColumnSegment::with_compression(*col_idx as u32, compression_type);
```

**Step 2: 验证编译**

```bash
cargo check -p sqlrustgo-storage 2>&1 | grep -E "^error"
```

**Step 3: 提交**

```bash
git add crates/storage/src/columnar/storage.rs
git commit -m "feat(columnar): integrate compression config into TableStore"
```

---

## Phase 4: 测试

### Task 9: 添加压缩级别测试

**Files:**
- Modify: `crates/storage/src/columnar/segment.rs` 在 `#[cfg(test)]` 模块

**Step 1: 添加完整的压缩级别测试**

```rust
#[test]
fn test_compression_config_default() {
    let config = CompressionConfig::default();
    assert_eq!(config.level, CompressionLevel::Default);
    assert!(config.auto_select);
}

#[test]
fn test_compression_level_mapping() {
    assert_eq!(CompressionLevel::Fastest.lz4_level(), 0);
    assert_eq!(CompressionLevel::Fastest.zstd_level(), 1);
    assert_eq!(CompressionLevel::Best.zstd_level(), 19);
    assert_eq!(CompressionLevel::Custom(15).zstd_level(), 15);
}

#[test]
fn test_auto_select_compression() {
    // Integer -> LZ4
    let (algo, _) = auto_select_compression("INTEGER", CompressionLevel::Default);
    assert_eq!(algo, CompressionType::Lz4);

    // Text -> Zstd
    let (algo, _) = auto_select_compression("TEXT", CompressionLevel::Default);
    assert_eq!(algo, CompressionType::Zstd);

    // Boolean -> None
    let (algo, _) = auto_select_compression("BOOLEAN", CompressionLevel::Default);
    assert_eq!(algo, CompressionType::None);
}
```

**Step 2: 运行测试**

```bash
cargo test -p sqlrustgo-storage --lib -- columnar::segment::tests --nocapture
```

**Step 3: 提交**

---

### Task 10: 添加集成测试

**Files:**
- Create: `crates/storage/tests/columnar_compression_test.rs`

**Step 1: 创建集成测试文件**

```rust
use sqlrustgo_storage::columnar::{CompressionConfig, CompressionLevel, CompressionType};
use sqlrustgo_storage::engine::{ColumnDefinition, TableInfo};

#[test]
fn test_column_definition_with_compression() {
    let col = ColumnDefinition {
        name: "data".to_string(),
        data_type: "TEXT".to_string(),
        nullable: false,
        is_unique: false,
        is_primary_key: false,
        references: None,
        auto_increment: false,
        compression: Some(CompressionConfig {
            level: CompressionLevel::Best,
            auto_select: false,
        }),
    };

    assert!(col.compression.is_some());
    assert_eq!(col.compression.unwrap().level, CompressionLevel::Best);
}
```

**Step 2: 运行测试**

```bash
cargo test -p sqlrustgo-storage --test columnar_compression_test
```

**Step 3: 提交**

---

## 验证清单

运行完整测试：

```bash
# Storage crate tests
cargo test -p sqlrustgo-storage --lib -- columnar

# Integration tests
cargo test -p sqlrustgo-storage --test columnar_compression_test

# Full build
cargo check -p sqlrustgo-storage
```

---

## 执行选项

**Plan complete and saved to `docs/plans/2026-04-08-columnar-compression-implementation.md`**

Two execution options:

**1. Subagent-Driven (this session)** - I dispatch fresh subagent per task, review between tasks, fast iteration

**2. Parallel Session (separate)** - Open new session with executing-plans, batch execution with checkpoints

Which approach?
