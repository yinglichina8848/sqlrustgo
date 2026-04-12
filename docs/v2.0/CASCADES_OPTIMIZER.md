# SQLRustGo 2.0 Cascades Optimizer 设计

> **版本**: v2.0
> **状态**: 架构设计
> **目标**: 工业级查询优化器（类似 SQL Server / CockroachDB / Greenplum / Orca）
> **更新日期**: 2026-03-11

---

## 一、Cascades Optimizer 总体架构

### 1.1 执行流程

```
SQL
  │
Parser
  │
Logical Plan
  │
Cascades Optimizer
  │
Physical Plan
  │
Execution Engine
```

### 1.2 Optimizer 内部结构

```
            Cascades Optimizer
                   │
        ┌──────────┼──────────┐
        │          │          │
      Memo      Rule Set   Cost Model
        │          │          │
        └──── Search Engine ──┘
```

### 1.3 核心组件

| 模块 | 作用 |
|------|------|
| Memo | 记录所有候选计划 |
| Rules | 规则变换 |
| Search Engine | 搜索最优计划 |
| Cost Model | 计算执行成本 |

---

## 二、Memo 数据结构

### 2.1 结构概览

```
Memo
 │
 ├── Group 1
 │     ├── LogicalExpr
 │     └── PhysicalExpr
 │
 ├── Group 2
 │     ├── LogicalExpr
 │     └── PhysicalExpr
 │
 └── Group 3
```

### 2.2 Group 说明

每个 Group 代表**等价表达式集合**。

例如：

```sql
SELECT *
FROM A
JOIN B
ON A.id = B.id
```

可能有：

```
Group1
 ├─ HashJoin(A,B)
 ├─ NestedLoop(A,B)
 └─ MergeJoin(A,B)
```

### 2.3 Rust 结构设计

```rust
/// Memo - Cascades 优化器的核心数据结构
/// 存储所有候选执行计划
pub struct Memo {
    pub groups: Vec<Group>,
}

impl Memo {
    pub fn new() -> Self {
        Self { groups: Vec::new() }
    }

    /// 将逻辑计划插入 Memo
    pub fn insert(&mut self, expr: LogicalExpr) -> GroupId {
        let group_id = GroupId(self.groups.len());
        self.groups.push(Group::new(group_id, expr));
        group_id
    }

    /// 获取等价表达式
    pub fn get_group(&self, id: GroupId) -> Option<&Group> {
        self.groups.get(id.0)
    }
}

/// Group - 等价表达式集合
pub struct Group {
    pub id: GroupId,
    pub logical_expressions: Vec<LogicalExpr>,
    pub physical_expressions: Vec<PhysicalExpr>,
    pub best_cost: Option<Cost>,
    pub best_plan: Option<PhysicalPlan>,
}

impl Group {
    pub fn new(id: GroupId, expr: LogicalExpr) -> Self {
        Self {
            id,
            logical_expressions: vec![expr],
            physical_expressions: Vec::new(),
            best_cost: None,
            best_plan: None,
        }
    }
}

/// Group ID - Group 的唯一标识
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GroupId(pub usize);
```

---

## 三、Logical Operators

### 3.1 定义

```rust
/// 逻辑算子 - 表示逻辑执行计划
#[derive(Clone, Debug, PartialEq)]
pub enum LogicalOperator {
    /// 表扫描
    Scan {
        table: String,
        columns: Vec<String>,
    },
    /// 过滤
    Filter {
        input: GroupId,
        predicate: Expression,
    },
    /// 连接
    Join {
        join_type: JoinType,
        left: GroupId,
        right: GroupId,
        condition: Expression,
    },
    /// 聚合
    Aggregate {
        input: GroupId,
        group_by: Vec<String>,
        aggregates: Vec<AggregateExpr>,
    },
    /// 投影
    Projection {
        input: GroupId,
        columns: Vec<String>,
    },
    /// 排序
    Sort {
        input: GroupId,
        order_by: Vec<OrderByExpr>,
    },
    /// 限制
    Limit {
        input: GroupId,
        limit: usize,
        offset: usize,
    },
}

/// 连接类型
#[derive(Clone, Debug, PartialEq)]
pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
    Cross,
}
```

### 3.2 示例

