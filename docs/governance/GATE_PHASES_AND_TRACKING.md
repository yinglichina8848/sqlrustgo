# SQLRustGo 分阶段门禁与跨版本追踪规范

> **版本**: 1.0
> **创建日期**: 2026-05-14
> **维护人**: hermes-z6g4
> **用途**: 定义 A/B/R/G 四级门禁的详细流程、追踪机制和跨版本延续规范
> **SSOT**: 门禁规范以 `GATE_SPEC_MASTER.md` 为唯一权威来源

> **关联文档**:
> - `GATE_SPEC_MASTER.md` — 门禁规范 SSOT
> - `GATE_CHECKLIST_TEMPLATE.md` — 门禁检查清单模版
> - `gate_lifecycle_tracking.md` — Issue 追踪闭环规范
> - `GOVERNANCE_STANDARD.md` — 治理标准总纲

---

## 一、门禁体系概述

### 1.1 四级门禁模型

```
A-Gate ──▶ B-Gate ──▶ R-Gate ──▶ G-Gate
 (α入口)    (β入口)    (RC入口)    (GA入口)
```

| 门禁 | 名称 | 入口条件 | 通过标准 | 预计时间 |
|------|------|----------|----------|----------|
| **A-Gate** | Alpha Gate | 开发完成，代码已提交 | 编译通过，测试≥80%，覆盖率≥50% | ~30min |
| **B-Gate** | Beta Gate | A-Gate PASS，无 P0/P1 Bug | 编译通过，测试≥90%，覆盖率≥75%，TPC-H SF=0.1 22/22 | ~2h |
| **R-Gate** | RC Gate | B-Gate PASS，TPC-H SF=1 可运行 | 编译通过，测试100%，覆盖率≥85%，QPS 退化≤5% | ~6h |
| **G-Gate** | GA Gate | R-Gate PASS，性能达标 | 所有 R-Gate 项 + Point Select≥10K QPS | ~12h |

### 1.2 门禁通过条件

```
门禁通过 = (所有 MANDATORY 项 PASS) + (所有 FAIL 项有 Issue/PR) + (所有豁免项已审批)
```

| 条件类型 | 说明 | 处理方式 |
|----------|------|----------|
| **全部 PASS** | 所有检查项通过 | 直接进入下一阶段 |
| **有 FAIL** | 存在未通过项 | 必须创建 Issue + 修复 PR |
| **有 SKIP** | 条件不满足 | 需人工判断是否豁免 |
| **有豁免** | 客观原因无法满足 | 审批后记录到 GATE_EXEMPTIONS.md |

---

## 二、门禁执行流程

### 2.1 流程图

