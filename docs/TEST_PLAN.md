# SQLRustGo 测试计划

> **版本**: master
> **日期**: 2026-05-08
> **目的**: 作为测试计划入口文档，建立与门禁的同步机制

---

## 一、版本导航

| 版本 | 测试计划 | 门禁规范 |
|------|----------|----------|
| **v3.0.0** | `docs/releases/v3.0.0/TEST_PLAN.md` | `docs/governance/gate_spec_v300.md` |
| v2.9.0 | `docs/releases/v2.9.0/TEST_PLAN.md` | `docs/governance/gate_spec.md` |
| v2.7.0 | `docs/releases/v2.7.0/TEST_PLAN.md` | `docs/governance/gate_spec.md` |

---

## 二、TEST_PLAN 与门禁同步机制

### 2.1 核心原则

```
规范定义 ←→ 脚本实现 ←→ 测试计划
    ↑              ↓            ↓
    ←── 双向同步 ──────────←
```

| 触发事件 | 同步要求 |
|----------|----------|
| gate_spec 新增检查项 | 测试计划必须包含对应测试方法 |
| check_*.sh 新增检查项 | gate_spec 必须同步更新（SSOT） |
| TEST_PLAN 新增测试场景 | 门禁脚本必须包含执行方式 |
| 门禁阈值变更 | 测试计划和 gate_spec 必须同时更新 |

### 2.2 同步检查命令

```bash
# 检查 gate_spec 与 check_*.sh 是否一致
bash scripts/gate/check_gate_sync.sh

# 检查 gate_spec 与 TEST_PLAN 是否一致
bash scripts/gate/check_test_plan_sync.sh
```

### 2.3 同步检查脚本

`scripts/gate/check_gate_sync.sh` — 检查门禁规范与脚本一致性：

```bash
#!/usr/bin/env bash
# check_gate_sync.sh — 检查 gate_spec 与 check_*.sh 一致性
set -euo pipefail

echo "=== 门禁规范与脚本同步检查 ==="

# 1. 检查 gate_spec 中定义的 G1-G13 是否在 check_ga_v300.sh 中
echo "[检查 1] gate_spec vs check_ga_v300.sh"
# ...

# 2. 检查 check_ga_v300.sh 中的检查项是否在 gate_spec 中定义
echo "[检查 2] check_ga_v300.sh vs gate_spec"
# ...

# 3. 检查 Beta/RC 门禁脚本是否与 gate_spec 一致
echo "[检查 3] Beta/RC 门禁脚本一致性"
# ...
```

`scripts/gate/check_test_plan_sync.sh` — 检查 TEST_PLAN 与门禁一致性：

```bash
#!/usr/bin/env bash
# check_test_plan_sync.sh — 检查 TEST_PLAN 与 gate_spec 一致性
set -euo pipefail

echo "=== TEST_PLAN 与门禁同步检查 ==="

# 1. 检查 TEST_PLAN 中的检查项是否在 gate_spec 中定义
echo "[检查 1] TEST_PLAN vs gate_spec"
# ...

# 2. 检查 gate_spec 中的检查项是否有对应测试
echo "[检查 2] gate_spec 覆盖项是否在 TEST_PLAN 中"
# ...

# 3. 检查阈值是否一致
echo "[检查 3] 阈值一致性"
# ...
```

---

## 三、v3.0.0 快速入口

### 3.1 门禁命令

| 阶段 | 命令 | 说明 |
|------|------|------|
| L0 冒烟 | `bash scripts/gate/check_l0_smoke.sh` | <5min |
| Alpha | `bash scripts/gate/check_alpha_v300.sh` | 基础门禁 |
| Beta | `bash scripts/gate/check_beta_v300.sh` | 功能冻结 |
| RC | `bash scripts/gate/check_rc_v300.sh` | 发布候选 |
| GA | `bash scripts/gate/check_ga_v300.sh` | 正式发布 |

### 3.2 测试命令

| 测试 | 命令 | 说明 |
|------|------|------|
| 核心 crate | `cargo test -p sqlrustgo-{types,parser,planner,optimizer,executor,storage,transaction,catalog} --lib -- --test-threads=8` | L1 模块回归 |
| SQL Corpus | `cargo test -p sqlrustgo-sql-corpus test_sql_corpus_all -- --nocapture` | SQL 兼容性 |
| TPC-H SF=0.1 | `bash scripts/gate/check_tpch.sh sf=0.1` | Beta 门禁 |
| TPC-H SF=1 | `bash scripts/gate/check_tpch.sh sf=1` | RC/GA 门禁 |
| 性能回归 | `bash scripts/gate/check_regression.sh` | QPS 基线 |

### 3.3 覆盖率

```bash
# 完整覆盖率（可能超时）
cargo llvm-cov --all-features --json --output-path /tmp/cov.json

# 核心 crate 覆盖率（推荐）
cargo llvm-cov \
  -p sqlrustgo-types \
  -p sqlrustgo-parser \
  -p sqlrustgo-planner \
  -p sqlrustgo-optimizer \
  -p sqlrustgo-executor \
  -p sqlrustgo-storage \
  -p sqlrustgo-transaction \
  -p sqlrustgo-catalog \
  --all-features --json
```

