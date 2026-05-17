# v3.3.0 开发测试计划

> **版本**: v1.0
> **创建日期**: 2026-05-18
> **维护人**: hermes-agent
> **分支**: `develop/v3.3.0` (`c97a709a`)
> **起点**: `539f2957` (v3.2.0 GA release point)
> **Milestone**: v3.3.0 (id=13)

---

## 一、版本目标与约束

### 1.1 核心目标

解决 v3.2.0 GA 遗留问题，消除 5 项豁免记录（EX-v320-001~004），建立 Coverage SSOT 测量规范。

### 1.2 版本约束

| 约束 | 说明 |
|------|------|
| 必须从 GA 释放点出发 | `539f2957`（v3.2.0 GA tag） |
| 所有豁免必须关闭或续期 | EX-v320-xxx → v3.3.0 Alpha 前复审 |
| Coverage 以 L1 CRATES 为 SSOT | 统一测量标准，消除 85.81% vs 68.8% 矛盾 |
| 大内存测试在 Z6G4 执行 | 本机 Mac 禁止 cargo llvm-cov |

---

## 二、分支策略

```
develop/v3.3.0  ← 主开发分支（alpha/v3.3.0 作为起点）
alpha/v3.3.0    ← v3.3.0 Alpha 起点（已包含 v3.2.0 post-GA 补丁）
beta/v3.3.0     ← Beta 阶段
release/v3.3.0  ← 正式发布分支
main             ← GA 释放点（只接受 release/v3.3.0 PR）
```

### 分支同步规则

| 源 | 目标 | 方法 | 频率 |
|----|------|------|------|
| develop/v3.3.0 | alpha/v3.3.0 | PR merge | 按需 |
| alpha/v3.3.0 | beta/v3.3.0 | PR merge | Alpha Gate 通过后 |
| beta/v3.3.0 | release/v3.3.0 | PR merge | Beta Gate 通过后 |
| release/v3.3.0 | main | PR merge | GA Gate 通过后 |

---

## 三、Issue 分配与责任人

### 3.1 P0 阻塞项（Alpha 必须关闭）

#### P0-1: executor 模块拆分 + 覆盖率（#1196, #1197）

| 属性 | 值 |
|------|-----|
| Issue | #1196 (覆盖率不达标), #1197 (模块拆分重构) |
| 当前状态 | executor 覆盖率 70.70%，目标 ≥85% |
| 差距 | -14.3%（约 1,294 行未覆盖） |
| 瓶颈 | `stored_proc.rs` 41.8% 覆盖，1,748 行未覆盖 |
| 修复路径 | 模块拆分 → 增量测试 → 覆盖率重测 |
| 验收标准 | `cargo llvm-cov test L1_CRATES --lib` ≥85% |

**Phase 1 — 模块拆分（Alpha 前完成）**：
```
crates/executor/src/stored_proc/
├── mod.rs           # 主入口（保持 API 兼容）
├── expression.rs    # 表达式求值（可独立测试）
├── cursor.rs        # 游标管理
├── handler.rs       # 异常处理
└── cte.rs          # CTE/递归
```

**Phase 2 — 增量覆盖（Alpha 内完成）**：
- 为每个子模块编写单元测试
- 目标：为 `expression.rs` 和 `cte.rs` 增加 800+ 行覆盖

#### P0-2: MySQL Protocol 握手（#1201）

| 属性 | 值 |
|------|-----|
| Issue | #1201 |
| 当前状态 | `mysql::Conn::new()` 返回 `DriverError { Could not setup connection }` |
| 根因 | sqlrustgo-mysql-server 握手状态机 / auth plugin 兼容性问题 |
| 修复路径 | 抓包分析 → 握手状态机修复 → auth plugin 对齐 |
| 验收标准 | 端到端 MySQL 连接成功（使用 mysql client 验证） |

**调试步骤**：
1. 在 Z6G4 启用 `tcpdump` 抓包：`tcpdump -i any port 3306 -w mysql_handshake.pcap`
2. 用 `wireshark` 分析握手包序列
3. 对照 MySQL 5.7 Protocol 规范（`auth_switch_request`, `caching_sha2_password`）
4. 参考 `docs/releases/v3.1.0/MYSQL_PROTOCOL_OPTIMIZATION.md`

### 3.2 P1 项（Alpha 目标）

#### P1-1: Coverage 测量矛盾（#1202）

| 属性 | 值 |
|------|-----|
| Issue | #1202 |
| 当前状态 | GA_GATE_CHECKLIST 85.81%，RC_TO_GA_REPORT 68.8%，差距 17% |
| 根因 | 测量工具/方法/范围不统一 |
| 修复路径 | 统一工具 + 明确范围 + SSOT 文档 |
| 验收标准 | gate_spec_v330.md 明确定义覆盖率测量规范 |

