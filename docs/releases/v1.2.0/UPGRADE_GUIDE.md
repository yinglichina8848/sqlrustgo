# SQLRustGo v1.1.x → v1.2.0 升级迁移指南

> 版本：v1.2.0
> 发布日期：2026-03-10
> 目标用户：v1.1.x 用户升级到 v1.2.0

---

## 一、升级概述

### 1.1 版本对比

| 项目 | v1.1.0 | v1.2.0 |
|------|--------|--------|
| 架构模式 | Client-Server + 嵌入式 | 接口抽象化 + 可插拔 |
| 存储引擎 | FileStorage (固定) | StorageEngine trait (可替换) |
| 执行方式 | Row-based | Vectorized + RecordBatch |
| 统计信息 | 简单计数器 | TableStats + ColumnStats + Collector |
| 优化器 | 硬编码规则 | CostModel + Memo 结构 |
| 数据规模 | 10万行 | 100万行 |

### 1.2 升级收益

- ✅ **性能提升**: 向量化执行，百万行数据处理能力
- ✅ **可扩展性**: 可插拔存储引擎支持自定义实现
- ✅ **优化能力**: 基于成本的优化器 (CBO)
- ✅ **统计信息**: ANALYZE 命令支持表/列统计

### 1.3 兼容性说明

**v1.2.0 完全向后兼容 v1.1.0 API**，现有代码无需修改即可运行。

---

## 二、升级前准备

### 2.1 系统要求

| 要求 | v1.1.0 | v1.2.0 |
|------|--------|--------|
| Rust 版本 | ≥1.85 | ≥1.85 |
| 操作系统 | Linux/macOS/Windows | Linux/macOS/Windows |
| 依赖项 | Tokio | Tokio + 新增统计相关 |

### 2.2 备份数据

```bash
# 备份数据文件
cp -r data/ data_backup_v1.1.0/

# 备份配置文件（如有）
cp config.toml config.toml.bak
```

### 2.3 检查依赖

```bash
# 确保 Rust 版本正确
rustc --version

# 检查项目依赖
cargo outdated
```

---

## 三、升级步骤

### 3.1 更新代码

```bash
# 拉取最新代码
git fetch origin
git checkout develop/v1.2.0
```

### 3.2 更新依赖

```bash
# 更新 Cargo.lock
cargo update

# 构建项目
cargo build --all-features
```

### 3.3 运行测试

```bash
# 运行所有测试
cargo test --all-features

# 检查覆盖率
cargo tarpaulin --all-features
```

---

## 四、API 变更

> ⚠️ **重要**: v1.1.0 现有 API 完全兼容，无需修改代码即可运行。

### 4.1 新增存储引擎接口

#### StorageEngine Trait (新增)

```rust
use sqlrustgo_storage::StorageEngine;
use sqlrustgo_storage::FileStorage;
use sqlrustgo_storage::MemoryStorage;

// 使用文件存储
let storage: Box<dyn StorageEngine> = Box::new(FileStorage::new("/data")?);

// 使用内存存储
let storage: Box<dyn StorageEngine> = Box::new(MemoryStorage::new());
```

### 4.2 新增向量化执行

#### RecordBatch (新增)

```rust
use sqlrustgo_executor::RecordBatch;
use sqlrustgo_types::ArrayRef;

// 创建 RecordBatch
let batch = RecordBatch::new(
    schema,
    columns,  // Vec<ArrayRef>
    row_count,
)?;
```

### 4.3 新增统计信息

#### TableStats (新增)

```rust
use sqlrustgo_optimizer::stats::TableStats;

// 获取表统计信息
let stats = table_stats.get_table_stats("users")?;
println!("Row count: {}", stats.row_count);
```

#### ANALYZE 命令 (新增)

```sql
-- 收集表统计信息
ANALYZE TABLE users;

-- 收集所有表统计信息
ANALYZE;
```

### 4.4 新增 CostModel

```rust
use sqlrustgo_optimizer::cost::CostModel;

// 创建成本模型
let cost_model = CostModel::new()
    .with_seq_scan_cost(1.0)
    .with_idx_scan_cost(1.5)
    .with_filter_cost(0.5)
    .with_join_cost(3.0);
```

---

## 五、配置变更

### 5.1 新增配置项

```toml
# config.toml

[storage]
# 存储类型: file | memory
backend = "file"

# 文件存储路径
data_dir = "./data"

[optimizer]
# 启用基于成本的优化
enable_cbo = true

# 启用统计信息收集
enable_stats = true

[stats]
# 自动 ANALYZE 阈值
auto_analyze_threshold = 10000
```

