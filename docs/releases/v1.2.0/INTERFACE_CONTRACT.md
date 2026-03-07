# v1.2.0 接口契约文档

> ⚠️ **重要更新**: 代码已迁移到 crates/ workspace 结构，以下路径可能已变更。
> 实际位置请参考各 crate 的 Cargo.toml 和 src/lib.rs。
> 
> 新路径对应关系:
> - __代码0__ → __代码1__
> - __代码0__ → __代码1__
> - __代码0__ → __代码1__
> - __代码0__ → __代码1__
> - __代码0__ → __代码1__
> - __代码0__ → __代码1__

本文档记录 v1.2.0 中所有核心接口 (trait) 的契约定义，基于实际代码。

---

## 一、核心接口总览

| 接口 | 原位置 | 新位置 | 状态 |
|------|--------|--------|------|
|__代码0__| `src/query/mod.rs` |__代码0__| ✅ |
|__代码0__| `src/catalog/mod.rs` |__代码0__| ✅ |
|__代码0__| `src/optimizer/mod.rs` |__代码0__| ✅ |
| `Rule` | `src/optimizer/mod.rs` |__代码0__| ✅ |
|__代码0__| `src/optimizer/mod.rs` |__代码0__| ✅ |
|__代码0__| `src/optimizer/stats.rs` |__代码0__| ✅ |
|__代码0__| `src/executor/mod.rs` |__代码0__| ✅ |
|__代码0__| `src/storage/engine.rs` |__代码0__| ✅ |
|__代码0__| `src/planner/physical_plan.rs` |__代码0__| ✅ |

---

## 二、接口详细定义

### 2.1 查询服务

**文件**: `src/query/mod.rs`

```rust
pub trait QueryService: Send + Sync {
    fn execute_query(&self, sql: &str) -> Result<RecordSet, SqlError>;
    fn get_catalog(&self) -> Arc<dyn Catalog>;
}
```

| 方法 | 输入 | 输出 | 说明 |
|------|------|------|------|
|__代码0__| SQL 字符串 |记录集| 执行查询 |
|__代码0__| - |弧\<动态目录\>| 获取目录服务 |

### 2.2 目录

**文件**: `src/catalog/mod.rs`

```rust
pub trait Catalog: Send + Sync {
    fn list_tables(&self) -> Result<Vec<String>, SqlError>;
    fn get_table(&self, name: &str) -> Result<TableMeta, SqlError>;
    fn create_table(&self, schema: Schema) -> Result<(), SqlError>;
    fn drop_table(&self, name: &str) -> Result<(), SqlError>;
}
```

| 方法 | 输入 | 输出 | 说明 |
|------|------|------|------|
|__代码0__| - |Vec\<字符串\>| 列出所有表 |
|__代码0__| 表名 |表元| 获取表元数据 |
|__代码0__|模式| () | 创建表 |
|__代码0__| 表名 | () | 删除表 |

### 2.3 优化器

**文件**: `src/optimizer/mod.rs`

```rust
pub trait Optimizer<Plan> {
    fn optimize(&self, plan: Plan) -> OptimizerResult<Plan>;
}
```

| 方法 | 输入 | 输出 | 说明 |
|------|------|------|------|
|__代码0__|逻辑计划|OptimizerResult\<物理计划\>| 执行优化 |

### 2.4 Rule

**文件**: `src/optimizer/mod.rs`

```rust
/// Rule trait - 优化规则接口
///
/// # What
/// 优化规则接口，每条规则负责特定的优化转换
///
/// # Why
/// 规则化设计便于扩展和维护优化规则
///
/// # How
/// - apply 方法尝试应用规则
/// - 返回是否发生了改变
pub trait Rule<Plan>: Send + Sync {
    fn apply(&self, plan: &Plan) -> Option<Plan>;
    fn name(&self) -> &str;
}
```

| 方法 | 输入 | 输出 | 说明 |
|------|------|------|------|
| `apply` | Plan |选项\<计划\>| 应用规则 |
| `name` | - | &str | 规则名称 |

### 2.5 成本模型

**文件**: `src/optimizer/mod.rs`

```rust
pub trait CostModel<Plan>: Send + Sync {
    fn estimate_cost(&self, plan: &Plan) -> Cost;
}
```

| 方法 | 输入 | 输出 | 说明 |
|------|------|------|------|
|__代码0__|物理计划| Cost | 估算执行成本 |

### 2.6 统计提供者

**文件**: `src/optimizer/stats.rs`

