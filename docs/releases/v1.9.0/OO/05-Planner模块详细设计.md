# SQLRustGo Planner 模块详细设计文档

> **版本**: v1.9.0
> **日期**: 2026-03-26
> **模块**: sqlrustgo-planner

---

## 1. 模块概述

Planner 模块负责将 SQL AST 转换为逻辑计划和物理计划。

### 1.1 模块职责

- 语义分析 (Semantic Analysis)
- 类型检查 (Type Checking)
- 列绑定 (Column Binding)
- 逻辑计划生成 (Logical Plan Generation)
- 物理计划生成 (Physical Plan Generation)

### 1.2 模块结构

```
crates/planner/
├── src/
│   ├── lib.rs           # 模块入口
│   ├── planner.rs       # 规划器主逻辑
│   ├── logical_plan.rs  # 逻辑计划定义
│   ├── physical_plan.rs # 物理计划定义
│   └── optimizer.rs     # 优化器
└── Cargo.toml
```

---

## 2. 核心类设计

### 2.1 规划器类图

```uml
@startuml

class Planner {
  -catalog: CatalogRef
  --
  +create_logical_plan(stmt): LogicalPlan
  +create_physical_plan(logical): PhysicalPlan
}

class Binder {
  -catalog: CatalogRef
  -scope: Scope
  --
  +bind_expression(expr): BoundExpression
  +bind_table_ref(table): BoundTable
  +resolve_column(name): ColumnRef
}

class TypeChecker {
  -schema: Schema
  --
  +check_expression(expr): DataType
  +coerce_types(left, right): (Expression, Expression)
}

class LogicalPlanner {
  -binder: Binder
  -type_checker: TypeChecker
  --
  +plan_select(stmt): LogicalPlan
  +plan_insert(stmt): LogicalPlan
  +plan_update(stmt): LogicalPlan
  +plan_delete(stmt): LogicalPlan
}

class PhysicalPlanner {
  -optimizer: Optimizer
  --
  +plan_to_physical(plan): PhysicalPlan
  +select_physical_operators(node): PhysicalOperator
}

Planner --> Binder
Binder --> TypeChecker
Planner --> LogicalPlanner
Planner --> PhysicalPlanner
LogicalPlanner --> Binder
PhysicalPlanner --> Optimizer

@enduml
```

---

## 3. 逻辑计划设计

### 3.1 逻辑计划节点

```uml
@startuml

abstract class LogicalPlanNode {
  -schema: Schema
  -children: Vec<LogicalPlan>
  --
  +schema(): Schema
  +children(): Vec<LogicalPlan>
}

class ScanNode {
  -table_name: String
  -filters: Vec<Expression>
}

class ProjectionNode {
  -input: LogicalPlan
  -expressions: Vec<Expression>
}

class FilterNode {
  -input: LogicalPlan
  -predicate: Expression
}

class JoinNode {
  -left: LogicalPlan
  -right: LogicalPlan
  -join_type: JoinType
  -condition: Expression
}

class AggregateNode {
  -input: LogicalPlan
  -group_by: Vec<Expression>
  -aggregates: Vec<AggregateFunction>
}

class SortNode {
  -input: LogicalPlan
  -order_by: Vec<OrderExpression>
}

class LimitNode {
  -input: LogicalPlan
  -limit: u64
  -offset: u64
}

class SetOperationNode {
  -left: LogicalPlan
  -right: LogicalPlan
  -set_op: SetOperationType
}

LogicalPlanNode <|-- ScanNode
LogicalPlanNode <|-- ProjectionNode
LogicalPlanNode <|-- FilterNode
LogicalPlanNode <|-- JoinNode
LogicalPlanNode <|-- AggregateNode
LogicalPlanNode <|-- SortNode
LogicalPlanNode <|-- LimitNode
LogicalPlanNode <|-- SetOperationNode

@enduml
```

### 3.2 逻辑计划示例

```sql
-- SQL
SELECT id, name 
FROM users 
WHERE age > 18 
ORDER BY name 
LIMIT 10

-- 逻辑计划树
Limit(10)
└── Sort(name ASC)
    └── Filter(age > 18)
        └── Projection(id, name)
            └── Scan(users)
```

---

## 4. 物理计划设计

### 4.1 物理计划节点

