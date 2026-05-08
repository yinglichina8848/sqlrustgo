---
marp: true
theme: gaia
paginate: true
backgroundColor: #fff
color: #333
---

<!-- _class: lead -->

# 第十讲：PR工作流与项目成熟度评估

## Harness工程：AI时代的质量保障

---

# 课程大纲

1. **PR工作流与Harness机制**（25分钟）
2. **项目成熟度评估**（25分钟）
3. **BetaPhase2治理架构**（20分钟）
4. **实践练习**（10分钟）

---

# Part 1: PR工作流与Harness机制

---

## 1.1 Why：LLM时代的质量困境

### AI辅助开发的上下文限制

```
┌─────────────────────────────────────────────────────┐
│           LLM 上下文窗口（Context Window）           │
├─────────────────────────────────────────────────────┤
│  ▲ 输入（历史对话 + 代码）   ▼ 输出（生成代码）      │
│         [有限]                    [有限]            │
│                                                     │
│  问题：AI无法"看到"全系统依赖关系                    │
│  问题：AI无法保证修改后其他模块不受影响               │
│  问题：AI的"局部最优"可能是"全局最差"               │
└─────────────────────────────────────────────────────┘
```

### AI CLI工具的能力边界

| 工具类型 | 能力 | 局限 |
|----------|------|------|
| AI Code Assistant | 生成代码 | 不理解系统级 invariant |
| LLM Chat | 分析代码 | 上下文窗口有限 |
| AI CLI | 执行命令 | 无法判断业务语义 |

### 核心矛盾

> **AI擅长"做"，但不擅长"保证做了之后系统不出问题"**

---

## 1.2 How：Harness工程的核心思想

### 外部流程补偿内部智能不足

```
传统模式：  AI生成代码 → 人工Review（人眼有限）
           ↓
Harness模式：AI生成代码 → CI/CD自动检查 → 门禁Gate → 发布
                        ↑
                  软件工程流程弥补AI不足
```

### ISO 9001 QA 的核心原则

| 原则 | Harness实现 |
|------|-------------|
| 过程控制 | CI/CD流水线定义每个阶段检查项 |
| 持续改进 | 每次PR后自动记录指标、发现回归 |
| 文档化 | Gate spec文档定义每阶段验收标准 |
| 可追溯 | Issue → PR → 门禁结果全链路追踪 |

### CMM-4/5 级别的核心特征

```
CMM-4：已管理（Managed）
  → 量化控制：每个Gate有明确的数值指标（覆盖率≥75%、QPS≥1000）

CMM-5：优化中（Optimizing）
  → 持续改进：回归检测 → 分析根因 → 流程优化
```

---

## 1.3 What：PR工作流在Harness中的角色

### 传统PR vs Harness PR

```
传统PR流程：
  创建PR → 人工Review → 讨论 → 合并

Harness PR流程：
  创建PR → [自动门禁] → Issue关联检查 → 人工Review → 合并
              ↓
        ┌──────────────────────────────────┐
        │  Gate检查（无人值守，自动执行）    │
        │  • 编译检查                       │
        │  • 测试覆盖率检查                  │
        │  • Clippy零警告                   │
        │  • TPC-H基准测试                  │
        │  • GitNexus影响范围分析           │
        └──────────────────────────────────┘
```

### PR与Gate的联动机制

```markdown
## PR描述模板（Gate驱动）

## 变更类型
- [ ] 新功能 / [x] Bug修复 / [ ] 重构

## Gate检查项自检
- [ ] cargo build --all-features  ✅
- [ ] cargo test --all-features  ✅ (通过率 ≥90%)
- [ ] cargo clippy --all-features -- -D warnings  ✅
- [ ] cargo llvm-cov --all-features --json  (覆盖率 ≥75%)

## 相关Issue
Closes #465  #466  #467  ...（Gate闭环追踪）

## GitNexus影响分析
运行 gitnexus_impact() → 风险等级：[LOW/HIGH/CRITICAL]
```

---

## 1.4 实践：PR工作流与成熟度关联

### SQLRustGo的成熟度演进

```
v1.0 (原型)  →  手动测试，人工质量判断
     ↓
v2.0 (Alpha) →  引入CI，编译+测试自动化
     ↓
v2.9 (Beta)  →  Gate系统，量化指标管控
     ↓
v3.0 (BetaPhase2) → 完整Harness，ISO 9001 + CMM-4/5
```

### 成熟度评估维度（v3.0 BetaPhase2）

| 维度 | BetaPhase2目标 | 衡量方式 |
|------|---------------|----------|
| Crash率 | 下降25% | 自动化压力测试 |
| 稳定性 | 提升20% | 长期运行指标 |
| SQL兼容性 | 提升20% | TPC-H通过率 |
| 审计完整性 | 提升15% | Audit Trail覆盖率 |
| 死锁率 | 下降15% | 并发测试 |
| 锁竞争 | 下降5% | 性能回归检测 |

