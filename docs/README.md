# SQLRustGo 文档索引

> **最后更新**: 2026-03-21
> **当前版本**: v1.6.1

---

## 一、文档目录结构

```
docs/
├── architecture/          # 系统架构、模块设计
├── benchmark/            # 性能测试报告和分析
├── governance/           # 分支治理、CI/CD、Release 策略
├── monitoring/           # 监控、日志、告警配置
├── plans/                # 开发计划、功能设计文档
├── releases/             # 版本发布记录和路线图
├── tutorials/            # 教学资料和实验指导
│   ├── 教学计划/         # 教学进度计划和大纲
│   └── 教学实践/         # 教学实践记录和学生手册
├── ai_collaboration/     # AI 协作开发文档和 Prompts
├── v1.0/                 # v1.0 版本文档
├── v2.0/                 # v2.0 版本规划文档
├── whitepaper/           # 技术白皮书
└── README.md             # 本文档
```

---

## 二、版本发布文档

| 版本 | 说明 |
|------|------|
| [v1.6.1](./releases/v1.6.1/) | 当前开发版本 |
| [v1.6.0](./releases/v1.6.0/) | 2026-03 |
| [v1.5.0](./releases/v1.5.0/) | 2026-02 |
| [v1.4.0](./releases/v1.4.0/) | 2026-02 |
| [v1.3.0](./releases/v1.3.0/) | 2026-01 |
| [v1.2.0](./releases/v1.2.0/) | 2025-12 |
| [v1.1.0](./releases/v1.1.0/) | 2025-11 |
| [v1.0.0](./releases/v1.0.0/) | 初始版本 |

### 版本路线图

- [版本路线图](./releases/VERSION_ROADMAP.md)
- [长期规划](./releases/LONG_TERM_ROADMAP.md)

---

## 三、架构文档

| 文档 | 说明 |
|------|------|
| [架构设计](./architecture/ARCHITECTURE.md) | 系统架构设计 |
| [架构总览](./architecture/ARCHITECTURE_OVERVIEW.md) | 架构概览 |
| [架构演进](./architecture/ARCHITECTURE_EVOLUTION.md) | 架构演进历史 |
| [目录结构](./architecture/DIRECTORY_STRUCTURE.md) | 项目目录设计 |
| [Cascades 优化器](./architecture/cascades_optimizer_design.md) | 优化器设计 |

---

## 四、开发计划

| 文档 | 说明 |
|------|------|
| [v1.7.0 开发计划](./plans/2026-03-19-v170-development-plan.md) | 当前版本开发计划 |
| [v1.6.1 Alpha 测试改进](./plans/2026-03-19-v160-alpha-test-improvement-plan.md) | 测试改进计划 |
| [Query Cache 设计](./plans/2026-03-19-p01-query-cache-design.md) | 查询缓存设计 |
| [Benchmark CI 设计](./plans/2026-03-20-b11-benchmark-ci-design.md) | Benchmark CI 设计 |
| [PostgreSQL Benchmark](./plans/2026-03-20-postgres-benchmark-implementation.md) | PostgreSQL 对比实现 |

---

## 五、治理文档

| 文档 | 说明 |
|------|------|
| [分支治理](./governance/BRANCH_GOVERNANCE.md) | 分支管理规范 |
| [贡献指南](./governance/CONTRIBUTING.md) | 贡献者指南 |
| [Release 策略](./governance/RELEASE_POLICY.md) | 发布策略 |
| [权限矩阵](./governance/PERMISSION_MATRIX.md) | GitHub 权限配置 |
| [GA 发布门禁清单](./governance/RC_TO_GA_GATE_CHECKLIST.md) | RC → GA 检查 |

---

## 六、监控与性能

### 监控配置
| 文档 | 说明 |
|------|------|
| [可观测性指南](./monitoring/OBSERVABILITY_GUIDE.md) | 监控配置指南 |
| [Prometheus 告警](./monitoring/PROMETHEUS_ALERTS.md) | 告警规则 |
| [Grafana 面板](./monitoring/grafana-dashboard.json) | 可视化面板 |

### Benchmark
| 文档 | 说明 |
|------|------|
| [Benchmark 报告](./benchmark/benchmark_report.md) | 性能测试结果 |
| [Benchmark CI 设计](./plans/2026-03-20-b11-benchmark-ci-design.md) | CI 集成设计 |

---

## 七、教学材料

### 教学计划 (教师材料)
| 文档 | 说明 |
|------|------|
| [教材大纲](./tutorials/教学计划/教材大纲.md) | 课程大纲 |
| [教学设计](./tutorials/教学计划/教学设计.md) | 教学实施方案 |
| [上机实验指导书](./tutorials/教学计划/上机实验指导书.md) | 实验指导 |
| [实验报告模版](./tutorials/教学计划/实验报告模版.md) | 报告模板 |

### 教学实践 (学生材料)
| 文档 | 说明 |
|------|------|
| [学生操作手册](./tutorials/教学实践/学生操作手册/) | 学生实践指南 |
| [助教检查手册](./tutorials/教学实践/助教检查手册/) | 助教检查指南 |

---

## 八、AI 协作开发

| 文档 | 说明 |
|------|------|
| [AI Agent 提示词](./ai_collaboration/AI增强软件工程/AI_AGENT_PROMPTS.md) | 4 AI Agent 完整提示词 |
| [多 Agent 配置](./ai_collaboration/AI增强软件工程/MULTI_AGENT_CONFIG.md) | 环境配置指南 |
| [多身份开发模式](./ai_collaboration/AI增强软件工程/MULTI_IDENTITY_DEVELOPMENT_MODEL.md) | 四账号权限体系 |
| [Copilot 评估报告](./ai_collaboration/AI增强软件工程/Copilot-Github评估报告.md) | GitHub Copilot 评估 |

---

## 九、2.0 规划文档

| 文档 | 说明 |
|------|------|
| [技术路线图](./v2.0/TECH_ROADMAP.md) | 1.x → 3.0 完整演进路线 |
| [架构设计 v2](./v2.0/ARCHITECTURE_V2.md) | 2.0 架构设计 |
| [Cascades 优化器](./v2.0/CASCADES_OPTIMIZER.md) | 工业级查询优化器设计 |
| [基准测试框架](./v2.0/BENCHMARK_FRAMEWORK.md) | 性能测试框架 |
| [2.0 路线图](./v2.0/SQLRUSTGO_2_0_ROADMAP.md) | 2.0 开发路线图 |
| [白皮书](./v2.0/WHITEPAPER.md) | 项目白皮书 |

---

## 十、白皮书

| 文档 | 说明 |
|------|------|
| [v1.2 Interface Freeze](./whitepaper/sqlrustgo_1.2_interface_freeze.md) | 接口冻结文档 |
| [v1.2 Release Whitepaper](./whitepaper/sqlrustgo_1.2_release_whitepaper.md) | 发布白皮书 |
| [v2.0 分布式框架](./whitepaper/sqlrustgo_2.0_distributed_framework.md) | 分布式架构白皮书 |

---

## 十一、变更历史

| 版本 | 日期 | 说明 |
|------|------|------|
| 2.0 | 2026-03-21 | 重组文档目录结构，新增 benchmark/、tutorials/、ai_collaboration/ |
| 1.1 | 2026-03-05 | 新增 v1.3.0 计划，重组教学材料目录 |
| 1.0 | 2026-03-04 | 初始版本，整合所有文档索引 |

---

*本文档由 SQLRustGo Team 维护*
