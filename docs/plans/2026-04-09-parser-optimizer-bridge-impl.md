# Parser → Optimizer 桥接实现计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 建立 Parser index_hints 到 Optimizer RuleContext 的桥接链路，使 USE/FORCE/IGNORE INDEX hints 能正确影响优化决策。

**Architecture:** 扩展 RuleContext 支持从 SelectStatement 创建，修改 Optimizer/Rule trait 添加上下文参数，所有规则实现支持通过上下文访问 index_hints。

**Tech Stack:** Rust, sqlrustgo-parser, sqlrustgo-optimizer, sqlrustgo-server

---

## 任务概览

| # | 任务 | 状态 |
|---|------|------|
| 1 | 扩展 RuleContext - 添加 session_vars 和 from_select_stmt | pending |
| 2 | 修改 Optimizer trait - 添加上下文参数 + 默认方法 | pending |
| 3 | 修改 Rule trait - 添加上下文参数 + 默认方法 | pending |
| 4 | 更新 ConstantFolding 规则 | pending |
| 5 | 更新 PredicatePushdown 规则 | pending |
| 6 | 更新 ProjectionPruning 规则 | pending |
| 7 | 更新 ExpressionSimplification 规则 | pending |
| 8 | 更新 IndexSelect 规则 - 添加 hint 感知 | pending |
| 9 | 更新 JoinReordering 规则 | pending |
| 10 | 更新 DefaultOptimizer | pending |
| 11 | 更新 server endpoints 创建 RuleContext | pending |
| 12 | 添加集成测试 | pending |
| 13 | 验证所有测试通过 | pending |

---

## Task 1: 扩展 RuleContext

**Files:**
- Modify: `crates/optimizer/src/rules.rs:1137-1180`

**Step 1: 添加 session_vars 字段到 RuleContext**

找到 RuleContext 定义（约第 1137 行），添加 session_vars 字段：

```rust
/// 优化器运行时上下文
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
```

**Step 2: 更新构造函数**

更新 `RuleContext::new()`:
```rust
pub fn new() -> Self {
    Self {
        index_hints: Vec::new(),
        session_vars: HashMap::new(),
        enable_rule_trace: false,
        depth: 0,
        rules_applied: 0,
        continue_optimization: true,
    }
}
```

**Step 3: 添加 from_select_stmt 工厂方法**

在 RuleContext impl 块中添加：
```rust
/// 从 Parser 的 SelectStatement 创建 RuleContext（Parser → Optimizer 桥接点）
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
```

**Step 4: 添加 use 语句**

在文件顶部添加：
```rust
use std::collections::HashMap;
use sqlrustgo_types::Value;
```

---

## Task 2: 修改 Optimizer trait

**Files:**
- Modify: `crates/optimizer/src/lib.rs:39-60`

**Step 1: 修改 Optimizer trait 定义**

找到 Optimizer trait（约第 39 行），修改为：

```rust
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

---

## Task 3: 修改 Rule trait

**Files:**
- Modify: `crates/optimizer/src/lib.rs:46-70`

**Step 1: 修改 Rule trait 定义**

找到 Rule trait，修改为：

```rust
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

---

## Task 4-9: 更新各规则实现

每个规则需要修改 apply 方法签名并添加向后兼容方法。

**Files:**
- Modify: `crates/optimizer/src/rules.rs` (所有规则实现)

### Task 4: ConstantFolding

约第 1360 行，找到 impl Rule for ConstantFolding，修改为：

```rust
impl Rule<Plan> for ConstantFolding {
    fn name(&self) -> &str {
        "ConstantFolding"
    }

    fn apply(&self, plan: &mut Plan, _ctx: &mut RuleContext) -> bool {
        // 现有逻辑不变
        self.fold_constants(plan)
    }
    
    fn apply_without_context(&self, plan: &mut Plan) -> bool {
        self.fold_constants(plan)
    }
}
```

### Task 5: PredicatePushdown

约第 1420 行，修改 apply 签名。

### Task 6: ProjectionPruning

约第 1480 行，修改 apply 签名。

### Task 7: ExpressionSimplification

约第 1540 行，修改 apply 签名。

### Task 8: IndexSelect (重要 - 添加 hint 感知)

约第 1600 行，修改为：

```rust
impl Rule<Plan> for IndexSelect {
    fn name(&self) -> &str {
        "IndexSelect"
    }

    fn apply(&self, plan: &mut Plan, ctx: &mut RuleContext) -> bool {
        // 检查 index hints
        if !self.should_use_index_for_context(ctx) {
            return false;
        }
        self.transform_to_index_scan(plan)
    }
    
    fn apply_without_context(&self, plan: &mut Plan) -> bool {
        self.transform_to_index_scan(plan)
    }
}
```

添加辅助方法：
```rust
impl IndexSelect {
    /// 判断是否应该使用索引（考虑 index hints）
    pub fn should_use_index_for_context(&self, ctx: &RuleContext) -> bool {
        // 如果没有 hints，使用原有逻辑
        if ctx.index_hints.is_empty() {
            return true;
        }
        
        // 遍历 hints 检查是否有针对此表的
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
        // 没有匹配的 hint，使用原有逻辑
        true
    }
}
```

