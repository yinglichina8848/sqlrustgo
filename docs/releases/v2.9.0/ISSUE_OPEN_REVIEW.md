# v2.9.0 Open Issue 逐项验证报告

**Date**: 2026-05-06
**Branch**: `develop/v2.9.0` @ `d370cd57`
**Total Open Issues**: 24

---

## 1. 可关闭的 Issue (Work Complete)

| Issue | Title | 关闭理由 |
|-------|-------|----------|
| #328 | [GA] E-09 UPDATE/DELETE QPS 优化完成 | E-09 优化已完成，DELETE 63K QPS，UPDATE 43K QPS，远超目标。性能基准已建立。相关工作已合并。 |
| #283 | GA 阶段 G2 混沌工程测试提前完成 | G2 混沌测试脚本已实现，CPU/网络/死锁测试均已验证通过。文档和 CI 集成已完成。 |
| #304 | v2.9.0 RC Coverage Report | RC Coverage Report 归档性文档，非门禁 blocker。Phase 2 覆盖率提升由 opencode 正在进行。 |
| #218 | [SQL兼容性] SQL Corpus测试报告 - Pass rate 92.6% | 92.6% 已远超 Beta 目标 80%。剩余 P0 问题（DATE_ADD/TRIM）属于 v3.0.0 改进项。 |
| #230 | [版本声明] SQL兼容性问题 v2.9.0 | 96.9% Pass rate，版本说明已清晰声明。不影响 GA 发布。 |

---

## 2. 无法关闭的 Issue (Work Incomplete)

### 2.1 P0 — GA Blocker

| Issue | Title | 状态 | 遗留工作 |
|-------|-------|------|----------|
| #285 | [GA-G3] 测试覆盖率提升：从 84.18% 提升至 ≥85% | **进行中** | 目标 85%，当前 84.18%，差 +0.82%。opencode 正在进行 parser 覆盖率提升。 |

### 2.2 P1 — 高优先级改进项

| Issue | Title | 状态 | 遗留工作 |
|-------|-------|------|----------|
| #298 | [E-08 Step 2] Hash Join O(n+m) 优化 | **未完成** | Hash Join 仍为 O(n×m) 嵌套循环，Q5/Q6/Q8/Q9 性能未达标。Step 2 未实施。 |
| #234 | [TPC-H] 测试改进：当前 9/22 查询可运行 | **未完成** | 当前 9/22，目标 ≥18/22。复杂 JOIN 查询 (Q2/Q5/Q7/Q8/Q9/Q11-Q18/Q21) 仍未实现。 |
| #175 | TPC-H Q1-Q22 真实数据测试 (SF=0.1) | **进行中** | 数据导入完成，性能数据已收集。但查询覆盖率仍不足。 |
| #277 | [HP Z6G4] TPC-H 全面测试：SQLite/MySQL/PostgreSQL 对比 | **进行中** | HP Z6G4 TPC-H 对比测试，SQLite/MySQL/PostgreSQL 对比未完成。 |
| #276 | [HP Z6G4] TPC-H 全面测试：SQLite/MySQL/PostgreSQL 对比 | **重复 Issue** | 与 #277 重复。 |
| #224 | [Phase 2] Cross Join - 笛卡尔积支持 | **未完成** | Cross Join 实现缺失 (Task 8/9 未完成)。P1 优先级。 |
| #227 | [Phase 2] Mini-Fuzz - 随机化测试生成 | **未完成** | Fuzz 测试框架未创建 (Task 17/18 未完成)。P2 优先级。 |
| #201 | [Formal Verification] Phase 2: Enhance Coverage | **部分完成** | P0 (S0-S05) 全部 VERIFIED ✅，但 Phase 2 增强覆盖率工作未完成。 |

### 2.3 P2 — 中优先级改进项

