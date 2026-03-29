# SQLRustGo v2.0.0 Planner 模块详细设计

> **版本**: v2.0.0
> **日期**: 2026-03-29
> **模块**: sqlrustgo-planner

---

## 1. 模块概述

Planner 模块负责将 AST 转换为逻辑计划，进行类型检查和绑定。

## 2. 核心组件

### 2.1 LogicalPlanner

```rust
pub struct LogicalPlanner {
    catalog: Arc<dyn Catalog>,
    next_id: u64,
}

impl LogicalPlanner {
    pub fn new(catalog: Arc<dyn Catalog>) -> Self;
    pub fn plan(&self, stmt: Statement) -> Result<LogicalPlan>;
    pub fn plan_select(&self, stmt: SelectStatement) -> Result<LogicalPlan>;
    pub fn plan_insert(&self, stmt: InsertStatement) -> Result<LogicalPlan>;
    pub fn plan_update(&self, stmt: UpdateStatement) -> Result<LogicalPlan>;
    pub fn plan_delete(&self, stmt: DeleteStatement) -> Result<LogicalPlan>;
    pub fn plan_copy(&self, stmt: CopyStatement) -> Result<LogicalPlan>;
}
```

### 2.2 Binder

```rust
pub struct Binder {
    catalog: Arc<dyn Catalog>,
    scopes: Vec<Scope>,
    parameters: Vec<Parameter>,
}

impl Binder {
    pub fn new(catalog: Arc<dyn Catalog>) -> Self;
    pub fn bind_expr(&mut self, expr: &Expr) -> Result<BoundExpr>;
    pub fn bind_select(&mut self, stmt: &SelectStatement) -> Result<BoundSelect>;
    pub fn push_scope(&mut self, scope: Scope);
    pub fn pop_scope(&mut self);
}
```

### 2.3 TypeChecker

```rust
pub struct TypeChecker {
    casts: TypeCasts,
}

impl TypeChecker {
    pub fn new() -> Self;
    pub fn check_expr(&self, expr: &BoundExpr, schema: &Schema) -> Result<DataType>;
    pub fn common_type(&self, t1: &DataType, t2: &DataType) -> Result<DataType>;
    pub fn can_cast(&self, from: &DataType, to: &DataType) -> bool;
}
```

---

## 3. 逻辑计划节点

### 3.1 LogicalPlan 枚举

```rust
pub enum LogicalPlan {
    TableScan(TableScanPlan),
    Projection(ProjectionPlan),
    Selection(SelectionPlan),
    Aggregate(AggregatePlan),
    Window(WindowPlan),
    Join(JoinPlan),
    CrossJoin(CrossJoinPlan),
    Limit(LimitPlan),
    Sort(SortPlan),
    EmptyRelation,
    Subquery(SubqueryPlan),
    Copy(CopyPlan),
}
```

### 3.2 TableScanPlan

```rust
pub struct TableScanPlan {
    pub table_name: String,
    pub table_schema: SchemaRef,
    pub projection: Vec<usize>,
    pub filters: Vec<Expr>,
    pub fetch: Option<usize>,
}
```

### 3.3 AggregatePlan

```rust
pub struct AggregatePlan {
    pub input: Arc<LogicalPlan>,
    pub group_expr: Vec<Expr>,
    pub aggr_expr: Vec<AggregateFunction>,
    pub filter_expr: Option<Expr>,
}

pub enum AggregateFunction {
    Count(Option<Box<Expr>>),
    Sum(Box<Expr>),
    Avg(Box<Expr>),
    Min(Box<Expr>),
    Max(Box<Expr>),
    BitAnd(Box<Expr>),
    BitOr(Box<Expr>),
    BitXor(Box<Expr>),
    ArrayAgg(Box<Expr>),
}
```

---

## 4. BoundExpr 类型

```rust
pub struct BoundExpr {
    pub id: ExprId,
    pub expr: BoundExprEnum,
    pub data_type: DataType,
    pub nullable: bool,
}

pub enum BoundExprEnum {
    ColumnReference(ColumnReference),
    Literal(LiteralValue),
    BinaryExpr {
        left: Box<BoundExpr>,
        op: BinaryOperator,
        right: Box<BoundExpr>,
    },
    UnaryExpr {
        op: UnaryOperator,
        expr: Box<BoundExpr>,
    },
    ScalarFunction {
        name: String,
        args: Vec<BoundExpr>,
    },
    AggregateFunction(AggregateFunction),
    WindowFunction {
        func: WindowFunction,
        partition_by: Vec<BoundExpr>,
        order_by: Vec<OrderByExpr>,
        window_frame: Option<WindowFrame>,
    },
    Cast {
        expr: Box<BoundExpr>,
        data_type: DataType,
    },
    InList {
        expr: Box<BoundExpr>,
        list: Vec<BoundExpr>,
        negated: bool,
    },
    Between {
        expr: Box<BoundExpr>,
        low: Box<BoundExpr>,
        high: Box<BoundExpr>,
        negated: bool,
    },
    Subquery {
        subquery: Box<LogicalPlan>,
        exists: bool,
    },
}
```

---

## 5. EXPLAIN 扩展

### 5.1 EXPLAIN 支持

```sql
EXPLAIN [FORMAT (TEXT|JSON)] <statement>
EXPLAIN [ANALYZE] <statement>
EXPLAIN (FORMAT text) SELECT * FROM orders;
```

### 5.2 ExplainOptions

```rust
pub struct ExplainOptions {
    pub analyze: bool,
    pub verbose: bool,
    pub format: ExplainFormat,
}

pub enum ExplainFormat {
    Text,
    Json,
    Graphviz,
}
```

---

*文档生成日期: 2026-03-29*
*版本: v2.0.0*
