# v2.9.0 开发计划 — 阶段目标与任务分配

> **Issue 标题**: v2.9.0 开发阶段计划 — MySQL 5.7 兼容 + SQL Corpus/TPC-H/Sysbench
> **文件**: docs/releases/v2.9.0/DEVELOPMENT_PLAN.md (更新版)
> **阶段**: 开发阶段 (develop/v2.9.0) → Alpha → Beta → RC → GA
> **铁律**: 所有功能开发在开发阶段完成，Alpha 起只做集成测试 + Bug 修复

---

## 一、现状评估与差距

### MySQL 5.7 兼容性差距 (45.5/100 → 目标: 90/100)

| 分类 | 支持 | 缺失 | 优先级 |
|------|------|------|--------|
| **DDL** | CREATE TABLE, DROP TABLE, CREATE INDEX, ALTER TABLE (基本), TRUNCATE, CREATE ROLE | ALTER TABLE ADD/DROP COLUMN, CREATE VIEW, REPLACE INTO | 🔴 P0 |
| **DML** | INSERT, UPDATE, DELETE, SELECT (基本) | INSERT ... ON DUPLICATE KEY UPDATE, DELETE/UPDATE with JOIN, LIMIT with OFFSET | 🔴 P0 |
| **表达式** | AND/OR/NOT, 比较运算, IN, BETWEEN?, LIKE, IS NULL | CASE WHEN, EXTRACT(), DATE() | 🟡 P1 |
| **JOIN** | INNER JOIN, LEFT JOIN (部分) | RIGHT JOIN, FULL OUTER JOIN, CROSS JOIN, NATURAL JOIN | 🟡 P1 |
| **子查询** | 基本子查询 | EXISTS, IN (subquery), 相关子查询, CTE/WITH | 🔴 P0 |
| **聚合** | COUNT, SUM, AVG, MIN, MAX, GROUP BY, HAVING | 窗口函数 (ROW_NUMBER, RANK), OVER 子句 | 🟢 P2 |
| **MySQL 协议** | 基本查询协议 | COM_STMT_PREPARE, COM_STMT_EXECUTE, 多结果集, EOF 包编码 | 🔴 P0 |
| **事务** | BEGIN/COMMIT/ROLLBACK, 基本隔离级别 | SAVEPOINT, XA (已完成), 自动提交模式 | 🟢 P2 |

### SQL Corpus 差距 (85.4% → 目标: ≥95%)

| 文件组 | 当前状态 | 缺失原因 | 优先级 |
|--------|---------|---------|--------|
| DML/INSERT (6文件) | ✅ 基础通过 | batch_operations, upsert_operations 需 REPLACE | 🔴 P0 |
| DML/DELETE (4文件) | ⚠️ 部分 | delete_complex, delete_variations 多表语法 | 🔴 P0 |
| DDL/ALTER_TABLE (2文件) | ⚠️ 部分 | column_operations ADD/DROP/MODIFY | 🔴 P0 |
| DDL/VIEW (1文件) | ❌ | CREATE VIEW / DROP VIEW 未实现 | 🟡 P1 |
| ADVANCED/JOINS (1文件) | ⚠️ 部分 | FULL OUTER JOIN, CROSS JOIN | 🟡 P1 |
| ADVANCED/UNION (1文件) | ❌ | UNION 实现不完整 (右孩子忽略 bug) | 🔴 P0 |
| ADVANCED/SUBQUERIES (1文件) | ❌ | EXISTS, IN(subquery) 不支持 | 🔴 P0 |
| PROCEDURES (1文件) | ❌ | CREATE PROCEDURE 未对接执行引擎 | 🟡 P1 |

### Sysbench 差距 (当前: point_select → 目标: 完整 OLTP ≥10K QPS)

| 场景 | 当前 | 要求 | 阻塞项 |
|------|------|------|--------|
| point_select | ✅ ~1,840 QPS | 基线 | 无 |
| oltp_read_only | ❌ | ≥1,500 QPS | COM_STMT_PREPARE 协议不完整 |
| oltp_write_only | ❌ | ≥500 QPS | INSERT ON DUPLICATE KEY UPDATE |
| oltp_read_write | ❌ | ≥1,000 QPS | 读写混合协议支持 |

### TPC-H 差距 (当前: 4/22 → 目标: 18+/22)