---

# Part 2: 项目成熟度评估

---

## 2.1 BetaPhase2的成熟度定义

### 为什么需要量化成熟度

> **没有量化，就没有管理。—— Peter Drucker**

AI生成代码的不可预测性要求更严格的量化标准：

```
AI生成的代码 → 可能是正确的 → 但不确定是否在所有边界条件下正确
                    ↓
           需要量化指标来"证明"而非"相信"
```

### BetaPhase2成熟度等级

```
┌─────────────────────────────────────────────────────────┐
│                    Beta v3.0.0-beta                      │
├─────────────────────────────────────────────────────────┤
│  特征：                                                   │
│  • 功能完整（JOIN、子查询、视图、TPC-H SF=1）              │
│  • 有Gate保护（BP1/BP2/BP3三阶段门禁）                     │
│  • 有量化指标（5类KPI追踪）                                │
│  • 有审计追溯（Issue→PR→Gate结果全链路）                   │
│  • 有自我优化（check_regression.sh回归检测）               │
├─────────────────────────────────────────────────────────┤
│  ISO 9001: 过程文件化 ✅ / 质量记录 ✅ / 持续改进 ✅         │
│  CMM-4:    量化控制 ✅                                    │
│  CMM-5:    持续优化 ✅                                    │
└─────────────────────────────────────────────────────────┘
```

---

## 2.2 成熟度评估流程

### Gate驱动的评估机制

```
Phase 1: Gate Spec定义（开发前）
  └→ docs/governance/gate_spec_v300.md
     ├→ BP1 静态检查（编译/格式/Clippy）
     ├→ BP2 集成检查（覆盖率/回归/TPC-H）
     └→ BP3 Beta检查（KPI指标/功能闭环）

Phase 2: Gate执行（每个PR）
  └→ scripts/gate/check_beta_v300.sh
     ├→ BP3-1: Full test suite ≥90% pass
     ├→ BP3-2: Coverage ≥75%
     ├→ BP3-3: TPC-H SF=1 all pass
     ├→ BP3-4: No critical GitNexus blast radius
     └→ BP3-5~BP3-7: Advisory checks

Phase 3: 指标收集（持续）
  └→ Nomad runner执行 → 结果上报 → SSOT存储
```

### 成熟度评估表格

```
┌────────────────────────────────────────────────────────┐
│        SQLRustGo v3.0.0 BetaPhase2 成熟度评估           │
├────────────────────────────────────────────────────────┤
│  质量维度        目标      实际      状态               │
│  ─────────────────────────────────────────             │
│  代码质量        ≥90%     92%       ✅ 达标            │
│  覆盖率          ≥75%     78%       ✅ 达标            │
│  SQL兼容性       TPC-H    22/22     ✅ 达标            │
│  性能基线        ≥1000    1200      ✅ 达标            │
│  审计完整性      100%     100%      ✅ 达标            │
│  死锁率          0次/月    0        ✅ 达标            │
├────────────────────────────────────────────────────────┤
│  Gate合规率     100%     100%      ✅                  │
│  Issue闭环率     100%     100%      ✅                  │
│  综合评分        —        A        ✅ 达到CMM-4       │
└────────────────────────────────────────────────────────┘
```

---

## 2.3 成熟度与Gate的闭环

### Issue → PR → Gate → 成熟度的全链路

```
Issue创建
    ↓
PR关联Issue（Closes #465 等）
    ↓
Gate自动检查（CI/CD触发）
    ↓ pass → PR合并 / fail → PR被block
    ↓
成熟度指标更新
    ↓
月度/季度评估报告
    ↓
流程优化（如果指标退化）
```

### 为什么需要这个闭环

| 问题 | 传统方式 | Harness方式 |
|------|----------|-------------|
| 人工Review疏漏 | 人眼有限，遗漏边界case | GitNexus impact分析 + 量化指标 |
| 回归未发现 | 上线后才发现 | check_regression.sh自动检测 |
| 指标退化不知 | 版本多了才发现 | Nomad runner持续监控 |
| Issue漂移 | Issue与PR脱节 | Closes #xxx强制关联 |

---

# Part 3: BetaPhase2治理架构

---

## 3.1 三层治理架构

### ISO 9001 + CMM-5 的Harness实现

```
┌─────────────────────────────────────────────────────────────┐
│                    治理层（Governance）                       │
│  BetaPhase2 目标设定 → KPI追踪 → 流程优化                     │
├─────────────────────────────────────────────────────────────┤
│                    Gate层（Gate System）                      │
│  BP1静态 → BP2集成 → BP3 Beta → E-09 Floor                  │
├─────────────────────────────────────────────────────────────┤
│                    执行层（Execution）                        │
│  Nomad Runner → Gitea Actions → GitNexus → GBrain SSOT       │
└─────────────────────────────────────────────────────────────┘
```

