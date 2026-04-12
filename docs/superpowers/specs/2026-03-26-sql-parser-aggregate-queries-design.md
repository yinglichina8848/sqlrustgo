# SQL Parser 聚合查询增强 - 设计规范

> **实现模式:** 使用 superpowers:subagent-driven-development (推荐) 或 superpowers:executing-plans
>
> **目标:** 实现 GROUP BY + HAVING + ORDER BY 功能，支持 SQL-92 标准

## 架构概述

```
┌─────────────────────────────────────────────────────────────┐
│                        Parser 层                            │
│  token.rs → lexer.rs → parser.rs → SelectStatement          │
│                           ↓                                  │
│  新增: GroupByClause, HavingClause, OrderByClause           │
└─────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────┐
│                       Planner 层                            │
│  新增: AggregateExec, SortExec                              │
│  修改: FilterExec 支持 HAVING                               │
└─────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────┐
│                      Executor 层                            │
│  新增: HashAggregateExecutor                                 │
│  修改: SeqScanExecutor 支持排序                             │
└─────────────────────────────────────────────────────────────┘
```

## 修改的文件

| 文件 | 修改内容 |
|------|---------|
| `crates/parser/src/token.rs` | 添加 `Group`、`By`、`Having`、`Order`、`Nulls`、`First`、`Last` 等 Token |
| `crates/parser/src/lexer.rs` | 添加关键字识别 |
| `crates/parser/src/parser.rs` | 修改 `SelectStatement` 结构，添加 `parse_group_by()`、`parse_having()`、`parse_order_by()` |
| `crates/planner/src/mod.rs` | 添加 `AggregateExec`、`SortExec` |
| `crates/executor/src/*.rs` | 实现 `HashAggregateExecutor`、`TopNExecutor` |

## 新增 AST 结构

### SelectStatement 修改

```rust
// crates/parser/src/parser.rs

pub struct SelectStatement {
    pub columns: Vec<SelectColumn>,      // SELECT 列
    pub table: String,                   // FROM 表名
    pub where_clause: Option<Expression>, // WHERE 条件
    pub join_clause: Option<JoinClause>,  // JOIN 子句
    pub aggregates: Vec<AggregateCall>,   // 聚合函数
    pub limit: Option<usize>,            // LIMIT
    pub offset: Option<usize>,           // OFFSET
    // 新增字段
    pub group_by: Option<GroupByClause>,  // GROUP BY 子句
    pub having: Option<Expression>,       // HAVING 条件
    pub order_by: Option<OrderByClause>, // ORDER BY 子句
}
```

### 新增数据结构

```rust
/// GROUP BY 子句
pub struct GroupByClause {
    pub columns: Vec<Expression>,
}

/// ORDER BY 子句
pub struct OrderByClause {
    pub items: Vec<OrderByItem>,
}

/// ORDER BY 单项
pub struct OrderByItem {
    pub expr: Expression,
    pub asc: bool,          // true = ASC, false = DESC
    pub nulls_first: bool,  // true = NULLS FIRST, false = NULLS LAST
}
```

## SQL-92 语法支持

### GROUP BY 语法

```sql
SELECT column, AGG(column)
FROM table
GROUP BY column

-- 多列分组
SELECT col1, col2, AGG(col3)
FROM table
GROUP BY col1, col2

-- GROUP BY with HAVING
SELECT category, COUNT(*)
FROM products
GROUP BY category
HAVING COUNT(*) > 1
```

### ORDER BY 语法

```sql
SELECT * FROM table ORDER BY column ASC

SELECT * FROM table ORDER BY column DESC

SELECT * FROM table ORDER BY column NULLS FIRST

SELECT * FROM table ORDER BY column DESC NULLS LAST

-- 多列排序
SELECT * FROM table ORDER BY col1 ASC, col2 DESC
```

## 实现顺序

### Phase 1: Token + Lexer
- [ ] 在 `token.rs` 添加新 Token 变体
- [ ] 在 `lexer.rs` 添加关键字识别
- [ ] 添加单元测试

### Phase 2: Parser
- [ ] 修改 `SelectStatement` 添加新字段
- [ ] 实现 `parse_group_by()` 函数
- [ ] 实现 `parse_having()` 函数
- [ ] 实现 `parse_order_by()` 函数
- [ ] 更新 `parse_select()` 整合新解析
- [ ] 添加解析器单元测试

### Phase 3: Planner
- [ ] 添加 `AggregateExec` 算子
- [ ] 添加 `SortExec` 算子
- [ ] 更新查询优化器
- [ ] 添加规划器测试

### Phase 4: Executor
- [ ] 实现 `HashAggregateExecutor`
- [ ] 实现 `TopNExecutor` (带排序的 LIMIT)
- [ ] 修改现有执行器支持新算子
- [ ] 添加执行器测试

### Phase 5: 集成测试
- [ ] 添加 teaching_scenario_test 测试用例
- [ ] 运行完整测试套件
- [ ] 验证 SQL-92 兼容性

## 依赖关系

```
Token/Lexer
    ↓
Parser (GROUP BY 解析)
    ↓
Planner (AggregateExec)
    ↓
Executor (HashAggregate)
```

## 风险与注意事项

1. **表达式解析依赖** - HAVING 和 ORDER BY 都依赖表达式解析，需确保 `parse_expression()` 足够强大
2. **执行器性能** - HashAggregate 需要处理大数据集，需考虑内存使用
3. **NULL 处理** - SQL-92 对 NULL 的排序有特殊规则（NULLS FIRST/LAST）

## 测试用例

```sql
-- 基础 GROUP BY
SELECT category, COUNT(*) FROM products GROUP BY category

-- GROUP BY + HAVING
SELECT category, COUNT(*) FROM products GROUP BY category HAVING COUNT(*) > 1

-- GROUP BY + ORDER BY
SELECT category, COUNT(*) FROM products GROUP BY category ORDER BY COUNT(*) DESC

-- 完整语法
SELECT category, SUM(price) FROM products WHERE price > 10 GROUP BY category HAVING SUM(price) > 100 ORDER BY SUM(price) DESC NULLS LAST

-- 多列分组
SELECT category, brand, COUNT(*) FROM products GROUP BY category, brand

-- 带表达式的 GROUP BY
SELECT (price > 100)::text as price_tier, COUNT(*) FROM products GROUP BY (price > 100)
```