```
┌─────────────────────────────────────────────────────────────────────┐
│                         开始门禁检查                                 │
└─────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│ Step 1: 识别门禁阶段                                                 │
│ - 检查当前分支 (develop/v{VERSION})                                  │
│ - 检查 milestone (v{VERSION}-{phase})                               │
│ - 确定执行的检查脚本                                                 │
│   └── scripts/gate/check_{phase}_v{VERSION}.sh                     │
└─────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│ Step 2: Pre-Gate 自检                                               │
│                                                                         │
│ 2.1 代码准备                                                         │
│     [ ] git log --oneline -1  (已提交)                               │
│     [ ] git push --dry-run  (已推送)                                │
│     [ ] git branch      (分支正确)                                  │
│     [ ] git status      (WORKSPACE 干净)                            │
│                                                                         │
│ 2.2 环境准备                                                         │
│     [ ] rustc --version  (要求: {version})                          │
│     [ ] cargo --version  (要求: {version})                          │
│     [ ] cargo llvm-cov --version  (要求: {version})                 │
│     [ ] cargo audit --version  (要求: {version})                    │
│                                                                         │
│ 2.3 数据准备                                                         │
│     [ ] TPC-H 数据 sf=0.1 已生成                                    │
│     [ ] TPC-H 数据 sf=1 已生成 (B-Gate 起)                          │
│     [ ] SQL Corpus 已准备                                           │
└─────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│ Step 3: 执行门禁检查                                                 │
│                                                                         │
│ 执行: bash scripts/gate/check_{phase}_v{VERSION}.sh                 │
│                                                                         │
│ 代码层: G1-G6 (Build/Test/Clippy/Format/Coverage/Security)          │
│ 文档层: G7-G7d (死链/文档存在/版本/用户指南)                         │
│ 性能层: G8-G13 (QPS/TPC-H/SQL/Proof)                               │
│ 稳定性: G14 (B-S1~S6)                                               │
│ 协议层: G15-G16 (MySQL/SSI)                                         │
│ 合规层: G17-G19 (CI/Issue/Branch)                                   │
│ 发布前: G20-G24 (Release Notes/Tag)                                 │
└─────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│ Step 4: 结果分类与处理                                               │
│                                                                         │
│ PASS ──────────────────────────────────────────▶ 记录证据，继续 Step 5 │
│     │                                                                  │
│ FAIL ──────────────────────────────────────────▶ 创建 Issue #N       │
│     │                                                │               │
│     │                                                ▼               │
│     │                                          修复 PR 创建           │
│     │                                                │               │
│     │                                                ▼               │
│     │                                          验证 PASS              │
│     │                                                │               │
│     │                                                ▼               │
│     └──────────────────────────────────────── 重新测试 (回到 Step 3)   │
│                                                                         │
│ SKIP ──────────────────────────────────────────▶ 人工判断            │
│     │                                                │               │
│     ├── 需要豁免? ──Yes──▶ 申请豁免 → 记录到 GATE_EXEMPTIONS.md      │
│     │                                                │               │
│     └── No ──────────────────────────────────────▶ 修复使检查可执行  │
└─────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│ Step 5: 生成检查报告                                                 │
│                                                                         │
│ 5.1 按 GATE_CHECKLIST_TEMPLATE.md 格式生成报告                        │
│                                                                         │
│ 5.2 门禁结果判定                                                     │
│     所有 MANDATORY PASS + 所有 FAIL 有 Issue/PR + 所有豁免已审批       │
│         │                                                               │
│         ├── PASS ──▶ 发布报告 → 更新 milestone → 通知团队            │
│         │                                                               │
│         └── FAIL ──▶ 不能发布 → 返回 Step 3 重新检查                  │
│                                                                         │
│ 5.3 证据格式示例                                                     │
│     {                                                                  │
│       "gate": "{PHASE}-GATE-v{VERSION}",                              │
│       "commit": "{sha}",                                              │
│       "status": "PASS|FAIL",                                          │
│       "summary": { "total": N, "passed": N, "failed": N },           │
│       "blockers": [...]                                               │
│     }                                                                  │
└─────────────────────────────────────────────────────────────────────┘
```

### 2.2 各阶段门禁检查项

#### A-Gate (Alpha Gate) 检查清单

| # | 检查项 | 命令/方法 | 通过标准 | 证据格式 |
|---|--------|----------|---------|---------|
| A1 | 编译检查 | `cargo build --workspace` | 无错误 | `{command, exit_code}` |
| A2 | 单元测试 | `cargo test --workspace` | ≥80% 通过 | `{passed, failed, exit_code}` |
| A3 | Clippy 检查 | `cargo clippy --all-features -- -D warnings` | 零警告 | `{warnings, exit_code}` |
| A4 | 格式化检查 | `cargo fmt --all -- --check` | 无格式错误 | `{diff_count, exit_code}` |
| A5 | 文档链接检查 | `bash scripts/gate/check_docs_links.sh` | 无死链 | `{broken_links}` |
| A6 | 覆盖率检查 | `cargo llvm-cov --all-features --lcov` | ≥50% | `{total_pct}` |
| A7 | 安全扫描 | `cargo audit` | 无高危漏洞 | `{vulnerabilities}` |

#### B-Gate (Beta Gate) 检查清单

