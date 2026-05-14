# SQLRustGo SQL 语句执行链路分析总计划

> **目标**: 对 SQL 的所有语句类型和综合查询进行纵向执行链路分析
> **版本**: v3.0.0
> **更新日期**: 2026-05-11

---

## 一、SQL 语句分类总览

```
┌─────────────────────────────────────────────────────────────────────┐
│                         SQL 语句分类体系                             │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐    │
│  │     DDL        │  │     DML        │  │     DCL        │    │
│  │ (数据定义)     │  │ (数据操作)     │  │ (数据控制)     │    │
│  ├─────────────────┤  ├─────────────────┤  ├─────────────────┤    │
│  │ CREATE         │  │ SELECT         │  │ GRANT          │    │
│  │ DROP           │  │ INSERT         │  │ REVOKE         │    │
│  │ ALTER          │  │ UPDATE         │  │ CREATE USER    │    │
│  │ TRUNCATE       │  │ DELETE         │  │ DROP USER      │    │
│  │ RENAME         │  │ MERGE          │  │ CREATE ROLE    │    │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘    │
│                                                                      │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐    │
│  │     TCL        │  │     DQL         │  │    分布式       │    │
│  │ (事务控制)     │  │ (数据查询)     │  │                 │    │
│  ├─────────────────┤  ├─────────────────┤  ├─────────────────┤    │
│  │ COMMIT         │  │ 子查询          │  │ XA BEGIN       │    │
│  │ ROLLBACK       │  │ CTE            │  │ XA END         │    │
│  │ SAVEPOINT      │  │ 窗口函数       │  │ XA PREPARE     │    │
│  │ SET TX         │  │ 集合运算       │  │ XA COMMIT      │    │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘    │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 二、执行链路功能矩阵

### 2.1 DDL 语句矩阵

| 语句 | 关键字 | Parser | Planner | Optimizer | Executor | Storage | WAL | MVCC | 覆盖率 | 状态 |
|------|--------|--------|---------|-----------|----------|---------|-----|------|--------|------|
| CREATE TABLE | create_table | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ~75% | ✅ |
| DROP TABLE | drop_table | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ~70% | ✅ |
| CREATE INDEX | create_index | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ~65% | ⚠️ |
| DROP INDEX | drop_index | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ~60% | ⚠️ |
| ALTER TABLE ADD | alter_add_col | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ~70% | ✅ |
| ALTER TABLE DROP | alter_drop_col | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ~65% | ⚠️ |
| ALTER TABLE MODIFY | alter_modify | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ~60% | ⚠️ |
| TRUNCATE | truncate | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ~40% | ❌ |
| RENAME TABLE | rename | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ~30% | ❌ |
| CREATE VIEW | create_view | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ~50% | ⚠️ |
| DROP VIEW | drop_view | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ~50% | ⚠️ |
| CREATE DATABASE | create_db | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ~60% | ⚠️ |
| DROP DATABASE | drop_db | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ~55% | ⚠️ |

### 2.2 DML 语句矩阵

| 语句 | 关键字 | Parser | Planner | Optimizer | Executor | Storage | WAL | MVCC | 覆盖率 | 状态 |
|------|--------|--------|---------|-----------|----------|---------|-----|------|--------|------|
| SELECT (simple) | select | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ~80% | ✅ |
| SELECT (WHERE) | filter | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ~75% | ✅ |
| SELECT (JOIN) | hash_join | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ~70% | ✅ |
| SELECT (GROUP BY) | aggregate | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ~70% | ✅ |
| SELECT (ORDER BY) | sort | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ~70% | ✅ |
| SELECT (LIMIT) | limit | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ~70% | ✅ |
| SELECT (子查询) | subquery | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ~60% | ⚠️ |
| SELECT (CTE) | cte | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ~55% | ⚠️ |
| SELECT (窗口函数) | window | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ~65% | ⚠️ |
| INSERT | insert | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ~75% | ✅ |
| INSERT...SELECT | ins_sel | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ~65% | ⚠️ |
| UPDATE | update | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ~70% | ✅ |
| UPDATE (multi-table) | upd_mul | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ~50% | ❌ |
| DELETE | delete | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ~70% | ✅ |
| DELETE (multi-table) | del_mul | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ~45% | ❌ |
| MERGE | merge | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | 0% | ❌ |

### 2.3 DCL 语句矩阵

| 语句 | 关键字 | Parser | Planner | Optimizer | Executor | Catalog | Auth | Audit | 覆盖率 | 状态 |
|------|--------|--------|---------|-----------|----------|---------|------|-------|--------|------|
| CREATE USER | create_user | ✅ | ✅ | ❌ | ❌ | ✅ | ✅ | ✅ | ~60% | ⚠️ |
| DROP USER | drop_user | ✅ | ✅ | ❌ | ❌ | ✅ | ✅ | ✅ | ~55% | ⚠️ |
| ALTER USER | alter_user | ✅ | ✅ | ❌ | ❌ | ✅ | ✅ | ✅ | ~50% | ❌ |
| RENAME USER | rename_user | ✅ | ✅ | ❌ | ❌ | ✅ | ✅ | ❌ | ~30% | ❌ |
| GRANT | grant | ✅ | ✅ | ❌ | ❌ | ✅ | ✅ | ✅ | ~55% | ⚠️ |
| REVOKE | revoke | ✅ | ✅ | ❌ | ❌ | ✅ | ✅ | ✅ | ~50% | ⚠️ |
| CREATE ROLE | create_role | ✅ | ✅ | ❌ | ❌ | ✅ | ✅ | ❌ | ~40% | ❌ |
| DROP ROLE | drop_role | ✅ | ✅ | ❌ | ❌ | ✅ | ✅ | ❌ | ~35% | ❌ |
| GRANT ROLE | grant_role | ✅ | ✅ | ❌ | ❌ | ✅ | ✅ | ❌ | ~30% | ❌ |
| REVOKE ROLE | revoke_role | ✅ | ✅ | ❌ | ❌ | ✅ | ✅ | ❌ | ~30% | ❌ |
| SET PASSWORD | set_pwd | ✅ | ✅ | ❌ | ❌ | ✅ | ✅ | ❌ | ~25% | ❌ |

### 2.4 TCL 语句矩阵

| 语句 | 关键字 | Parser | Planner | Optimizer | Executor | Transaction | WAL | MVCC | 覆盖率 | 状态 |
|------|--------|--------|---------|-----------|----------|-------------|-----|------|--------|------|
| BEGIN | tx_begin | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ~85% | ✅ |
| COMMIT | commit | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ~80% | ✅ |
| ROLLBACK | rollback | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ~80% | ✅ |
| SAVEPOINT | savepoint | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ~60% | ⚠️ |
| RELEASE SAVEPOINT | rel_sp | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ~50% | ❌ |
| ROLLBACK TO | rollback_to | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ~55% | ⚠️ |
| SET TRANSACTION | set_tx | ✅ | ✅ | ❌ | ✅ | ✅ | ❌ | ✅ | ~40% | ❌ |
| START TRANSACTION | start_tx | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | ~75% | ✅ |

### 2.5 综合查询矩阵

| 查询类型 | 关键字 | Parser | Planner | Optimizer | Executor | 执行算法 | 覆盖率 | 状态 |
|----------|--------|--------|---------|-----------|----------|----------|--------|------|
| 标量子查询 | scalar_subq | ✅ | ✅ | ✅ | ✅ | Hash Join | ~60% | ⚠️ |
| 行子查询 | row_subq | ✅ | ✅ | ✅ | ✅ | 比较 | ~55% | ⚠️ |
| 表子查询 | table_subq | ✅ | ✅ | ✅ | ✅ | Semi Join | ~55% | ⚠️ |
| EXISTS 子查询 | exists_subq | ✅ | ✅ | ✅ | ✅ | EXISTS | ~60% | ⚠️ |
| IN 子查询 | in_subq | ✅ | ✅ | ✅ | ✅ | IN | ~60% | ⚠️ |
| WITH (CTE) | cte | ✅ | ✅ | ✅ | ✅ | Recursive | ~55% | ⚠️ |
| 递归 CTE | r_cte | ✅ | ✅ | ✅ | ✅ | Recursive | ~50% | ⚠️ |
| ROW_NUMBER | row_num | ✅ | ✅ | ✅ | ✅ | WindowAgg | ~70% | ✅ |
| RANK | rank | ✅ | ✅ | ✅ | ✅ | WindowAgg | ~70% | ✅ |
| DENSE_RANK | dense_rank | ✅ | ✅ | ✅ | ✅ | WindowAgg | ~70% | ✅ |
| NTILE | ntile | ❌ | ❌ | ❌ | ❌ | - | 0% | ❌ |
| LEAD | lead | ❌ | ❌ | ❌ | ❌ | - | 0% | ❌ |
| LAG | lag | ❌ | ❌ | ❌ | ❌ | - | 0% | ❌ |
| FIRST_VALUE | first_val | ❌ | ❌ | ❌ | ❌ | - | 0% | ❌ |
| LAST_VALUE | last_val | ❌ | ❌ | ❌ | ❌ | - | 0% | ❌ |
| NTH_VALUE | nth_val | ❌ | ❌ | ❌ | ❌ | - | 0% | ❌ |

### 2.6 JOIN 类型矩阵

| JOIN 类型 | 关键字 | Parser | Planner | Optimizer | Executor | 算法 | 覆盖率 | 状态 |
|----------|--------|--------|---------|-----------|----------|------|--------|------|
| INNER JOIN | inner | ✅ | ✅ | ✅ | ✅ | Hash Join | ~75% | ✅ |
| LEFT JOIN | left | ✅ | ✅ | ✅ | ✅ | Hash Join | ~75% | ✅ |
| RIGHT JOIN | right | ✅ | ✅ | ✅ | ✅ | Hash Join | ~70% | ✅ |
| FULL JOIN | full | ✅ | ✅ | ✅ | ✅ | Hash Join | ~65% | ⚠️ |
| CROSS JOIN | cross | ✅ | ✅ | ✅ | ✅ | Nested Loop | ~60% | ⚠️ |
| NATURAL JOIN | natural | ✅ | ✅ | ✅ | ✅ | Hash Join | ~55% | ⚠️ |
| USING clause | using | ✅ | ✅ | ✅ | ✅ | Hash Join | ~65% | ⚠️ |
| STRAIGHT_JOIN | straight | ✅ | ✅ | ✅ | ✅ | Hash Join | ~50% | ❌ |
| INDEX JOIN | index | ❌ | ❌ | ❌ | ❌ | - | 0% | ❌ |
| NO_MERGE | no_merge | ❌ | ❌ | ❌ | ❌ | - | 0% | ❌ |
| BNL | bnl | ❌ | ❌ | ❌ | ❌ | - | 0% | ❌ |
| HASH_JOIN | hash_join | ✅ | ✅ | ✅ | ✅ | Hash Join | ~75% | ✅ |
| NESTED_LOOP | nl | ✅ | ✅ | ✅ | ✅ | Nested Loop | ~60% | ⚠️ |
| MERGE JOIN | merge | ❌ | ❌ | ❌ | ❌ | - | 0% | ❌ |

### 2.7 集合运算矩阵

| 运算类型 | 关键字 | Parser | Planner | Optimizer | Executor | 覆盖率 | 状态 |
|----------|--------|--------|---------|-----------|----------|--------|------|
| UNION | union | ✅ | ✅ | ✅ | ✅ | Union | ~70% | ✅ |
| UNION ALL | union_all | ✅ | ✅ | ✅ | ✅ | Concat | ~70% | ✅ |
| INTERSECT | intersect | ✅ | ✅ | ✅ | ✅ | Intersect | ~45% | ❌ |
| EXCEPT | except | ✅ | ✅ | ✅ | ✅ | Except | ~40% | ❌ |
| MINUS | minus | ✅ | ✅ | ✅ | ✅ | Except | ~40% | ❌ |

### 2.8 高级特性矩阵

| 特性 | 关键字 | Parser | Planner | Optimizer | Executor | 覆盖率 | 状态 |
|------|--------|--------|---------|-----------|----------|--------|------|
| 触发器 | trigger | ✅ | ✅ | ✅ | ✅ | ✅ | ~55% | ⚠️ |
| 存储过程 | proc | ✅ | ✅ | ❌ | ✅ | ✅ | ~50% | ⚠️ |
| 存储函数 | func | ✅ | ✅ | ❌ | ✅ | ✅ | ~45% | ❌ |
| 事件调度器 | event | ❌ | ❌ | ❌ | ❌ | ❌ | 0% | ❌ |
| 预处理语句 | prepare | ✅ | ✅ | ✅ | ✅ | ✅ | ~60% | ⚠️ |
| 绑定变量 | bind | ✅ | ✅ | ✅ | ✅ | ✅ | ~60% | ⚠️ |

### 2.9 分布式语句矩阵

| 语句 | 关键字 | XA | Coordinator | Participant | 覆盖率 | 状态 |
|------|--------|-----|-------------|--------------|--------|------|
| XA BEGIN | xa_begin | ✅ | ✅ | ✅ | ~70% | ✅ |
| XA END | xa_end | ✅ | ✅ | ✅ | ~65% | ✅ |
| XA PREPARE | xa_prepare | ✅ | ✅ | ✅ | ~65% | ✅ |
| XA COMMIT | xa_commit | ✅ | ✅ | ✅ | ~65% | ✅ |
| XA ROLLBACK | xa_rollback | ✅ | ✅ | ✅ | ~65% | ✅ |
| XA RECOVER | xa_recover | ✅ | ✅ | ✅ | ~50% | ⚠️ |

---

## 三、阶段执行计划

### 阶段一: DML 语句 (最高优先级)

| 任务 | 优先级 | 文档 | 状态 |
|------|--------|------|------|
| SELECT 完整执行链路 | P0 | `oo/dml/SELECT_EXECUTION.md` | ✅ |
| INSERT 完整执行链路 | P0 | `oo/dml/INSERT_EXECUTION.md` | ✅ |
| UPDATE 完整执行链路 | P0 | `oo/dml/UPDATE_EXECUTION.md` | ✅ |
| DELETE 完整执行链路 | P0 | `oo/dml/DELETE_EXECUTION.md` | ✅ |
| MERGE 执行链路 | P1 | `oo/dml/MERGE_EXECUTION.md` | ❌ |

### 阶段二: DDL 语句

| 任务 | 优先级 | 文档 | 状态 |
|------|--------|------|------|
| CREATE TABLE 完整链路 | P0 | `oo/ddl/DDL_EXECUTION.md` | ✅ |
| DROP TABLE 完整链路 | P0 | `oo/ddl/DDL_EXECUTION.md` | ✅ |
| ALTER TABLE 完整链路 | P0 | `oo/ddl/ALTER_EXECUTION.md` | ❌ |
| TRUNCATE 执行链路 | P2 | `oo/ddl/TRUNCATE_EXECUTION.md` | ❌ |
| CREATE INDEX 完整链路 | P1 | `oo/ddl/INDEX_EXECUTION.md` | ❌ |
| CREATE VIEW 完整链路 | P2 | `oo/ddl/VIEW_EXECUTION.md` | ❌ |

### 阶段三: DCL 语句

| 任务 | 优先级 | 文档 | 状态 |
|------|--------|------|------|
| GRANT/REVOKE 链路 | P1 | `oo/dcl/DCL_EXECUTION.md` | ✅ |
| CREATE USER 链路 | P1 | `oo/dcl/USER_MANAGEMENT.md` | ❌ |
| 角色管理链路 | P2 | `oo/dcl/ROLE_MANAGEMENT.md` | ❌ |
| 权限检查链路 | P1 | `oo/dcl/PRIVILEGE_CHECK.md` | ❌ |

### 阶段四: TCL 语句

| 任务 | 优先级 | 文档 | 状态 |
|------|--------|------|------|
| COMMIT/ROLLBACK 链路 | P0 | `oo/transaction/TX_MANAGEMENT.md` | ❌ |
| SAVEPOINT 链路 | P2 | `oo/transaction/SAVEPOINT.md` | ❌ |
| MVCC 实现 | P0 | `oo/transaction/MVCC_IMPLEMENTATION.md` | ❌ |

### 阶段五: 综合查询

| 任务 | 优先级 | 文档 | 状态 |
|------|--------|------|------|
| 子查询执行链路 | P1 | `oo/query/SUBQUERY_EXECUTION.md` | ❌ |
| CTE 执行链路 | P1 | `oo/query/CTE_EXECUTION.md` | ❌ |
| 窗口函数实现 | P1 | `oo/query/WINDOW_FUNCTIONS.md` | ❌ |
| 递归 CTE | P2 | `oo/query/RECURSIVE_CTE.md` | ❌ |

### 阶段六: JOIN 算法

| 任务 | 优先级 | 文档 | 状态 |
|------|--------|------|------|
| Hash Join 算法 | P0 | `oo/join/JOIN_ALGORITHMS.md` | ✅ |
| Nested Loop Join | P1 | `oo/join/NESTED_LOOP_JOIN.md` | ❌ |
| Merge Join | P2 | `oo/join/MERGE_JOIN.md` | ❌ |
| Join Order 优化 | P1 | `oo/cbo/CBO_JOIN_ORDERING.md` | ❌ |

### 阶段七: 集合运算

| 任务 | 优先级 | 文档 | 状态 |
|------|--------|------|------|
| UNION 执行链路 | P1 | `oo/setops/SET_OPERATIONS.md` | ❌ |
| INTERSECT/EXCEPT | P2 | `oo/setops/SET_OPERATIONS.md` | ❌ |

### 阶段八: 高级特性

| 任务 | 优先级 | 文档 | 状态 |
|------|--------|------|------|
| 触发器执行链路 | P1 | `oo/advanced/TRIGGER_EXECUTION.md` | ❌ |
| 存储过程执行链路 | P2 | `oo/advanced/PROCEDURE_EXECUTION.md` | ❌ |
| Prepared Statement | P2 | `oo/advanced/PREPARED_STATEMENT.md` | ❌ |
| 事件调度器 | P3 | `oo/advanced/EVENT_SCHEDULER.md` | ❌ |

### 阶段九: 分布式

| 任务 | 优先级 | 文档 | 状态 |
|------|--------|------|------|
| XA 事务链路 | P0 | `oo/distributed/XA_TRANSACTION.md` | ✅ |
| 主从复制 | P1 | `oo/distributed/REPLICATION.md` | ❌ |
| 半同步复制 | P1 | `oo/distributed/SEMISYNC.md` | ✅ |
| MTS 并行复制 | P1 | `oo/distributed/MTS.md` | ✅ |

---

## 四、文档模板

每个执行链路文档应包含以下章节：

```markdown
# {语句名} 执行链路

