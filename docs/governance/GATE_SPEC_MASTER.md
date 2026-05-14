# SQLRustGo 门禁规范 - 主模板（SSOT）

> **版本**: 1.0
> **创建日期**: 2026-05-14
> **目的**: 建立所有版本、所有阶段门禁的标准化模板，杜绝 AI 自由发挥定义门禁

> **SSOT 声明**: 本文档是 SQLRustGo 门禁定义的唯一权威来源（Single Source of Truth）。所有版本（v2.9.0, v3.0.0, v3.1.0...）的门禁检查必须基于本文档，不得自行定义或修改。

---

## 一、门禁体系概述

### 1.1 四级门禁模型

```
A-Gate → B-Gate → R-Gate → G-Gate
 (α入口)  (β入口)  (RC入口)  (GA入口)
```

| 门禁 | 名称 | 阶段目标 | 覆盖率目标 | 性能目标 |
|------|------|----------|------------|----------|
| A-Gate | Alpha Gate | 开发完成，可运行原型 | ≥50% | 基线建立 |
| B-Gate | Beta Gate | 功能冻结，进入稳定期 | ≥75% | TPC-H SF=0.1 22/22 |
| R-Gate | RC Gate | 发布候选，性能优化完成 | ≥85% | TPC-H SF=1 22/22 + QPS 基线 |
| G-Gate | GA Gate | 正式发布，所有门槛达标 | ≥85% | Point Select ≥10K QPS |

### 1.2 门禁通过条件

| 条件类型 | 说明 |
|----------|------|
| **全部通过** | 所有门禁检查项必须 PASS |
| **有豁免记录** | 不满足的项必须在 `GATE_EXEMPTIONS.md` 有审批通过的豁免记录 |
| **无阻塞性 OPEN Issue** | milestone 下的 OPEN Issue 不能阻塞门禁 |

### 1.3 门禁失败处理原则

```
门禁 FAIL → 必须创建 Issue → 必须有修复 PR → 必须验证通过 → 才能关闭 Issue
```

---

## 二、A-Gate (Alpha Gate)

### 2.1 入口条件

- 所有计划的 feature 已实现
- 核心功能可运行
- 无 P0 Bug

### 2.2 检查清单

| # | 检查项 | 命令/方法 | 通过标准 | 证据格式 |
|---|--------|----------|---------|---------|
| A1 | 编译检查 | `cargo build --workspace` | 无错误 | `{command, exit_code}` |
| A2 | 单元测试 | `cargo test --workspace` | ≥80% 通过 | `{passed, failed, exit_code}` |
| A3 | Clippy 检查 | `cargo clippy --all-features -- -D warnings` | 零警告 | `{warnings, exit_code}` |
| A4 | 格式化检查 | `cargo fmt --all -- --check` | 无格式错误 | `{diff_count, exit_code}` |
| A5 | 文档链接检查 | `bash scripts/gate/check_docs_links.sh` | 无死链 | `{broken_links}` |
| A6 | 覆盖率检查 | `cargo llvm-cov --all-features --lcov --output-path lcov.info` | ≥50% | `{total_pct}` |
| A7 | 安全扫描 | `cargo audit` | 无高危漏洞 | `{vulnerabilities}` |

### 2.3 覆盖率要求

| 模块 | 目标 |
|------|------|
| executor | ≥45% |
| optimizer | ≥40% |
| storage | ≥15% |
| catalog | ≥50% |
| parser | ≥50% |
| **整体** | **≥50%** |

### 2.4 证据格式

```json
{
  "gate": "A-GATE-v{VERSION}",
  "commit": "<sha>",
  "status": "PASS|FAIL",
  "evidence": {
    "A1_build": {"command": "cargo build --all-features", "exit_code": 0},
    "A2_test": {"command": "cargo test --all-features", "passed": N, "failed": M, "exit_code": 0},
    "A3_clippy": {"command": "cargo clippy --all-features", "warnings": 0, "exit_code": 0},
    "A4_fmt": {"command": "cargo fmt --all -- --check", "diff_count": 0},
    "A5_docs": {"command": "bash scripts/gate/check_docs_links.sh", "broken_links": 0},
    "A6_coverage": {"total_pct": N, "pass": true},
    "A7_security": {"command": "cargo audit", "vulnerabilities": 0}
  }
}
```