| Operator | SQL |
|----------|-----|
| LogicalScan | `SELECT * FROM table` |
| LogicalFilter | `WHERE id > 10` |
| LogicalJoin | `A JOIN B ON A.id = B.id` |
| LogicalAggregate | `GROUP BY dept` |

---

## 四、Physical Operators

### 4.1 定义

```rust
/// 物理算子 - 可执行的物理计划
#[derive(Clone, Debug, PartialEq)]
pub enum PhysicalOperator {
    /// 顺序扫描
    SeqScan {
        table: String,
        projection: Vec<String>,
        filters: Vec<Expression>,
    },
    /// 索引扫描
    IndexScan {
        table: String,
        index_name: String,
        range: IndexRange,
    },
    /// 哈希连接
    HashJoin {
        left: Box<PhysicalPlan>,
        right: Box<PhysicalPlan>,
        join_type: JoinType,
        condition: Expression,
        build_side: BuildSide,
    },
    /// 合并连接
    MergeJoin {
        left: Box<PhysicalPlan>,
        right: Box<PhysicalPlan>,
        join_type: JoinType,
        left_key: String,
        right_key: String,
    },
    /// 嵌套循环连接
    NestedLoopJoin {
        left: Box<PhysicalPlan>,
        right: Box<PhysicalPlan>,
        join_type: JoinType,
        condition: Option<Expression>,
    },
    /// 哈希聚合
    HashAggregate {
        input: Box<PhysicalPlan>,
        group_by: Vec<String>,
        aggregates: Vec<AggregateExpr>,
    },
    /// 排序聚合
    SortAggregate {
        input: Box<PhysicalPlan>,
        group_by: Vec<String>,
        aggregates: Vec<AggregateExpr>,
    },
    /// 投影
    Projection {
        input: Box<PhysicalPlan>,
        columns: Vec<String>,
    },
    /// 排序
    Sort {
        input: Box<PhysicalPlan>,
        order_by: Vec<OrderByExpr>,
    },
    /// 限制
    Limit {
        input: Box<PhysicalPlan>,
        limit: usize,
        offset: usize,
    },
}

/// 构建侧
#[derive(Clone, Debug, PartialEq)]
pub enum BuildSide {
    Left,
    Right,
}

/// 索引范围
#[derive(Clone, Debug, PartialEq)]
pub struct IndexRange {
    pub lower: Option<Value>,
    pub upper: Option<Value>,
    pub lower_inclusive: bool,
    pub upper_inclusive: bool,
}
```

---

## 五、Rule System

### 5.1 Rule Trait

```rust
/// 优化规则 trait
pub trait Rule: Send + Sync {
    /// 规则名称
    fn name(&self) -> &str;

    /// 模式匹配 - 返回可以应用此规则的表达式类型
    fn pattern(&self) -> Pattern;

    /// 应用规则 - 返回变换后的新表达式
    fn apply(&self, expr: &Expr, memo: &Memo) -> Vec<Expr>;

    /// 规则优先级
    fn priority(&self) -> usize {
        100
    }
}

/// 模式匹配
#[derive(Clone, Debug, PartialEq)]
pub struct Pattern {
    pub operator_type: OperatorType,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum OperatorType {
    Scan,
    Filter,
    Join,
    Aggregate,
    Projection,
    Sort,
    Limit,
}
```

### 5.2 规则类型

| 类型 | 示例 |
|------|------|
| Rewrite | Predicate Pushdown |
| Exploration | Join Reordering |
| Implementation | Logical → Physical |

### 5.3 Predicate Pushdown 示例

**原始 SQL**:
```sql
SELECT *
FROM A
JOIN B
WHERE A.id > 10
```

**优化前**:
```
Filter(A.id > 10)
   │
Join
 ├─ A
 └─ B
```

**优化后**:
```
Join
 ├─ Filter(A.id > 10)
 │   └─ A
 └─ B
```

### 5.4 规则实现示例

