# 集合运算执行链路

> Set Operations: UNION, UNION ALL, INTERSECT, EXCEPT

## 1. 集合运算概述

### 1.1 集合运算类型

```
┌─────────────────────────────────────────────────────────────┐
│                    集合运算分类                               │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────┐ │
│  │   UNION         │  │  UNION ALL     │  │  INTERSECT  │ │
│  │                 │  │                 │  │             │ │
│  │  A ∪ B          │  │  A ∪ B (含重复) │  │  A ∩ B      │ │
│  │  (去重)         │  │                 │  │             │ │
│  │                 │  │                 │  │             │ │
│  └─────────────────┘  └─────────────────┘  └─────────────┘ │
│                                                              │
│  ┌─────────────────┐                                        │
│  │   EXCEPT        │                                        │
│  │                 │                                        │
│  │  A - B          │                                        │
│  │  (差集)         │                                        │
│  └─────────────────┘                                        │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 1.2 SQL 语法

```sql
-- 基本 UNION (去重)
SELECT col FROM t1
UNION
SELECT col FROM t2;

-- UNION ALL (保留重复)
SELECT col FROM t1
UNION ALL
SELECT col FROM t2;

-- INTERSECT (交集)
SELECT col FROM t1
INTERSECT
SELECT col FROM t2;

-- EXCEPT (差集)
SELECT col FROM t1
EXCEPT
SELECT col FROM t2;

-- 混合运算
SELECT col FROM t1
UNION
SELECT col FROM t2
INTERSECT
SELECT col FROM t3;
```

## 2. 集合运算执行架构

### 2.1 UNION 执行流程

```
SELECT name FROM employees
UNION
SELECT name FROM contractors
    │
    ▼
┌─────────────────────────────────────────────┐
│              Parser                           │
│  SetOperation {                             │
│    op: UNION,                              │
│    left: SELECT name FROM employees,      │
│    right: SELECT name FROM contractors,    │
│    distinct: true                          │
│  }                                          │
└─────────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────────┐
│              Planner                         │
│  SetExec {                                  │
│    op: UNION,                              │
│    left: SeqScan(employees),               │
│    right: SeqScan(contractors),            │
│    distinct: true,                         │
│    schema: [name: VARCHAR]                │
│  }                                          │
└─────────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────────┐
│              Executor                        │
│  ┌─────────────────────────────────────┐  │
│  │  if UNION ALL:                       │  │
│  │      直接拼接左右结果                  │  │
│  │                                      │  │
│  │  if UNION (distinct):                │  │
│  │      1. 收集所有行                  │  │
│  │      2. 排序                        │  │
│  │      3. 去除相邻重复                │  │
│  └─────────────────────────────────────┘  │
└─────────────────────────────────────────────┘
```

### 2.2 UNION ALL vs UNION (distinct)

```
┌─────────────────────────────────────────────────────────────┐
│                 UNION vs UNION ALL                          │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  UNION ALL:                                                │
│  ─────────────────────────────────────────────────────────  │
│  employees: [Alice, Bob, Alice]                          │
│  contractors: [Carol, Alice, Dave]                        │
│                                                              │
│  Result: [Alice, Bob, Alice, Carol, Alice, Dave]          │
│  (直接拼接，6 行)                                          │
│                                                              │
│  UNION (distinct):                                         │
│  ─────────────────────────────────────────────────────────  │
│  1. 收集所有行:                                            │
│     [Alice, Bob, Alice, Carol, Alice, Dave]                 │
│                                                              │
│  2. 排序:                                                   │
│     [Alice, Alice, Alice, Bob, Carol, Dave]                  │
│                                                              │
│  3. 去除相邻重复:                                            │
│     [Alice, Bob, Carol, Dave]                               │
│  (4 行)                                                    │
│                                                              │
│  性能: UNION ALL > UNION (distinct)                        │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## 3. INTERSECT 执行

### 3.1 INTERSECT 执行流程