**统一测量命令**：
```bash
cargo llvm-cov test --lib \
  -p sqlrustgo \
  -p sqlrustgo-executor \
  -p sqlrustgo-optimizer \
  -p sqlrustgo-storage \
  -p sqlrustgo-types
```

#### P1-2: TPC-H SF=1 数据生成（#1198）

| 属性 | 值 |
|------|-----|
| Issue | #1198 |
| 当前状态 | `tpch_data/` 目录不存在 |
| 修复路径 | Z6G4 生成 SF=1 数据 → 执行 check_tpch.sh |
| 验收标准 | 22/22 查询通过，无 OOM |
| 执行环境 | **必须在 Z6G4**（大内存服务器） |

```bash
# 在 Z6G4 执行
ssh openclaw@192.168.0.252
cd /home/openclaw/dev/yinglichina163/sqlrustgo
bash scripts/gate/setup_tpch_env.sh --sf 1
bash scripts/gate/check_tpch.sh --sf1
```

### 3.3 P2 项（Beta/RC 目标）

#### P2-1: 72h 稳定性测试（#1198）

| 属性 | 值 |
|------|-----|
| 关联 Issue | #1198 |
| 执行环境 | Z6G4 |
| 验收标准 | 72h 无崩溃，无数据丢失 |

---

## 四、Alpha 阶段门禁（Alpha Gate）

### 4.1 入口条件

- [x] develop/v3.3.0 基于 `539f2957` 创建
- [x] 所有 P0 Issue 已分配 milestone v3.3.0
- [ ] P0-1: executor 覆盖率 ≥85%
- [ ] P0-2: MySQL 握手成功
- [ ] P1-1: Coverage SSOT 规范建立
- [ ] P1-2: TPC-H SF=1 22/22 通过

### 4.2 Alpha Gate 检查清单

| # | 检查项 | 命令 | 执行环境 | 责任人 |
|---|--------|------|----------|--------|
| A1 | Build | `cargo build --release --workspace` | 本机 | hermes-agent |
| A2 | Test | `cargo test --lib` | 本机 | hermes-agent |
| A3 | Clippy | `cargo clippy --all-features -- -D warnings` | 本机 | hermes-agent |
| A4 | Format | `cargo fmt --all -- --check` | 本机 | hermes-agent |
| A5 | Executor Coverage | `cargo llvm-cov test L1_CRATES --lib` | Z6G4 | hermes-agent |
| A6 | MySQL Protocol | `mysql -h <host> -u root -e "SELECT 1"` | Z6G4 | hermes-agent |
| A7 | TPC-H SF=1 | `bash scripts/gate/check_tpch.sh --sf1` | Z6G4 | hermes-agent |
| A8 | Coverage SSOT | 文档审查 | 本机 | hermes-agent |

### 4.3 Alpha Gate 脚本

```bash
# scripts/gate/check_alpha_v330.sh
#!/bin/bash
set -e

PROJECT_ROOT="$(cd "$(dirname "$0")/.. && pwd)"
cd "$PROJECT_ROOT"

echo "=== v3.3.0 Alpha Gate ==="

# A1: Build
echo "[A1] Build..."
cargo build --release --workspace
echo "  ✅ PASS"

# A2: Test
echo "[A2] Test..."
cargo test --lib
echo "  ✅ PASS"

# A3: Clippy
echo "[A3] Clippy..."
cargo clippy --all-features -- -D warnings
echo "  ✅ PASS"

# A4: Format
echo "[A4] Format..."
cargo fmt --all -- --check
echo "  ✅ PASS"

# A8: SSOT 文档检查
echo "[A8] SSOT Coverage Spec..."
[ -f "docs/governance/gate_spec_v330.md" ] && echo "  ✅ PASS" || echo "  ❌ FAIL"

echo ""
echo "=== Alpha Gate: Local checks 5/5 PASS ==="
echo "=== Remaining: A5 (Coverage), A6 (MySQL), A7 (TPC-H) on Z6G4 ==="
```

---

## 五、测试计划

### 5.1 测试分层

| 层级 | 测试类型 | 执行环境 | 频率 |
|------|----------|----------|------|
| L0 | `cargo test --lib`（快速冒烟） | 本机 | 每次 commit |
| L1 | `cargo test`（完整单元） | 本机 | 每次 PR |
| L2 | Integration + TPC-H | Z6G4 | Beta 前 |
| L3 | Performance QPS | Z6G4 | Beta 前 |
| L4 | Stability 72h | Z6G4 | RC 前 |

### 5.2 覆盖率提升计划