```rust
pub trait StatisticsProvider: Send + Sync {
    fn get_table_stats(&self, table: &str) -> Option<TableStats>;
    fn get_column_stats(&self, table: &str, column: &str) -> Option<ColumnStats>;
}
```

| 方法 | 输入 | 输出 | 说明 |
|------|------|------|------|
|__代码0__| 表名 |选项\<表统计\>| 获取表统计 |
|__代码0__| 表名, 列名 |选项\<列统计\>| 获取列统计 |

### 2.7 执行者

**文件**: `src/executor/mod.rs`

```rust
pub trait Executor: Send + Sync {
    fn execute(&self, plan: Box<dyn PhysicalPlan>) -> Result<RecordBatch, SqlError>;
}
```

| 方法 | 输入 | 输出 | 说明 |
|------|------|------|------|
|__代码0__|物理计划|记录批次| 执行物理计划 |

### 2.8 存储引擎

**文件**: `src/storage/engine.rs`

```rust
pub trait StorageEngine: Send + Sync {
    fn scan(&self, table: &str) -> Result<Box<dyn Iterator<Item = Row>> + '_>;
    fn insert(&self, table: &str, row: Row) -> Result<(), SqlError>;
    fn delete(&self, table: &str, key: &Value) -> Result<(), SqlError>;
    fn update(&self, table: &str, key: &Value, row: Row) -> Result<(), SqlError>;
    fn create_table(&self, schema: &Schema) -> Result<(), SqlError>;
    fn drop_table(&self, name: &str) -> Result<(), SqlError>;
}
```

| 方法 | 输入 | 输出 | 说明 |
|------|------|------|------|
| `scan` | 表名 |迭代器| 扫描表 |
|__代码0__| 表名, Row | () | 插入行 |
|__代码0__| 表名, Key | () | 删除行 |
|__代码0__|表名、键、行| () | 更新行 |
|__代码0__|模式| () | 创建表 |
|__代码0__| 表名 | () | 删除表 |

### 2.9 物理计划

**文件**: `src/planner/physical_plan.rs`

```rust
pub trait PhysicalPlan: Send + Sync {
    fn schema(&self) -> &Schema;
    fn children(&self) -> Vec<Arc<dyn PhysicalPlan>>;
    fn execute(&self) -> Result<RecordBatch, SqlError>;
}
```

| 方法 | 输入 | 输出 | 说明 |
|------|------|------|------|
|__代码0__| - |架构(&S)| 获取输出 Schema |
|__代码0__| - |Vec\<物理计划\>| 获取子计划 |
|__代码0__| - |记录批次| 执行计划 |

---

## 三、返回类型说明

### sql错误

**文件**: `src/error/mod.rs`

```rust
pub enum SqlError {
    ParseError(String),
    ExecutionError(String),
    StorageError(String),
    CatalogError(String),
    NetworkError(String),
    // ...
}
```

### 记录集/记录批处理

```rust
pub struct RecordBatch {
    columns: Vec<Array>,
    row_count: usize,
}
```

### Cost

**文件**: `src/optimizer/cost.rs`

```rust
pub struct Cost {
    pub cpu_cost: f64,
    pub io_cost: f64,
    pub memory_cost: f64,
}
```

---

## 四、接口依赖关系

```
QueryService
    │
    ├── Catalog
    │       └── (依赖 StorageEngine)
    │
    ├── Optimizer
    │       ├── Rule
    │       ├── CostModel
    │       └── StatisticsProvider
    │               └── (依赖 Catalog)
    │
    └── Executor
            └── PhysicalPlan
                    └── StorageEngine
```

---

## 五、版本状态

| 接口 | 状态 | 说明 |
|------|------|------|
|查询服务| ✅ 稳定 | 已定义并实现 |
|目录| ✅ 稳定 |SimpleCatalog 实现|
|优化器| ✅ 稳定 |NoOpOptimizer 基础实现|
| Rule | ✅ 稳定 | 接口定义 |
|成本模型| ✅ 稳定 | 接口定义 |
|统计提供者| ✅ 稳定 |InMemory 实现|
|执行者| ✅ 稳定 | 接口定义 |
|存储引擎| ✅ 稳定 | 接口定义 |
|物理计划| ✅ 稳定 | 7 个算子实现 |

---

## 六、文档版本

| 版本 | 日期 | 说明 |
|------|------|------|
| 1.0.0 | 2026-03-05 | 初始版本 |

---

*本文档基于 v1.2.0 develop 分支实际代码生成*