---

## 三、B-Gate (Beta Gate)

### 3.1 入口条件

- A-Gate 已通过
- TPC-H SF=0.1 22/22 查询可运行（无 OOM）
- 无 P0/P1 Bug

### 3.2 检查清单

| # | 检查项 | 命令/方法 | 通过标准 | 证据格式 |
|---|--------|----------|---------|---------|
| B1 | 编译检查 | `cargo build --release --workspace` | 无错误 | `{command, exit_code}` |
| B2 | 全量测试 | `cargo test --all-features` | ≥90% 通过 | `{passed, failed, exit_code}` |
| B3 | Clippy 检查 | `cargo clippy --all-features -- -D warnings` | 零警告 | `{warnings, exit_code}` |
| B4 | 格式化检查 | `cargo fmt --all -- --check` | 无格式错误 | `{diff_count, exit_code}` |
| B5 | 覆盖率检查 | `cargo llvm-cov --all-features --lcov --output-path lcov.info` | ≥75% | `{total_pct}` |
| B6 | 安全扫描 | `cargo audit` | 无高危漏洞 | `{vulnerabilities}` |
| B7 | 文档链接检查 | `bash scripts/gate/check_docs_links.sh` | 无死链 | `{broken_links}` |
| B8 | TPC-H SF=0.1 | `scripts/gate/check_tpch.sh sf=0.1` | 22/22 通过，无 OOM | `{passed, total, oom_count}` |
| B9 | SQL Corpus | `cargo test -p sqlrustgo-sql-corpus` | ≥85% | `{passed, total, pct}` |

### 3.3 稳定性测试清单

| # | 检查项 | 命令/方法 | 通过标准 | 证据格式 |
|---|--------|----------|---------|---------|
| B-S1 | concurrency_stress_test | `cargo test --test concurrency_stress_test` | 全部通过 | `{passed, total}` |
| B-S2 | crash_recovery_test | `cargo test --test crash_recovery_test` | 全部通过 | `{passed, total}` |
| B-S3 | long_run_stability_test | `cargo test --test long_run_stability_test` | 全部通过 | `{passed, total}` |
| B-S4 | wal_integration_test | `cargo test --test wal_integration_test` | 全部通过 | `{passed, total}` |
| B-S5 | network_tcp_smoke_test | `cargo test --test network_tcp_smoke_test` | 全部通过 | `{passed, total}` |
| B-S6 | ssi_stress_test | `cargo test -p sqlrustgo-transaction --test ssi_stress_test` | 全部通过 | `{passed, total}` |

### 3.4 覆盖率要求

| 模块 | 目标 |
|------|------|
| executor | ≥60% |
| optimizer | ≥50% |
| storage | ≥20% |
| catalog | ≥60% |
| parser | ≥60% |
| **整体** | **≥75%** |

---

## 四、R-Gate (RC Gate)

### 4.1 入口条件

- B-Gate 已通过
- TPC-H SF=1 22/22 查询可运行（无 OOM）
- SQL Corpus ≥95%
- 无 P0/P1 Bug

### 4.2 检查清单

| # | 检查项 | 命令/方法 | 通过标准 | 证据格式 |
|---|--------|----------|---------|---------|
| R1 | Build | `cargo build --release --workspace` | 无错误 | `{command, exit_code}` |
| R2 | Test | `cargo test --all-features` | 100% 通过 | `{passed, failed, exit_code}` |
| R3 | Clippy | `cargo clippy --all-features -- -D warnings` | 零警告 | `{warnings, exit_code}` |
| R4 | Format | `cargo fmt --all -- --check` | 无格式错误 | `{diff_count, exit_code}` |
| R5 | Coverage | `cargo llvm-cov --all-features --lcov` | ≥85% | `{total_pct, module_pcts}` |
| R6 | Security | `cargo audit` | 无高危漏洞 | `{vulnerabilities}` |
| R7 | Docs | R7a 死链 + R7b 文档存在 + R7c 版本一致 + R7d 文档一致 | 无死链/缺失/不一致 | `{broken_links, missing_docs}` |
| R8 | SQL Compat | `cargo test -p sqlrustgo-sql-corpus` | ≥95% | `{passed, total, pct}` |
| R9 | TPC-H SF=1 | `scripts/gate/check_tpch.sh sf=1` | 22/22 可运行 | `{passed, total, oom_count}` |
| R10 | Performance Baseline | `cargo bench && scripts/gate/check_perf_baseline.sh` | QPS 退化 ≤5% | `{baseline_path, delta_pct, pass}` |
| R11 | Sysbench Gate | `scripts/gate/check_sysbench.sh` | Point/UPDATE/INSERT 对比 baseline | `{point_qps, update_qps, insert_qps, delta}` |
| R12 | MySQL Protocol | mysql:5.7 容器握手测试 | 连接成功 | `{handshake, query_response}` |