```uml
@startuml

abstract class PhysicalPlanNode {
  -schema: Schema
  --
  +execute(ctx): RecordBatch
}

class SeqScanNode {
  -table_name: String
  -filter: Option<Expression>
}

class IndexScanNode {
  -table_name: String
  -index_name: String
  -key_range: KeyRange
}

class HashJoinNode {
  -join_type: JoinType
  -left_keys: Vec<Expression>
  -right_keys: Vec<Expression>
  -build_side: BuildSide
}

class MergeJoinNode {
  -join_type: JoinType
  -left_keys: Vec<Expression>
  -right_keys: Vec<Expression>
  -left_order: Vec<Order>
  -right_order: Vec<Order>
}

class HashAggNode {
  -group_by: Vec<Expression>
  -aggregates: Vec<AggregateExpr>
}

class SortNode {
  -order_by: Vec<OrderExpression>
}

class ProjectNode {
  -expressions: Vec<Expression>
}

class FilterNode {
  -predicate: Expression
}

class LimitNode {
  -limit: u64
  -offset: u64
}

class NestedLoopJoinNode {
  -left: PhysicalPlan
  -right: PhysicalPlan
  -join_type: JoinType
  -condition: Expression
}

PhysicalPlanNode <|-- SeqScanNode
PhysicalPlanNode <|-- IndexScanNode
PhysicalPlanNode <|-- HashJoinNode
PhysicalPlanNode <|-- MergeJoinNode
PhysicalPlanNode <|-- HashAggNode
PhysicalPlanNode <|-- SortNode
PhysicalPlanNode <|-- ProjectNode
PhysicalPlanNode <|-- FilterNode
PhysicalPlanNode <|-- LimitNode
PhysicalPlanNode <|-- NestedLoopJoinNode

@enduml
```

---

## 5. Schema 设计

### 5.1 Schema 结构

```rust
pub struct Schema {
    fields: Vec<Field>,
}

pub struct Field {
    name: String,
    data_type: DataType,
    nullable: bool,
}

impl Schema {
    pub fn new(fields: Vec<Field>) -> Self;
    pub fn field(&self, name: &str) -> Option<&Field>;
    pub fn field_index(&self, name: &str) -> Option<usize>;
    pub fn merge(&self, other: &Schema) -> Schema;
}
```

---

## 6. 绑定器设计

### 6.1 绑定流程

```uml
@startuml

actor User

User -> Planner: SELECT id, name FROM users

Planner -> Binder: bind(statement)

Binder -> Catalog: get_table("users")

Catalog --> Binder: TableInfo

Binder -> TypeChecker: check_types(expression)

TypeChecker --> Binder: DataType

Binder --> Planner: LogicalPlan

Planner -> Optimizer: optimize(plan)

Optimizer --> Planner: OptimizedPlan

Planner -> PhysicalPlanner: create_physical(plan)

PhysicalPlanner --> User: PhysicalPlan

@enduml
```

### 6.2 列绑定

```rust
impl Binder {
    pub fn bind_column(&mut self, name: &str) -> Result<ColumnRef, SqlError> {
        // 1. 在当前作用域中查找
        for scope in self.scope.iter().rev() {
            if let Some(col) = scope.resolve_column(name) {
                return Ok(col);
            }
        }
        
        // 2. 在表引用中查找
        for table in &self.active_tables {
            if let Some(col) = table.resolve_column(name) {
                return Ok(col);
            }
        }
        
        Err(SqlError::ColumnNotFound(name.to_string()))
    }
}
```

---

## 7. 与代码对应检查

### 7.1 模块文件对应

| 设计内容 | 代码文件 | 状态 |
|----------|----------|------|
| 逻辑计划定义 | `logical_plan.rs` | ✅ 对应 |
| 物理计划定义 | `physical_plan.rs` | ✅ 对应 |
| 规划器逻辑 | `planner.rs` | ✅ 对应 |
| 优化器 | `optimizer.rs` | ✅ 对应 |

### 7.2 功能覆盖检查

| 功能 | 代码实现 | 状态 |
|------|----------|------|
| SELECT 计划 | ✅ | ✅ |
| INSERT 计划 | ✅ | ✅ |
| UPDATE 计划 | ✅ | ✅ |
| DELETE 计划 | ✅ | ✅ |
| JOIN 计划 | ✅ | ✅ |
| 聚合计划 | ✅ | ✅ |
| 排序计划 | ✅ | ✅ |
| 限制计划 | ✅ | ✅ |
| UNION 计划 | ✅ | ✅ |
| 子查询计划 | ✅ | ✅ |

---

## 8. 测试设计

### 8.1 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_select_plan() {
        let planner = Planner::new(catalog);
        let plan = planner.create_logical_plan(
            parse("SELECT id, name FROM users").unwrap()
        );
        assert!(plan.is_ok());
    }
    
    #[test]
    fn test_join_plan() {
        let planner = Planner::new(catalog);
        let plan = planner.create_logical_plan(
            parse("SELECT * FROM a JOIN b ON a.id = b.a_id").unwrap()
        );
        assert!(plan.is_ok());
    }
    
    #[test]
    fn test_aggregate_plan() {
        let planner = Planner::new(catalog);
        let plan = planner.create_logical_plan(
            parse("SELECT COUNT(*), SUM(age) FROM users GROUP BY name").unwrap()
        );
        assert!(plan.is_ok());
    }
}
```

---

**文档版本历史**

| 版本 | 日期 | 作者 | 变更 |
|------|------|------|------|
| 1.0 | 2026-03-26 | OpenCode | 初始版本 |

**文档状态**: ✅ 已完成
