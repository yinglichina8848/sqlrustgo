# v2.9.0 迁移指南

> **版本**: v2.9.0
> **从**: v2.8.0
> **日期**: 2026-05-05

---

## 一、迁移路径

### 兼容性说明

v2.9.0 相比 v2.8.0：
- ✅ 向后兼容：大多数 v2.8.0 SQL 语句无需修改即可运行
- ⚠️ 部分破坏性变更：见下方详细说明
- ✅ 新增功能：CTE、窗口函数、CASE 表达式、JSON 操作

### 迁移检查清单

- [ ] 备份现有数据目录
- [ ] 更新 Cargo.toml 中的 sqlrustgo 版本
- [ ] 运行 `cargo test` 验证兼容性
- [ ] 更新 SHOW INDEX 和 Catalog 相关脚本
- [ ] 检查 WAL 配置（如使用自定义检查点间隔）

---

## 二、破坏性变更详细说明

### 2.1 SHOW INDEX 输出格式

**变更**: `SHOW INDEX FROM table` 输出列顺序调整

**旧格式**:
```
Table | Non_unique | Key_name | Seq_in_index | Column_name | ...
```

**新格式**:
```
Table | Key_name | Column_name | Non_unique | Seq_in_index | ...
```

**迁移**: 检查使用列位置的脚本，改为使用列名匹配

### 2.2 WAL 检查点默认值

**变更**: `wal.checkpoint_interval` 默认值从 1000 改为 500

**影响**: 日志文件稍大，崩溃恢复更快

**迁移**: 如需还原，在 `sqlrustgo.toml` 中显式设置：
```toml
[wal]
checkpoint_interval = 1000
```

---

## 三、新功能迁移示例

### 3.1 CTE (WITH) 语法

```sql
-- v2.9.0 支持
WITH regional_sales AS (
    SELECT region, SUM(amount) AS total_sales
    FROM orders
    GROUP BY region
)
SELECT region, total_sales
FROM regional_sales
WHERE total_sales > 10000;
```

### 3.2 窗口函数

```sql
-- v2.9.0 支持
SELECT
    department,
    employee_id,
    salary,
    RANK() OVER (PARTITION BY department ORDER BY salary DESC) as rank,
    ROW_NUMBER() OVER (PARTITION BY department ORDER BY employee_id) as row_num
FROM employees;
```

### 3.3 CASE 表达式

```sql
SELECT
    product_name,
    CASE
        WHEN price > 1000 THEN 'Premium'
        WHEN price > 500 THEN 'Mid-range'
        ELSE 'Budget'
    END as category
FROM products;
```

### 3.4 JSON 操作

```sql
SELECT
    id,
    json_extract(metadata, '$.author') as author,
    json_extract(metadata, '$.tags[0]') as first_tag
FROM articles;
```

---

## 四、API 迁移

### 4.1 Rust API

```rust
// v2.8.0
let engine = ExecutionEngine::new();

// v2.9.0 (推荐)
let engine = ExecutionEngine::new()
    .with_config(EngineConfig::default())
    .with_max_connections(256);
```

### 4.2 配置文件

```toml
# v2.8.0
[vector]
hnsw_enable = true

# v2.9.0
[vector]
index_type = "hnsw"
```

---

## 五、验证步骤

```bash
# 1. 编译通过
cargo build --all

# 2. 运行测试
cargo test --all-features

# 3. 快速功能验证
cargo run --bin sqlrustgo -- -e "SELECT 1+1"

# 4. TPC-H 验证
cargo run --bin bench-cli -- tpch bench --queries Q1 --sf 0.1

# 5. 性能基准
cargo run --bin bench-cli -- sysbench point_select --threads 4 --time 10
```

---

*本文档由 Hermes Agent 维护*
*更新日期: 2026-05-05*