| Issue | Title | 状态 | 遗留工作 |
|-------|-------|------|----------|
| #235 | [PROOF-026] Write Skew / SSI — FROZEN | **冻结** | PROOF-026 Write Skew / SSI 处于 FROZEN 状态，未完成。 |
| #120 | E-01~E-08: 生产就绪 | **Beta 阶段** | E-07 Connection Pooling 完整集成测试未完成，SYSBENCH QPS ≥5K 未验证。 |
| #118 | C-01~C-06: SQL 兼容性 | **Beta 阶段** | SQL Corpus 92.6% (>80% Beta 目标)，但窗口函数完整支持未完成 (Beta 目标)。 |
| #116 | G-01~G-05: 可信任治理体系 | **Beta 阶段** | R6/R7 CI 验证、R9 回归检测刚建立但 concurrent_select_8t 不稳定。 |

### 2.4 Meta / 协调类

| Issue | Title | 状态 | 说明 |
|-------|-------|------|------|
| #321 | [External AI Review] 治理文档整改 Brainstorming | **进行中** | Brainstorming 阶段，尚未形成具体开发计划。 |
| #11 | [META] Hermes/OpenCode 分布式开发协作手册 | **长期** | META Issue，协调手册，持续更新中。 |
| #216 | test: Phase 1-3 done K1-K3 pending | **状态不明** | 内容极少 (Phase 1-3 done. K1-K3 pending.)，无法判断工作内容。 |

---

## 3. v2.9.0 GA 遗留问题汇总

### 3.1 GA Blocker (Must Fix for GA)

| 遗留项 | 当前 | 目标 | 负责人 |
|--------|------|------|--------|
| 总覆盖率 | 84.18% | ≥85% | opencode (进行中) |

### 3.2 性能优化遗留 (P1)

| 遗留项 | 当前 | 目标 | 状态 |
|--------|------|------|------|
| Hash Join | O(n×m) 嵌套循环 | O(n+m) Hash Join | 未实施 |
| TPC-H 查询支持 | 9/22 (41%) | ≥18/22 (82%) | 未完成 |
| concurrent_select_8t 稳定性 | 56% 波动 | ≤5% 稳定 | 需改进测试方法 |

### 3.3 SQL 兼容性遗留 (P1)

| 遗留项 | 当前 | 目标 | 状态 |
|--------|------|------|------|
| SQL Corpus Pass Rate | 92.6% | 95%+ | 接近目标 |
| DATE_ADD INTERVAL 语法 | 缺失 | 支持 | P0 改进项 |
| TRIM LEADING/TRAILING | 缺失 | 支持 | P0 改进项 |
| 窗口函数 | 部分支持 | 完整支持 | Beta 目标 |
| Cross Join | 不支持 | 支持 | 未实施 |

### 3.4 形式化验证遗留 (P2)

| 遗留项 | 当前 | 状态 |
|--------|------|------|
| PROOF-026 Write Skew / SSI | FROZEN | 未完成 |
| Formal Verification Phase 2 | 部分完成 | P0 verified，S1-S5 pending |

### 3.5 测试基础设施遗留 (P2)

| 遗留项 | 当前 | 状态 |
|--------|------|------|
| Mini-Fuzz 框架 | 未实现 | Task 17/18 pending |
| TPC-H 全面对比 | 部分进行 | SQLite/MySQL/PostgreSQL 对比未完成 |

---

## 4. 建议行动

### 立即 (GA 发布后)

1. **#285 覆盖率** — 继续由 opencode 完成 parser 覆盖率提升
2. **#298 Hash Join** — v3.0.0 开发周期内实施 O(n+m) 优化
3. **#234 TPC-H** — 优先实现 Q5/Q6/Q8/Q9 (Hash Join 受益查询)
4. **#224 Cross Join** — v3.0.0 实施计划中

### v3.0.0 规划

- DATE_ADD INTERVAL 语法修复
- TRIM LEADING/TRAILING 支持
- 窗口函数完整实现
- Write Skew / SSI (PROOF-026)
- TPC-H Q5-Q22 完整支持
- Mini-Fuzz 测试框架

---

## 5. Issue 关闭建议

| 操作 | Issue 列表 |
|------|-----------|
| **立即关闭** | #328, #283, #304, #218, #230 |
| **保持 Open** | #285 (GA blocker), #298, #234, #175, #277, #276, #224, #227, #201 |
| **降级为 v3.0.0** | #235 (PROOF-026 FROZEN), #120, #118, #116 |
| **待确认** | #321 (brainstorming), #11 (meta), #216 (状态不明) |