| # | 检查项 | 命令/方法 | 通过标准 | 证据格式 |
|---|--------|----------|---------|---------|
| B1 | 编译检查 | `cargo build --release --workspace` | 无错误 | `{command, exit_code}` |
| B2 | 全量测试 | `cargo test --all-features` | ≥90% 通过 | `{passed, failed, exit_code}` |
| B3 | Clippy 检查 | `cargo clippy --all-features -- -D warnings` | 零警告 | `{warnings, exit_code}` |
| B4 | 格式化检查 | `cargo fmt --all -- --check` | 无格式错误 | `{diff_count, exit_code}` |
| B5 | 覆盖率检查 | `cargo llvm-cov --all-features --lcov` | ≥75% | `{total_pct}` |
| B6 | 安全扫描 | `cargo audit` | 无高危漏洞 | `{vulnerabilities}` |
| B7 | 文档链接检查 | `bash scripts/gate/check_docs_links.sh` | 无死链 | `{broken_links}` |
| B8 | TPC-H SF=0.1 | `scripts/gate/check_tpch.sh sf=0.1` | 22/22 通过，无 OOM | `{passed, total, oom_count}` |
| B9 | SQL Corpus | `cargo test -p sqlrustgo-sql-corpus` | ≥85% | `{passed, total, pct}` |
| B-S1 | concurrency_stress_test | `cargo test --test concurrency_stress_test` | 全部通过 | `{passed, total}` |
| B-S2 | crash_recovery_test | `cargo test --test crash_recovery_test` | 全部通过 | `{passed, total}` |
| B-S3 | long_run_stability_test | `cargo test --test long_run_stability_test` | 全部通过 | `{passed, total}` |
| B-S4 | wal_integration_test | `cargo test --test wal_integration_test` | 全部通过 | `{passed, total}` |
| B-S5 | network_tcp_smoke_test | `cargo test --test network_tcp_smoke_test` | 全部通过 | `{passed, total}` |
| B-S6 | ssi_stress_test | `cargo test -p sqlrustgo-transaction --test ssi_stress_test` | 全部通过 | `{passed, total}` |

#### R-Gate (RC Gate) 检查清单

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
| R10 | Performance Baseline | `cargo bench && scripts/gate/check_perf_baseline.sh` | QPS 退化≤5% | `{baseline_path, delta_pct, pass}` |
| R11 | Sysbench Gate | `scripts/gate/check_sysbench.sh` | Point/UPDATE/INSERT 对比 baseline | `{point_qps, update_qps, insert_qps, delta}` |
| R12 | MySQL Protocol | mysql:5.7 容器握手测试 | 连接成功 | `{handshake, query_response}` |

#### G-Gate (GA Gate) 检查清单

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

---

## 三、Issue 追踪闭环

### 3.1 闭环流程图

```
┌─────────────────────────────────────────────────────────────────────┐
│                        门禁检查失败 (FAIL)                            │
└─────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│ Step 1: Issue 创建                                                  │
│                                                                         │
│ Issue 标题: [{GATE_ITEM}] {简短描述}                                  │
│                                                                         │
│ Issue 内容必须包含:                                                   │
│ - 门禁来源: check_{phase}_v{VERSION}.sh                              │
│ - 检查项: {GATE_ITEM} (如 B9, R10, B-S1 等)                          │
│ - 检查命令: {实际执行的命令}                                           │
│ - 失败输出: {命令输出摘要}                                             │
│ - 根因分析: {分析结果}                                                 │
│ - 影响范围: {对阶段转换的阻塞影响}                                      │
│ - 验收条件: [ ] {条件1} [ ] {条件2}                                   │
│ - milestone: v{VERSION}-{phase}                                      │
│ - labels: source/gate-{phase}, type/{type}                           │
└─────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│ Step 2: PR 修复                                                     │
│                                                                         │
│ 2.1 创建修复分支                                                      │
│     git checkout -b fix/gate-{issue_number}-{short_desc}            │
│                                                                         │
│ 2.2 修复代码                                                          │
│     - 编写修复代码                                                     │
│     - 添加或修复相关测试                                               │
│     - 确保所有检查通过                                                 │
│                                                                         │
│ 2.3 创建 PR                                                          │
│     - PR 标题: fix: #{issue_number} {简短描述}                        │
│     - PR body 关联 Issue: Closes #{issue_number}                     │
│     - 确保 CI 全部通过                                                 │
└─────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│ Step 3: 验证                                                         │
│                                                                         │
│ 3.1 本地验证                                                          │
│     cargo test --all-features                                         │
│     bash scripts/gate/check_{phase}_v{VERSION}.sh                     │
│                                                                         │
│ 3.2 CI 验证                                                          │
│     - 确保所有 CI 检查通过                                             │
│     - 确保覆盖率未下降                                                 │
│                                                                         │
│ 3.3 Issue 关联验证                                                    │
│     gh issue view {id} --json closedByPullRequestsReferences          │
│     # 结果非空 → 继续                                                  │
│     # 结果为空 → 禁止关闭                                              │
└─────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│ Step 4: Issue 关闭                                                   │
│                                                                         │
│ 关闭前验证 (强制):                                                    │
│ [ ] gh issue view {id} --json closedByPullRequestsReferences 非空     │
│ [ ] gh pr view {pr_number} --json state,mergedAt state=MERGED         │
│ [ ] 相关测试在 CI 通过                                                │
│ [ ] 门禁重新检查 PASS                                                │
│                                                                         │
│ 关闭操作:                                                             │
│ gh issue close {id} --comment "Fixed in PR #{pr_number}"              │
└─────────────────────────────────────────────────────────────────────┘
```

