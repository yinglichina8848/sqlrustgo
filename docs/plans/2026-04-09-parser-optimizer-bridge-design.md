# Parser → Optimizer 桥接架构设计

**日期**: 2026-04-09
**状态**: Approved
**版本**: 1.0

## 1. 背景与目标

### 1.1 问题

当前 `Parser` 解析 SQL 后生成的 `SelectStatement` 包含 `index_hints` 字段，但该字段没有被传递到 `Optimizer`。导致用户指定的 `USE INDEX`、`FORCE INDEX`、`IGNORE INDEX` 提示无法生效。

### 1.2 目标

建立 **Parser → Optimizer** 的桥接链路：
1. 将 `SelectStatement.index_hints` 正确传递到 Optimizer 的 `RuleContext`
2. 所有优化规则都能通过 `RuleContext` 访问 index hints
3. 设计可扩展的上下文框架，支持未来更多优化提示

### 1.3 架构原则

```
数据流层级（必须遵守）:
Parser → AST (index_hints) → Server/Endpoint → Planner → Optimizer (RuleContext here) → Executor
```

- **RuleContext 创建位置 = Server/Endpoint 层**：index_hints 是 AST 元数据，不是 Optimizer 内部推导信息
- **不能放 Optimizer 内部**：会丢失 AST 信息
- **不能放 Planner 内部**：职责污染

## 2. 架构设计

### 2.1 数据流

```
SQL: SELECT * FROM t USE INDEX (idx) WHERE id = 1
         ↓
Parser: SelectStatement { index_hints: [IndexHint { hint_type: UseIndex, index_names: ["idx"] }] }
         ↓
Server/Endpoint: build_rule_context(select_stmt)
         ↓
RuleContext { index_hints: Vec<IndexHint>, session_vars: HashMap, ... }
         ↓
Planner: LogicalPlan
         ↓
Optimizer.optimize(plan, &mut RuleContext)
         ↓
Executor
```

### 2.2 RuleContext 扩展设计

```rust
// optimizer/src/rules.rs

/// 优化器运行时上下文 - 持有优化所需的全部上下文信息
#[derive(Debug, Clone)]
pub struct RuleContext {
    /// Index hints from SQL query (USE/FORCE/IGNORE INDEX)
    pub index_hints: Vec<IndexHint>,
    /// Session optimizer variables (future extension)
    pub session_vars: HashMap<String, sqlrustgo_types::Value>,
    /// 是否启用规则追踪调试
    pub enable_rule_trace: bool,
    // === 内部状态 ===
    pub depth: usize,
    pub rules_applied: usize,
    pub continue_optimization: bool,
}

impl RuleContext {
    /// 创建空的 RuleContext
    pub fn new() -> Self { ... }
    
    /// 从 SelectStatement 创建 RuleContext（Parser → Optimizer 桥接点）
    pub fn from_select_stmt(stmt: &sqlrustgo_parser::SelectStatement) -> Self {
        Self {
            index_hints: stmt.index_hints.clone(),
            session_vars: HashMap::new(),
            enable_rule_trace: false,
            depth: 0,
            rules_applied: 0,
            continue_optimization: true,
        }
    }
}
```

### 2.3 Optimizer Trait 新签名

```rust
// optimizer/src/lib.rs

/// 优化器 trait - 支持传入 RuleContext
pub trait Optimizer {
    /// 使用上下文优化计划
    fn optimize(&mut self, plan: &mut dyn std::any::Any, ctx: &mut RuleContext) -> OptimizerResult<()>;
    
    /// 向后兼容的默认方法（不带上下文）
    fn optimize_without_context(&mut self, plan: &mut dyn std::any::Any) -> OptimizerResult<()> {
        let mut ctx = RuleContext::new();
        self.optimize(plan, &mut ctx)
    }
}
```

### 2.4 Rule Trait 新签名

```rust
// optimizer/src/lib.rs

/// 优化规则 trait - 支持访问 RuleContext
pub trait Rule<Plan> {
    fn name(&self) -> &str;
    
    /// 应用规则到计划，传入上下文以支持 index hints 等
    fn apply(&self, plan: &mut Plan, ctx: &mut RuleContext) -> bool;
    
    /// 向后兼容的默认方法
    fn apply_without_context(&self, plan: &mut Plan) -> bool {
        let mut ctx = RuleContext::new();
        self.apply(plan, &mut ctx)
    }
}
```

### 2.5 IndexSelect 规则集成

```rust
// optimizer/src/rules.rs - IndexSelect 实现

impl IndexSelect {
    /// 判断是否应该使用索引（考虑 index hints）
    pub fn should_use_index(&self, table: &str, ctx: &RuleContext) -> bool {
        // 1. 检查是否有该表的 index hints
        for hint in &ctx.index_hints {
            match hint.hint_type {
                IndexHintType::UseIndex | IndexHintType::ForceIndex => {
                    // 检查目标索引是否在 hint 列表中
                    if hint.index_names.iter().any(|idx| idx == self.index_name()) {
                        return true;
                    }
                }
                IndexHintType::IgnoreIndex => {
                    // 检查是否要忽略该索引
                    if hint.index_names.iter().any(|idx| idx == self.index_name()) {
                        return false;
                    }
                }
            }
        }
        // 2. 没有 hint 时，使用原有 CBO 逻辑
        self.cbo_selects_index()
    }
}
```

## 3. 改动范围

### 3.1 文件改动清单

| 文件 | 改动内容 |
|------|----------|
| `optimizer/src/rules.rs` | 扩展 RuleContext，添加 from_select_stmt |
| `optimizer/src/lib.rs` | 修改 Optimizer/Rule trait 签名 |
| `optimizer/src/*.rs` | 所有规则实现更新 |
| `server/src/openclaw_endpoints.rs` | 创建 RuleContext 并传入 |
| `server/src/teaching_endpoints.rs` | 同上 |

### 3.2 受影响的规则实现

需要更新以下规则的 `apply` 方法签名：

- `ConstantFolding`
- `PredicatePushdown`
- `ProjectionPruning`
- `ExpressionSimplification`
- `IndexSelect`
- `JoinReordering`

## 4. 向后兼容性

- **旧的 optimize() 调用**：自动调用 `optimize_without_context()`
- **旧的 Rule 实现**：自动调用 `apply_without_context()`
- **现有的测试**：继续工作，因为使用默认上下文

## 5. 扩展计划

未来可支持：

```rust
pub struct RuleContext {
    pub index_hints: Vec<IndexHint>,
    pub session_vars: HashMap<String, Value>,
    pub enable_rule_trace: bool,
    pub statistics: Option<Arc<dyn StatisticsProvider>>,
    pub cost_model: Option<Arc<dyn CostModel>>,
}
```

支持的优化提示：
- `USE INDEX` / `FORCE INDEX` / `IGNORE INDEX`
- `SET optimizer = 'cost-based'`
- `SET enable_rewrite = off`

## 6. 验收标准

1. ✅ `SELECT * FROM t USE INDEX (idx) WHERE id = 1` 能正确解析并传递到 Optimizer
2. ✅ IndexSelect 规则能根据 USE/IGNORE INDEX 调整决策
3. ✅ 所有现有测试通过
4. ✅ 向后兼容：现有代码无需改动

## 7. 实现顺序

1. 扩展 RuleContext（添加 from_select_stmt）
2. 修改 Optimizer trait（添加上下文参数 + 默认方法）
3. 修改 Rule trait（添加上下文参数 + 默认方法）
4. 更新所有规则实现
5. 更新 server endpoints
6. 添加集成测试
