# SQLRustGo 文档索引

> **最后更新**: 2026-03-12
> **当前版本**: v1.2.0-RC (RC 阶段)

---

## 一、文档目录结构

```
docs/
├── releases/                    # 版本发布文档
│   ├── v1.0.0/                 # v1.0.0 发布文档
│   ├── v1.1.0/                 # v1.1.0 发布文档
│   ├── v1.2.0/                 # v1.2.0 发布文档 (当前)
│   └── v1.3.0/                 # v1.3.0 版本计划
│
├── v1.0/                        # v1.0 开发过程文档
│   ├── alpha/                   # Alpha 阶段文档
│   ├── beta/                    # Beta 阶段文档
│   ├── rc1/                     # RC1 验收文档
│   ├── 草稿/                    # 开发草稿
│   ├── 草稿计划/                # 规划文档
│   └── 评估改进/                # 评估报告
│
├── v2.0/                        # 2.0 规划文档
│   └── 网络设计/
│
├── AI增强软件工程/              # AI 协作规范
│
├── governance/                  # 治理文档
│
├── plans/                       # 计划文档
│
├── 教学计划/                    # 教师准备材料 (PPT、大纲)
│
└── 教学实践/                    # 学生实践材料 (手册、模板)
```

> 文档中文化进度说明见：[文档中文化推进说明](文档中文化推进说明.md)

---

## 二、版本发布文档

### v1.2.0 (当前版本: Alpha)

| 文档 | 说明 |
|------|------|
| [文档索引](releases/v1.2.0/README.md) | v1.2.0 文档总入口 |
| [版本计划](releases/v1.2.0/VERSION_PLAN.md) | v1.2.0 版本计划 |
| [门禁检查清单](releases/v1.2.0/RELEASE_GATE_CHECKLIST.md) | 发布门禁 |
| [任务矩阵](releases/v1.2.0/TASK_MATRIX.md) | v1.2.0 任务跟踪 |
| [分支治理规范](releases/v1.2.0/BRANCH_STAGE_GOVERNANCE.md) |Draft/Alpha/Beta/RC/GA 规则|
| [测试计划](releases/v1.2.0/TEST_PLAN.md) | 测试目标与阶段安排 |
| [发布说明](releases/v1.2.0/RELEASE_NOTES.md) | 发布文档草稿 |

### v1.1.0 (已发布)

| 文档 | 说明 |
|------|------|
| [Release Notes](releases/v1.1.0/RELEASE_NOTES.md) | 版本发布说明 |
| [门禁检查清单](releases/v1.1.0/RELEASE_GATE_CHECKLIST.md) | 发布门禁 |
| [API 文档](releases/v1.1.0/API_DOCUMENTATION.md) | API 参考 |

### v1.2.0 (计划中)

| 文档 | 说明 |
|------|------|
| [版本计划](releases/v1.2.0/VERSION_PLAN.md) | v1.2.0 版本计划 |
| [门禁检查清单](releases/v1.2.0/RELEASE_GATE_CHECKLIST.md) | 发布门禁 |

> 说明：`v1.2.0-draft` 属于历史阶段标识，文档保留用于追溯；当前执行口径为 `alpha/v1.2.0` + `develop/v1.2.0`。

### v1.3.0 (计划中)

| 文档 | 说明 |
|------|------|
| [版本计划](releases/v1.3.0/VERSION_PLAN.md) | v1.3.0 版本计划 |
| [门禁检查清单](releases/v1.3.0/RELEASE_GATE_CHECKLIST.md) | 发布门禁 |

### v1.0.0

| 文档 | 说明 |
|------|------|
| [Release Summary](releases/v1.0.0/00-release-summary.md) | 发布总结 |
| [Release Notes](releases/v1.0.0/01-release-notes.md) | 发布说明 |
| [Approval Record](releases/v1.0.0/10-approval-record.md) | 审批记录 |

---

## 三、2.0 规划文档

| 文档 | 说明 |
|------|------|
| [技术路线图](v2.0/TECH_ROADMAP.md) | 1.x → 3.0 完整演进路线 |
| [架构设计](v2.0/ARCHITECTURE_V2.md) | 2.0 架构设计 |
| [Cascades 优化器](v2.0/CASCADES_OPTIMIZER.md) | 工业级查询优化器设计 |
| [基准测试框架](v2.0/BENCHMARK_FRAMEWORK.md) | 性能测试框架 |
| [路线图](v2.0/SQLRUSTGO_2_0_ROADMAP.md) | 2.0 开发路线图 |
| [分布式接口设计](v2.0/DISTRIBUTED_INTERFACE_DESIGN.md) | 3.0 分布式接口 |
| [风险矩阵](v2.0/RISK_MATRIX.md) | 风险评估 |
| [白皮书](v2.0/WHITEPAPER.md) | 项目白皮书 |
| [白皮书 v3](v2.0/WHITEPAPER_V3.md) | 3.0 分布式接口与治理白皮书 |
| [权限模型](v2.0/GIT_PERMISSION_MODEL.md) |Git & GitHub 权限体系|
| [企业级权限](v2.0/GIT_PERMISSION_MODEL_V3.md) | 3.0 企业级权限推演 |
| [展示材料](v2.0/PRESENTATION_MATERIALS.md) | 多受众展示版本 |
| [分支策略](v2.0/BRANCH_STRATEGY.md) | 分支管理策略 |
| [网络增强计划](v2.0/网络设计/NETWORK_ENHANCEMENT_PLAN.md) | 网络层增强计划 |