### 5.2 环境变量

| 变量 | 说明 | 默认值 |
|------|------|--------|
| `SQLRUSTGO_STORAGE_BACKEND` | 存储后端 | `file` |
| `SQLRUSTGO_CBO_ENABLED` | 启用 CBO | `true` |
| `SQLRUSTGO_STATS_ENABLED` | 启用统计 | `true` |

---

## 六、数据迁移

### 6.1 数据兼容性

v1.2.0 **完全兼容** v1.1.0 数据格式，无需迁移数据。

| 数据类型 | 兼容性 |
|----------|--------|
| 表数据文件 | ✅ 兼容 |
| 索引文件 | ✅ 兼容 |
| WAL 日志 | ✅ 兼容 |

### 6.2 数据验证

```bash
# 启动服务
cargo run --bin sqlrustgo

# 执行 ANALYZE 收集统计
ANALYZE TABLE users;

# 验证查询
SELECT COUNT(*) FROM users;
EXPLAIN SELECT * FROM users WHERE id > 100;
```

---

## 七、性能优化建议

### 7.1 使用 ANALYZE

```sql
-- 在执行复杂查询前先收集统计信息
ANALYZE TABLE orders;
ANALYZE TABLE customers;

-- 执行 JOIN 查询
SELECT * FROM orders o
JOIN customers c ON o.customer_id = c.id
WHERE o.amount > 1000;
```

### 7.2 向量化执行

v1.2.0 自动使用向量化执行，无需额外配置。确保：

- 数据类型使用标准类型（Integer, Varchar, Float）
- 避免频繁的类型转换

### 7.3 索引优化

```sql
-- 创建索引
CREATE INDEX idx_orders_customer ON orders(customer_id);

-- 使用索引
SELECT * FROM orders WHERE customer_id = 123;
```

---

## 八、回滚方案

### 8.1 回滚步骤

```bash
# 1. 停止服务
pkill sqlrustgo

# 2. 切换回 v1.1.0
git checkout develop/v1.1.0

# 3. 恢复数据（如有变更）
cp -r data_backup_v1.1.0/ data/

# 4. 重新构建
cargo build --release
```

### 8.2 兼容性保证

- v1.2.0 数据文件可被 v1.1.0 读取
- 新增功能（如 CBO）不影响基础查询
- LocalExecutor API 保持不变

---

## 九、常见问题

### Q1: 升级后编译失败？

**A**: 确保 Rust 版本 ≥ 1.85

```bash
rustup update stable
rustc --version
```

### Q2: 如何使用新的存储引擎？

**A**: 参考 [API 变更](#四api-变更) 章节：

```rust
use sqlrustgo_storage::StorageEngine;
use sqlrustgo_storage::MemoryStorage;

let storage: Box<dyn StorageEngine> = Box::new(MemoryStorage::new());
```

### Q3: ANALYZE 命令不生效？

**A**: 确保：
- 表已创建并有数据
- 查询计划器启用了统计信息

```sql
-- 检查统计信息
EXPLAIN ANALYZE SELECT * FROM users;
```

### Q4: 性能没有提升？

**A**: 检查是否：
- 执行了 ANALYZE 收集统计信息
- 查询涉及大数据量
- 使用了 JOIN 或聚合查询

### Q5: 索引相关测试失败？

**A**: v1.2.0 索引功能正在完善中，索引相关测试可能不稳定。建议：
- 避免在生产环境依赖索引
- 报告问题到 GitHub Issues

---

## 十、获取帮助

### 10.1 文档资源

- [Release Notes](./RELEASE_NOTES.md)
- [CHANGELOG](../../CHANGELOG.md)
- [API 文档](https://docs.rs/sqlrustgo)
- [性能报告](./PERFORMANCE_REPORT.md)

### 10.2 社区支持

- GitHub Issues: https://github.com/minzuuniversity/sqlrustgo/issues
- GitHub Discussions: https://github.com/minzuuniversity/sqlrustgo/discussions

---

## 附录 A: Breaking Changes

v1.2.0 **无 Breaking Changes**，完全向后兼容。

| 变更类型 | 影响 |
|----------|------|
| 新增 trait | 无影响 |
| 新增 struct | 无影响 |
| 新增 API | 可选使用 |

---

## 附录 B: 新增 Crate

v1.2.0 新增以下 crate：

| Crate | 说明 |
|-------|------|
| `sqlrustgo-common` | 公共工具和类型 |
| `sqlrustgo-optimizer` | 优化器和统计信息 |

---

*本文档由 Claude Code 生成*
*最后更新: 2026-03-10*
