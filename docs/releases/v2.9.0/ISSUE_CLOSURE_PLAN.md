# v2.9.0 遗留问题闭环开发计划

> **版本**: 1.0
> **日期**: 2026-05-06
> **目标**: 所有 16 个 Open Issue 有明确的解决路径，形成闭环
> **版本归属**: v2.9.0 GA 后清理 → v3.0.0 剩余

---

## 一、总览

当前 Gitea 上有 16 个 Open Issue（不含已关闭的 7 个）。按解决时限分为三组：

| 组别 | 数量 | 时限 | 要求 |
|------|------|------|------|
| **A 组 — v2.9.0 解决** | 7 个 | 立即（GA 后 1-2 周） | 可在此版本内完成的修复/文档 |
| **B 组 — v3.0.0 解决** | 6 个 | v3.0.0 Development 阶段 | 需要较大功能开发 |
| **C 组 — 长期跟踪** | 3 个 | 持续 | META/Brainstorming |

---

## 二、A 组 — v2.9.0 内解决（7 个）

### A-1: #285 测试覆盖率 84.18% → ≥85%

| 项目 | 内容 |
|------|------|
| **当前** | 84.18%，距 GA 目标 85% 差 **0.82%** |
| **瓶颈** | `crates/parser/src/parser.rs` 仅 94 个 `#\[test\]`，`lexer.rs` 仅 11 个 |
| **方案** | 在 parser crate 增加 ~20-30 个测试用例，覆盖已知未测试的语法分支 |
| **估算** | 2-3 天（opencode 进行中） |
| **关闭条件** | `tarpaulin` 报告总覆盖率 ≥85%，parser 模块 ≥80% |
| **责任人** | opencode （已有分配） |

### A-2: #224 Cross Join 笛卡尔积支持

| 项目 | 内容 |
|------|------|
| **当前** | `JoinType::Cross` 在 planner 中存在，但 executor 返回空结果 |
| **源码证据** | `crates/executor/src/local_executor.rs` — HashJoin 的 `_ =>` 分支返回 empty |
| **方案** | 在 `execute_hash_join` 中为 `JoinType::Cross` 添加笛卡尔积实现（`cartesian_product` 函数已存在） |
| **估算** | 1 天 |
| **关闭条件** | `SELECT * FROM a CROSS JOIN b` 返回正确笛卡尔积 |
| **紧急性** | 低（非生产阻塞）但简单可做 |

### A-3: #227 Mini-Fuzz 随机化测试生成

| 项目 | 内容 |
|------|------|
| **当前** | 未实现 |
| **方案** | 创建 `crates/fuzz/` 简单模糊测试框架，基于 SQL Corpus 模板随机生成 SQL 并对比 SQLite |
| **估算** | 2-3 天 |
| **关闭条件** | 模糊测试脚本可运行，发现至少 1 个 SQL 解析差异 |
| **注意** | 简单版本即可，不需要完整 AFL/libfuzzer 集成 |

### A-4: #120 E-01~E-08 生产就绪（收尾）

| 项目 | 内容 |
|------|------|
| **当前** | E-01~E-08 核心实现已完成（E-07 连接池、E-08 QPS 优化、E-09 合并） |
| **遗留** | E-07 Connection Pooling 完整集成测试、Sysbench QPS ≥5K 验证 |
| **方案** | 1) 检查 QPS benchmark 当前数值（已 ≥10K） 2) 确认连接池与 E-08/E-09 集成测试通过 |
| **估算** | 1 天 |
| **关闭条件** | 验证所有 E-01~E-08 + E-09 功能已实现，写关闭总结 |

### A-5: #116 G-01~G-05 可信任治理体系（收尾）

| 项目 | 内容 |
|------|------|
| **当前** | R4/R7/R9/B1-B5 门禁已建立，scripts/gate/ 目录 28 个脚本 |
| **新增** | R9 (check_regression.sh), R10 (check_tpch.sh), GA (check_ga.sh) |
| **方案** | 梳理 G-01~G-05 各子项完成状态，关闭已完成项 |
| **估算** | 0.5 天 |
| **关闭条件** | 门禁文档与实现匹配，无未完成子项 |

### A-6: #263 测试基础设施说明与 CI/CD 集成指南

