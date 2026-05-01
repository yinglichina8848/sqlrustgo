# MySQL 5.7 兼容性增强计划 — v2.8.0

> **版本**: v2.8.0 (GA)
> **日期**: 2026-05-02
> **基于**: SQL Corpus (40.8% 通过率) + FEATURE_MATRIX.md + PARSER_COVERAGE_ANALYSIS.md

---

## 1. 兼容性现状

| 维度 | v2.7.0 | v2.8.0 | 变化 |
|------|--------|--------|------|
| MySQL 5.7 功能覆盖率 | 83% | 83% → 92%（目标） | **目标提升** |
| SQL Corpus 通过率 | - | **40.8%** (174/426) | **基线建立** |
| Parser 行覆盖率 | - | **54.70%** | **基数低** |
| DDL 支持度 | 基础 | TRUNCATE TABLE 新增 | 1 项 |
| DML 支持度 | 基础 | REPLACE INTO 新增 | 1 项 |
| JOIN 支持度 | INNER/LEFT/RIGHT | FULL OUTER JOIN 修复 | 1 项 |

---

## 2. SQL Corpus 现状分析

### 2.1 整体通过率

根据 `crates/sql-corpus/tests/corpus_test.rs` 集成测试结果 (`INTEGRATION_TEST_PLAN.md` 行 36)：

| 指标 | 数值 |
|------|------|
| 总用例数 | 426 |
| 通过 | 174 |
| 失败 | 252 |
| **通过率** | **40.8%** |

### 2.2 失败分布

SQL Corpus 覆盖以下类别，失败集中在以下领域：

| 分类 | 预计通过率 | 主要失败原因 |
|------|-----------|-------------|
| **DDL (CREATE/ALTER/DROP)** | ~60% | ALTER TABLE 部分支持、CREATE VIEW 不完整 |
| **DML (INSERT/UPDATE/DELETE)** | ~70% | 复杂 WHERE 条件、多表 UPDATE 限制 |
| **SELECT (JOIN)** | ~80% | FULL OUTER JOIN 已修复，NATURAL/CROSS JOIN 缺 |
| **SELECT (子查询)** | ~45% | 相关子查询、IN/EXISTS 嵌套深度限制 |
| **表达式** | ~55% | 三值逻辑 NULL 传播、CASE 表达式限制 |
| **函数** | ~35% | 日期函数、字符串函数、聚合函数参数类型 |
| **事务** | ~70% | SAVEPOINT、ROLLBACK TO 限制 |
| **窗口函数** | ~50% | PARTITION BY 复杂表达式、RANGE 窗口 |
| **存储过程/触发器** | ~30% | 复合语句、游标、异常处理 |
| **视图** | ~20% | CREATE VIEW + SELECT * 限制 |
| **特殊语法** | ~10% | FULLTEXT、HANDLER、分区语法 |

---

## 3. Parser 覆盖率分析

**来源**: `docs/releases/v2.8.0/PARSER_COVERAGE_ANALYSIS.md`

### 3.1 当前覆盖率

| 指标 | 当前值 | 目标 (v2.8.0) | 差距 |
|------|--------|---------------|------|
| Line 覆盖率 | **54.70%** | 85% | **-30.30%** |
| Region 覆盖率 | **52.01%** | 80% | **-27.99%** |
| Func 覆盖率 | **70.41%** | 90% | **-19.59%** |

### 3.2 关键未覆盖函数

| 函数 | 位置 | 缺失分支 |
|------|------|----------|
| `parse_join_clause` | ~行 1194 | CROSS JOIN、NATURAL JOIN、JOIN USING |
| `parse_expression` | ~行 1506 | IS NULL、IS NOT NULL、三值逻辑 AND/OR/NOT |
| `parse_truncate` | ~行 2098 | **完全未测试** |

### 3.3 预期提升路径

| 测试组 | 测试数 | 预期 Line % 提升 | 当前状态 |
|--------|--------|------------------|----------|
| 三值逻辑（IS NULL/AND/OR/NOT） | 6 | +8% | ⏳ 待添加 |
| JOIN 补全（CROSS/NATURAL/USING） | 4 | +5% | ⏳ 待添加 |
| TRUNCATE TABLE 解析 | 2 | +3% | ✅ v2.8.0 已实现 |
| 表达式优先级 | 4 | +4% | ⏳ 待添加 |
| 错误处理（语法错误恢复） | 4 | +5% | ⏳ 待添加 |
| **合计** | **20** | **+25% → ~80%** | |

---

## 4. FEATURE_MATRIX.md 差距分析

### 4.1 DDL 差距