```
SELECT name FROM employees
INTERSECT
SELECT name FROM contractors
    │
    ▼
┌─────────────────────────────────────────────┐
│              策略: Hash Semi Join            │
│  ┌─────────────────────────────────────┐  │
│  │ 1. 构建 Hash Set (较小表)           │  │
│  │    employees: {Alice, Bob}          │  │
│  │                                      │  │
│  │ 2. 遍历较大表，检查存在性           │  │
│  │    contractors: {Carol, Alice, Dave}│  │
│  │                                      │  │
│  │ 3. 保留存在的行                      │  │
│  │    Result: {Alice}                  │  │
│  └─────────────────────────────────────┘  │
└─────────────────────────────────────────────┘
```

### 3.2 INTERSECT 状态机

```
                  ┌──────────────────┐
                  │    INITIAL       │
                  └────────┬─────────┘
                           │ start INTERSECT
                           ▼
                  ┌──────────────────┐
                  │   BUILD_SET     │
                  └────────┬─────────┘
                           │ build hash set from left
                           ▼
                  ┌──────────────────┐
                  │   PROBE_RIGHT    │
                  └────────┬─────────┘
                           │ scan right, check existence
                           ▼
                  ┌──────────────────┐
                  │    MATCH_FOUND    │
                  └────────┬─────────┘
                           │ emit matching row
                           ▼
                  ┌──────────────────┐
                  │   MORE_RIGHT     │
                  └────────┬─────────┘
                           │ more rows?
                           │
            ┌─────────────┴─────────────┐
            │                           │
            ▼                           ▼
     ┌──────────┐              ┌──────────┐
     │   YES    │              │    NO    │
     └──────────┘              └──────────┘
          │                           │
          ▼                           ▼
     ┌──────────┐              ┌──────────┐
     │  BACK TO  │              │   DONE   │
     │ PROBE_RIGHT│              └──────────┘
     └──────────┘
```

## 4. EXCEPT 执行

### 4.1 EXCEPT 执行流程

```
SELECT name FROM employees
EXCEPT
SELECT name FROM contractors
    │
    ▼
┌─────────────────────────────────────────────┐
│              策略: Hash Anti Join             │
│  ┌─────────────────────────────────────┐  │
│  │ 1. 构建 Hash Set (右表)            │  │
│  │    contractors: {Carol, Alice, Dave} │  │
│  │                                      │  │
│  │ 2. 遍历左表，检查不存在性           │  │
│  │    employees: {Alice, Bob}           │  │
│  │                                      │  │
│  │ 3. 保留不存在的行                    │  │
│  │    Result: {Bob}                    │  │
│  └─────────────────────────────────────┘  │
└─────────────────────────────────────────────┘
```

### 4.2 三种运算对比

```
┌─────────────────────────────────────────────────────────────┐
│              INTERSECT vs EXCEPT vs UNION                     │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  INTERSECT (交集):                                          │
│  ─────────────────────────────────────────────────────────  │
│  A = {1, 2, 3}                                            │
│  B = {2, 3, 4}                                            │
│  A ∩ B = {2, 3}                                           │
│                                                              │
│  EXCEPT (差集):                                            │
│  ─────────────────────────────────────────────────────────  │
│  A - B = {1}  (A 有 B 没有的)                            │
│  B - A = {4}  (B 有 A 没有的)                            │
│                                                              │
│  UNION (并集):                                             │
│  ─────────────────────────────────────────────────────────  │
│  A ∪ B = {1, 2, 3, 4}                                     │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## 5. 集合运算实现

### 5.1 执行器实现

```rust
pub struct SetExecutor {
    op: SetOperator,
    left: Box<dyn Executor>,
    right: Box<dyn Executor>,
    distinct: bool,
    // 用于 distinct 的去重
    seen: HashSet<Vec<Value>>,
}

pub enum SetOperator {
    Union,
    UnionAll,
    Intersect,
    Except,
}

