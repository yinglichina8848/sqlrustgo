# B 组遗留问题实施计划 — TPC-H 扩展 + 形式化验证

> **版本**: 1.0
> **日期**: 2026-05-06
> **基于**: v2.9.0 develop/v2.9.0
> **对应 v3.0.0 阶段**: Phase 0-3

---

## 一、总览

B 组共有 4 个未关闭 Issue，覆盖 TPC-H 查询扩展和形式化验证两个领域：

| # | 标题 | 类型 | 难度 | 估算 |
|---|------|------|------|------|
| #234 | TPC-H 9/22 → 18/22 | 功能 + 性能 | 🔴 H | 5-7 w |
| #277 | TPC-H 三平台对比 | 测试 | 🟡 M | 3-5 d |
| #235 | PROOF-026 Write Skew / SSI | 形式化验证 | 🔴 H | 3-4 w |
| #175 | TPC-H SF=0.1 测试 | 测试 | 🟢 E | 1-2 d |

**关键路径**: #234（5-7 周）主导时间线。#235 可并行。#277/#175 在 #234 完成后收尾。

---

## 二、依赖关系

```
Phase 0 (2w)           Phase 1 (4w)         Phase 2 (3w)       Phase 3 (2w)
─────────────────────────────────────────────────────────────────────────
D-04 SSI加固 ────────→ #235 PROOF-026 S2-S5 (并行 3-4w) ───────────→ ✅
                         │
D-01 CBO实现 ─────────→ PP-01 CBO完善 ──→ #234 TPC-H (5-7w) ─────→ #277对比
                                          │                       │
                                          └──→ #175 SF=0.1 ──────┘
```

---

## 三、#234 — TPC-H 9/22 → 18/22（5-7 周）

### 3.1 当前状态

| 工作状态 | 查询 | SF=0.1 耗时 | 阻塞项 |
|----------|------|------------|--------|
| ✅ 通过 | Q1, Q3, Q4, Q6, Q10, Q13, Q14, Q19, Q20, Q22 | 0.3-1.5s | 无 |
| ✅ 通过 | Q15 | 待验证 VIEW | 需要 CREATE VIEW 执行器支持 |
| ⚠️ 超时 | Q2, Q5, Q7, Q8, Q9, Q11, Q12, Q16, Q21 | - | 多表 JOIN + 聚合 + 子查询 |
| ❌ OOM | Q17, Q18 | - | 相关子查询 (correlated subquery) |

**当前 10 个查询可用**，目标 ≥18/22，还需 **8-12 个查询**。

### 3.2 实施步骤

#### Step 1: 验证基线（1 天）

```bash
cargo run -p sqlrustgo-bench-cli -- tpch-bench --ddl scripts/tpch/tpch_schema.sql --data ~/sqlrustgo-tpch/data --queries Q1,Q3,Q4,Q6,Q10,Q13,Q14,Q19,Q20,Q22 --iterations 3
```
**验收**: 10 个查询全部可重复运行，中位耗时 < 5s。

#### Step 2: 中难度查询实现（2-3 周）

| 查询 | SQL 特征 | 需要实现 | 优先 | 关键文件 |
|------|---------|---------|------|---------|
| Q2 | 多表 JOIN + MIN + 子查询 | IN → LeftSemi Join | P0 | `planner/src/planner.rs` |
| Q5 | 6 表 JOIN + GROUP BY + ORDER BY | 多表 join order | P0 | `planner/src/planner.rs` |
| Q7 | 派生表 JOIN + 字符串函数 | 子查询 as 表 | P1 | `executor/src/local_executor.rs` |
| Q8 | CASE WHEN + 子查询 | CASE WHEN 完善 | P1 | `src/execution_engine.rs` |
| Q9 | 表达式计算 + 聚合 | 表达式 eval 扩展 | P1 | `src/execution_engine.rs` |
| Q11 | HAVING + 子查询 | HAVING 完善 | P1 | `src/execution_engine.rs` |
| Q12 | CASE WHEN + GROUP BY | CASE WHEN | P1 | `src/execution_engine.rs` |
| Q15 | CREATE VIEW | parser/executor 联动 | P0 | `crates/parser/src/parser.rs` |
| Q16 | NOT IN / NOT EXISTS | LeftAnti Join | P0 | `planner/src/planner.rs` |
| Q21 | EXISTS + NOT EXISTS | EXISTS 执行 | P1 | `executor/src/local_executor.rs` |

**验收**: 新增 8 个查询可运行，总计 ≥18/22。

#### Step 3: 困难查询实现（2-3 周）

| 查询 | SQL 特征 | 需要实现 | 关键文件 |
|------|---------|---------|---------|
| Q17 | 相关子查询 (WHERE qty < 0.2*AVG) | 子查询去关联 | `crates/optimizer/src/rules.rs` |
| Q18 | 相关子查询 (HAVING SUM > 300) | 同上 | `crates/optimizer/src/rules.rs` |

**验收**: 3 个困难查询可运行（SF=0.1 ≤60s）。

#### Step 4: 性能调优（1 周）

对 ≥18/22 查询进行 CBO 收益验证，确保 Q1/Q6 耗时减少 ≥50%。

---

## 四、#235 — PROOF-026 Write Skew / SSI（3-4 周，并行）

### 4.1 当前状态

