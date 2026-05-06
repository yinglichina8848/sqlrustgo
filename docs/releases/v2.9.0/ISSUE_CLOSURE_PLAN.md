# v2.9.0 遗留问题闭环开发计划（已更新）

> **版本**: 2.0
> **日期**: 2026-05-06
> **目标**: 所有 Open Issue 有明确的解决路径，形成闭环
> **版本归属**: v2.9.0 GA 后清理 → v3.0.0 剩余

---

## 一、总览

| 组别 | 原始 | 已关闭 | 剩余 | 内容 |
|------|------|--------|------|------|
| **A 组 — v2.9.0 解决** | 7 | 6 | **1** | #285 覆盖率（opencode 进行中） |
| **B 组 — v3.0.0 解决** | 6 | 2 | **4** | #234 #277 #235 #175 |
| **C 组 — 长期跟踪** | 3 | 0 | **3** | #321 #11 #336 |
| **合计** | 16 | **8** | **8** | 闭环率 50% |

---

## 二、A 组 — v2.9.0 内解决（6/7 关闭）

| # | 标题 | 状态 | 关闭理由 |
|---|------|------|---------|
| #224 | Cross Join 笛卡尔积 | ✅ **已关闭** | 解析器+执行器+测试全部通过（255/255） |
| #227 | Mini-Fuzz 框架 | ✅ **已关闭** | `crates/fuzz/` 创建，`cargo run -p sqlrustgo-fuzz` 可用 |
| #120 | E-01~E-08 生产就绪 | ✅ **已关闭** | E-09 验证 DELETE 63K/UPDATE 43K QPS |
| #116 | G-01~G-05 治理体系 | ✅ **已关闭** | 28 个门禁脚本 + RC/GA 门禁 |
| #263 | CI/CD 集成指南 | ✅ **已关闭** | `docs/guides/TEST_INFRASTRUCTURE_GUIDE.md` 创建 |
| #216 | K1-K3 确认 | ✅ **已关闭** | 过期，由 #336 承接 |
| **#285** | **覆盖率 84.18%→≥85%** | 🔴 **剩余** | opencode 进行中（parser 测试） |

---

## 三、B 组 — v3.0.0 解决（2/6 关闭）

### 已关闭

| # | 标题 | 关闭理由 | 验证数据 |
|---|------|---------|---------|
| #118 | C-01~C-06 SQL 兼容性 | SQL Corpus 96.7% (469/485 passed)，远超 80% 目标 | ✅ 实测通过 |
| #201 | Formal Verification Phase 2 | S0-S5 死锁防护全部闭环，19 个 formal specs | ✅ FORMAL_SYSTEM_STATUS.md 确认 |

### 剩余（需 v3.0.0 实现）

| # | 标题 | 当前状态 | v3.0.0 阶段 | 关闭条件 |
|---|------|---------|-------------|---------|
| #234 | TPC-H 9/22→18/22 | Q1/Q6 可运行 (OOM on full) | Phase 2 (SQL) | SF=0.1 无 OOM，≥18/22 |
| #277 | TPC-H SQLite/MySQL/PG 对比 | SQLite 22 查询完成，MySQL/PG 未做 | Phase 3 (Infra) | 三平台统一对比 |
| #235 | PROOF-026 Write Skew/SSI | FROZEN，Phase C 处理 | Phase 0 (SSI 加固) | Proof verified |
| #175 | TPC-H SF=0.1 测试 | 数据已导入，随 #234 关闭 | Phase 2 (SQL) | 随 #234 关闭 |

---

## 四、C 组 — 长期跟踪（3 个）

| # | 标题 | 状态 | 策略 |
|---|------|------|------|
| #321 | 治理文档 Brainstorming | 进行中 | 每 sprint 审查 |
| #11 | Hermes/OpenCode 协作手册 | 长期 | META，持续更新 |
| #336 | 本汇总 Issue | 当前 | 所有子项关闭后关闭 |

---

## 五、时间线

```
当前: A 组 — #285 覆盖率（opencode）
         其余 6 个 A 组问题已全部关闭 ✅

v3.0.0 Development:
  Phase 0 (Debt Sprint):  #235 PROOF-026 (SSI 加固)
  Phase 2 (SQL):           #234 TPC-H 18/22, #175 TPC-H SF=0.1
  Phase 3 (Infrastructure): #277 TPC-H 三平台对比
```

---

## 六、各 Issue 关闭条件汇总

| # | 标题 | 关闭条件 | 版本 |
|---|------|---------|------|
| **285** | 覆盖率 ≥85% | tarpaulin ≥85%, parser ≥80% | **v2.9.0** |
| 234 | TPC-H 18/22 | SF=0.1 无 OOM，≥18/22 | v3.0.0 Phase 2 |
| 277 | TPC-H 对比 | SQLite/MySQL/PG 统一对比 | v3.0.0 Phase 3 |
| 235 | PROOF-026 | Proof verified | v3.0.0 Phase 0 |
| 175 | TPC-H SF=0.1 | 随 #234 关闭 | v3.0.0 Phase 2 |

---

## 七、实测验证记录

| Check | 结果 | 日期 |
|-------|------|------|
| SQL Corpus pass rate | **96.7%** (469/485) | 2026-05-06 |
| Formal specs count | **19** (docs/formal/) | 2026-05-06 |
| Dafny proofs | **3** (PROOF-011,013,014) | 2026-05-06 |
| TLA+ proofs | **1** (PROOF-012) | 2026-05-06 |
| Phase B S0-S5 | ✅ All closed | 2026-05-03 |
| SQLite TPC-H | 22/22 queries | 2026-05-06 |
| SQLRustGo TPC-H | Q1(2.8s), Q6(1.9s) | 2026-05-06 |

---

*文档版本: 2.0*
*原始 16 个 Issue 中 8 个已关闭。剩余 8 个：1 个 A 组 + 4 个 B 组 + 3 个 C 组。*
