# v3.1.0 覆盖缺口整改计划

> **Issue**: 基于 Issue #626 覆盖缺口扫描结果
> **Date**: 2026-05-12
> **Status**: 整改计划待评审

---

## 一、覆盖缺口汇总

基于 `scripts/coverage/scan_coverage_gaps.py` 扫描结果：

| 类别 | 缺口数量 | Critical |
|------|---------|----------|
| 总条目 | 76 | — |
| 覆盖缺口 | 43 | — |
| Critical (0%) | 1 | MERGE |

---

## 二、Critical Gap: MERGE 语句 (0%)

### 2.1 现状

| 组件 | 状态 | 证据 |
|------|------|------|
| Parser | ✅ 存在 | `parse_merge()` in parser.rs |
| Planner | ❓ 需验证 | Statement::Merge 存在 |
| Optimizer | ❓ 需验证 | 可能有 logical plan |
| Executor | ⚠️ 疑似缺失 | `execute_merge` 存在于 local_executor.rs 但可能 stub |
| 测试 | ❌ 无 | 无 execution_chain_regression_test |

**证据**:
```rust
// crates/parser/src/parser.rs
Some(Token::Merge) => self.parse_merge(),  // line 928
fn parse_merge(&mut self) -> Result<Statement, String>  // line 4013

// crates/executor/src/local_executor.rs
"Merge" => self.execute_merge(plan),  // line 290
```

### 2.2 整改任务

| 任务 | 优先级 | 估计工时 |
|------|--------|---------|
| 验证 MERGE parser 完整性 | P1 | 2h |
| 验证 MERGE planner 逻辑 | P1 | 2h |
| 验证/实现 MERGE executor | P1 | 4h |
| 添加 MERGE 集成测试 | P1 | 2h |

---

## 三、高优先级缺口 (< 70% 覆盖)

### 3.1 窗口函数 (NTILE/LEAD/LAG/FIRST_VALUE/LAST_VALUE/NTH_VALUE)

**现状**:
- Parser 存在：parsing 测试通过
- Executor **疑似不完整**：window_executor.rs 存在但可能只有框架
- 测试：window_function_test.rs 只有 parsing 测试

**证据**:
```rust
// tests/window_function_test.rs - 只测试 parsing
fn test_parse_lead() { ... }
fn test_parse_lag() { ... }
// 无 execution 测试
```

**整改任务**:

| 任务 | 优先级 | 估计工时 |
|------|--------|---------|
| 审计 window_executor.rs 实现 | P1 | 4h |
| 补全 LEAD/LAG execution | P2 | 8h |
| 补全 NTILE/FIRST_VALUE/LAST_VALUE/NTH_VALUE | P2 | 12h |
| 添加窗口函数集成测试 | P1 | 4h |

### 3.2 多表 DML (UPDATE/DELETE multi-table)

**现状**:
- Parser 支持存在
- Executor **疑似缺失**
- 测试：execution_chain_regression_test 标记为 `#[ignore]`

**证据**:
```rust
// tests/execution_chain_regression_test.rs
#[ignore]
fn test_execution_chain_multi_table_update()  // 失败
#[ignore]
fn test_execution_chain_multi_table_delete()  // 失败
```

**整改任务**:

| 任务 | 优先级 | 估计工时 |
|------|--------|---------|
| 审计 multi-table UPDATE executor | P1 | 4h |
| 审计 multi-table DELETE executor | P1 | 4h |
| 实现缺失逻辑 | P1 | 8h |
| 添加集成测试 | P1 | 2h |

### 3.3 TRUNCATE / RENAME TABLE

**现状**:
- Parser 支持存在
- Executor **疑似 stub**
- 测试：execution_chain_regression_test 标记为 `#[ignore]`

**整改任务**:

| 任务 | 优先级 | 估计工时 |
|------|--------|---------|
| 审计 TRUNCATE executor | P2 | 2h |
| 审计 RENAME TABLE executor | P2 | 2h |
| 实现缺失逻辑 | P2 | 4h |
| 添加测试 | P2 | 2h |

### 3.4 子查询 (Scalar/Row/Table subquery)

**现状**:
- Parser 支持存在
- Executor **疑似部分实现**
- 测试：execution_chain_regression_test 标记为 `#[ignore]`

**整改任务**:

| 任务 | 优先级 | 估计工时 |
|------|--------|---------|
| 审计 subquery executor | P1 | 4h |
| 实现缺失的子查询类型 | P2 | 8h |
| 添加测试 | P1 | 4h |

---

## 四、中等优先级缺口

### 4.1 DDL 语句

| 语句 | 覆盖率 | 优先级 |
|------|--------|--------|
| CREATE VIEW | 50% | P2 |
| DROP VIEW | 50% | P2 |
| CREATE DATABASE | 60% | P2 |
| DROP DATABASE | 55% | P2 |
| ALTER TABLE DROP | 65% | P2 |
| ALTER TABLE MODIFY | 60% | P2 |

### 4.2 DCL 语句

| 语句 | 覆盖率 | 优先级 |
|------|--------|--------|
| ALTER USER | 50% | P2 |
| RENAME USER | 30% | P3 |
| CREATE ROLE | 40% | P3 |
| DROP ROLE | 35% | P3 |
| GRANT ROLE | 30% | P3 |
| REVOKE ROLE | 30% | P3 |
| SET PASSWORD | 25% | P3 |

### 4.3 TCL 语句

| 语句 | 覆盖率 | 优先级 |
|------|--------|--------|
| RELEASE SAVEPOINT | 50% | P2 |
| ROLLBACK TO | 55% | P2 |
| SET TRANSACTION | 40% | P3 |

### 4.4 集合运算