| 阶段 | 内容 | 状态 |
|------|------|------|
| S0 | TLA+ 模型文件存在 | ⚠️ 缺 SSI .cfg |
| S1 | Write Skew violated | ✅ |
| S2 | SSI Atomic pass | ⚠️ commitTs 语义弱 |
| S3 | SerializationGraph Rust 实现 | 🔴 未开始 |
| S4 | 并发 SSI 测试 | 🔴 未开始 |
| S5 | 闭环验证 | 🔴 未完成 |

### 4.2 实施步骤

#### Phase 0（1 周）— SSI 加固 + TLA+ 模型修复

| 任务 | 文件 | 工时 |
|------|------|------|
| D-04a: SSI cycle 检测优化 | `crates/transaction/src/ssi.rs` | 1d |
| D-04b: 修复 SSI TLA+ 模型 (commitTs) | `docs/formal/PROOF_016_mvcc_ssi.tla` | 2d |
| D-04c: 添加 .cfg 文件 | `docs/formal/PROOF_016_mvcc_ssi.cfg` | 0.5d |
| D-04d: 并发压力测试 (100 并发) | `crates/transaction/tests/` | 1d |

#### S3（1-2 周）— SerializationGraph

| 任务 | 文件 |
|------|------|
| 设计 Graph 数据结构 | `crates/transaction/src/ssi.rs` |
| 实现 `detect_cycle()` / `would_create_cycle()` | 同上 |
| 集成到事务 commit 路径 | `crates/transaction/src/transaction_manager.rs` |
| 单元测试 | `crates/transaction/tests/` |

#### S4（1 周）— 并发 SSI 测试

| 测试 | 目标 |
|------|------|
| Write Skew 检测 | 2 事务写交叉 |
| 无假阳性 | 100 并发合法操作 |
| TPC-H 并发 | 22 查询写入无冲突 |

#### S5（0.5 周）— 闭环

```bash
python3 docs/proof/verification_engine.py --verify ssi
cargo test --test ssi_integration_test
./scripts/formal/formal_smoke.sh
```

---

## 五、#277 — TPC-H 三平台对比（3-5 天）

**前提**: #234 TPC-H ≥18/22 可用 + docker 可用。

| 步骤 | 内容 | 耗时 |
|------|------|------|
| 1 | MySQL docker + TPC-H 数据导入 | 1d |
| 2 | PostgreSQL docker + 数据导入 | 1d |
| 3 | 运行 18+ 查询于三平台，收集耗时 | 1d |
| 4 | 生成 `tpch_comparison.json` | 0.5d |

---

## 六、#175 — TPC-H SF=0.1 测试（1-2 天）

**依赖**: #234。22 查询 SF=0.1 全量运行并记录到 `tpch_sf01_full_report.json`。

---

## 七、时间线

```
         W1   W2   W3   W4   W5   W6   W7   W8   W9   W10  W11  W12
         ──────────────────────────────────────────────────────────────
Phase 0  [SSI加固################]
               [#235 SerializationGraph + 测试 + 闭环 ████████████████████]
Phase 1         [CBO 完善 ████████████████]
                     [#234 Step2 中难度查询 ████████████████]
Phase 2                                    [#234 Step3 困难 ████████████]
Phase 3                                                    [调优]
                                                                [#277]
                                                                  [#175]
```

| 里程碑 | 时间 | 内容 |
|--------|------|------|
| M1 | Phase 0 末 | SSI 加固完成 + 新 TLA+ 模型 |
| M2 | Phase 1 末 | CBO 可用, #234 Step2 完成(≥16/22) |
| M3 | Dev 末 | #234 ≥18/22 + PROOF-026 闭环 |
| M4 | Alpha 末 | TPC-H 三平台对比完成 |

---

## 八、Agent 分配

| 任务 | Agent | 负责文件 |
|------|-------|---------|
| #234 Step 2 (中难度) | Executor Team | `planner/src/`, `executor/src/`, `src/execution_engine.rs` |
| #234 Step 3 (困难) | Optimizer Team | `optimizer/src/rules.rs` (子查询去关联) |
| #235 S3-S5 (PROOF-026) | Formal Team | `transaction/src/ssi.rs`, `docs/formal/*.tla` |
| #277 + #175 | Test Team | docker, bench-cli, comparison scripts |

---

## 九、风险矩阵

| 风险 | 影响 | 缓解 |
|------|------|------|
| Q17/Q18 相关子查询超 2w | TPC-H → 16/22 | v3.0.0 降级, Q17/Q18 → v3.1.0 |
| CBO 延后影响 TPC-H | Step3 无法开始 | Step2 不依赖 CBO |
| PROOF-026 复杂度超出预期 | v3.0.0 不含 SSI | Phase C 已有完整设计文档 |
| MySQL docker 不可用 | 对比不完整 | 仅 SQLite vs SQLRustGo |

---

## 十、关闭条件

| # | 标题 | 关闭条件 |
|---|------|---------|
| #234 | TPC-H 18/22 | `tpch-bench --queries all` ≥18/22 通过，无 OOM |
| #277 | TPC-H 对比 | `tpch_comparison.json` 含 4 平台数据 |
| #235 | PROOF-026 | `formal_smoke.sh` 含 SSI + `ssi_integration_test` 通过 |
| #175 | TPC-H SF=0.1 | 22 查询 SF=0.1 全部运行并记录 |

---

*版本 1.0 | 2026-05-06*
*B 组 4 个 Issue 全部分配到 v3.0.0 五阶段。关键路径 12 周。#235 可并行。*