---

## 四、门禁与测试对应关系

### 4.1 Alpha 门禁 (A1-A7)

| ID | 检查项 | 测试计划对应项 |
|----|--------|---------------|
| A1 | 编译 | 构建测试 |
| A2 | 单元测试 | 单元测试覆盖率 |
| A3 | Clippy | 代码质量检查 |
| A4 | 格式 | 代码格式检查 |
| A5 | 文档链接 | 文档完整性 |
| A6 | 覆盖率 ≥50% | 覆盖率模块级要求 |
| A7 | 安全扫描 | 安全测试 |

### 4.2 Beta 门禁 (B1-B9, B-S*)

| ID | 检查项 | 测试计划对应项 |
|----|--------|---------------|
| B1 | 编译 | 构建测试 |
| B2 | 核心测试 ≥90% | L1 模块回归 |
| B3 | Clippy | 代码质量检查 |
| B4 | 格式 | 代码格式检查 |
| B5 | 覆盖率 ≥75% | 覆盖率模块级要求 |
| B6 | 安全扫描 | 安全测试 |
| B7 | 文档链接 | 文档完整性 |
| B8 | TPC-H SF=0.1 22/22 | TPC-H 测试 |
| B9 | SQL Corpus ≥85% | SQL Corpus 测试 |
| B-S1 | concurrency_stress_test | 并发压力测试 |
| B-S2 | crash_recovery_test | 崩溃恢复测试 |
| B-S3 | long_run_stability_test | 长时间稳定性测试 |
| B-S4 | wal_integration_test | WAL 集成测试 |
| B-S5 | network_tcp_smoke_test | 网络稳定性测试 |
| B-S10 | SQL operations ≥20% | operations 类别测试 |

### 4.3 RC 门禁 (R1-R12)

| ID | 检查项 | 测试计划对应项 |
|----|--------|---------------|
| R1 | 编译 | 构建测试 |
| R2 | 全量测试 100% | L0+L1+L2 完整测试 |
| R3 | Clippy | 代码质量检查 |
| R4 | 格式 | 代码格式检查 |
| R5 | 覆盖率 ≥85% | 覆盖率模块级要求 |
| R6 | 安全扫描 | 安全测试 |
| R7 | 文档完整性 | 文档完整性检查 |
| R8 | SQL Compat ≥95% | SQL Corpus 测试 |
| R9 | TPC-H SF=1 22/22 | TPC-H SF=1 测试 |
| R10 | 性能基线 | 性能回归测试 |
| R11 | Sysbench | Sysbench 测试 |
| R12 | MySQL Protocol | 协议兼容性测试 |

### 4.4 GA 门禁 (G1-G13)

| ID | 检查项 | 测试计划对应项 |
|----|--------|---------------|
| G1 | 编译 | 构建测试 |
| G2 | 全量测试 100% | L0+L1+L2 完整测试 |
| G3 | Clippy | 代码质量检查 |
| G4 | 格式 | 代码格式检查 |
| G5 | 覆盖率 ≥85% | 覆盖率模块级要求 |
| G6 | 安全扫描 | 安全测试 |
| G7 | Point SELECT ≥10K QPS | QPS 性能测试 |
| G8 | UPDATE ≥5K QPS | QPS 性能测试 |
| G9 | DELETE ≥2K QPS | QPS 性能测试 |
| G10 | TPC-H SF=1 22/22 | TPC-H SF=1 测试 |
| G11 | SQL Corpus ≥98% | SQL Corpus 测试 |
| G12 | B-S 稳定性 | 稳定性测试 |
| G13 | MySQL Protocol | 协议兼容性测试 |

---

## 五、同步检查清单

每次 gate_spec、check_*.sh 或 TEST_PLAN 更新后，必须执行以下检查：

```
[ ] gate_spec 新增检查项 → check_*.sh 是否实现？
[ ] check_*.sh 新增检查项 → gate_spec 是否同步？
[ ] gate_spec 阈值变更 → TEST_PLAN 是否同步？
[ ] TEST_PLAN 新增测试 → 门禁脚本是否包含？
[ ] 新增门禁项 → 是否有对应 Issue 追踪？
```

---

## 六、相关文档

| 文档 | 作用 |
|------|------|
| `docs/governance/gate_spec_v300.md` | 门禁规范 SSOT |
| `docs/governance/gate_lifecycle_tracking.md` | 门禁失败项追踪 |
| `docs/governance/governance_self_improvement.md` | 治理自我进化机制 |
| `docs/releases/v3.0.0/TEST_PLAN.md` | v3.0.0 测试计划详细版 |
| `docs/governance/TEST_PLAN_v3.md` | 覆盖率弱项分析 |
| `scripts/gate/check_gate_sync.sh` | 门禁规范与脚本同步检查 |
| `scripts/gate/check_test_plan_sync.sh` | TEST_PLAN 与门禁同步检查 |

---

*最后更新: 2026-05-08*
