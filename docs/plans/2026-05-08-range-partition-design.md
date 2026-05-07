# ISSUE 442 - RANGE Partition 分区裁剪设计方案

> **日期**: 2026-05-08
> **Issue**: #442
> **状态**: 设计完成，待实现
> **分支**: feat/range-partition-442

---

## 一、目标

实现 RANGE 分区表的 Planner-level pruning，支持分区裁剪以提升查询性能。

### 验收条件
```sql
SELECT * FROM batch_record WHERE batch_date = '2025-06-01';
-- 只扫描 p2025 分区，不扫描 p2024 和 p_future
```

### 支持的功能
- ✅ partition pruning (optimizer)
- ✅ INSERT routing
- ✅ metadata 查询 (via Catalog)

### 不支持
- ❌ global secondary index
- ❌ online repartition
- ❌ subpartition

---

## 二、架构决策

| 决策点 | 选择 | 理由 |
|--------|------|------|
| 裁剪层面 | Storage-level | 每个分区独立文件存储 |
| 裁剪时机 | Optimizer 层 | 支持 CBO 代价选择 |
| 元数据存储 | Catalog | 统一管理，保证一致性 |
| 分区信息传递 | Physical Plan | 早期裁剪，无运行时开销 |
| 边界计算 | 预计算边界值 | 二分查找，性能最优 |

---

## 三、存储架构

### 3.1 物理存储
```
data/
  batch_record/
    p2024.data      # 分区数据文件
    p2025.data
    p_future.data
    metadata.json    # 可选的备份元数据
```

### 3.2 Catalog 分区存储

```rust
// TableSchema 新增 partition 字段
struct TableSchema {
    name: String,
    columns: Vec<Column>,
    partition: Option<PartitionDefinition>,
}

// 分区定义
enum PartitionType {
    Range,
    Hash,
    List,
    Key,
}

struct PartitionDefinition {
    partition_type: PartitionType,
    expr: Expr,                    // 分区表达式，如 YEAR(batch_date)
    partitions: Vec<Partition>,
}

// 单个分区
struct Partition {
    name: String,
    bound_less_than: Option<Value>, // VALUES LESS THAN 值
    is_max_value: bool,            // MAXVALUE 特殊分区
}
```

---

## 四、分区裁剪流程

### 4.1 裁剪流程图

```
SQL: SELECT * FROM batch_record WHERE batch_date = '2025-06-01'
                        ↓
Parser → 生成 AST
                        ↓
Planner → 生成 LogicalPlan (带 WHERE 条件)
                        ↓
Optimizer → 分区裁剪
    1. 从 Catalog 获取表分区定义
    2. 根据 WHERE 条件计算目标分区
       WHERE batch_date = '2025-06-01'
       → YEAR(batch_date) = 2025
       → 二分查找定位分区 p2025
    3. 生成 PhysicalPlan 带 partitions: [p2025]
                        ↓
PhysicalPlan: PartitionedTableScan {
    table_name: "batch_record",
    partitions: [p2025_id],
    ...
}
                        ↓
Executor → 并行扫描目标分区
```

### 4.2 分区裁剪算法

```rust
fn prune_partitions(
    partitions: &[Partition],
    predicate: &Expr,
) -> Vec<usize> {
    // 简化逻辑：
    // 1. 解析谓词，提取分区表达式的值
    // 2. 二分查找确定目标分区
    // 3. 返回匹配的分区索引列表
}
```

---

## 五、Physical Plan 设计

### 5.1 分区扫描节点

```rust
struct PartitionedTableScan {
    table_name: String,
    partition_ids: Vec<u32>,        // 裁剪后的分区 ID 列表
    projection: Vec<usize>,
    filter: Option<Expr>,            // 可能还有额外过滤条件
}
```

### 5.2 Plan 转换

```
LogicalPlan::TableScan { table_name, ... }
        ↓
PhysicalPlan::PartitionedTableScan {
    table_name,
    partition_ids: [p2025_id],  // 裁剪结果
    ...
}
```

---

## 六、INSERT 路由

### 6.1 INSERT 流程

```sql
INSERT INTO batch_record VALUES (1, '2025-06-01')
```
```
Parser → 生成 INSERT LogicalPlan
                ↓
Catalog → 获取分区定义
                ↓
计算 YEAR('2025-06-01') = 2025
                ↓
定位目标分区 p2025
                ↓
Executor → 写入 p2025.data
```

---

## 七、实现任务分解

### 7.1 Parser 层
- [ ] 解析 `PARTITION BY RANGE (expr)` 语法
- [ ] 解析 `PARTITION ... VALUES LESS THAN (value)` 语法
- [ ] 解析 `MAXVALUE` 特殊值

### 7.2 Catalog 层
- [ ] 在 `TableSchema` 中添加 `partition: Option<PartitionDefinition>`
- [ ] 实现 `create_table_with_partition()` 方法
- [ ] 实现 `get_table_partitions()` 方法

### 7.3 Planner 层
- [ ] 在 `LogicalPlan::CreateTable` 中支持分区定义
- [ ] 传递分区元数据到执行层

### 7.4 Optimizer 层
- [ ] 实现 `partition_pruning()` 优化规则
- [ ] 根据 WHERE 条件计算目标分区
- [ ] 生成 `PartitionedTableScan` 节点

### 7.5 Executor 层
- [ ] 实现 `PartitionedTableScan` 执行器
- [ ] 并行扫描多个分区
- [ ] 实现 INSERT 分区路由

### 7.6 Storage 层
- [ ] 每个分区独立数据文件
- [ ] 分区数据文件读写

### 7.7 测试
- [ ] 创建分区表测试
- [ ] 分区裁剪测试
- [ ] INSERT 路由测试
- [ ] 多分区扫描测试

---

## 八、文件变更

| 文件 | 变更 |
|------|------|
| `crates/parser/src/parser.rs` | 解析 PARTITION 语法 |
| `crates/parser/src/token.rs` | 新增 PARTITION 关键字 |
| `crates/parser/src/lexer.rs` | 新增 PARTITION 词法 |
| `crates/planner/src/logical_plan.rs` | 添加分区结构 |
| `crates/planner/src/planner.rs` | 分区表创建逻辑 |
| `crates/planner/src/optimizer.rs` | 分区裁剪优化规则 |
| `crates/catalog/src/lib.rs` | Catalog 分区存储 |
| `crates/executor/src/partition_executor.rs` | 新增：分区执行器 |
| `crates/storage/src/partition_storage.rs` | 新增：分区存储 |
| `tests/partition_test.rs` | 新增：分区测试 |

---

## 九、风险与备选方案

### 风险 1：表达式解析复杂度
- **风险**: 分区表达式可能很复杂（如 `YEAR(batch_date) - 2000`）
- **缓解**: 初期仅支持简单表达式 YEAR(), MONTH(), DAY()

### 风险 2：分区裁剪准确性
- **风险**: 边界条件处理不当
- **缓解**: 严格测试边界值，包括 MAXVALUE

---

## 十、参考文档

- MySQL 8.0 Partitioning: https://dev.mysql.com/doc/refman/8.0/en/partitioning.html
- PostgreSQL Partitioning: https://www.postgresql.org/docs/current/ddl-partitioning.html

---

*本文档由 Sisyphus 创建*
*日期: 2026-05-08*