```rust
/// Predicate Pushdown 规则 - 将过滤条件下推到扫描
pub struct PredicatePushdownRule;

impl Rule for PredicatePushdownRule {
    fn name(&self) -> &str {
        "PredicatePushdown"
    }

    fn pattern(&self) -> Pattern {
        Pattern {
            operator_type: OperatorType::Filter,
        }
    }

    fn apply(&self, expr: &Expr, memo: &Memo) -> Vec<Expr> {
        // 实现下推逻辑
        // 1. 找到 Filter 的子节点
        // 2. 将 predicate 传递给子节点
        // 3. 返回优化后的表达式
        vec![]
    }

    fn priority(&self) -> usize {
        200 // 高优先级
    }
}

/// Join Reordering 规则 - 重新排列 Join 顺序
pub struct JoinReorderingRule;

impl Rule for JoinReorderingRule {
    fn name(&self) -> &str {
        "JoinReordering"
    }

    fn pattern(&self) -> Pattern {
        Pattern {
            operator_type: OperatorType::Join,
        }
    }

    fn apply(&self, expr: &Expr, memo: &Memo) -> Vec<Expr> {
        // 实现 Join 重排逻辑
        // 枚举所有可能的 Join 顺序
        // 使用 Cost Model 选择最优
        vec![]
    }

    fn priority(&self) -> usize {
        100 // 中等优先级
    }
}

/// Logical to Physical 规则 - 逻辑算子转换为物理算子
pub struct ImplementRule;

impl Rule for ImplementRule {
    fn name(&self) -> &str {
        "Implement"
    }

    fn pattern(&self) -> Pattern {
        Pattern {
            operator_type: OperatorType::Join, // 匹配所有需要实现的算子
        }
    }

    fn apply(&self, expr: &Expr, memo: &Memo) -> Vec<Expr> {
        // 为逻辑算子生成物理实现
        // 例如: LogicalJoin -> HashJoin, MergeJoin, NestedLoopJoin
        vec![]
    }

    fn priority(&self) -> usize {
        50 // 低优先级，最后应用
    }
}
```

### 5.5 Rule Set

```rust
/// 规则集合
pub struct RuleSet {
    rules: Vec<Box<dyn Rule>>,
}

impl RuleSet {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn add_rule(&mut self, rule: Box<dyn Rule>) {
        self.rules.push(rule);
    }

    /// 按优先级排序
    pub fn sort_by_priority(&mut self) {
        self.rules.sort_by(|a, b| a.priority().cmp(&b.priority()));
    }

    /// 获取所有规则
    pub fn rules(&self) -> &[Box<dyn Rule>] {
        &self.rules
    }

    /// 预定义规则集
    pub fn default_rules() -> Self {
        let mut set = Self::new();
        set.add_rule(Box::new(PredicatePushdownRule {}));
        set.add_rule(Box::new(ProjectionPushdownRule {}));
        set.add_rule(Box::new(JoinReorderingRule {}));
        set.add_rule(Box::new(ColumnPruningRule {}));
        set.add_rule(Box::new(ImplementRule {}));
        set.sort_by_priority();
        set
    }
}
```

---

## 六、Search Engine

### 6.1 Top-Down Search

```rust
/// Cascades 搜索引擎
pub struct SearchEngine {
    memo: Memo,
    rules: RuleSet,
    cost_model: CostModel,
}

impl SearchEngine {
    /// 优化逻辑计划
    pub fn optimize(&mut self, logical_plan: LogicalPlan) -> PhysicalPlan {
        // 1. 插入 LogicalPlan 到 Memo
        let root_group = self.memo.insert(logical_plan);

        // 2. 递归优化
        self.optimize_group(root_group);

        // 3. 返回最优物理计划
        self.memo
            .get_group(root_group)
            .and_then(|g| g.best_plan.clone())
            .unwrap_or_else(|| PhysicalPlan::empty())
    }

    fn optimize_group(&mut self, group_id: GroupId) {
        // 应用所有规则
        for rule in self.rules.rules() {
            let group = self.memo.get_group(group_id).unwrap();
            for expr in &group.logical_expressions {
                let alternatives = rule.apply(expr, &self.memo);
                for alt in alternatives {
                    self.memo.add_expression(group_id, alt);
                }
            }
        }

        // 实现物理算子
        self.implement_group(group_id);

        // 计算成本
        self.estimate_cost(group_id);
    }

    fn implement_group(&mut self, group_id: GroupId) {
        // 为每个逻辑表达式生成物理实现
        // 例如: LogicalJoin -> HashJoin, MergeJoin, NestedLoopJoin
    }

    fn estimate_cost(&mut self, group_id: GroupId) {
        // 使用 Cost Model 计算每个物理计划的成本
        // 选择成本最低的作为 best_plan
    }
}
```

---

## 七、Cost Model

### 7.1 成本结构

