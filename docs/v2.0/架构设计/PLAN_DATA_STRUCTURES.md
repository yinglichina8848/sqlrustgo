# LogicalPlan / PhysicalPlan 数据结构设计

> 版本：v1.0
> 日期：2026-02-18
> 设计目标：企业级可扩展

---

## 一、LogicalPlan 设计

### 1.1 核心结构

```rust
#[derive(Clone, Debug, PartialEq)]
pub enum LogicalPlan {
    Projection {
        input: Box<LogicalPlan>,
        expr: Vec<Expr>,
        schema: Schema,
    },
    Filter {
        input: Box<LogicalPlan>,
        predicate: Expr,
    },
    Aggregate {
        input: Box<LogicalPlan>,
        group_expr: Vec<Expr>,
        aggr_expr: Vec<Expr>,
        schema: Schema,
    },
    Join {
        left: Box<LogicalPlan>,
        right: Box<LogicalPlan>,
        join_type: JoinType,
        on: Vec<(Expr, Expr)>,
        filter: Option<Expr>,
        schema: Schema,
    },
    TableScan {
        table_name: String,
        source: Arc<dyn TableSource>,
        projection: Option<Vec<usize>>,
        filters: Vec<Expr>,
        limit: Option<usize>,
    },
    Sort {
        input: Box<LogicalPlan>,
        expr: Vec<SortExpr>,
    },
    Limit {
        input: Box<LogicalPlan>,
        n: usize,
    },
    Values {
        values: Vec<Vec<Expr>>,
        schema: Schema,
    },
    EmptyRelation {
        produce_one_row: bool,
        schema: Schema,
    },
    Subquery {
        subquery: Box<LogicalPlan>,
    },
    Union {
        inputs: Vec<LogicalPlan>,
        schema: Schema,
    },
}
```

### 1.2 表达式设计

```rust
#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Column(Column),
    Literal(ScalarValue),
    BinaryExpr {
        left: Box<Expr>,
        op: Operator,
        right: Box<Expr>,
    },
    UnaryExpr {
        op: Operator,
        expr: Box<Expr>,
    },
    AggregateFunction {
        func: AggregateFunction,
        args: Vec<Expr>,
        distinct: bool,
        filter: Option<Box<Expr>>,
    },
    ScalarFunction {
        func: String,
        args: Vec<Expr>,
    },
    Case {
        expr: Option<Box<Expr>>,
        when_then_expr: Vec<(Box<Expr>, Box<Expr>)>,
        else_expr: Option<Box<Expr>>,
    },
    Cast {
        expr: Box<Expr>,
        data_type: DataType,
    },
    IsNull {
        expr: Box<Expr>,
        negated: bool,
    },
    IsTrue {
        expr: Box<Expr>,
        negated: bool,
    },
    Negative {
        expr: Box<Expr>,
    },
    InList {
        expr: Box<Expr>,
        list: Vec<Expr>,
        negated: bool,
    },
    Between {
        expr: Box<Expr>,
        negated: bool,
        low: Box<Expr>,
        high: Box<Expr>,
    },
    Alias {
        expr: Box<Expr>,
        name: String,
    },
    Wildcard,
    QualifiedWildcard {
        qualifier: String,
    },
}
```