**当前状态（v3.2.0 GA）**：
```
executor:       70.70%  [瓶颈: stored_proc.rs 41.8%]
optimizer:       85.81%  [超过 85% 目标]
storage:         88.30%  [超过 85% 目标]
types:          91.50%  [超过 85% 目标]
```

**目标（v3.3.0 Alpha）**：
```
executor:       ≥85.00%  [+14.3%]
optimizer:       ≥85.00%  [维持]
storage:         ≥85.00%  [维持]
types:           ≥85.00%  [维持]
overall:         ≥85.00%
```

**覆盖率测试命令（Z6G4）**：
```bash
cd /home/openclaw/dev/yinglichina163/sqlrustgo
cargo llvm-cov test --lib \
  -p sqlrustgo \
  -p sqlrustgo-executor \
  -p sqlrustgo-optimizer \
  -p sqlrustgo-storage \
  -p sqlrustgo-types \
  --llvm-cov --codecov
```

### 5.3 TPC-H SF=1 测试计划

**数据生成（Z6G4）**：
```bash
bash scripts/gate/setup_tpch_env.sh --sf 1
# 预计时间: 5-10 分钟
# 生成文件: tpch_data/sf1/
```

**测试执行（Z6G4）**：
```bash
bash scripts/gate/check_tpch.sh --sf1
# 22 个查询，单查询超时 30s
# 预期: 22/22 PASS
```

---

## 六、豁免复审计划

### 6.1 EX-v320-001: executor 覆盖率 70.7%

| 属性 | 值 |
|------|-----|
| 豁免 ID | EX-v320-001 |
| 复审条件 | executor 覆盖率 ≥85% |
| 续期条件 | 覆盖率提升 ≥5%，有明确路线图 |
| 关闭条件 | 覆盖率 ≥85%，无未覆盖的关键路径 |

### 6.2 EX-v320-002: MySQL Protocol 握手

| 属性 | 值 |
|------|-----|
| 豁免 ID | EX-v320-002 |
| 关联 Issue | #1201 |
| 复审条件 | MySQL 连接成功建立 |
| 关闭条件 | `mysql -h <host> -u root -p -e "SELECT 1"` 成功 |

### 6.3 EX-v320-003: TPC-H SF=1 数据缺失

| 属性 | 值 |
|------|-----|
| 豁免 ID | EX-v320-003 |
| 关联 Issue | #1198 |
| 复审条件 | SF=1 数据已生成，22/22 查询通过 |
| 关闭条件 | TPC-H SF=1 22/22 PASS |

### 6.4 EX-v320-004: Sysbench 服务器环境

| 属性 | 值 |
|------|-----|
| 豁免 ID | EX-v320-004 |
| 关联 Issue | #1198 |
| 复审条件 | 72h 稳定性测试完成 |
| 关闭条件 | 72h 无崩溃 |

---

## 七、版本信息

| 属性 | 值 |
|------|-----|
| 起点 | `539f2957` (v3.2.0 GA release point) |
| develop/v3.3.0 | `c97a709a` |
| alpha/v3.3.0 | `c97a709a`（同 develop/v3.3.0） |
| Milestone | v3.3.0 (id=13) |
| 豁免文件 | `docs/governance/GATE_EXEMPTIONS.md` v1.1 |
| 门禁规范 | `docs/governance/gate_spec_v320.md`（参考） |

---

## 九、AI 辅助开发规划

### 9.1 多模型评估机制

v3.3.0 引入**多模型交叉评估**模式，参考 v1.3.0 CRAFT_PLAN 的成功实践。

#### 评估模型分工

| AI 模型 | 角色 | 擅长领域 | 评估重点 |
|---------|------|----------|----------|
| **Claude Code** | 主开发 Agent | Rust/系统编程、架构设计 | 代码质量、内存安全、clippy |
| **DeepSeek** | 文档与分析 Agent | 根因分析、测试设计 | 测试覆盖、边界条件 |
| **MiniMax (本 Agent)** | 首席架构师 | 编排与决策 | 版本规划、风险评估、进度追踪 |

#### 评估触发规则

| 场景 | 触发条件 | 评估模型 |
|------|----------|----------|
| P0 Issue 开始前 | — | Claude + DeepSeek 双重评估 |
| P0 Issue PR 合并前 | — | Claude + DeepSeek 交叉 Review |
| Alpha Gate 前 | — | 三模型综合评审 |
| Beta Gate 前 | — | 三模型综合评审 |
| 豁免复审 | EX-v320-xxx 复审 | DeepSeek 主导分析 |

### 9.2 AI 辅助开发任务

#### 9.2.1 AI Test Generator（参考 QA_ENHANCEMENT_PLAN.md）

基于 LLM 的测试生成器，用于快速构建覆盖率测试：