```rust
/// 执行成本
#[derive(Clone, Debug, PartialEq)]
pub struct Cost {
    /// CPU 成本
    pub cpu: f64,
    /// I/O 成本
    pub io: f64,
}

impl Cost {
    pub fn new(cpu: f64, io: f64) -> Self {
        Self { cpu, io }
    }

    /// 总成本
    pub fn total(&self) -> f64 {
        self.cpu + self.io
    }

    /// 加法
    pub fn add(&self, other: &Cost) -> Cost {
        Cost::new(self.cpu + other.cpu, self.io + other.io)
    }

    /// 乘法
    pub fn mul(&self, factor: f64) -> Cost {
        Cost::new(self.cpu * factor, self.io * factor)
    }
}

impl Default for Cost {
    fn default() -> Self {
        Self::new(0.0, 0.0)
    }
}

impl std::ops::Add for Cost {
    type Output = Cost;

    fn add(self, other: Cost) -> Cost {
        self.add(&other)
    }
}

impl std::ops::Mul<f64> for Cost {
    type Output = Cost;

    fn mul(self, factor: f64) -> Cost {
        self.mul(factor)
    }
}
```

### 7.2 成本计算公式

**Scan Cost**:
```
cost = rows * cpu_tuple_cost
```

示例: 100k rows → 100k cost

**Join Cost**:

Hash Join:
```
cost = build + probe
build = rows_left
probe = rows_right * hash_lookup_cost
```

### 7.3 Cost Model trait

```rust
/// Cost Model trait
pub trait CostModel: Send + Sync {
    /// 计算扫描成本
    fn scan_cost(&self, stats: &TableStats) -> Cost;

    /// 计算连接成本
    fn join_cost(
        &self,
        left_stats: &TableStats,
        right_stats: &TableStats,
        join_type: &JoinType,
    ) -> Cost;

    /// 计算聚合成本
    fn aggregate_cost(&self, input_stats: &TableStats, group_by: &[String]) -> Cost;

    /// 计算排序成本
    fn sort_cost(&self, input_stats: &TableStats) -> Cost;
}

/// 默认成本模型
pub struct DefaultCostModel {
    cpu_tuple_cost: f64,
    io_page_cost: f64,
}

impl DefaultCostModel {
    pub fn new() -> Self {
        Self {
            cpu_tuple_cost: 1.0,
            io_page_cost: 10.0,
        }
    }
}

impl Default for DefaultCostModel {
    fn default() -> Self {
        Self::new()
    }
}

impl CostModel for DefaultCostModel {
    fn scan_cost(&self, stats: &TableStats) -> Cost {
        Cost::new(
            stats.row_count as f64 * self.cpu_tuple_cost,
            (stats.row_count as f64 / 1000.0) * self.io_page_cost,
        )
    }

    fn join_cost(
        &self,
        left_stats: &TableStats,
        right_stats: &TableStats,
        _join_type: &JoinType,
    ) -> Cost {
        let build = left_stats.row_count as f64 * self.cpu_tuple_cost;
        let probe = right_stats.row_count as f64 * self.cpu_tuple_cost;
        Cost::new(build + probe, 0.0)
    }

    fn aggregate_cost(&self, input_stats: &TableStats, _group_by: &[String]) -> Cost {
        Cost::new(
            input_stats.row_count as f64 * self.cpu_tuple_cost,
            0.0,
        )
    }

    fn sort_cost(&self, input_stats: &TableStats) -> Cost {
        Cost::new(
            input_stats.row_count as f64 * self.cpu_tuple_cost * 2.0,
            0.0,
        )
    }
}
```

---

## 八、Statistics System

### 8.1 表统计信息

