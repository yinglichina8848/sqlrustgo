# v3.0.0 Alpha Release Gate Checklist

> **版本**: v3.0.0-alpha
> **日期**: 2026-05-06
> **用途**: Alpha 阶段结束前必须全部通过的门禁

---

## A-Gate 门禁清单

### A-OPT: 优化器激活 ✅

- [x] ConstantFolding 激活 (`SELECT 1+2` → `3`)
- [x] PredicatePushdown 激活（filter 下推至 TableScan）
- [x] ProjectionPruning 激活（只读必要列）
- [x] 优化器桥接验证测试（4 个测试全部通过）

### A-SQL: SQL 兼容性 ✅

- [x] IN 语法 (`WHERE id IN (1,2,3)`)
- [x] DISTINCT (`SELECT DISTINCT c FROM t`)
- [x] CASE 表达式 (`CASE WHEN c>0 THEN 1 ELSE 0 END`)
- [x] COALESCE (`COALESCE(NULL, c)`)
- [x] IN 子查询 (`WHERE c IN (SELECT ...)`)
- [x] EXISTS 子查询 (`WHERE EXISTS (SELECT 1)`)

### A-EXEC: 执行引擎 ✅

- [x] HashJoin 结果正确
- [x] 聚合函数正确 (COUNT/SUM/AVG)
- [x] GROUP BY 正确
- [x] ORDER BY 正确
- [x] LIMIT/OFFSET 正确

### A-TX: 事务隔离 ✅

- [x] Read Committed 正确
- [x] Snapshot Isolation 正确
- [x] 写冲突检测正确
- [x] 事务回滚正确

### A-HYG: 代码质量 ⚠️

- [ ] `cargo test --all-features --workspace` ≥80% 通过
- [ ] `cargo llvm-cov --all --all-features` 整体 ≥50%
- [x] `cargo clippy --all-features -- -D warnings` 零警告
- [x] `cargo fmt --all -- --check` 零差异
- [ ] `bash scripts/gate/check_docs_links.sh` 零死链
- [ ] `cargo audit` 无高危漏洞

---

## 覆盖率门槛

| 模块 | Alpha 目标 | 当前状态 |
|------|-----------|---------|
| executor | ≥45% | ⚠️ 待实测 |
| optimizer | ≥40% | ⚠️ 待实测 |
| parser | ≥50% | ⚠️ 待实测 |
| storage | ≥15% | ⚠️ 待实测 |
| catalog | ≥50% | ⚠️ 待实测 |
| **整体** | **≥50%** | **⚠️ 待实测** |

---

## 性能门槛

| 指标 | 门槛 | 当前状态 |
|------|------|---------|
| UPDATE QPS | ≥10,000 | ⚠️ 待实测 |
| DELETE QPS | ≥10,000 | ⚠️ 待实测 |
| concurrent_select_8t | ≥5,000 | ⚠️ 待实测 |
| TPC-H SF=0.1 | 22/22 | ✅ 已验证 |

---

## Alpha → Beta 晋升条件

所有 P0 项（A-OPT/A-SQL/A-EXEC/A-TX）全部 ✅ +
A-HYG 全部项 ✅ 或有明确 Beta 补完计划 +
覆盖率 ≥50% 达成 → 可晋升 Beta 阶段

---

## 文档完整性

- [x] RELEASE_NOTES.md
- [x] CHANGELOG.md
- [x] FEATURE_MATRIX.md
- [x] INTEGRATION_STATUS.md
- [x] TEST_PLAN.md
- [x] PERFORMANCE_TARGETS.md
- [x] MIGRATION_GUIDE.md
- [ ] RELEASE_GATE_CHECKLIST.md (本文档)
- [ ] ALPHA_INTEGRATION_TESTING_PLAN.md (补充中)