impl Executor for SetExecutor {
    fn next(&mut self) -> Result<Option<Row>, Error> {
        match self.op {
            SetOperator::UnionAll => {
                // 直接拼接
                if let Some(row) = self.left.next()? {
                    return Ok(Some(row));
                }
                return self.right.next();
            }
            SetOperator::Union => {
                // Union = Union All + Distinct
                self.union_distinct()
            }
            SetOperator::Intersect => {
                // Intersect = Anti Join (保留存在的)
                self.intersect()
            }
            SetOperator::Except => {
                // Except = Anti Join (保留不存在的)
                self.except()
            }
        }
    }
}
```

### 5.2 UNION DISTINCT 实现

```rust
fn union_distinct(&mut self) -> Result<Option<Row>, Error> {
    loop {
        if let Some(row) = self.left.next()? {
            let key = row_to_key(&row);
            if !self.seen.contains(&key) {
                self.seen.insert(key);
                return Ok(Some(row));
            }
        } else {
            // 左表耗尽，切换到右表
            break;
        }
    }

    loop {
        if let Some(row) = self.right.next()? {
            let key = row_to_key(&row);
            if !self.seen.contains(&key) {
                self.seen.insert(key);
                return Ok(Some(row));
            }
        } else {
            return Ok(None);
        }
    }
}
```

## 6. ORDER BY 与 LIMIT

### 6.1 集合运算的 ORDER BY

```sql
-- 集合运算的 ORDER BY 在最后
SELECT name FROM employees
UNION
SELECT name FROM contractors
ORDER BY name DESC
LIMIT 10;
```

```
执行顺序:
1. 执行 UNION
2. 对结果集排序
3. 应用 LIMIT
```

### 6.2 集合运算的列名

```sql
-- 结果列名来自第一个 SELECT
SELECT name AS n FROM employees
UNION
SELECT username FROM contractors;  -- username 会变成 n
```

## 7. 测试计划

### 7.1 UNION 测试

| 测试编号 | 测试内容 | 预期结果 |
|----------|----------|----------|
| SET-T01 | UNION 简单 | 去重合并 |
| SET-T02 | UNION ALL | 保留重复 |
| SET-T03 | UNION 空表 | 正确处理 |
| SET-T04 | UNION 不同列数 | 报错 |

### 7.2 INTERSECT 测试

| 测试编号 | 测试内容 | 预期结果 |
|----------|----------|----------|
| SET-T10 | INTERSECT 有交集 | 返回交集 |
| SET-T11 | INTERSECT 无交集 | 返回空 |
| SET-T12 | INTERSECT 全包含 | 返回较小的 |

### 7.3 EXCEPT 测试

| 测试编号 | 测试内容 | 预期结果 |
|----------|----------|----------|
| SET-T20 | EXCEPT 有差集 | 返回差集 |
| SET-T21 | EXCEPT 无差集 | 返回空 |
| SET-T22 | A EXCEPT B vs B EXCEPT A | 结果不同 |

### 7.4 复杂测试

| 测试编号 | 测试内容 | 预期结果 |
|----------|----------|----------|
| SET-T30 | 嵌套集合运算 | 正确优先级 |
| SET-T31 | 集合运算 + ORDER BY | 正确排序 |
| SET-T32 | 集合运算 + LIMIT | 正确限制 |

## 8. 覆盖率差距分析

### 8.1 当前覆盖率

| 组件 | 行覆盖率 | 说明 |
|------|----------|------|
| Union Executor | ~70% | UNION/UNION ALL |
| Intersect | ~45% | 基础实现 |
| Except | ~40% | 基础实现 |
| 多重集合运算 | ~35% | A ∪ B ∩ C 等 |

### 8.2 差距原因

1. **INTERSECT/EXCEPT 优化**: 未使用更高效的算法
2. **多重集合运算**: 括号决定优先级未正确处理
3. **ALL 修饰符**: INTERSECT ALL/EXCEPT ALL 未实现

### 8.3 提升计划

| 阶段 | 任务 | 目标覆盖率 |
|------|------|-----------|
| v3.1.0 | 优化 INTERSECT 算法 | 70% |
| v3.1.0 | 实现多重集合运算 | 65% |
| v3.2.0 | INTERSECT ALL/EXCEPT ALL | 60% |

## 9. 核心文件索引

| 文件 | 说明 |
|------|------|
| `crates/executor/src/set_op.rs` | 集合运算执行器 |
| `crates/planner/src/` | 集合运算规划 |

## 10. 相关文档

| 文档 | 说明 |
|------|------|
| [DML_EXECUTION.md](./DML_EXECUTION.md) | DML 执行链路 |
| [SUBQUERY_EXECUTION.md](./SUBQUERY_EXECUTION.md) | 子查询执行 |