### 4.3 R7 扩展说明

R7 包含四个子检查：

| 子项 | 检查内容 | 命令/方法 |
|------|----------|-----------|
| R7a | 死链检查 | `bash scripts/gate/check_docs_links.sh` |
| R7b | 必选文档存在性 | 检查 `docs/governance/VERSION_DOCS_SPEC.md` 定义的最小文档集 |
| R7c | 版本号一致性 | 所有文档头部版本号与当前版本一致，无遗留旧版本号 |
| R7d | 文档与代码状态一致性 | 代码中标注的 feature 与文档描述匹配，Issue 引用有效 |

### 4.4 R10 性能回归检查规范

**基线文件**: `perf_baselines/v{VERSION}/baseline.json`

**退化判定**:
- QPS 退化 ≤5% → PASS
- QPS 退化 5%-20% → 需人工解释
- QPS 退化 >20% → FAIL

### 4.5 覆盖率要求

| 模块 | 目标 |
|------|------|
| executor | ≥75% |
| optimizer | ≥70% |
| storage | ≥40% |
| catalog | ≥70% |
| parser | ≥40% |
| **整体** | **≥85%** |

---

## 五、G-Gate (GA Gate)

### 5.1 入口条件

- R-Gate 已通过
- Point Select QPS ≥10,000
- UPDATE QPS ≥5,000
- DELETE QPS ≥2,000
- TPC-H SF=1 22/22 无 OOM
- SQL Corpus ≥98%
- 所有已知问题已关闭

### 5.2 检查清单

| # | 检查项 | 命令/方法 | 通过标准 | 证据格式 |
|---|--------|----------|---------|---------|
| G1 | Build | `cargo build --release --workspace` | 无错误 | `{command, exit_code}` |
| G2 | Test | `cargo test --all-features` | 100% 通过 | `{passed, failed, exit_code}` |
| G3 | Clippy | `cargo clippy --all-features -- -D warnings` | 零警告 | `{warnings, exit_code}` |
| G4 | Format | `cargo fmt --all -- --check` | 无格式错误 | `{diff_count, exit_code}` |
| G5 | Coverage | `cargo llvm-cov --all-features --lcov` | ≥85% | `{total_pct, module_pcts}` |
| G6 | Security | `cargo audit` | 无高危漏洞 | `{vulnerabilities}` |
| G7 | Point Select QPS | `cargo bench -- point_select` | **≥10,000 ops/s** | `{qps, threshold, pass}` |
| G8 | UPDATE QPS | `cargo bench -- update_simple` | **≥5,000 ops/s** | `{qps, threshold, pass}` |
| G9 | DELETE QPS | `cargo bench -- delete_simple` | **≥2,000 ops/s** | `{qps, threshold, pass}` |
| G10 | TPC-H SF=1 | `scripts/gate/check_tpch.sh sf=1` | 22/22 通过，无 OOM | `{passed, total, oom_count}` |
| G11 | SQL Corpus | `cargo test -p sqlrustgo-sql-corpus` | **≥98%** | `{passed, total, pct}` |
| G12 | B-S 稳定性测试 | B-S1~B-S6 全部 | 全部 PASS | `{b_s1_pass, ..., b_s6_pass}` |
| G13 | MySQL Protocol | mysql:5.7 容器握手测试 | 连接成功 | `{handshake, query_response}` |

### 5.3 G7/G8/G9 性能测量要求

> **注**: G7/G8/G9 要求实际运行 `cargo bench` 并解析 ops/s 输出，禁止仅依赖 check_regression.sh 的回归检测。

### 5.4 覆盖率要求

