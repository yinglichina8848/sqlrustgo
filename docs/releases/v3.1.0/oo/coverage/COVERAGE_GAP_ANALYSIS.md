# v3.1.0 OO 执行链路覆盖缺口分析

> **Issue**: #626
> **Parent**: #624 (OO 文档落地追踪总控)
> **Status**: In Progress
> **Date**: 2026-05-12

## 背景

`SQL_EXECUTION_MATRIX.md` 分析了 v3.0.0 SQL 语句的执行链路覆盖情况。本文档追踪覆盖缺口自动扫描和链路回归测试的落地情况。

## 扫描工具

创建了 `scripts/coverage/scan_coverage_gaps.py` 自动扫描脚本：

```bash
python scripts/coverage/scan_coverage_gaps.py [--matrix PATH] [--tests PATH]
```

### 功能
- 解析 `SQL_EXECUTION_MATRIX.md` 中的覆盖矩阵
- 扫描 `tests/` 目录查找匹配的测试文件
- 输出覆盖缺口报告

## 扫描结果摘要

| 指标 | 数值 |
|------|------|
| 矩阵总条目 | 76 |
| 覆盖缺口数量 | 43 |

## Critical Gaps (0% 覆盖)

| Statement | Keyword | 状态 |
|-----------|---------|------|
| MERGE | merge | ❌ 无测试 |

## 高优先级缺口 (< 70% 覆盖)

### DDL 语句

| Statement | 覆盖率 | 缺口 |
|-----------|--------|------|
| TRUNCATE | 40% | 无测试 |
| RENAME TABLE | 30% | 无测试 |
| CREATE VIEW | 50% | 无测试 |
| DROP VIEW | 50% | 无测试 |
| CREATE DATABASE | 60% | 无测试 |
| DROP DATABASE | 55% | 无测试 |
| ALTER TABLE DROP | 65% | 无测试 |
| ALTER TABLE MODIFY | 60% | 无测试 |

### DML 语句

| Statement | 覆盖率 | 缺口 |
|-----------|--------|------|
| UPDATE (multi-table) | 50% | 无测试 |
| DELETE (multi-table) | 45% | 无测试 |
| INSERT...SELECT | 65% | 无测试 |

### DCL 语句

| Statement | 覆盖率 | 缺口 |
|-----------|--------|------|
| CREATE USER | 60% | 无测试 |
| DROP USER | 55% | 无测试 |
| ALTER USER | 50% | 无测试 |
| RENAME USER | 30% | 无测试 |
| GRANT | 55% | 无测试 |
| REVOKE | 50% | 无测试 |
| CREATE ROLE | 40% | 无测试 |
| DROP ROLE | 35% | 无测试 |
| GRANT ROLE | 30% | 无测试 |
| REVOKE ROLE | 30% | 无测试 |
| SET PASSWORD | 25% | 无测试 |

### TCL 语句

| Statement | 覆盖率 | 缺口 |
|-----------|--------|------|
| SAVEPOINT | 60% | 无测试 |
| RELEASE SAVEPOINT | 50% | 无测试 |
| ROLLBACK TO | 55% | 无测试 |
| SET TRANSACTION | 40% | 无测试 |

### 窗口函数

| Statement | 覆盖率 | 缺口 |
|-----------|--------|------|
| NTILE | 0% | 无实现 |
| LEAD | 0% | 无实现 |
| LAG | 0% | 无实现 |
| FIRST_VALUE | 0% | 无实现 |
| LAST_VALUE | 0% | 无实现 |
| NTH_VALUE | 0% | 无实现 |

### 集合运算

| Statement | 覆盖率 | 缺口 |
|-----------|--------|------|
| INTERSECT | 45% | 无测试 |
| EXCEPT | 40% | 无测试 |
| MINUS | 40% | 无测试 |

### 子查询

| Statement | 覆盖率 | 缺口 |
|-----------|--------|------|
| 标量子查询 | 60% | 无测试 |
| 行子查询 | 55% | 无测试 |
| 表子查询 | 55% | 无测试 |

## 已创建回归测试

文件: `tests/execution_chain_regression_test.rs`

### 测试用例

| 测试名称 | 覆盖链路 |
|---------|---------|
| `test_execution_chain_insert_select` | INSERT...SELECT |
| `test_execution_chain_multi_table_update` | 多表 UPDATE |
| `test_execution_chain_multi_table_delete` | 多表 DELETE |
| `test_execution_chain_truncate` | TRUNCATE |
| `test_execution_chain_rename_table` | RENAME TABLE |
| `test_execution_chain_savepoint_release` | RELEASE SAVEPOINT |
| `test_execution_chain_rollback_to_savepoint` | ROLLBACK TO SAVEPOINT |
| `test_execution_chain_set_transaction` | SET TRANSACTION |
| `test_execution_chain_scalar_subquery` | 标量子查询 |
| `test_execution_chain_cte` | CTE (WITH) |
| `test_execution_chain_intersect` | INTERSECT |
| `test_execution_chain_except` | EXCEPT |
| `test_execution_chain_create_drop_view` | CREATE/DROP VIEW |
| `test_execution_chain_create_drop_database` | CREATE/DROP DATABASE |
| `test_execution_chain_create_drop_role` | CREATE/DROP ROLE |
| `test_execution_chain_grant_revoke` | GRANT/REVOKE |
| `test_execution_chain_alter_user` | ALTER USER |
| `test_execution_chain_set_password` | SET PASSWORD |

## 下一步

### 短期 (v3.1.0-alpha)
- [ ] 运行 `execution_chain_regression_test.rs` 确保所有测试通过
- [ ] 为 MERGE 语句创建占位测试（标记为 `#[ignore]` 待实现）
- [ ] 将 `scan_coverage_gaps.py` 集成到 CI

### 中期 (v3.1.0)
- [ ] 窗口函数完整实现 (NTILE, LEAD, LAG, FIRST_VALUE, LAST_VALUE, NTH_VALUE)
- [ ] MERGE 语句实现
- [ ] INTERSECT/EXCEPT 测试覆盖

### 长期
- [ ] 覆盖矩阵与代码覆盖率工具集成
- [ ] 自动生成测试模板

## 相关文档

- `docs/releases/v3.0.0/oo/SQL_EXECUTION_MATRIX.md` - SQL 执行链路矩阵
- `scripts/coverage/scan_coverage_gaps.py` - 覆盖缺口扫描脚本
- `tests/execution_chain_regression_test.rs` - 链路回归测试