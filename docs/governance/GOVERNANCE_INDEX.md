# SQLRustGo Governance 文档体系索引

> **版本**: 1.0
> **创建日期**: 2026-05-14
> **维护人**: hermes-z6g4
> **用途**: AI 和开发者必须遵循的文档导航指南

> **SSOT 声明**: 本文档定义了 SQLRustGo governance 文档体系的权威索引。所有 AI 在执行版本开发、测试计划、门禁检查等任务时，必须先阅读本文档确定要使用的模版和流程文档。

---

## 一、文档体系分类

### 1.1 核心 SSOT 文档（必须遵循）

| 文档 | 用途 | 强制级别 |
|------|------|----------|
| **GATE_SPEC_MASTER.md** | 全版本、全阶段门禁定义唯一权威 | **强制** |
| **DEVELOPMENT_PLAN_TEMPLATE.md** | 版本开发计划模版 | **强制** |
| **TEST_PLAN_TEMPLATE.md** | 测试计划模版 | **强制** |
| **GATE_CHECKLIST_TEMPLATE.md** | 门禁检查清单模版 | **强制** |

### 1.2 治理流程文档

| 文档 | 用途 | 强制级别 |
|------|------|----------|
| RELEASE_LIFECYCLE.md | 四级门禁模型、阶段转换规则 | 强制 |
| gate_lifecycle_tracking.md | Issue 追踪闭环、版本延续机制 | 强制 |
| governance_self_improvement.md | 治理自我进化、反馈机制 | 强制 |
| GATE_EXEMPTIONS.md | 门禁豁免/延期记录 | 强制 |
| RELEASE_POLICY.md | 发布策略、版本号规范 | 强制 |

### 1.3 测试文档

| 文档 | 用途 | 强制级别 |
|------|------|----------|
| GATE-RC-TEST-PLAN.md | RC 阶段测试计划 (~6h) | 强制 |
| P15-rc-ga-test-system.md | 双测试系统架构 | 强制 |
| ALPHA_INTEGRATION_TESTING_PLAN.md | Alpha 阶段测试计划 | 参考 |
| BETA_TEST_PLAN.md | Beta 阶段测试计划 | 参考 |

### 1.4 发布文档

| 文档 | 用途 | 强制级别 |
|------|------|----------|
| RC_TO_GA_GATE_CHECKLIST.md | RC→GA 清单 | 强制 |
| GA_RELEASE_TIMELINE.md | GA 发布时间表 | 参考 |
| RELEASE_GATE_CHECKLIST.md | 版本发布检查 | 参考 |

---

## 二、AI 执行任务时的文档导航

### 2.1 开始新版本开发

```
任务: 开始 v{X}.{Y}.{Z} 开发
↓
阅读本文档 (GOVERNANCE_INDEX.md)
↓
使用 DEVELOPMENT_PLAN_TEMPLATE.md 创建开发计划
↓
创建 docs/releases/v{X}.{Y}.{Z}/DEVELOPMENT_PLAN.md
↓
参考 gate_lifecycle_tracking.md 建立追踪机制
```

### 2.2 创建测试计划

```
任务: 创建 {PHASE} 测试计划
↓
阅读本文档 (GOVERNANCE_INDEX.md)
↓
使用 TEST_PLAN_TEMPLATE.md 创建测试计划
↓
确保测试覆盖 gate_spec 定义的所有门禁检查项
↓
将测试结果映射到门禁检查清单
```

### 2.3 执行门禁检查

```
任务: 执行 {PHASE}-Gate 检查
↓
阅读本文档 (GOVERNANCE_INDEX.md)
↓
使用 GATE_CHECKLIST_TEMPLATE.md 创建检查清单
↓
执行 scripts/gate/check_{phase}_v{VERSION}.sh
↓
记录所有检查结果
↓
失败项 → 创建 Issue → 修复 PR → 验证
↓
豁免项 → 申请审批 → 记录到 GATE_EXEMPTIONS.md
↓
通过 → 发布报告 → 更新 milestone
```

### 2.4 版本延续

```
任务: 将未完成任务延续到下版本
↓
阅读 gate_lifecycle_tracking.md §七
↓
在 DEVELOPMENT_PLAN.md §6 建立映射
↓
确保原 Issue 已关闭或有豁免记录
↓
新版本 DEVELOPMENT_PLAN.md 引用原 Issue
```

---

## 三、文档使用决策树

```
你需要执行 governance 相关任务吗？
    │
    ├── 是 → 继续 ↓
    │
    └── 否 → 执行其他任务

任务类型是什么？
    │
    ├── 版本开发计划
    │   └── 使用 DEVELOPMENT_PLAN_TEMPLATE.md
    │
    ├── 测试计划
    │   └── 使用 TEST_PLAN_TEMPLATE.md
    │
    ├── 门禁检查
    │   ├── 创建检查清单 → 使用 GATE_CHECKLIST_TEMPLATE.md
    │   ├── 执行检查 → 执行 scripts/gate/check_{phase}_v{VERSION}.sh
    │   └── 记录结果 → 使用 GATE_CHECKLIST_TEMPLATE.md §七
    │
    ├── Issue 追踪
    │   └── 阅读 gate_lifecycle_tracking.md
    │
    ├── 豁免申请
    │   └── 阅读 GATE_EXEMPTIONS.md
    │
    ├── 治理改进
    │   └── 阅读 governance_self_improvement.md
    │
    └── 发布
        ├── RC→GA → 阅读 RC_TO_GA_GATE_CHECKLIST.md
        └── GA → 阅读 GA_RELEASE_TIMELINE.md
```

---

## 四、门禁检查完整流程