### 1.3 辅助类型

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct Column {
    pub relation: Option<String>,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
    Cross,
    LeftSemi,
    LeftAnti,
    RightSemi,
    RightAnti,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SortExpr {
    pub expr: Expr,
    pub asc: bool,
    pub nulls_first: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AggregateFunction {
    Count,
    Sum,
    Avg,
    Min,
    Max,
    Variance,
    Stddev,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Operator {
    Eq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulo,
    And,
    Or,
    Not,
    Like,
    NotLike,
    IsDistinctFrom,
    IsNotDistinctFrom,
    RegexMatch,
    RegexNotMatch,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    BitwiseShiftLeft,
    BitwiseShiftRight,
    StringConcat,
}
```

### 1.4 Schema 定义

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct Schema {
    pub fields: Vec<Field>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Field {
    pub name: String,
    pub data_type: DataType,
    pub nullable: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DataType {
    Boolean,
    Int8,
    Int16,
    Int32,
    Int64,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Float32,
    Float64,
    Utf8,
    LargeUtf8,
    Binary,
    LargeBinary,
    Date32,
    Date64,
    Timestamp(TimeUnit, Option<String>),
    Time32(TimeUnit),
    Time64(TimeUnit),
    Duration(TimeUnit),
    Interval(IntervalUnit),
    List(Box<DataType>),
    Struct(Vec<Field>),
    Decimal(usize, usize),
}
```

---

## 二、PhysicalPlan 设计

### 2.1 核心 trait

```rust
pub trait PhysicalPlan: Send + Sync + Debug {
    fn schema(&self) -> &Schema;
    fn execute(&self, partition: usize) -> Result<Box<dyn ExecutionPlan>>;
    fn children(&self) -> Vec<Arc<dyn PhysicalPlan>>;
    fn with_new_children(&self, children: Vec<Arc<dyn PhysicalPlan>>) -> Result<Arc<dyn PhysicalPlan>>;
    fn output_partitioning(&self) -> Partitioning;
    fn required_input_ordering(&self) -> Vec<Option<Vec<SortExpr>>>;
    fn output_ordering(&self) -> Option<Vec<SortExpr>>;
}
```

### 2.2 具体实现

#### HashJoinExec

```rust
#[derive(Debug)]
pub struct HashJoinExec {
    pub left: Arc<dyn PhysicalPlan>,
    pub right: Arc<dyn PhysicalPlan>,
    pub on: Vec<(Column, Column)>,
    pub filter: Option<JoinFilter>,
    pub join_type: JoinType,
    pub mode: JoinMode,
    pub schema: Schema,
}

impl PhysicalPlan for HashJoinExec {
    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn execute(&self, partition: usize) -> Result<Box<dyn ExecutionPlan>> {
        let left_stream = self.left.execute(partition)?;
        let right_stream = self.right.execute(partition)?;
        
        Ok(Box::new(HashJoinStream::new(
            left_stream,
            right_stream,
            self.on.clone(),
            self.join_type,
        )))
    }

    fn children(&self) -> Vec<Arc<dyn PhysicalPlan>> {
        vec![self.left.clone(), self.right.clone()]
    }
}
```

#### SeqScanExec

```rust
#[derive(Debug)]
pub struct SeqScanExec {
    pub table_name: String,
    pub source: Arc<dyn TableSource>,
    pub projection: Option<Vec<usize>>,
    pub filters: Vec<Expr>,
    pub limit: Option<usize>,
    pub schema: Schema,
}

impl PhysicalPlan for SeqScanExec {
    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn execute(&self, partition: usize) -> Result<Box<dyn ExecutionPlan>> {
        let stream = self.source.scan(
            self.projection.as_ref(),
            &self.filters,
            self.limit,
        )?;
        Ok(stream)
    }

    fn children(&self) -> Vec<Arc<dyn PhysicalPlan>> {
        vec![]
    }
}
```

#### ProjectionExec

```rust
#[derive(Debug)]
pub struct ProjectionExec {
    pub input: Arc<dyn PhysicalPlan>,
    pub expr: Vec<Arc<dyn PhysicalExpr>>,
    pub schema: Schema,
}

impl PhysicalPlan for ProjectionExec {
    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn execute(&self, partition: usize) -> Result<Box<dyn ExecutionPlan>> {
        let input_stream = self.input.execute(partition)?;
        Ok(Box::new(ProjectionStream::new(
            input_stream,
            self.expr.clone(),
        )))
    }

    fn children(&self) -> Vec<Arc<dyn PhysicalPlan>> {
        vec![self.input.clone()]
    }
}
```

#### FilterExec

```rust
#[derive(Debug)]
pub struct FilterExec {
    pub input: Arc<dyn PhysicalPlan>,
    pub predicate: Arc<dyn PhysicalExpr>,
}

impl PhysicalPlan for FilterExec {
    fn schema(&self) -> &Schema {
        self.input.schema()
    }

    fn execute(&self, partition: usize) -> Result<Box<dyn ExecutionPlan>> {
        let input_stream = self.input.execute(partition)?;
        Ok(Box::new(FilterStream::new(
            input_stream,
            self.predicate.clone(),
        )))
    }

    fn children(&self) -> Vec<Arc<dyn PhysicalPlan>> {
        vec![self.input.clone()]
    }
}
```

#### AggregateExec

```rust
#[derive(Debug)]
pub struct AggregateExec {
    pub input: Arc<dyn PhysicalPlan>,
    pub group_expr: Vec<Arc<dyn PhysicalExpr>>,
    pub aggr_expr: Vec<Arc<dyn AggregateExpr>>,
    pub schema: Schema,
    pub mode: AggregateMode,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AggregateMode {
    Partial,
    Final,
    FinalPartitioned,
}

impl PhysicalPlan for AggregateExec {
    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn execute(&self, partition: usize) -> Result<Box<dyn ExecutionPlan>> {
        let input_stream = self.input.execute(partition)?;
        Ok(Box::new(AggregateStream::new(
            input_stream,
            self.group_expr.clone(),
            self.aggr_expr.clone(),
            self.mode.clone(),
        )))
    }

    fn children(&self) -> Vec<Arc<dyn PhysicalPlan>> {
        vec![self.input.clone()]
    }
}
```

### 2.3 物理表达式

```rust
pub trait PhysicalExpr: Send + Sync + Debug {
    fn data_type(&self, schema: &Schema) -> Result<DataType>;
    fn nullable(&self, schema: &Schema) -> Result<bool>;
    fn evaluate(&self, batch: &RecordBatch) -> Result<ColumnarValue>;
}

pub trait AggregateExpr: Send + Sync + Debug {
    fn field(&self) -> Result<Field>;
    fn create_accumulator(&self) -> Result<Box<dyn Accumulator>>;
}

pub trait Accumulator: Send + Sync + Debug {
    fn update_batch(&mut self, values: &[ArrayRef]) -> Result<()>;
    fn merge_batch(&mut self, values: &[ArrayRef]) -> Result<()>;
    fn evaluate(&self) -> Result<ScalarValue>;
}
```

---

## 三、执行流程

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          执行流程                                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   LogicalPlan                                                               │
│       │                                                                      │
│       ▼                                                                      │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                     Optimizer Rules                                  │   │
│   │  ├── PredicatePushdown                                              │   │
│   │  ├── ProjectionPruning                                              │   │
│   │  ├── JoinReorder                                                    │   │
│   │  └── ConstantFolding                                                │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│       │                                                                      │
│       ▼                                                                      │
│   Optimized LogicalPlan                                                     │
│       │                                                                      │
│       ▼                                                                      │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                     Physical Planner                                 │   │
│   │  ├── 选择 Join 策略 (HashJoin vs NestedLoop)                        │   │
│   │  ├── 选择 Scan 策略 (IndexScan vs SeqScan)                          │   │
│   │  └── 选择 Aggregate 策略                                            │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│       │                                                                      │
│       ▼                                                                      │
│   PhysicalPlan                                                              │
│       │                                                                      │
│       ▼                                                                      │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                     Execution Engine                                 │   │
│   │  ├── execute() → RecordBatch                                        │   │
│   │  └── 流式处理                                                        │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│       │                                                                      │
│       ▼                                                                      │
│   QueryResult                                                               │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 四、设计原则

### 4.1 LogicalPlan 原则

| 原则 | 说明 |
|:-----|:-----|
| 声明式 | 描述"做什么"，不描述"怎么做" |
| 可序列化 | 支持网络传输 |
| 可优化 | 可被优化器变换 |
| 与物理无关 | 不涉及具体执行策略 |

### 4.2 PhysicalPlan 原则

| 原则 | 说明 |
|:-----|:-----|
| 可执行 | 可以直接执行 |
| 流式处理 | 支持 RecordBatch 流 |
| 可并行 | 支持分区执行 |
| 可向量化 | 支持批量处理 |

---

## 五、扩展点

### 5.1 新增 LogicalPlan 节点

```rust
pub enum LogicalPlan {
    Window {
        input: Box<LogicalPlan>,
        window_expr: Vec<Expr>,
        schema: Schema,
    },
    Repartition {
        input: Box<LogicalPlan>,
        partitioning: Partitioning,
    },
    Explain {
        plan: Box<LogicalPlan>,
        verbose: bool,
    },
}
```

### 5.2 新增 PhysicalPlan 节点

```rust
pub struct SortExec {
    pub input: Arc<dyn PhysicalPlan>,
    pub expr: Vec<PhysicalSortExpr>,
}

pub struct LimitExec {
    pub input: Arc<dyn PhysicalPlan>,
    pub n: usize,
}

pub struct UnionExec {
    pub inputs: Vec<Arc<dyn PhysicalPlan>>,
}
```

---

*本文档由 TRAE (GLM-5.0) 创建*