### 3.2 Issue 创建标准

| 门禁项 | Issue 标题模板 | 必须包含内容 |
|--------|---------------|-------------|
| A/B/R/G-N 测试通过率 | `[{N}] 全量测试通过率 {X%}，低于 {Y%} 要求` | 失败测试清单、最后通过率记录 |
| A/B/R/G-N 覆盖率 | `[{N}] {模块} 覆盖率 {X%}，低于 {Y%} 要求` | 差距分析、预期增加覆盖的模块 |
| B8/B9/R9/G10 TPC-H | `[{N}] TPC-H SF={SF} {M}/22 通过` | 失败查询号、超时/OOM/结果错误分类 |
| B-S1~B-S6/R-S1~R-S6 稳定性 | `[{N}] {测试名} {M}/{K} 通过` | 失败用例名称、复现步骤 |
| SQL Corpus | `[{N}] SQL Corpus {X%}，低于 {Y%} 要求` | 不支持语法分类清单 |
| R7/R8/R11/G7/G8/G9 性能 | `[{N}] {指标} {X}，低于 {Y} 要求` | 性能数据、退化比例 |

### 3.3 Issue 关闭验证命令

```bash
# Step 1: 检查是否有 PR 关闭该 Issue
gh issue view {id} --json closedByPullRequestsReferences
# 结果非空 → 可以关闭
# 结果为空 → 禁止手动关闭

# Step 2: 验证 PR 已合并
gh pr view <pr_number> --json state,mergedAt
# state = MERGED 且 mergedAt 有值 → 继续
# 否则 → 禁止关闭

# Step 3: 验证测试通过
cargo test -p sqlrustgo-{package} --test {test_name}
# 必须相关测试全部通过

# Step 4: 确认覆盖率未下降
cargo llvm-cov --all-features | grep total
# 必须覆盖率 ≥ 目标值
```

### 3.4 禁止的模式

```text
❌ 门禁 FAIL → 跳过 → 合并代码 → Issue 未创建 → 问题丢失
❌ Issue 已创建 → 未关联 PR → 无人追踪
❌ 检查通过 → 未记录证据 → 后续无法复现
❌ 豁免未申请 → 直接忽略 → 违反流程
❌ 没有 PR 证据就关闭 Issue
❌ PR 未合并就关闭 Issue
❌ 测试未通过就关闭 Issue
```

### 3.5 正确的模式

```text
✅ 门禁检查 → 记录结果 → FAIL → 创建 Issue → 修复 PR → 验证 PASS → 关闭 Issue
✅ 门禁检查 → 记录结果 → FAIL → 评估豁免 → 申请审批 → 记录到 GATE_EXEMPTIONS.md
✅ 门禁通过 → 记录证据 → 发布报告 → 更新 milestone → 通知团队
```

---

## 四、跨版本追踪机制

### 4.1 版本延续判定

满足以下任一条件，必须将任务延续到下个版本：

| 条件 | 说明 |
|------|------|
| 修复需要 3 人周以上 | 超出当前版本开发周期 |
| 涉及架构变更 | 必须在下一个大版本迭代 |
| 优先级冲突 | 当前版本有更高优先级的 P0 任务 |
| 需要等待其他依赖完成 | 如 CBO 需要先完成索引选择 |

### 4.2 版本延续流程

```
v3.0.0 B-Gate FAIL
         │
         ├── Issue #451 创建 (milestone: v3.0.0-beta)
         │
         ├── 判定: 修复需要 3 人周以上 → 触发版本延续
         │
         ▼
v3.1.0 DEVELOPMENT_PLAN.md §6 建立映射
┌──────────────────────────────────────┐
│ v3.1.0 延续任务（来自 v3.0.0）      │
├──────────────────────────────────────┤
│ #451 → SQL Operations 语法支持        │
│ 原状态: 20% (11/55)                 │
│ 目标: ≥80% (44/55)                 │
│ 优先级: P0                           │
│ 验收: test_sql_corpus_operations     │
│       通过率 ≥80%                    │
└──────────────────────────────────────┘
         │
         ▼
v3.1.0 开发过程中
Issue #451 修复 → PR #XXX → 验证 PASS
         │
         ▼
Issue #451 关闭（需 PR 证据）
```