| 语句 | 覆盖率 | 优先级 |
|------|--------|--------|
| INTERSECT | 45% | P2 |
| EXCEPT | 40% | P2 |
| MINUS | 40% | P3 |

---

## 五、MVCC 可见性形式化验证 (Issue #624)

### 5.1 现状

根据 `docs/formal/FORMAL_SYSTEM_STATUS.md`:
- MVCC (PROOF-016) 已有 Spec→Code→Test
- 但 **Issue #624 要求"MVCC 可见性形式化验证 (TLA+)"**

### 5.2 差距分析

| 项目 | 现状 | 需求 |
|------|------|------|
| MVCC Spec | 存在 | 需要反例测试 |
| TLA+ 规格 | 需验证是否存在 | 需要完整 TLA+ |
| 可见性规则证明 | 无 | 需要 |
| 写偏斜测试 | 无 | 需要 |

### 5.3 整改任务

| 任务 | 优先级 | 估计工时 |
|------|--------|---------|
| 审计现有 MVCC TLA+ 规格 | P1 | 4h |
| 补充可见性规则 TLA+ | P1 | 8h |
| 添加反例测试 | P1 | 4h |
| 添加写偏斜压力测试 | P2 | 4h |

---

## 六、OO 文档补全 (Issue #627)

### 6.1 现状

根据 `OO_DOCUMENT_ANALYSIS.md`，大量 OO 执行链路文档缺失：

| README 标记 | 实际文件 | 状态 |
|------------|---------|------|
| dml/DML_EXECUTION.md | ❌ 不存在 | 需创建 |
| ddl/DDL_EXECUTION.md | ❌ 不存在 | 需创建 |
| ddl/ALTER_EXECUTION.md | ❌ 不存在 | 需创建 |
| ddl/INDEX_EXECUTION.md | ❌ 不存在 | 需创建 |
| dcl/DCL_EXECUTION.md | ❌ 不存在 | 需创建 |
| join/JOIN_ALGORITHMS.md | ❌ 不存在 | 需创建 |
| wal/WAL_PROTOCOL.md | ❌ 不存在 | 需创建 |
| recovery/CRASH_RECOVERY.md | ❌ 不存在 | 需创建 |
| distributed/DISTRIBUTED_SYNC.md | ❌ 不存在 | 需创建 |
| transaction/TX_MANAGEMENT.md | ❌ 不存在 | 需创建 |
| transaction/MVCC_IMPLEMENTATION.md | ❌ 不存在 | 需创建 |
| query/SUBQUERY_EXECUTION.md | ❌ 不存在 | 需创建 |
| query/WINDOW_FUNCTIONS.md | ❌ 不存在 | 需创建 |
| query/RECURSIVE_CTE.md | ❌ 不存在 | 需创建 |
| cbo/CBO_DESIGN.md | ❌ 不存在 | 需创建 |
| cbo/CBO_COST_MODEL.md | ❌ 不存在 | 需创建 |
| cbo/CBO_JOIN_ORDERING.md | ❌ 不存在 | 需创建 |
| advanced/TRIGGER_EXECUTION.md | ❌ 不存在 | 需创建 |
| advanced/STORED_PROCEDURE.md | ❌ 不存在 | 需创建 |
| setops/SET_OPERATIONS.md | ❌ 不存在 | 需创建 |

**共 20 个文档缺失**。

### 6.2 整改任务

| 任务 | 优先级 | 估计工时 |
|------|--------|---------|
| 创建核心 OO 文档 (10个) | P0 | 40h |
| 创建高级 OO 文档 (10个) | P1 | 40h |

---

## 七、整改优先级排序

### P0 (v3.1.0-alpha 前必须完成)

| Issue | 任务 | 依赖 |
|-------|------|------|
| #626 | MERGE executor 验证/实现 | Parser 已有 |
| #624 | MVCC 可见性 TLA+ | 现有 MVCC 代码 |
| #627 | 10 个核心 OO 文档 | 执行链路分析 |

### P1 (v3.1.0-beta 前完成)

| 任务 | 依赖 |
|------|------|
| 窗口函数完整实现 | Parser 已有 |
| 多表 DML 实现 | 已有部分 |
| 子查询完善 | 已有部分 |
| 10 个高级 OO 文档 | P0 文档 |

### P2 (v3.1.0-RC 前完成)

| 任务 |
|------|
| DDL 缺口补全 |
| DCL 缺口补全 |
| TCL 缺口补全 |

---

## 八、工时估算汇总

| 类别 | 任务数 | 估计工时 |
|------|--------|---------|
| Critical (MERGE) | 4 | 10h |
| P1 功能 | 8 | 40h |
| P2 功能 | 12 | 30h |
| MVCC TLA+ | 4 | 20h |
| OO 文档 (核心) | 10 | 40h |
| OO 文档 (高级) | 10 | 40h |
| **总计** | **48** | **180h** |

---

## 九、后续行动

1. **评审本文档** → 确认整改范围和优先级
2. **创建 Issue #624** → MVCC 可见性形式化验证
3. **创建 Issue #627** → OO 文档补全
4. **更新 Issue #626** → 添加 MERGE 专项任务
5. **分配资源** → 开始 P0 任务

---

## 附录：扫描脚本输出

```
Critical Gaps (0% coverage, ❌ status):
| Statement | Keyword | Reason |
|-----------|---------|--------|
| MERGE | merge | No test coverage |

All Gaps (<70% coverage):
| Statement | 覆盖率 | 缺口 |
|-----------|--------|------|
| TRUNCATE | 40% | 无测试 |
| RENAME TABLE | 30% | 无测试 |
| CREATE VIEW | 50% | 无测试 |
| DROP VIEW | 50% | 无测试 |
| ... |
```