### 4.1 门禁执行流程图

```
┌─────────────────────────────────────────────────────────────┐
│                      开始门禁检查                            │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│ Step 1: Pre-Gate 自检                                       │
│ - 代码已提交、已推送                                         │
│ - 环境准备就绪 (Rust/cargo/llvm-cov)                       │
│ - 数据准备完成 (TPC-H/SQL Corpus)                          │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│ Step 2: 执行 gate_spec 定义的检查                           │
│                                                              │
│ 代码层: G1-G6 (Build/Test/Clippy/Format/Coverage/Security) │
│ 文档层: G7-G7d (死链/文档存在/版本/用户指南)               │
│ 性能层: G8-G13 (QPS/TPC-H/SQL/Proof)                       │
│ 稳定性: G14 (B-S1~S6)                                      │
│ 协议层: G15-G16 (MySQL/SSI)                                │
│ 合规层: G17-G19 (CI/Issue/Branch)                          │
│ 自我优化: G20-G22 (TODO/Proof/GMP)                          │
│ 发布前: G23-G24 (Release Notes/Tag)                        │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│ Step 3: 结果分类                                            │
│                                                              │
│ PASS → 记录证据，继续下一步                                  │
│ FAIL → 创建 Issue → 修复 PR → 重新测试 → 验证 PASS          │
│ SKIP → 人工判断 → 需要豁免? → 是 → 申请豁免                 │
│ N/A  → 记录原因，继续下一步                                  │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│ Step 4: 生成检查报告                                         │
│                                                              │
│ {                                                           │
│   "gate": "{PHASE}-GATE-v{VERSION}",                       │
│   "commit": "{sha}",                                        │
│   "status": "PASS|FAIL",                                   │
│   "summary": { "total": N, "passed": N, "failed": N },    │
│   "blockers": [...]                                         │
│ }                                                           │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│ Step 5: 门禁通过/失败判定                                    │
│                                                              │
│ 所有 MANDATORY PASS + 所有 FAIL 有 Issue/PR/豁免              │
│                          │                                   │
│            ┌─────────────┴─────────────┐                    │
│            ▼                           ▼                     │
│        PASS                          FAIL                     │
│            │                           │                     │
│            ▼                           ▼                     │
│    发布报告               不能发布，返回 Step 2               │
│    更新 milestone                                          │
└─────────────────────────────────────────────────────────────┘
```

### 4.2 门禁检查时间要求

| 门禁 | 预计时间 | 触发时机 |
|------|----------|----------|
| A-Gate | ~30min | Alpha→Beta |
| B-Gate | ~2h | Beta→RC |
| R-Gate | ~6h | RC→GA |
| G-Gate | ~12h | GA 发布前 |

---

## 五、文档同步要求

### 5.1 规范变更必须同步

每当发生以下情况，必须更新关联文档：

| 触发事件 | 必须同步的文档 |
|----------|----------------|
| gate_spec 新增检查项 | scripts/gate/check_{phase}_v{VERSION}.sh |
| 模版变更 | 所有使用该模版的文档 |
| 新版本开发计划 | 版本目录下的所有文档 |
| 豁免申请 | GATE_EXEMPTIONS.md |
| 门禁失败 | gate_lifecycle_tracking.md |

### 5.2 文档版本对齐检查

```bash
# 检查 gate_spec 与脚本是否同步
for gate in $(grep "^|| [A-Z][0-9]" gate_spec.md | awk '{print $2}'); do
    if ! grep -q "$gate" scripts/gate/check_*v*.sh; then
        echo "MISSING: $gate not in any gate script"
    fi
done
```

---

## 六、快速参考

### 6.1 常用命令

| 命令 | 用途 |
|------|------|
| `bash scripts/gate/check_alpha_v*.sh` | Alpha 门禁 |
| `bash scripts/gate/check_beta_v*.sh` | Beta 门禁 |
| `bash scripts/gate/check_rc_v*.sh` | RC 门禁 |
| `bash scripts/gate/check_ga_v*.sh` | GA 门禁 |
| `bash scripts/gate/check_docs_links.sh` | 文档链接检查 |

### 6.2 关键文件路径

| 用途 | 路径 |
|------|------|
| 开发计划模版 | `docs/governance/DEVELOPMENT_PLAN_TEMPLATE.md` |
| 测试计划模版 | `docs/governance/TEST_PLAN_TEMPLATE.md` |
| 门禁清单模版 | `docs/governance/GATE_CHECKLIST_TEMPLATE.md` |
| 门禁规范 | `docs/governance/GATE_SPEC_MASTER.md` |
| 版本开发计划 | `docs/releases/v{VERSION}/DEVELOPMENT_PLAN.md` |
| 版本测试计划 | `docs/releases/v{VERSION}/{PHASE}_TEST_PLAN.md` |
| 版本门禁清单 | `docs/releases/v{VERSION}/{PHASE}_GATE_CHECKLIST.md` |

### 6.3 豁免申请流程

```
发现门禁不满足 → 评估是否可豁免
        │
        ├── 否 → 修复代码/配置使检查通过
        │
        └── 是 → 填写豁免记录
                   │
                   ▼
              Tech Lead 审批
                   │
                   ▼
              记录到 GATE_EXEMPTIONS.md
                   │
                   ▼
              门禁通过（带豁免记录）
```

---

## 七、变更历史

| 版本 | 日期 | 变更 | 作者 |
|------|------|------|------|
| 1.0 | 2026-05-14 | 初始版本 | hermes-z6g4 |

---

*本文档是 SQLRustGo governance 文档体系的导航索引。所有 AI 在执行 governance 相关任务前必须阅读本文档。*