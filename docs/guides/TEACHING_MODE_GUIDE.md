# 教学模式

> **版本**: 1.0
> **更新日期**: 2026-05-06

---

## 一、概述

SQLRustGo 提供教学模式（Teaching Mode），通过 `SQLRUSTGO_TEACHING_MODE=1` 环境变量启用。

教学模式提供标准 MySQL 不具备的教学增强功能：
- 查询管道可视化
- 执行计划逐步展示
- 表达式评估分解

---

## 二、启用

### 环境变量

```bash
SQLRUSTGO_TEACHING_MODE=1 cargo run --bin sqlrustgo
```

### HTTP 端点

启用后，HTTP 服务新增以下端点：

| 端点 | 说明 |
|------|------|
| `GET /teaching/pipeline` | 查询管道可视化 HTML 页面 |
| `GET /teaching/pipeline/json` | 管道数据 JSON 格式 |

### Cargo 特性

教学模式通过 `teaching` feature 控制：

```bash
# 默认启用
cargo run --bin sqlrustgo --features teaching

# 禁用
cargo run --bin sqlrustgo
```

---

## 三、教学功能

### 3.1 查询管道可视化

`GET /teaching/pipeline?sql=SELECT ...`

展示查询执行的完整管道：
1. Parser → AST
2. Planner → Logical Plan
3. Optimizer → Optimized Plan  
4. Executor → Physical Execution

### 3.2 与 CBO、CTE、窗口函数的兼容性

| 功能 | 教学模式 | 说明 |
|------|---------|------|
| CBO (Predicate Pushdown) | ✅ 可见化 | 学生可看到优化前后的计划对比 |
| CTE (WITH 子句) | ✅ 可见化 | CTE 展开步骤可见 |
| 窗口函数 | ✅ 可见化 | PARTITION BY / ORDER BY 分解 |
| EXPLAIN ANALYZE | ✅ 逐步展示 | 每步代价行数对比 |

---

## 四、验证

### 快速验证

```bash
# 启动教学模式
SQLRUSTGO_TEACHING_MODE=1 cargo run --bin sqlrustgo &

# 测试管道可视化
curl "http://localhost:8080/teaching/pipeline?sql=SELECT+1"

# 测试管道 JSON
curl "http://localhost:8080/teaching/pipeline/json?sql=SELECT+1"

# 关闭
kill %1
```

### 教学实验

教学模式支持以下教学实验（12 个）：

| # | 实验 | SQL | 教学点 |
|---|------|-----|--------|
| 1 | 基础 SELECT | `SELECT 1` | 查询管道 |
| 2 | WHERE 过滤 | `SELECT * FROM t WHERE id = 1` | 过滤下推 |
| 3 | JOIN | `SELECT * FROM a JOIN b ON a.id = b.id` | Hash Join |
| 4 | 聚合 | `SELECT COUNT(*) FROM t GROUP BY col` | 分组聚合 |
| 5 | 子查询 | `SELECT * FROM t WHERE id IN (SELECT id FROM s)` | 子查询转 Join |
| 6 | CTE | `WITH x AS (SELECT 1) SELECT * FROM x` | CTE 展开 |
| 7 | 窗口函数 | `SELECT ROW_NUMBER() OVER (ORDER BY id) FROM t` | 窗口评估 |
| 8 | ORDER BY | `SELECT * FROM t ORDER BY name` | 排序策略 |
| 9 | DISTINCT | `SELECT DISTINCT name FROM t` | 去重 |
| 10 | LIMIT | `SELECT * FROM t LIMIT 10` | 提前终止 |
| 11 | CBO 对比 | `EXPLAIN ANALYZE SELECT * FROM t WHERE id < 100` | 代价估算 |
| 12 | 索引 | `CREATE INDEX ON t(id); SELECT * FROM t WHERE id = 1` | 索引加速 |

---

## 五、维护

### 添加新教学端点

`crates/server/src/teaching_endpoints.rs` — 在此文件中添加新端点。

### 教学模式开关

教学模式通过 `TeachingEndpoints` 结构体控制每个功能的启用/禁用：

```rust
let endpoints = TeachingEndpoints {
    enable_pipeline_viz: true,
    enable_step_by_step: true,
    // ...
};
```

---

*本文档由 SQLRustGo Team 维护*