### 各层职责

| 层级 | 职责 | 关键产物 |
|------|------|----------|
| 治理层 | 目标设定、KPI监控、流程改进 | gate_spec_v300.md, TEST_PLAN_v3.md |
| Gate层 | BP1/BP2/BP3检查、E-09豁免 | check_beta_v300.sh, check_regression.sh |
| 执行层 | CI运行、代码分析、知识管理 | Nomad, Gitea Actions, GitNexus, GBrain |

---

## 3.2 BetaPhase2的31个Issues追踪

### Issue闭环管理

```
创建Issue (#465-495)
    ↓
分配到PR
    ↓
Gate检查
    ↓ pass → Issue关联PR merged
    ↓ fail → Issue open + 修复PR
    ↓
验证通过 → Issue closed
```

### Issue与Gate的联动

| Issue类型 | 关联Gate | 闭环要求 |
|-----------|----------|----------|
| 功能缺失 | BP2覆盖率 | 必须有测试证明 |
| 性能回归 | BP3性能基线 | 必须有benchmark数据 |
| SQL兼容性 | BP3 TPC-H | 必须22 queries全通过 |
| 死锁问题 | BP3并发测试 | 必须无死锁记录 |
| 审计缺陷 | BP3审计完整性 | 必须Audit Trail可查 |

---

## 3.3 E-09 Floor：风险容忍机制

### 为什么需要E-09 Floor

> **Gate失败 ≠ 项目停止。风险可接受时，应该有条件推进。**

### E-09机制

```
正常Gate流程：
  BP3检查失败 → PR blocked → 必须修复

E-09 Floor流程：
  BP3检查失败 → 评估是否属于已知/可接受风险
                → 是 → 记录E-09豁免 → 继续推进
                → 否 → 仍需修复
```

### E-09 vs 绕过Gate的区别

| 行为 | E-09 Floor | 绕过Gate |
|------|------------|----------|
| 是否记录 | ✅ 有日志 | ❌ 无记录 |
| 是否分析 | ✅ 有根因分析 | ❌ 无分析 |
| 是否可追溯 | ✅ SSOT可查 | ❌ 不可追溯 |
| 是否合规 | ✅ ISO 9001合规 | ❌ 违规 |

---

# Part 4: 实践练习

---

## 4.1 PR创建与Gate自检

### 任务

1. 创建功能分支 `git checkout -b feature/xxx`
2. 实现功能，编写测试
3. 提交PR前，手动运行Gate检查

### 自检命令

```bash
# BP1静态检查
cargo build --all-features && cargo fmt --check && cargo clippy --all-features -- -D warnings

# BP2集成检查
cargo test --all-features  # ≥90% pass
cargo llvm-cov --all-features --json --output-file cov.json  # ≥75%

# BP3功能检查
bash scripts/gate/check_tpch.sh  # TPC-H SF=1
bash scripts/gate/check_regression.sh  # 无性能回归
```

---

## 4.2 Issue关联与闭环追踪

### 任务

1. 创建一个Issue描述问题
2. 创建PR修复该Issue
3. 在PR描述中使用 `Closes #xxx` 关联
4. 验证Gate通过后Issue自动关闭

### Issue闭环验证

```bash
# 检查Issue是否有关联的PR
gh issue view <id> --json closedByPullRequestsReferences

# 结果非空 → 正常闭环
# 结果为空 → 禁止手动关闭（除非任务取消）
```

---

# 核心知识点总结

---

## 1. Harness工程思想

- **Why**：LLM上下文有限、AI CLI能力有限，需要外部流程补偿
- **How**：CI/CD + Gate + 量化指标 + 闭环追踪
- **What**：ISO 9001 QA + CMM-4/5 级别的质量保障

## 2. PR工作流

- **传统PR**：人工Review为主
- **Harness PR**：Gate自动检查 + Issue闭环 + 量化指标

## 3. 成熟度评估

- **BetaPhase2**：5类KPI指标，31个Issues追踪
- **Gate合规**：BP1/BP2/BP3三阶段门禁
- **E-09 Floor**：风险容忍机制，保证合规可追溯

---

# 课后作业

---

## 任务

1. 分析自己项目的成熟度现状（对照BetaPhase2表格）
2. 设计一个Gate检查项，补充到现有流程
3. 使用Issue→PR→Closes方式关联一次修改

## 预习

- CI/CD与Nomad自动化
- Nomad Runner与Gitea Actions联动

---

<!-- _class: lead -->

# 谢谢！

## 下节课：CI/CD与Nomad自动化

