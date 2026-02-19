# SQLRustGo v1.0 文档目录

> 版本：v1.0.0
> 日期：2026-02-19
> 状态：Alpha 阶段

---

## 一、目录结构

```
docs/v1.0/
├── README.md                   # 本文档
├── alpha/                      # Alpha 阶段文档
│   └── README.md              # Alpha 目标与质量门禁
├── 草稿/                       # 早期草稿文档
│   ├── 1.0.0版本的对话记录.md
│   ├── 2026-02-13-sqlcc-rust-impl-plan.md
│   ├── 2026-02-13-sqlcc-rust-redesign-design.md
│   ├── 改进计划.md
│   └── 阶段性工作报告.md
├── 草稿计划/                   # 早期计划文档
│   ├── 2026-02-14-sqlrustgo-feature-completion.md
│   ├── 2026-02-16-ai-collaboration-guide.md
│   ├── 2026-02-16-branch-strategy.md
│   ├── 2026-02-16-executable-repo-rules-checklist.md
│   ├── 2026-02-16-parallel-development-guide.md
│   ├── 2026-02-16-pr-workflow.md
│   ├── 2026-02-16-review-approve-merge-policy.md
│   ├── 2026-02-16-source-code-documentation-plan.md
│   ├── 2026-02-16-test-coverage-impl-plan.md
│   ├── 2026-02-16-test-coverage-improvement.md
│   ├── 2026-02-16-version-evolution-plan.md
│   └── 2026-02-18-docs-completion-plan.md
├── 评估改进/                   # 评估报告
│   ├── 01-AI协作开发评估.md
│   ├── 02-TDD开发流程评估.md
│   ├── 03-代码审查评估.md
│   ├── 04-待实现功能分析.md
│   ├── 05-错误处理评估.md
│   ├── 06-性能指标评估.md
│   ├── 07-数据安全评估.md
│   ├── 08-Evaluation演进评估.md
│   └── 综合改进计划.md
├── dev/                        # 开发文档
│   └── DEVELOP.md
├── meeting-notes/              # 会议记录
│   ├── 2026-02-18-gaoxiaoyao-liying.md
│   └── feishu_messages (1).md
├── 对话记录.md                 # 关键对话记录
├── 小龙虾的群聊记录.md
├── 飞书-龙虾群聊记录.md
├── 2026-02-13-sqlcc-rust-impl-plan.md
├── 2026-02-13-sqlcc-rust-redesign-design.md
└── github 多账号配置.rtf
```

---

## 二、文档分类

### 2.1 Alpha 阶段文档

| 文档 | 说明 |
|:-----|:-----|
| [alpha/README.md](alpha/README.md) | Alpha 阶段目标、质量门禁、验收口径 |

### 2.2 草稿文档

早期开发过程中的原始文档，保留了项目演进的完整轨迹：

| 文档 | 说明 |
|:-----|:-----|
| [阶段性工作报告.md](草稿/阶段性工作报告.md) | 开发历程总结 |
| [改进计划.md](草稿/改进计划.md) | 改进计划 |
| [2026-02-13-sqlcc-rust-redesign-design.md](草稿/2026-02-13-sqlcc-rust-redesign-design.md) | 设计文档 |
| [2026-02-13-sqlcc-rust-impl-plan.md](草稿/2026-02-13-sqlcc-rust-impl-plan.md) | 实施计划 |

### 2.3 草稿计划

开发过程中制定的各类计划文档：

| 文档 | 说明 |
|:-----|:-----|
| [2026-02-16-ai-collaboration-guide.md](草稿计划/2026-02-16-ai-collaboration-guide.md) | AI 工具协作开发指南 |
| [2026-02-16-branch-strategy.md](草稿计划/2026-02-16-branch-strategy.md) | 分支管理策略 |
| [2026-02-16-version-evolution-plan.md](草稿计划/2026-02-16-version-evolution-plan.md) | 版本演化规划 |
| [2026-02-16-test-coverage-impl-plan.md](草稿计划/2026-02-16-test-coverage-impl-plan.md) | 测试覆盖率提升计划 |

### 2.4 评估改进

项目各维度的评估报告：

| 文档 | 说明 |
|:-----|:-----|
| [综合改进计划.md](评估改进/综合改进计划.md) | 问题汇总、改进措施 |
| [01-AI协作开发评估.md](评估改进/01-AI协作开发评估.md) | AI 协作评估 |
| [02-TDD开发流程评估.md](评估改进/02-TDD开发流程评估.md) | TDD 流程评估 |
| [03-代码审查评估.md](评估改进/03-代码审查评估.md) | 代码审查评估 |
| [04-待实现功能分析.md](评估改进/04-待实现功能分析.md) | 功能缺失分析 |

### 2.5 对话记录

项目开发过程的真实记录：

| 文档 | 说明 |
|:-----|:-----|
| [对话记录.md](对话记录.md) | 项目创建过程中的关键对话 |
| [小龙虾的群聊记录.md](小龙虾的群聊记录.md) | 多 Agent 协作的实时沟通记录 |
| [飞书-龙虾群聊记录.md](飞书-龙虾群聊记录.md) | 飞书群聊记录 |

---

## 三、阅读建议

### 新用户

```
1. alpha/README.md → 了解当前阶段目标
2. 草稿/阶段性工作报告.md → 了解项目历程
3. 评估改进/综合改进计划.md → 了解待解决问题
```

### 贡献者

```
1. 草稿计划/2026-02-16-ai-collaboration-guide.md → 学习 AI 辅助开发
2. 草稿计划/2026-02-16-branch-strategy.md → 理解分支策略
3. dev/DEVELOP.md → 开发环境和规范
```

---

## 四、版本状态

| 指标 | 当前值 | 目标值 |
|:-----|:-------|:-------|
| 测试覆盖率 | ~76% | 85% |
| 功能完整度 | 基础 SQL | SQL-92 子集 |
| 文档完整度 | 进行中 | 完整 |

---

*本文档由 TRAE (GLM-5.0) 创建*