### 4.3 版本延续标准格式

```markdown
## v{NEXT_VERSION} 延续任务（来自 v{CURRENT_VERSION} 未完成项）

| 原 Issue | 任务描述 | 原版本状态 | v{NEXT_VERSION} 目标 | 验收条件 | 优先级 |
|----------|----------|------------|---------------------|----------|--------|
| #451 | SQL Operations 语法支持 | 20% (11/55) | ≥80% (44/55) | test_sql_corpus_operations ≥80% | P0 |
| #379 | 事务状态机压力测试 | 未开始 | B-S2 PASS | crash_recovery_test 全部通过 | P0 |
```

### 4.4 版本 milestone 映射

```
v3.0.0 milestone              v3.1.0 milestone
(v3.0.0-alpha/beta/rc)       (v3.1.0-alpha/beta/rc)
     │                              │
     │ #451 延续                    │
     └──────────────────────────────┼──▶ Issue #451 (open, milestone=v3.1.0-beta)
                                   │
                                   ▼
                              验证 test_sql_corpus_operations ≥ 80%
                                   │
                                   ▼
                              关闭 Issue #451
```

---

## 五、门禁脚本规范

### 5.1 脚本命名规范

```text
scripts/gate/
├── check_alpha_v{VERSION}.sh     # Alpha 门禁脚本
├── check_beta_v{VERSION}.sh      # Beta 门禁脚本
├── check_rc_v{VERSION}.sh        # RC 门禁脚本
├── check_ga_v{VERSION}.sh        # GA 门禁脚本
├── check_docs_links.sh          # 文档链接检查（通用）
└── check_tpch.sh                # TPC-H 检查（通用）
```

### 5.2 脚本输出格式规范

每个门禁脚本必须输出以下格式：

```bash
# check_beta_v310.sh 输出规范
[beta-v3.1.0] B1: PASS
[beta-v3.1.0] B2: FAIL (87% < 90%, 19 tests failed in 3 suites)
  → 失败 suites: sqlrustgo-executor, sqlrustgo-optimizer, sqlrustgo-storage
  → 建议: 创建 Issue 追踪，优先处理 executor 失败项
  → 影响: B-Gate 无法通过，需修复后重新检查
[beta-v3.1.0] B3: PASS
...
[beta-v3.1.0] B8: PASS (TPC-H SF=0.1: 22/22, OOM: 0)
[beta-v3.1.0] B9: FAIL (SQL Corpus 85.0% < 90%, 8 tests failed)
  → 失败类别: {具体类别}
  → 建议: 创建 Issue 追踪
  → 影响: SQL 兼容性不足

=== Beta Gate Results: PASS=12 / 14, BLOCKERS=2 ===
=== 未通过项 ===
  - B2: 全量测试通过率 87% < 90%
  - B9: SQL Corpus 85.0% < 90%
=== 建议行动 ===
  1. 为每个 BLOCKER 创建 Gitea Issue（milestone: v3.1.0-beta）
  2. 修复后重新运行 check_beta_v310.sh
  3. 如当前版本无法修复，将任务延续到 v3.1.1 或 v3.2.0 DEVELOPMENT_PLAN.md
```

### 5.3 必需的错误分类

对于失败的检查项，脚本必须输出：

```bash
echo "[gate-v{VERSION}] {ID}: FAIL"
echo "  → 失败原因: {具体原因}"
echo "  → 建议操作: {修复建议或 Issue 创建指引}"
echo "  → 影响范围: {对其他门禁项的连带影响}"
```

---

## 六、追踪检查清单

### 6.1 门禁检查前

```markdown
## 门禁检查前检查清单

### 环境和代码
- [ ] 确认在正确分支运行（develop/v{VERSION}）
- [ ] 确认代码已提交（git log --oneline -1）
- [ ] 确认代码已推送（git push --dry-run）
- [ ] 确认 milestone 已创建（v{VERSION}-{phase}）
- [ ] 确认有写入权限创建 Issue

### 环境和数据
- [ ] Rust/cargo 版本正确
- [ ] TPC-H 数据已生成（sf=0.1 或 sf=1）
- [ ] SQL Corpus 已准备
- [ ] mysql:5.7 容器可用（如需协议测试）
```

### 6.2 门禁检查后（FAIL 时）