```rust
/// 表统计信息
#[derive(Clone, Debug)]
pub struct TableStats {
    /// 行数
    pub row_count: usize,
    /// 列统计信息
    pub column_stats: HashMap<String, ColumnStats>,
}

impl TableStats {
    pub fn new(row_count: usize) -> Self {
        Self {
            row_count,
            column_stats: HashMap::new(),
        }
    }

    pub fn with_column(mut self, name: &str, stats: ColumnStats) -> Self {
        self.column_stats.insert(name.to_string(), stats);
        self
    }
}

/// 列统计信息
#[derive(Clone, Debug)]
pub struct ColumnStats {
    /// NDV (Number of Distinct Values)
    pub ndv: usize,
    /// 最小值
    pub min: Option<Value>,
    /// 最大值
    pub max: Option<Option<Value>>,
    /// 空值比例
    pub null_fraction: f64,
    /// 列基数 (row_count / ndv)
    pub cardinality: f64,
}

impl ColumnStats {
    pub fn new(ndv: usize) -> Self {
        Self {
            ndv,
            min: None,
            max: None,
            null_fraction: 0.0,
            cardinality: 0.0,
        }
    }

    pub fn with_range(mut self, min: Value, max: Value) -> Self {
        self.min = Some(min);
        self.max = Some(Some(max));
        self
    }

    pub fn with_null_fraction(mut self, fraction: f64) -> Self {
        self.null_fraction = fraction;
        self
    }

    pub fn calculate_cardinality(&mut self, row_count: usize) {
        self.cardinality = if self.ndv > 0 {
            row_count as f64 / self.ndv as f64
        } else {
            0.0
        };
    }
}
```

---

## 九、Join Order Optimization

### 9.1 问题说明

Join 顺序决定性能。

例如:
```sql
A JOIN B JOIN C
```

如果:
- A = 1M rows
- B = 10 rows
- C = 1M rows

**最优**: `(B JOIN A) JOIN C`

**最差**: `(A JOIN C) JOIN B`

### 9.2 实现

```rust
/// Join 重排算法
pub struct JoinReordering {
    /// 最大重排表数量
    max_tables: usize,
}

impl JoinReordering {
    pub fn new(max_tables: usize) -> Self {
        Self { max_tables }
    }

    /// 枚举所有可能的 Join 顺序
    pub fn enumerate(&self, tables: &[String]) -> Vec<Vec<String>> {
        if tables.len() <= self.max_tables {
            self.permute(tables)
        } else {
            // 对于大量表，使用贪心算法
            self.greedy_join_order(tables)
        }
    }

    fn permute(&self, tables: &[String]) -> Vec<Vec<String>> {
        let mut result = Vec::new();
        self.permute_recursive(tables, &mut Vec::new(), &mut result);
        result
    }

    fn permute_recursive(
        &self,
        remaining: &[String],
        current: &mut Vec<String>,
        result: &mut Vec<Vec<String>>,
    ) {
        if remaining.is_empty() {
            result.push(current.clone());
            return;
        }

        for i in 0..remaining.len() {
            let mut new_remaining = remaining.to_vec();
            let table = new_remaining.remove(i);
            current.push(table);
            self.permute_recursive(&new_remaining, current, result);
            current.pop();
        }
    }

    fn greedy_join_order(&self, tables: &[String]) -> Vec<Vec<String>> {
        // 贪心策略：每次选择最小的表先Join
        let mut result = tables.to_vec();
        result.sort_by(|a, b| a.cmp(b)); // 实际应该根据统计信息排序
        vec![result]
    }
}
```

---

## 十、Optimizer 执行流程

### 10.1 完整流程

```
1. Parse SQL
2. Build LogicalPlan
3. Insert into Memo
4. Apply Rules
5. Generate Alternatives
6. Estimate Cost
7. Choose Best Plan
8. Output PhysicalPlan
```

### 10.2 流程图

```
                SQL
                 │
              Parser
                 │
            LogicalPlan
                 │
           Cascades Optimizer
                 │
     ┌───────────┼───────────┐
     │           │           │
   Memo       Rule Set    Cost Model
     │           │           │
     └────── Search Engine ──┘
                 │
            PhysicalPlan
                 │
           Execution Engine
```

---

## 十一、性能收益

| 优化 | 收益 |
|------|------|
| Join Order | 10x |
| Predicate Pushdown | 5x |
| Projection Pushdown | 2x |

**整体**: 复杂查询 10x – 100x 提升

---

## 十二、开发阶段

### Phase 1 - 基础

- [ ] LogicalPlan 定义
- [ ] Memo 数据结构
- [ ] Rule Engine 框架

### Phase 2 - 实现

- [ ] Cost Model
- [ ] Statistics
- [ ] Physical Operators

### Phase 3 - 高级优化

- [ ] Join Reordering
- [ ] Subquery optimization
- [ ] Cost-based 选择

---

## 十三、变更历史

| 版本 | 日期 | 说明 |
|------|------|------|
| 1.0 | 2026-03-11 | 初始版本 |