### Task 9: JoinReordering

约第 1700 行，修改 apply 签名。

---

## Task 10: 更新 DefaultOptimizer

**Files:**
- Modify: `crates/optimizer/src/lib.rs:80-150`

**Step 1: 修改 optimize 方法**

找到 DefaultOptimizer::optimize，修改为：

```rust
impl Optimizer for DefaultOptimizer {
    fn optimize(&mut self, plan: &mut dyn std::any::Any, ctx: &mut RuleContext) -> OptimizerResult<()> {
        // 现有逻辑，但在调用 rule.apply 时传入 ctx
        let plan = plan.downcast_mut::<Plan>().map_err(|_| {
            OptimizerError::DowncastError("Failed to downcast plan".to_string())
        })?;

        let mut changed = true;
        let mut iterations = 0;
        
        while changed && iterations < self.max_iterations {
            changed = false;
            
            for rule in &self.rules {
                if self.is_disabled(&rule.name()) {
                    continue;
                }
                
                if rule.apply(plan, ctx) {
                    changed = true;
                    ctx.rules_applied += 1;
                }
            }
            
            iterations += 1;
        }
        
        Ok(())
    }
}
```

---

## Task 11: 更新 server endpoints

**Files:**
- Modify: `crates/server/src/openclaw_endpoints.rs:817-857`
- Modify: `crates/server/src/teaching_endpoints.rs:503-600`

**Step 1: 在 openclaw_endpoints.rs 添加 RuleContext 创建**

在 `Statement::Select` 分支中，在调用执行前创建 RuleContext：

```rust
sqlrustgo_parser::Statement::Select(select) => {
    // ... 现有验证代码 ...
    
    // 创建 RuleContext（Parser → Optimizer 桥接）
    let rule_context = sqlrustgo_optimizer::RuleContext::from_select_stmt(&select);
    
    // 如果有 index hints，使用 optimizer
    if !rule_context.index_hints.is_empty() {
        // TODO: 调用 optimizer.optimize_with_context()
    }
    
    // ... 现有执行代码 ...
}
```

**Step 2: 添加依赖**

在文件顶部添加：
```rust
use sqlrustgo_optimizer::{Optimizer, DefaultOptimizer, RuleContext};
```

---

## Task 12: 添加集成测试

**Files:**
- Create: `crates/optimizer/src/tests/rule_context_tests.rs`

```rust
#[cfg(test)]
mod rule_context_tests {
    use super::*;
    use sqlrustgo_parser::{SelectStatement, IndexHint, IndexHintType};
    
    #[test]
    fn test_rule_context_from_select_stmt_with_hints() {
        let mut select = SelectStatement {
            table: "users".to_string(),
            // ... 其他字段 ...
            index_hints: vec![
                IndexHint {
                    hint_type: IndexHintType::UseIndex,
                    index_names: vec!["idx_id".to_string()],
                },
            ],
        };
        
        let ctx = RuleContext::from_select_stmt(&select);
        assert_eq!(ctx.index_hints.len(), 1);
        assert_eq!(ctx.index_hints[0].hint_type, IndexHintType::UseIndex);
    }
    
    #[test]
    fn test_index_select_respects_use_hint() {
        let rule = IndexSelect::new().with_index("users", "idx_id");
        let mut ctx = RuleContext::from_select_stmt(&SelectStatement {
            table: "users".to_string(),
            index_hints: vec![IndexHint {
                hint_type: IndexHintType::UseIndex,
                index_names: vec!["idx_id".to_string()],
            }],
            // ... 其他必需字段 ...
        });
        
        // IndexSelect 应该使用索引
        assert!(rule.should_use_index_for_context(&ctx));
    }
    
    #[test]
    fn test_index_select_respects_ignore_hint() {
        let rule = IndexSelect::new().with_index("users", "idx_id");
        let mut ctx = RuleContext::from_select_stmt(&SelectStatement {
            table: "users".to_string(),
            index_hints: vec![IndexHint {
                hint_type: IndexHintType::IgnoreIndex,
                index_names: vec!["idx_id".to_string()],
            }],
            // ... 其他必需字段 ...
        });
        
        // IndexSelect 应该忽略索引
        assert!(!rule.should_use_index_for_context(&ctx));
    }
}
```

---

## Task 13: 验证测试

**Step 1: 运行 optimizer 测试**

```bash
cargo test -p sqlrustgo-optimizer --lib 2>&1 | tail -20
```

**Step 2: 运行 parser 测试**

```bash
cargo test -p sqlrustgo-parser --lib 2>&1 | tail -10
```

**Step 3: 运行 workspace 检查**

```bash
cargo check --workspace 2>&1 | tail -10
```

**Step 4: 提交代码**

```bash
git add -A && git commit -m "feat(optimizer): add Parser → Optimizer bridge with RuleContext

- Extend RuleContext with session_vars and from_select_stmt
- Update Optimizer/Rule traits with context parameter
- IndexSelect respects USE/IGNORE INDEX hints
- All rules support backward-compatible apply_without_context"
```