| 查询 | 当前 | 需要 | 阻塞项 |
|------|------|------|--------|
| Q1, Q3, Q6, Q11 | ✅ | Pricing/Priority/Forecast/Inventory | - |
| Q2, Q4, Q5, Q7, Q8, Q9 | ❌ | LEFT JOIN, EXISTS, 复杂子查询 | 子查询 + JOIN 能力 |
| Q10 | ❌ | GROUP BY + ORDER BY 多列 | ORDER BY 列排序 |
| Q12-Q22 | ❌ | LEFT JOIN, CREATE VIEW, EXISTS, 复杂聚合 | 多个语言特性 |

### 测试框架差距

| 类型 | 当前 | 目标 | 阶段 |
|------|------|------|------|
| 单元测试 | 3630 (#[test]) | ≥4000 | 开发 |
| 集成测试 | 28 文件, 手动 | CI 自动全量 | Alpha |
| SQL Corpus | 103 文件, 426 用例 | 150 文件, 600 用例 | 开发+Alpha |
| 性能基准 | 无基线 | sysbench + TPC-H | Beta |
| 覆盖率 | 仅收集, 无门禁 | Develop≥50%, Beta≥65%, GA≥85% | Alpha→GA |

---

## 二、开发阶段任务 (develop/v2.9.0) — 当前必须完成

### Sprint 1: MySQL 5.7 核心命令补齐 (P0, ~7天)

| # | 任务 | 文件 | 工时 | 验收标准 |
|---|------|------|------|---------|
| 1.1 | REPLACE INTO 和 INSERT ON DUPLICATE KEY UPDATE | parser.rs → execution_engine.rs | 2d | sysbench oltp_write_only prepare 成功 |
| 1.2 | ALTER TABLE ADD/DROP/MODIFY COLUMN | parser.rs + AlterTableExec | 2d | column_operations.sql 全部通过 |
| 1.3 | DELETE/UPDATE with JOIN | parser.rs + executor | 1d | delete_variations.sql 通过 |
| 1.4 | TRUNCATE TABLE 全实现 | parser.rs + executor | 0.5d | DDL 测试通过 |
| 1.5 | SHOW DATABASES / TABLES / CREATE TABLE | mysql-server 协议层 | 1d | sysbench prepare 连接正常 |
| 1.6 | LIMIT with OFFSET | parser -> planner -> executor | 0.5d | limit_statements.sql 通过 |

### Sprint 2: 子查询与高级 JOIN (P0, ~5天)

| # | 任务 | 文件 | 工时 | 验收标准 |
|---|------|------|------|---------|
| 2.1 | EXISTS / NOT EXISTS 子查询 | parser.rs -> executor | 2d | subquery_statements.sql 通过, TPC-H Q2/Q4 |
| 2.2 | IN (subquery) / NOT IN (subquery) | parser.rs -> executor | 1d | 子查询测试 |
| 2.3 | UNION bug 修复 (右孩子被忽略) | planner/src/planner.rs:210 | 0.5d | union_statements.sql 通过 |
| 2.4 | CTE / WITH 子句 | parser.rs | 2d | C-02 测试 (32 用例) |

### Sprint 3: Sysbench 协议兼容 (P0, ~3天)

| # | 任务 | 文件 | 工时 | 验收标准 |
|---|------|------|------|---------|
| 3.1 | COM_STMT_PREPARE / COM_STMT_EXECUTE | mysql-server/src/protocol.rs | 2d | sysbench oltp_read_only 可运行 |
| 3.2 | 多结果集支持 | mysql-server/src/protocol.rs | 0.5d | sysbench prepare 返回正确 |
| 3.3 | EOF packet 编码修正 | mysql-server/src/protocol.rs | 0.5d | sysbench 全部场景连接正常 |

### Sprint 4: TPC-H 关键查询 (P1, ~5天)

| # | 任务 | 文件 | 工时 | 验收标准 |
|---|------|------|------|---------|
| 4.1 | LEFT JOIN 全部实现 | planner/executor | 1d | TPC-H Q4/Q7/Q10 可运行 |
| 4.2 | CREATE VIEW / DROP VIEW | parser + catalog | 1d | view_operations.sql 通过 |
| 4.3 | GROUP BY 多列 + ORDER BY 排序 | executor | 1d | TPC-H Q10 通过 |
| 4.4 | EXTRACT(), DATE() 函数 | parser + executor | 1d | TPC-H Q7/Q9/Q14 |
| 4.5 | CASE WHEN / COALESCE | parser -> executor | 1d | TPC-H Q6/Q17 改进 |

### Sprint 5: 测试框架建设 (P1, ~3天)

| # | 任务 | 文件 | 工时 | 验收标准 |
|---|------|------|------|---------|
| 5.1 | 集成测试自动化脚本 | scripts/test/run_integration.sh | 1d | ✅ 已创建 |
| 5.2 | CI test-regression job | .gitea/workflows/ci.yml | 1d | PR 触发自动运行 |
| 5.3 | SQL Corpus 新增用例 | sql_corpus/ | 1d | C-02~C-06 框架搭建 |

---

## 三、Alpha 阶段任务 — 仅集成 + 验证 (不做新功能)

| # | 任务 | 验收标准 |
|---|------|---------|
| A1 | SQL Corpus ≥85% 门禁 | 103 文件, 426 用例全量 |
| A2 | Sysbench OLTP 套件运行 | 4 场景全部可执行 |
| A3 | 覆盖率 ≥50% | tarpaulin 报告验证 |
| A4 | Alpha 回归测试 | cargo test --all-features 100% |
| A5 | 集成测试 CI 强制执行 | test-alpha job 在 PR 拦截失败 |

## 四、Beta 阶段任务 — 性能 + 安全

| # | 任务 | 验收标准 |
|---|------|---------|
| B1 | TPC-H 全量 ≥18/22 | SF 0.01 全部通过 |
| B2 | Sysbench ≥5K QPS | oltp_read_write |
| B3 | 覆盖率 ≥65% | 强制门禁 |
| B4 | 安全测试 100% | 81 tests PASS |
| B5 | 性能基线记录 | benchmark_baseline.json |

## 五、RC 阶段任务 — 生产级验证

| # | 任务 | 验收标准 |
|---|------|---------|
| R1 | TPC-H SF 0.1 全量 | 全部通过 |
| R2 | Sysbench ≥10K QPS | 性能目标达成 |
| R3 | 形式化证明 P011-P012 | verified |
| R4 | 覆盖率 ≥80% | 强制门禁 |
| R5 | 72h 稳定性 | 无 crash |

## 六、GA 阶段任务 — 最终审计

| # | 任务 | 验收标准 |
|---|------|---------|
| G1 | 错误修正回溯 | 全部已知问题已修复 |
| G2 | 混沌工程 | CPU 80% + 30% 丢包 |
| G3 | 覆盖率 ≥85% | 最终门禁 |
| G4 | GA 发布检查清单 | 全部 ☐ → ✅ |

---

## 七、总体时间线

```
Week 1 (5/4-5/10): Sprint 1 MySQL命令补齐 ─────────────────
Week 2 (5/11-5/17): Sprint 2 子查询+JOIN + Sprint 3 Sysbench
Week 3 (5/18-5/24): Sprint 4 TPC-H + Sprint 5 测试框架
                    ↓ 全部功能开发截止 ↓
Week 4 (5/25-5/31): Alpha — 集成测试 + SQL Corpus全面验证
Week 5 (6/1-6/7):   Beta — TPC-H + Sysbench + 安全
Week 6 (6/8-6/14):  RC — 全量 + 性能 + 形式化证明
Week 7 (6/15-6/21): GA — 最终审计 + 混沌工程 + 发布
```

## 八、关键交付物检查表

```
☐ Sprint 1: MySQL 5.7 核心命令补齐
  ☐ REPLACE INTO
  ☐ ALTER TABLE ADD/DROP COLUMN
  ☐ DELETE/UPDATE JOIN
  ☐ TRUNCATE TABLE
  ☐ SHOW DATABASES/TABLES
  ☐ LIMIT OFFSET

☐ Sprint 2: 子查询与高级 JOIN
  ☐ EXISTS / NOT EXISTS
  ☐ IN (subquery)
  ☐ UNION bug 修复
  ☐ CTE/WITH

☐ Sprint 3: Sysbench 协议
  ☐ COM_STMT_PREPARE
  ☐ COM_STMT_EXECUTE
  ☐ 多结果集
  ☐ EOF packet 编码

☐ Sprint 4: TPC-H
  ☐ LEFT JOIN 完整
  ☐ CREATE VIEW
  ☐ EXTRACT/DATE 函数
  ☐ CASE WHEN
  ☐ GROUP BY + ORDER BY 多列

☐ Sprint 5: 测试框架
  ☐ run_integration.sh
  ☐ CI test-regression
  ☐ SQL Corpus 新增用例 (C-02~C-06)
```
