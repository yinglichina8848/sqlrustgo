# Issue #837 Teaching Scenario Tests - 设计文档

## 目标
创建 30 个教学场景测试，使 v1.9.0 支持完整的 SQL 教学场景。

## 现状问题

| 问题 | 当前状态 | 影响 |
|------|---------|------|
| UPDATE SET 表达式 | 只支持 `column = value` | 无法测试 `balance = balance - 200` |
| SELECT 限定列名 | 不支持 `table.column` | 无法测试 `customers.name` |
| 子查询 | planner 已支持，parser 未解析 | 无法测试 `(SELECT ...)` |
| GROUP BY/HAVING | 基本支持 | 需验证 |
| 触发器 | 未实现 | 无法测试触发器 |

## 实现计划

### 阶段 1：增强解析器 (Priority: P0)

#### 1.1 Expression 枚举扩展
```rust
pub enum Expression {
    Literal(String),
    Identifier(String),
    BinaryOp(Box<Expression>, String, Box<Expression>),
    Wildcard,
    FunctionCall(String, Vec<Expression>),
    // 新增
    Subquery(Box<SelectStatement>),  // 子查询
    QualifiedColumn(String, String),  // table.column
}
```

#### 1.2 UPDATE SET 表达式支持
- 当前: `column = value`
- 目标: `column = column +/- value`, `column = expression`

修改 `parse_update` 中的 set_clauses 解析，使用 `parse_expression()` 替代简单值解析。

#### 1.3 SELECT 列解析增强
- 支持 `table.column` 限定列名
- 支持子查询 `(SELECT avg(salary) FROM employees)`
- 支持列别名 `column AS alias`

#### 1.4 触发器实现

**Parser 部分:**
```rust
// Statement 新增
Statement::CreateTrigger(CreateTriggerStatement),
Statement::DropTrigger(DropTriggerStatement),

// 结构体
pub struct CreateTriggerStatement {
    pub name: String,
    pub table_name: String,
    pub timing: TriggerTiming,  // BEFORE, AFTER
    pub event: TriggerEvent,    // INSERT, UPDATE, DELETE
    pub body: TriggerBody,
}

pub enum TriggerBody {
    Statement(Box<Statement>),
    // FOR EACH ROW 简单实现
}
```

**执行部分:**
- 在 `ExecutionEngine` 中添加触发器管理
- 在 INSERT/UPDATE/DELETE 执行前后触发相应触发器

### 阶段 2：创建测试 (Priority: P1)

创建 30 个 teaching scenario 测试:

#### 基础 CRUD (5 tests)
1. `test_teaching_insert_basic`
2. `test_teaching_select_basic`
3. `test_teaching_update_basic`
4. `test_teaching_delete_basic`
5. `test_teaching_transaction_basic`

#### 事务 (5 tests)
6. `test_teaching_transaction_commit`
7. `test_teaching_transaction_rollback`
8. `test_teaching_savepoint`
9. `test_teaching_transaction_isolation`
10. `test_teaching_deadlock_detection`

#### JOIN (5 tests)
11. `test_teaching_join_inner`
12. `test_teaching_join_left`
13. `test_teaching_join_right`
14. `test_teaching_join_full`
15. `test_teaching_complex_join`

#### 子查询 (4 tests)
16. `test_teaching_subquery_basic`
17. `test_teaching_subquery_in_where`
18. `test_teaching_subquery_in_select`
19. `test_teaching_correlated_subquery`

#### 聚合与分组 (4 tests)
20. `test_teaching_aggregate_count`
21. `test_teaching_aggregate_sum_avg`
22. `test_teaching_group_by`
23. `test_teaching_having`

#### 高级功能 (7 tests)
24. `test_teaching_view_basic`
25. `test_teaching_index_creation`
26. `test_teaching_foreign_key`
27. `test_teaching_trigger_before_insert`
28. `test_teaching_trigger_before_update`
29. `test_teaching_trigger_after_delete`
30. `test_teaching_stored_procedure_basic`

## 文件修改

| 文件 | 修改内容 |
|------|---------|
| `crates/parser/src/parser.rs` | Expression 扩展, 解析器增强 |
| `crates/parser/src/token.rs` | 新增 Trigger 相关 Token |
| `src/lib.rs` | 触发器执行逻辑 |
| `tests/integration/teaching_scenario_test.rs` | 30 个测试 |

## 验收标准

- [ ] 解析器支持 SET 表达式
- [ ] 解析器支持限定列名
- [ ] 解析器支持子查询
- [ ] 触发器 CREATE/DROP 语句可解析
- [ ] 触发器在 DML 操作时执行
- [ ] 30 个 teaching 测试通过 ≥ 21 个 (70%)
- [ ] 覆盖率提升 ≥ 2%

## 时间预估

- 解析器增强: 1-2 天
- 触发器实现: 1-2 天
- 测试创建: 1 天
- 总计: 3-5 天