| MySQL 5.7 功能 | v2.8.0 状态 | 差距 |
|----------------|-------------|------|
| CREATE DATABASE | ✅ 稳定 | 无 |
| DROP DATABASE | ✅ 稳定 | 无 |
| CREATE TABLE | ✅ 稳定 | 无 |
| ALTER TABLE (ADD/DROP/MODIFY) | ⚠️ 部分 | 重命名列、更改数据类型 |
| CREATE INDEX | ✅ 稳定 | 无 |
| CREATE VIEW | ⚠️ 开发中 | 复杂视图查询 |
| TRUNCATE TABLE | ✅ **v2.8.0 新增** | 无 |
| 分区表 (RANGE/LIST/HASH) | ⏳ 规划中 | **完全缺失** |
| CHECK 约束 | ⏳ 部分 | 数据结构已支持，解析/执行缺 |

### 4.2 DML 差距

| MySQL 5.7 功能 | v2.8.0 状态 | 差距 |
|----------------|-------------|------|
| SELECT (基础) | ✅ 稳定 | 无 |
| SELECT (复杂WHERE) | ⚠️ 部分 | 三值逻辑、NOT IN 边界 |
| INSERT | ✅ 稳定 | 无 |
| INSERT ... ON DUPLICATE KEY UPDATE | ✅ 稳定 | 无 |
| REPLACE INTO | ✅ **v2.8.0 新增** | 无 |
| UPDATE (多表) | ⚠️ 部分 | JOIN UPDATE 限制 |
| DELETE (多表) | ⚠️ 部分 | JOIN DELETE 限制 |
| LOAD DATA INFILE | ❌ | **完全缺失** |
| SELECT ... FOR UPDATE | ⚠️ 部分 | 行锁实现不完整 |

### 4.3 表达式与函数差距

| MySQL 5.7 功能 | v2.8.0 状态 | 差距 |
|----------------|-------------|------|
| 算术运算 | ✅ | 无 |
| 比较运算 | ✅ | 无 |
| 逻辑运算 (AND/OR/NOT) | ⚠️ 部分 | 三值逻辑 NULL 传播 |
| IS NULL / IS NOT NULL | ⚠️ 部分 | 表达式解析覆盖 |
| CASE 表达式 | ⚠️ 部分 | 复杂条件 |
| 字符串函数 (CONCAT/SUBSTR) | ⚠️ 部分 | 参数数量/类型检查 |
| 日期函数 (DATE_FORMAT/NOW) | ⚠️ 部分 | 格式化选项不全 |
| 聚合函数 (COUNT/SUM/AVG) | ✅ | 基础可用 |
| 窗口函数 (RANK/DENSE_RANK) | ✅ v2.8.0 | ROW_NUMBER/RANK/DENSE_RANK |
| JSON 函数 | ⚠️ 部分 | 基础可用 |

### 4.4 JOIN 差距

| MySQL 5.7 功能 | v2.8.0 状态 | 差距 |
|----------------|-------------|------|
| INNER JOIN | ✅ 稳定 | 无 |
| LEFT JOIN | ✅ 稳定 | 无 |
| RIGHT JOIN | ✅ 稳定 | 无 |
| FULL OUTER JOIN | ✅ **v2.8.0 修复** | PR#1733 |
| CROSS JOIN | ❌ | parser 未覆盖 |
| NATURAL JOIN | ❌ | parser 未覆盖 |
| JOIN USING | ❌ | parser 未覆盖 |
| STRAIGHT_JOIN | ❌ | MySQL 特定语法 |

### 4.5 事务差距

| MySQL 5.7 功能 | v2.8.0 状态 | 差距 |
|----------------|-------------|------|
| BEGIN/COMMIT/ROLLBACK | ✅ 稳定 | 无 |
| SAVEPOINT | ⚠️ 开发中 | 不支持 ROLLBACK TO SAVEPOINT |
| REPEATABLE READ | ✅ | MVCC 实现 |
| READ COMMITTED | ✅ | 支持 |
| READ UNCOMMITTED | ✅ | 支持 |
| SERIALIZABLE | ❌ | **完全缺失** |
| XA 分布式事务 | ❌ | **完全缺失** |

### 4.6 MySQL 5.7 特定语法

| 语法 | v2.8.0 状态 | 优先级 |
|------|-------------|--------|
| `SHOW TABLES` | ✅ | P0 |
| `SHOW DATABASES` | ✅ | P0 |
| `SHOW CREATE TABLE` | ⚠️ 部分 | P1 |
| `DESCRIBE table` | ✅ | P0 |
| `EXPLAIN SELECT` | ✅ | P0 |
| `SET NAMES utf8` | ⚠️ 部分 | P1 |
| `SET autocommit=0` | ✅ | P0 |
| `USE database` | ❌ | P0 |
| `HANDLER table OPEN` | ❌ | P3 |
| `LOAD DATA INFILE` | ❌ | P1 |
| `DO expr` | ❌ | P2 |
| `SELECT ... INTO OUTFILE` | ❌ | P2 |

