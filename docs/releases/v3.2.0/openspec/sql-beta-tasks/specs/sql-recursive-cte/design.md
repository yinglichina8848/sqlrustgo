# SQL-1: RECURSIVE CTE Implementation Design

## Context

RECURSIVE CTE 允许查询引用自身，实现递归数据遍历。SQL 语法：
```sql
WITH RECURSIVE cte AS (
  SELECT base_case UNION ALL
  SELECT recursive_case FROM cte WHERE condition
)
SELECT * FROM cte;
```

## Current State

### Parser
- ✅ `WITH RECURSIVE` 语法已支持
- ✅ `WithClause { recursive, ctes }` AST 节点已定义

### Planner
- ✅ `LogicalPlan::With` 处理
- ✅ `create_physical_plan_with_cte_ctx` CTE 上下文传播

### Executor
- ⚠️ `execute_cte_subquery` 存在
- ❌ 递归执行有 bug（UNION ALL 处理问题）

## Design

### 1. Recursive CTE Execution

递归 CTE 执行使用迭代方式：

```
1. 执行 anchor member (base case)
2. 将结果加入 working table
3. 执行 recursive member，使用 working table
4. 如果 recursive member 返回新行，加入 working table，重复步骤 3
5. 当 recursive member 返回空，停止
6. 返回所有行的并集
```

### 2. Fix evaluate_binary_op

修复 `evaluate_binary_op` 以支持算术运算符（+、-、*、/）：

```rust
fn evaluate_binary_op(
    left: &Value,
    op: &Operator,
    right: &Value,
) -> Result<Value, String> {
    match op {
        Operator::Plus => add(left, right),
        Operator::Minus => subtract(left, right),
        Operator::Multiply => multiply(left, right),
        Operator::Divide => divide(left, right),
        // ... other operators
    }
}
```

### 3. UNION ALL 处理

递归 CTE 使用 UNION ALL 连接结果：

```rust
match set_expr {
    SetExpr::SetOperation {
        op: SetOperator::UnionAll,
        left,
        right,
    } => {
        let left_records = execute_cte_subquery(left, ctx)?;
        let right_records = execute_cte_subquery(right, ctx)?;
        Ok(concat_records(left_records, right_records))
    }
    // ...
}
```

## Implementation Plan

### Phase 1: Fix evaluate_binary_op
- 在 `executor/src/stored_proc.rs` 添加算术运算符支持
- 单元测试验证

### Phase 2: Fix UNION ALL in CTE
- 确保 UNION ALL 正确连接左右结果集
- 验证递归终止条件

### Phase 3: Integration Tests
- 测试简单递归: `WITH RECURSIVE cte AS (SELECT 1 UNION ALL SELECT n+1 FROM cte WHERE n < 10) SELECT * FROM cte`
- 测试多重递归 CTE
- 测试递归深度限制

## Files to Modify

- `crates/executor/src/stored_proc.rs`: 修复 evaluate_binary_op 和 UNION ALL 处理
- `crates/executor/tests/cte_tests.rs`: 添加 CTE 测试

## Verification

```bash
cargo test -p sqlrustgo-executor -- cte
cargo test -p sqlrustgo-executor -- recursive
```