## 1. 概述
- 语句语法
- 功能描述
- 适用场景

## 2. 架构图
- 模块交互图
- 数据流图

## 3. 时序图
- Parser → Planner → Optimizer → Executor → Storage

## 4. 状态图
- 关键状态转换

## 5. 算法实现
- 核心算法 Rust 代码
- 复杂度分析

## 6. 测试计划
- 功能测试用例
- 边界测试用例
- 性能测试用例

## 7. 覆盖率分析
- 当前覆盖率
- 差距原因
- 提升计划

## 8. 相关文件
- 核心实现文件索引
```

---

## 五、进度跟踪

### 已完成文档

| 文档 | 完成日期 |
|------|----------|
| `oo/dml/DML_EXECUTION.md` | 2026-05-11 |
| `oo/ddl/DDL_EXECUTION.md` | 2026-05-11 |
| `oo/dcl/DCL_EXECUTION.md` | 2026-05-11 |
| `oo/join/JOIN_ALGORITHMS.md` | 2026-05-11 |
| `oo/wal/WAL_PROTOCOL.md` | 2026-05-11 |
| `oo/recovery/CRASH_RECOVERY.md` | 2026-05-11 |
| `oo/distributed/DISTRIBUTED_SYNC.md` | 2026-05-11 |
| `oo/query/SUBQUERY_EXECUTION.md` | 2026-05-11 |
| `oo/query/WINDOW_FUNCTIONS.md` | 2026-05-11 |
| `oo/advanced/TRIGGER_EXECUTION.md` | 2026-05-11 |
| `oo/advanced/STORED_PROCEDURE.md` | 2026-05-11 |
| `oo/setops/SET_OPERATIONS.md` | 2026-05-11 |

### 进行中文档

| 文档 | 状态 |
|------|------|
| - | - |

### 待完成文档

| 优先级 | 文档数 |
|--------|--------|
| P0 | 4 |
| P1 | 9 |
| P2 | 8 |
| P3 | 2 |
| **总计** | **23** |

---

## 六、附录

### A. 覆盖率计算方法

```
覆盖率 = (已实现的测试用例数 / 理论测试用例总数) × 100%
```

### B. 状态说明

| 状态 | 说明 |
|------|------|
| ✅ 完成 | 覆盖率 ≥ 70%，测试完整 |
| ⚠️ 部分 | 覆盖率 40-70%，有差距 |
| ❌ 缺失 | 覆盖率 < 40% 或完全未实现 |

### C. 核心文件索引

| 模块 | 核心文件 |
|------|----------|
| Parser | `crates/parser/src/*.rs` |
| Planner | `crates/planner/src/*.rs` |
| Optimizer | `crates/optimizer/src/*.rs` |
| Executor | `crates/executor/src/*.rs` |
| Storage | `crates/storage/src/*.rs` |
| Transaction | `crates/transaction/src/*.rs` |
| Catalog | `crates/catalog/src/*.rs` |
| Distributed | `crates/distributed/src/*.rs` |