| 模块 | 目标 |
|------|------|
| executor | ≥80% |
| optimizer | ≥70% |
| storage | ≥40% |
| catalog | ≥75% |
| parser | ≥40% |
| **整体** | **≥85%** |

---

## 六、Issue 追踪闭环要求

### 6.1 核心原则

```
门禁失败 → 必须有 Issue → 必须有修复 PR → 必须验证通过 → 才能关闭 Issue
```

### 6.2 Issue 创建标准

| 门禁项 | Issue 标题模板 |
|--------|---------------|
| A/B/R/G-N 测试 | `[{N}] 全量测试通过率 {X%}，低于 {Y%} 要求` |
| A/B/R/G-N 覆盖率 | `[{N}] {模块} 覆盖率 {X%}，低于 {Y%} 要求` |
| R/G-N 性能 | `[{N}] {指标} {X}，低于 {Y} 要求` |
| R/G-N TPC-H | `[{N}] TPC-H SF={N} {N}/22 通过` |
| B-SN/R-SN 稳定性 | `[{N}] {测试名} {M}/{K} 通过` |

### 6.3 Issue 关闭验证（强制）

**禁止在没有 PR 证据的情况下关闭 Issue。**

关闭前必须验证：
1. `gh issue view {id} --json closedByPullRequestsReferences` 结果非空
2. `gh pr view {pr_number} --json state,mergedAt` state=MERGED
3. 相关测试在 CI 通过
4. 门禁重新检查 PASS

---

## 七、版本延续追踪要求

### 7.1 延续触发条件

满足以下任一条件，必须将任务延续到下个版本：

| 条件 | 说明 |
|------|------|
| 修复需要 3 人周以上 | 超出当前版本开发周期 |
| 涉及架构变更 | 必须在下一个大版本迭代 |
| 优先级冲突 | 当前版本有更高优先级的 P0 任务 |
| 需要等待其他依赖 | 如 CBO 需要先完成索引选择 |

### 7.2 延续标准格式

```markdown
## v{NEXT_VERSION} 延续任务（来自 v{CURRENT_VERSION} 未完成项）

| 原 Issue | 任务描述 | 原版本状态 | v{NEXT_VERSION} 目标 | 验收条件 |
|----------|----------|------------|---------------------|----------|
| #{issue} | {任务} | {当前状态} | {目标} | {验收条件} |
```

---

## 八、豁免规则

| 豁免类型 | 条件 | 审批人 |
|----------|------|--------|
| 覆盖率豁免 | 新增代码可证明难以测试 | Tech Lead |
| 性能豁免 | 性能测试环境不稳定 | QA Lead |
| 文档豁免 | 文档更新不影响功能 | Docs Lead |
| TPC-H 豁免 | Q17/Q18 证明是存储层限制非查询逻辑错误 | Architect |

豁免记录必须写入 `docs/governance/GATE_EXEMPTIONS.md`。

---

## 九、门禁脚本与规范同步要求

### 9.1 同步原则

```
规范定义 ←→ 脚本实现
    ↑              ↓
    ←── 差距 ───←
```

每当发生以下情况，必须同步更新：

| 触发事件 | 同步要求 |
|----------|----------|
| 规范新增检查项 | 脚本必须在下一版本实现 |
| 脚本新增检查项 | 必须同步到规范 SSOT |
| 规范修改阈值 | 脚本必须同步更新 |
| 脚本发现更好的实现 | 必须回填到规范并说明理由 |

### 9.2 同步检查

```bash
# 检查规范中定义的检查项是否都在脚本中实现
for gate in $(grep "^|| [A-Z][0-9]" gate_spec.md | awk '{print $2}'); do
    if ! grep -q "$gate" scripts/gate/check_*v*.sh; then
        echo "MISSING: $gate not in any gate script"
    fi
done

# 检查脚本中的检查项是否都在规范中定义
for script_check in $(grep "check_output\|TOTAL=\$((TOTAL" scripts/gate/check_*v*.sh | grep -oE '[A-Z][0-9]+'); do
    if ! grep -q "$script_check" gate_spec.md; then
        echo "NOT IN SPEC: $script_check in script but not in spec"
    fi
done
```

---

*本文档由 hermes-z6g4 维护。SSOT: GATE_SPEC_MASTER.md 是 SQLRustGo 门禁唯一权威来源。*