---

## 5. 增强计划

### 5.1 Phase 1: Parser 补齐（当前 Sprint）

| 功能 | 优先级 | 测试数 | 预期提升 |
|------|--------|--------|----------|
| 三值逻辑 (IS NULL/AND/OR/NOT) | P0 | 6 | Corpus +5%, Line +8% |
| CROSS JOIN 解析 | P0 | 1 | Corpus +2%, Line +2% |
| NATURAL JOIN 解析 | P0 | 1 | Corpus +2% |
| JOIN USING 解析 | P0 | 1 | Corpus +2% |
| 表达式优先级 | P1 | 4 | Line +4% |
| 语法错误恢复 | P1 | 4 | Line +5% |

**预期**: Corpus 通过率从 **40.8% → ~55%**，Parser Line 从 **54.70% → ~75%**

### 5.2 Phase 2: 执行器补齐

| 功能 | 优先级 | 依赖 |
|------|--------|------|
| ALTER TABLE MODIFY COLUMN | P0 | 执行器 DDL |
| USE database | P0 | Catalog/Session |
| 多表 UPDATE/DELETE | P1 | JOIN 合成 |
| SAVEPOINT + ROLLBACK TO | P1 | 事务 |
| LOAD DATA INFILE | P1 | 解析 + 执行 |
| SELECT ... FOR UPDATE | P1 | 行锁 |

**预期**: Corpus 通过率从 **~55% → ~70%**

### 5.3 Phase 3: 函数补齐

| 功能 | 优先级 | 说明 |
|------|--------|------|
| DATE_FORMAT 完整格式化 | P1 | MySQL 格式化字符串 |
| CONCAT_WS/GROUP_CONCAT | P1 | 常用字符串聚合 |
| COALESCE/IFNULL/NULLIF | P1 | NULL 处理 |
| LAST_INSERT_ID | P1 | 自增 ID 获取 |
| FOUND_ROWS | P2 | 查询行数 |

**预期**: Corpus 通过率从 **~70% → ~80%**

### 5.4 Phase 4: MySQL 协议补齐

| 功能 | 优先级 | 影响 |
|------|--------|------|
| COM_STATISTICS | P1 | 客户端兼容 |
| COM_PING | P1 | 连接池兼容 |
| COM_PROCESS_KILL | P1 | kill 查询 |
| COM_CHANGE_USER | P2 | 用户切换 |
| CLIENT_INTERACTIVE | P2 | 交互超时 |

---

## 6. 目标与里程碑

| 里程碑 | 日期 | Corpus 通过率 | Parser Line | MySQL 5.7 功能覆盖 |
|--------|------|---------------|-------------|-------------------|
| **v2.8.0 当前** | 2026-05-02 | **40.8%** | **54.70%** | **83%** |
| Phase 1 完成 | 2026-05-09 | 55% | 75% | 86% |
| Phase 2 完成 | 2026-05-16 | 70% | 82% | 88% |
| Phase 3 完成 | 2026-05-23 | 80% | 85% | 90% |
| **v2.8.0 目标** | GA | **≥ 80%** | **≥ 85%** | **≥ 92%** |
| v2.9.0 目标 | TBD | **≥ 92%** | **≥ 90%** | **≥ 95%** |

---

## 7. 风险与依赖

| 风险 | 影响 | 缓解 |
|------|------|------|
| Parser 重构风险大 | 影响现有功能 | 增量修改，覆盖率驱动 |
| Corpus 用例非 MySQL 5.7 专用 | 部分失败非兼容问题 | 分析失败原因，分类 |
| 函数实现依赖数据类型系统 | 日期/JSON 类型复杂度高 | 逐步实现，优先常用函数 |
| 缺少测试基础设施 | 回归风险 | 在 Corpus 中添加 CI 集成 |

---

## 8. 参考链接

- [功能矩阵](./FEATURE_MATRIX.md)
- [Parser 覆盖率分析](./PARSER_COVERAGE_ANALYSIS.md)
- [集成测试计划](./INTEGRATION_TEST_PLAN.md)
- [稳定性测试报告](./STABILITY_REPORT.md)
- [SQL Corpus 源码](../../../crates/sql-corpus/tests/corpus_test.rs)
- [Integration status](./INTEGRATION_STATUS.md)
- [开发计划](./DEVELOPMENT_PLAN.md)
- [v2.7.0 功能矩阵](../v2.7.0/FEATURE_MATRIX.md)

---

*本文档由 SQLRustGo Team 维护*
*更新日期: 2026-05-02*