---

## 四、AI 增强软件工程

| 文档 | 说明 |
|------|------|
| [AI Agent 提示词体系](AI增强软件工程/AI_AGENT_PROMPTS.md) | 4 AI Agent 完整提示词 |
| [多 Agent 配置说明](AI增强软件工程/MULTI_AGENT_CONFIG.md) | 环境配置指南 |
| [多身份隔离开发模式](AI增强软件工程/MULTI_IDENTITY_DEVELOPMENT_MODEL.md) | 四账号权限体系 |
| [GitHub 多账号配置指南](AI增强软件工程/GitHub多账号配置指南.md) | 多账号配置 |
| [Copilot 评估报告](AI增强软件工程/Copilot-Github评估报告.md) |GitHub Copilot 评估|

---

## 五、治理文档

| 文档 | 说明 |
|------|------|
| [分支清理报告](governance/BRANCH_CLEANUP_REPORT_20260222.md) | 分支清理记录 |
| [GA 发布时间线](governance/GA_RELEASE_TIMELINE.md) | GA 发布流程 |
| [不可变发布架构](governance/IMMUTABLE_RELEASE_ARCHITECTURE.md) | 发布架构设计 |
| [权限检查报告](governance/PERMISSION_CHECK_REPORT_20260222.md) | 权限检查记录 |
| [RC 到 GA 门禁清单](governance/RC_TO_GA_GATE_CHECKLIST.md) | RC → GA 检查清单 |
| [企业 GitHub 权限模型](governance/enterprise-github-minimal-permission-model.md) | 企业权限模型 |
| [企业治理最佳实践](governance/enterprise-governance-best-practices.md) | 治理最佳实践 |

---

## 六、开发文档

### v1.0 开发过程文档

| 文档 | 说明 |
|------|------|
| [v1.0 文档索引](v1.0/README.md) | v1.0.x 开发过程文档索引 |
| [综合改进计划](v1.0/评估改进/综合改进计划.md) | v1.0.0 综合改进计划 |
| [性能指标评估](v1.0/评估改进/06-性能指标评估.md) | 性能评估报告 |

### 架构文档

| 文档 | 说明 |
|------|------|
| [架构设计](architecture.md) | 系统架构设计（含 Mermaid 架构图） |
| [架构演进](ARCHITECTURE_EVOLUTION.md) | 架构演进历史 |
| [设计文档](2026-02-13-sqlcc-rust-redesign-design.md) | 重构设计文档 |
| [实施计划](2026-02-13-sqlcc-rust-impl-plan.md) | 实施计划 |

---

## 七、教学材料

### 教学计划 (教师材料)

| 文档 | 说明 |
|------|------|
| [教学进度计划](教学计划/AI增强软件工程-教学进度计划.md) | 教学进度安排 |
| [上机实验指导书](教学计划/上机实验指导书.md) | 实验指导 |
| [实验报告模版](教学计划/实验报告模版.md) | 报告模板 |

### PPT 讲义

| 讲次 | 主题 | 文档 |
|------|------|------|
| 第1讲 | 软件工程概述与项目导论 | [PPT](教学计划/PPT/第1讲-软件工程概述与项目导论.md) |
| 第2讲 | 结构化设计与UML基础 | [PPT](教学计划/PPT/第2讲-结构化设计与UML基础.md) |
| 第3讲 | 面向对象设计与类图 | [PPT](教学计划/PPT/第3讲-面向对象设计与类图.md) |
| 第4讲 | 顺序图状态图与架构设计 | [PPT](教学计划/PPT/第4讲-顺序图状态图与架构设计.md) |
| 第5讲 | 架构设计原理与SQLRustGo架构 | [PPT](教学计划/PPT/第5讲-架构设计原理与SQLRustGo架构.md) |
| 第6讲 | 功能模块划分与接口设计 | [PPT](教学计划/PPT/第6讲-功能模块划分与接口设计.md) |
| 第7讲 | AI辅助核心模块实现 | [PPT](教学计划/PPT/第7讲-AI辅助核心模块实现.md) |
| 第8讲 | 测试驱动开发与Alpha版本 | [PPT](教学计划/PPT/第8讲-测试驱动开发与Alpha版本.md) |
| 第9讲 | 软件治理与分支策略 | [PPT](教学计划/PPT/第9讲-软件治理与分支策略.md) |
| 第10讲 | PR工作流与项目成熟度评估 | [PPT](教学计划/PPT/第10讲-PR工作流与项目成熟度评估.md) |

### 教学实践 (学生材料)

| 文档 | 说明 |
|------|------|
| [教学实践索引](教学实践/README.md) | 学生实践材料索引 |
| [学生执行手册](教学实践/v1.1.0-beta/handbook-student.md) | 学生可复现步骤 |
| [助教执行手册](教学实践/v1.1.0-beta/handbook-ta.md) | PR 证据链示例 |
| [任务看板](教学实践/v1.1.0-beta/task-board.md) | Beta 阶段任务追踪 |
| [日报模板](教学实践/templates/daily-template.md) | 课堂用日报模板 |

---

## 八、变更历史

| 版本 | 日期 | 说明 |
|------|------|------|
| 1.1 | 2026-03-05 | 新增 v1.3.0 计划，重组教学材料目录 |
| 1.0 | 2026-03-04 | 初始版本，整合所有文档索引 |

---

*本文档由 yinglichina8848 维护*
