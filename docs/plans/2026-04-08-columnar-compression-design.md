# Issue #1302 - 列式存储压缩增强设计

## 目标

实现灵活的列式存储压缩配置，支持：
1. **压缩配置灵活性** - 为不同列选择不同压缩算法和级别
2. **自动压缩策略** - 根据数据特征自动选择最优压缩算法

## 验收标准

- [x] LZ4 压缩支持 - 已实现
- [x] Zstd 压缩支持 - 已实现
- [x] 压缩率 > 2x - 已验证 (LZ4: 244x, Zstd: 3815x)
- [ ] 压缩配置灵活性
- [ ] 自动压缩策略

---

## 1. 核心数据结构

### 1.1 压缩级别

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

### 1.2 压缩配置

```rust
/// Compression configuration for a column
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionConfig {
    /// Compression level
    pub level: CompressionLevel,
    /// Auto-select algorithm based on data type
    /// If true, algorithm is chosen based on data type, level still applies
    /// If false, use explicitly specified settings
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

### 1.3 ColumnDefinition 扩展

在 `ColumnDefinition` 中添加可选的压缩配置：

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

---

## 2. 压缩算法自动选择策略

### 2.1 策略表

| 数据类型 | 推荐算法 | Fastest | Default | Best |
|---------|---------|---------|---------|------|
| Integer | LZ4 | fastest | default | high |
| Float | LZ4 | fastest | default | high |
| Boolean | None | - | - | - |
| Text (<100B) | Snappy | fast | default | slow |
| Text (≥100B) | Zstd | level 1 | level 3 | level 19 |
| Binary | LZ4 | fastest | default | high |

### 2.2 算法级别映射

```rust
impl CompressionLevel {
    /// Get LZ4 compression level (0 = fastest)
    pub fn lz4_level(&self) -> i32 {
        match self {
            CompressionLevel::Fastest => 0,    // lz4_flex::block::CompressionInfo::fastest()
            CompressionLevel::Default => 12,    // lz4_flex default
            CompressionLevel::Best => 17,       // lz4_flex best
            CompressionLevel::Custom(n) => *n,
        }
    }

    /// Get Zstd compression level
    pub fn zstd_level(&self) -> i32 {
        match self {
            CompressionLevel::Fastest => 1,
            CompressionLevel::Default => 3,
            CompressionLevel::Best => 19,
            CompressionLevel::Custom(n) => *n,
        }
    }
}
```

---

## 3. 组件设计

### 3.1 ColumnSegment 扩展

修改 `ColumnSegment::with_compression` 以支持压缩配置：

```rust
impl ColumnSegment {
    /// Create with compression config (auto-select algorithm based on data type)
    pub fn with_compression_config(column_id: u32, config: CompressionConfig) -> Self;

    /// Create with explicit compression type and level
    pub fn with_explicit_compression(
        column_id: u32,
        compression: CompressionType,
        level: CompressionLevel,
    ) -> Self;
}
```

### 3.2 TableStore 扩展

修改表存储以支持压缩配置：

```rust
impl TableStore {
    /// Create table store with column compression configs
    pub fn with_compression(
        info: TableInfo,
        column_configs: HashMap<usize, CompressionConfig>,
    ) -> Self;
}
```

---

## 4. 实现任务

### Phase 1: 核心数据结构
1. [ ] 添加 `CompressionLevel` 枚举
2. [ ] 添加 `CompressionConfig` 结构体
3. [ ] 实现 `CompressionLevel` 的级别映射方法

### Phase 2: 算法自动选择
4. [ ] 添加 `CompressionStrategy` 选择逻辑
5. [ ] 修改 `write_to_file` 支持动态算法选择
6. [ ] 修改 `read_from_file` 自动检测压缩类型

### Phase 3: 配置集成
7. [ ] 扩展 `ColumnDefinition` 添加 `compression` 字段
8. [ ] 修改 `TableStore` 支持压缩配置
9. [ ] 修改 `ColumnarStorage` 应用列压缩配置

### Phase 4: 测试
10. [ ] 添加压缩级别测试
11. [ ] 添加自动选择策略测试
12. [ ] 添加配置继承测试

---

## 5. 文件变更

```
crates/storage/src/
├── engine.rs                          # ColumnDefinition 扩展
├── columnar/
│   ├── mod.rs                         # 导出新类型
│   ├── segment.rs                     # ColumnSegment 扩展
│   └── storage.rs                    # TableStore 扩展
```

---

## 6. 向后兼容性

- `compression: None` 在 ColumnDefinition 中表示使用默认自动选择
- 现有测试保持通过
- 现有数据文件无需迁移（自动检测压缩类型）