| 项目 | 内容 |
|------|------|
| **当前** | 文档缺失 |
| **方案** | 创建 `docs/guides/TEST_INFRASTRUCTURE_GUIDE.md`，说明 gate 脚本、QPS benchmark、TPC-H 的工作流 |
| **估算** | 1 天 |
| **关闭条件** | 指南文档创建 |

### A-7: #216 Phase 1-3 done K1-K3 pending

| 项目 | 内容 |
|------|------|
| **当前** | 内容极少（"Phase 1-3 done. K1-K3 pending."） |
| **方案** | 确认 K1-K3 具体内容，如已过期/无意义则关闭。否则明确任务范围 |
| **估算** | 0.5 天 |
| **关闭条件** | 关闭或明确重开 |

---

## 三、B 组 — v3.0.0 解决（6 个）

### B-1: #234 TPC-H 9/22 → 18/22

| 项目 | 内容 |
|------|------|
| **当前** | Q1/Q6 可运行（OOM on full run with 866K rows） |
| **阻塞原因** | LEFT JOIN 不完整、子查询/EXISTS 性能差、内存不足 |
| **方案** | Phase 0 Debt Sprint + Phase 1 Performance Pocket 后，TPC-H 查询自然改善 |
| **v3.0.0 阶段** | Phase 2 (SQL Completeness) 后验证 |
| **关闭条件** | SF=0.1 环境下 ≥18/22 查询可运行无 OOM |

### B-2: #277 TPC-H SQLite/MySQL/PostgreSQL 对比

| 项目 | 内容 |
|------|------|
| **当前** | SQLite 22 查询已完成。MySQL/PostgreSQL 未做 |
| **依赖** | #234 TPC-H 查询可运行 |
| **方案** | 在 TPC-H 查询覆盖率达到目标后，用 SQLite/MySQL(docker)/PostgreSQL(docker) 统一对比 |
| **v3.0.0 阶段** | Phase 3 (Infrastructure) |

### B-3: #201 Formal Verification Phase 2

| 项目 | 内容 |
|------|------|
| **当前** | P0 (S0-S05) 已 VERIFIED。S1-S5 未完成 |
| **方案** | Phase 2 增强覆盖率：DDL、MVCC SSI、UPDATE/DELETE 的形式化证明 |
| **v3.0.0 阶段** | Phase 3 (Infrastructure) 或独立 parallel track |
| **关闭条件** | S1-S5 全部 VERIFIED |

### B-4: #235 PROOF-026 Write Skew / SSI

| 项目 | 内容 |
|------|------|
| **当前** | FROZEN 状态 |
| **方案** | 在 SSI 脆弱性加固（Phase 0 D-04）完成后，重新评估并实现 PROOF-026 |
| **v3.0.0 阶段** | Phase 0 D-04 (SSI 加固) |
| **关闭条件** | Proof verified + SSI 实现正确性确认 |

### B-5: #118 C-01~C-06 SQL 兼容性

| 项目 | 内容 |
|------|------|
| **当前** | SQL Corpus 92.6%（远超 Beta 目标 80%） |
| **遗留** | DATE_ADD INTERVAL 语法、TRIM LEADING/TRAILING 修饰符、窗口函数完整支持 |
| **方案** | 余下语法问题归入 v3.0.0 Phase 2 (SQL Completeness) |
| **v3.0.0 阶段** | Phase 2 F-02 (窗口函数补全) + 独立 DATE_ADD/TRIM 修复 |
| **关闭条件** | SQL Corpus ≥95% |

### B-6: #175 TPC-H Q1-Q22 真实数据测试（SF=0.1）

| 项目 | 内容 |
|------|------|
| **当前** | 数据导入已完成，性能数据已收集 |
| **依赖** | #234 TPC-H 查询可运行 |
| **方案** | 与 #234 合并处理，作为 TPC-H 查询实现的一部分 |
| **关闭条件** | 关闭 #234 时一并关闭 |

---

## 四、C 组 — 长期跟踪（3 个）