```
crates/qa/src/ai_test_generator.rs
├── generate_tests(module, context) -> Vec<TestCase>
├── call_llm(prompt) -> String
└── parse_test_cases(response) -> Vec<TestCase>
```

**适用场景**：executor 子模块（expression.rs, cte.rs, cursor.rs）单元测试快速生成

#### 9.2.2 Intelligent Test Selection

基于历史数据预测哪些测试最可能失败，减少 CI 时间：

```
scripts/qa/intelligent_test_selection.py
├── predict_failure_probability(code_changes)
├── select_tests(code_changes, max_tests=100)
└── retrain(new_data)
```

#### 9.2.3 AI-Powered Bug Triage

自动分类和路由 Bug：

```
crates/qa/src/ai_bug_triage.rs
├── BugTriageAI::triage(report) -> TriageResult
│   ├── predict_severity
│   ├── predict_module
│   ├── find_similar_bugs
│   └── suggest_fix
```

### 9.3 Self-Healing CI/CD

#### 9.3.1 Flaky Test Detection

```rust
// crates/qa/src/flaky_detector.rs
pub struct FlakyTestDetector {
    history_db: HistoryDatabase,
    significance_threshold: f64,  // 0.05
    min_runs: u32,                 // 10
}

pub enum FlakyAction {
    Retry(RetryConfig),
    MarkKnownFlaky(KnownFlaky),
    RequiresInvestigation,
    NeedsFix,
}
```

#### 9.3.2 CI 自动修复流程

```
Flaky Test Detection → Auto-Retry (3x) → Known Flaky DB Update
                                              ↓
Build Failure → Anomaly Detection → Auto-Isolate Failed Jobs
                              ↓
Performance Regression → Alert + Rollback
```

### 9.4 工具链配置

```bash
# .env 示例
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...
DEEPSEEK_API_KEY=sk-...

# 模型路由配置 (models.toml)
[models.router]
default = "claude-sonnet-4"

[models.routing]
rust_code_generation = "claude-sonnet-4"
test_generation = "deepseek-chat"
bug_analysis = "deepseek-chat"
document_generation = "openai-gpt-4o"
```

### 9.5 AI 辅助开发 Issue

| # | 标题 | 验收标准 |
|---|------|----------|
| #1225 | AI Test Generator — executor 子模块测试生成 | expression.rs/cte.rs/cursor.rs 覆盖+800 行 |
| #1226 | Intelligent Test Selection — PR 智能测试选择 | 预测准确率 >70% |
| #1227 | Bug Triage AI — Issue 自动分类与路由 | precision >80% |
| #1228 | Flaky Test Detector — 稳定性测试自动标记 | pass_rate tracking |

---

## 十、多模型协作策略

### 10.1 Agent 分工（v3.3.0）

| Agent | 工作目录 | 负责任务 | 主力模型 |
|-------|---------|----------|----------|
| **hermes-z6g4** | `/home/openclaw/dev/yinglichina163/sqlrustgo` | P0 开发和测试执行 | Claude Code |
| **hermes-local** | `/Users/liying/workspace/dev/yinglichina163/sqlrustgo` | 文档、Issue、PR 管理 | MiniMax |
| **deepseek** (外部) | — | 根因分析、测试设计审查 | DeepSeek |

### 10.2 协作流程

```
Issue 创建 (hermes-local)
    ↓
根因分析 (deepseek)
    ↓
双重评估 (Claude Code + deepseek)
    ↓
PR 创建 (hermes-z6g4)
    ↓
交叉 Review (Claude Code review + deepseek analysis)
    ↓
合并 (hermes-local)
    ↓
智能测试选择 (intelligent_test_selection.py)
    ↓
CI 执行 (Nomad/Z6G4)
```

### 10.3 v3.3.0 AI 成熟度目标

| 维度 | v3.2.0 基线 | v3.3.0 目标 |
|------|-------------|-------------|
| AI 测试生成 | 无 | executor 子模块覆盖+800 行 |
| 智能测试选择 | 无 | PR 覆盖减少 30% |
| Bug 自动分类 | 无 | precision >80% |
| Flaky Test Detection | 无 | 自动标记已知 flaky |
| CI 自愈 | 无 | Auto-retry 3x |

---

## 八、变更历史

||| 版本 | 日期 | 说明 |
|||------|------|------|
||| 1.0 | 2026-05-18 | 初始版本，基于 v3.2.0 GA 遗留问题分析 |
||| 1.1 | 2026-05-18 | 新增第九节 AI 辅助开发规划、第十节多模型协作策略，整合 v1.3.0 CRAFT_PLAN 和 v3.3.0 QA_ENHANCEMENT_PLAN |