```markdown
## 门禁检查后（FAIL）检查清单

### Issue 创建
- [ ] 提取所有 FAIL 项的 ID、命令、输出
- [ ] 为每个 FAIL 项创建 Issue（milestone 绑定到当前版本）
- [ ] Issue 包含：来源门禁、检查命令、失败输出、验收条件
- [ ] Issue 标签包含：source/gate-{phase}

### 修复追踪
- [ ] 评估修复复杂度
- [ ] 如修复需要 3 人周以上 → 触发版本延续
- [ ] 在 v{NEXT} DEVELOPMENT_PLAN.md §6 建立映射

### 豁免评估
- [ ] 评估 FAIL 项是否可以豁免
- [ ] 如可豁免 → 申请 Tech Lead 审批
- [ ] 豁免批准后记录到 GATE_EXEMPTIONS.md
```

### 6.3 Issue 关闭前

```markdown
## Issue 关闭前检查清单

### PR 验证
- [ ] gh issue view {id} --json closedByPullRequestsReferences 非空
- [ ] gh pr view {pr_number} --json state,mergedAt state=MERGED

### 测试验证
- [ ] 相关测试在本地通过
- [ ] 相关测试在 CI 通过
- [ ] 门禁重新检查 PASS

### 文档更新
- [ ] DEVELOPMENT_PLAN.md 状态已更新为 DONE
- [ ] gate_lifecycle_tracking.md §7 已更新
```

---

## 七、门禁时间要求

| 门禁 | 预计时间 | 触发时机 |
|------|----------|----------|
| A-Gate | ~30min | Alpha→Beta |
| B-Gate | ~2h | Beta→RC |
| R-Gate | ~6h | RC→GA |
| G-Gate | ~12h | GA 发布前 |

---

## 八、版本回顾机制

### 8.1 回顾触发条件

| 时机 | 触发条件 | 执行人 |
|------|----------|--------|
| 版本发布后 | Tag vX.Y.Z 创建后 48h 内 | Agent |
| 重大豁免后 | EX-XXX 豁免被批准时 | Agent |

### 8.2 回顾内容

每次版本发布后，在 `gate_lifecycle_tracking.md` §10 添加：

```markdown
## v{X}.{Y}.{Z} 周期回顾 — {日期}

### 门禁执行统计
| 门禁 | 检查次数 | PASS | FAIL | 阻塞次数 |
|------|----------|------|------|----------|
| Alpha | N | M | K | J |
| Beta | N | M | K | J |
| RC | N | M | K | J |
| GA | N | M | K | J |

### Issue 追踪统计
| 指标 | 数值 |
|------|------|
| 新建 Issue | N |
| 已关闭（含 PR 证据）| M |
| 延续到下版本 | K |
| 豁免登记 | J |

### 规范 vs 执行差距
| 差距类型 | 数量 | 示例 |
|----------|------|------|
| 规范缺失 | N | {描述} |
| 规范过时 | N | {描述} |
| 执行缺失 | N | {描述} |
| 阈值偏差 | N | {描述} |

### 治理改进建议
1. {建议 1}
2. {建议 2}

### 遗留问题清单
| Issue | 类型 | 状态 | 延续到 |
|-------|------|------|--------|
| #N | GA-GAP | OPEN | v{X+1}.Y |
```

---

## 九、相关文档

| 文档 | 作用 | 路径 |
|------|------|------|
| GATE_SPEC_MASTER.md | 门禁规范 SSOT | `docs/governance/GATE_SPEC_MASTER.md` |
| GATE_CHECKLIST_TEMPLATE.md | 门禁检查清单模版 | `docs/governance/GATE_CHECKLIST_TEMPLATE.md` |
| gate_lifecycle_tracking.md | Issue 追踪闭环规范 | `docs/governance/gate_lifecycle_tracking.md` |
| GOVERNANCE_STANDARD.md | 治理标准总纲 | `docs/governance/GOVERNANCE_STANDARD.md` |
| GATE_EXEMPTIONS.md | 门禁豁免记录 | `docs/governance/GATE_EXEMPTIONS.md` |

---

## 十、变更历史

| 版本 | 日期 | 变更 | 作者 |
|------|------|------|------|
| 1.0 | 2026-05-14 | 初始版本，建立分阶段门禁与跨版本追踪规范 | hermes-z6g4 |

---

*本文档由 hermes-z6g4 维护。门禁规范权威来源: GATE_SPEC_MASTER.md*

*最后更新: 2026-05-14*