| Issue | 标题 | 状态 | 策略 |
|-------|------|------|------|
| #321 | 治理文档整改 Brainstorming | 进行中 | 保持 open，每 sprint 审查一次进展 |
| #11 | Hermes/OpenCode 协作手册 | 长期 | META Issue，持续更新不关闭 |
| #336 | 本汇总 Issue | 当前 | 保持 open 作为跟踪索引 |

---

## 五、时间线

```
Week 1 (5/7-5/13): A 组 — v2.9.0 清理
  A-1 #285 覆盖率 (opencode 进行中)
  A-2 #224 Cross Join (1d)
  A-3 #227 Mini-Fuzz 简单版 (2-3d)
  A-4 #120 E 系列收尾 (1d)
  A-5 #116 G 系列收尾 (0.5d)
  A-6 #263 CI/CD 指南 (1d)
  A-7 #216 确认关闭 (0.5d)

Week 2+ (v3.0.0 Development 阶段): B 组
  Phase 0 (Debt Sprint):  #235 PROOF-026 前置条件 (SSI 加固)
  Phase 2 (SQL):           #118 SQL 兼容性、#234 TPC-H
  Phase 3 (Infrastructure): #277 TPC-H 对比、#201 Formal Verification
                           #175 TPC-H 测试
```

---

## 六、各 Issue 关闭条件汇总

| # | 标题 | 关闭条件 | 版本 | 时限 |
|---|------|---------|------|------|
| 285 | 覆盖率 ≥85% | tarpaulin 报告 ≥85%，parser ≥80% | **v2.9.0** | 1-2 周 |
| 224 | Cross Join | CROSS JOIN SQL 返回正确笛卡尔积 | **v2.9.0** | 1 周 |
| 227 | Mini-Fuzz | 模糊测试脚本可运行 | **v2.9.0** | 1-2 周 |
| 120 | E-01~E-08 收尾 | 验证所有 E 系列功能实现 | **v2.9.0** | 1 周 |
| 116 | G-01~G-05 收尾 | 门禁文档与实现匹配 | **v2.9.0** | 0.5 周 |
| 263 | CI/CD 指南 | GUIDE.md 创建 | **v2.9.0** | 1 周 |
| 216 | K1-K3 确认 | 确认是否有效，关闭或重建 | **v2.9.0** | 0.5 周 |
| 234 | TPC-H 18/22 | SF=0.1 无 OOM，≥18/22 可运行 | v3.0.0 | Phase 2 |
| 277 | TPC-H 对比 | 三平台统一对比 | v3.0.0 | Phase 3 |
| 201 | Formal V Phase 2 | S1-S5 VERIFIED | v3.0.0 | Phase 3 |
| 235 | PROOF-026 | 证明通过 | v3.0.0 | Phase 0 |
| 118 | C-01~C-06 SQL | SQL Corpus ≥95% | v3.0.0 | Phase 2 |
| 175 | TPC-H SF=0.1 | 随 #234 关闭 | v3.0.0 | Phase 2 |
| 321 | 治理 Brainstorming | 持续跟踪 | 长期 | 每个 Sprint |
| 11 | 协作手册 | 持续更新 | 长期 | 持续 |
| 336 | 本索引 Issue | 所有子项关闭后关闭 | 长期 | — |

---

## 七、执行建议

### v2.9.0 GA 后立即（A 组）

1. **#285 覆盖率** — 是当前唯一 GA blocker。opencode 应优先完成 parser 测试
2. **#224 Cross Join** — 改动极小（新增 ~20 行），可作为预备练习
3. **#227 Mini-Fuzz** — 简单模糊测试可以快速切入，提升长期质量
4. **#120 #116 #263 #216** — 文档和确认工作，0.5-1 天

### v3.0.0 启动（B 组）

按现有 v3.0.0 五阶段计划进行：
- **Phase 0 (Debt Sprint)** — 解决 SSI 脆弱性（#235 前置条件）
- **Phase 2 (SQL Completeness)** — TPC-H 查询 + SQL 兼容性（#234 #118）
- **Phase 3 (Infrastructure)** — 对比测试 + 形式化验证（#277 #201）

---

*本文档已将 16 个 Open Issue 全部分配了解决路径。A 组 7 个在 v2.9.0 内解决，B 组 6 个在 v3.0.0 解决，C 组 3 个长期跟踪。形成闭环。*
