# SQLRustGo 文档索引

> **最后更新**: 2026-05-15
> **当前版本**: v3.2.0 (Beta Phase)
> **状态**: Alpha Gate ✅ Conditional Passed

---

## 一、文档目录结构

```
docs/
├── releases/                    # 版本发布文档
│   ├── v3.2.0/                  # v3.2.0 (当前开发版本: Beta)
│   ├── v3.1.0/                  # v3.1.0 (GA)
│   ├── v3.0.0/                  # v3.0.0 (GA)
│   ├── v2.9.0/                  # v2.9.0
│   ├── v2.8.0/                  # v2.8.0 (GA)
│   └── ...

├── governance/                  # 治理文档
│   ├── GOVERNANCE_INDEX.md     # 治理文档索引
│   ├── GATE_SPEC_MASTER.md     # 门禁规范 (A/B/R/G)
│   └── GATE_CI_CD.md          # CI/CD 流水线

├── proof/                       # 形式化验证
│   ├── REGISTRY_INDEX.md       # Proof 注册表索引
│   └── TOOLCHAIN_INSTALLATION.md

├── gmp/                        # GMP 合规文档
├── gmp-compliance/             # GMP 合规评估
└── ...
```

---

## 二、版本发布文档

### v3.2.0 (当前版本: Beta Phase)

| 文档 | 说明 |
|------|------|
| [综合说明](releases/v3.2.0/README.md) | v3.2.0 完整文档 |
| [发布说明](releases/v3.2.0/RELEASE_NOTES.md) | 版本发布说明 |
| [变更日志](releases/v3.2.0/CHANGELOG.md) | 详细变更记录 |
| [功能矩阵](releases/v3.2.0/FEATURE_MATRIX.md) | 功能实现状态 |
| [开发计划](releases/v3.2.0/DEVELOPMENT_PLAN.md) | 开发计划 |
| [Alpha 门禁报告](releases/v3.2.0/ALPHA_GATE_REPORT.md) | Alpha Gate 检查结果 |
| [Beta 门禁报告](releases/v3.2.0/BETA_GATE_REPORT.md) | Beta Gate 检查结果 |
| [测试计划](releases/v3.2.0/TEST_PLAN.md) | 测试目标与阶段安排 |
| [安装指南](releases/v3.2.0/INSTALL.md) | 安装配置 |
| [部署指南](releases/v3.2.0/DEPLOYMENT_GUIDE.md) | 生产部署 |
| [快速开始](releases/v3.2.0/QUICK_START.md) | 快速入门 |

### v3.1.0 (已发布: GA 2026-05-11)

| 文档 | 说明 |
|------|------|
| [综合说明](releases/v3.1.0/README.md) | v3.1.0 完整文档 |
| [发布说明](releases/v3.1.0/RELEASE_NOTES.md) | 版本发布说明 |
| [变更日志](releases/v3.1.0/CHANGELOG.md) | 详细变更记录 |
| [功能矩阵](releases/v3.1.0/FEATURE_MATRIX.md) | 功能实现状态 |
| [开发计划](releases/v3.1.0/DEVELOPMENT_PLAN.md) | 开发计划 |
| [开发分析](releases/v3.1.0/DEVELOPMENT_ANALYSIS.md) | 开发阶段分析 |
| [开发指导](releases/v3.1.0/DEVELOPMENT_GUIDANCE.md) | 开发阶段指导 |
| [状态报告](releases/v3.1.0/STATUS_REPORT.md) | 版本状态报告 |
| [系统瓶颈分析](releases/v3.1.0/SYSTEM_BOTTLENECK_ANALYSIS.md) | 瓶颈分析与优化路线 |
| [性能目标](releases/v3.1.0/PERFORMANCE_TARGETS.md) | 性能目标 |
| [测试计划](releases/v3.1.0/TEST_PLAN.md) | 测试目标与阶段安排 |
| [门禁脚本](../scripts/gate/) | Alpha/Beta/RC/GA 门禁脚本 |

### v2.9.0 (已发布: Beta → GA 2026-05-08)

| 文档 | 说明 |
|------|------|
| [文档索引](releases/v2.8.0/README.md) | v2.8.0 文档总入口 |
| [版本计划](releases/v2.8.0/VERSION_PLAN.md) | v2.8.0 版本计划 |

### v2.7.0 (已发布: GA)

| 文档 | 说明 |
|------|------|
| [文档索引](releases/v2.7.0/README.md) | v2.7.0 文档总入口 |

---

## 三、治理文档

| 文档 | 说明 |
|------|------|
| [治理文档索引](governance/GOVERNANCE_INDEX.md) | 治理文档导航 |
| [门禁规范](governance/GATE_SPEC_MASTER.md) | A/B/R/GA 四级门禁规格 |
| [CI/CD 流水线](governance/GATE_CI_CD.md) | 门禁 CI/CD 实现 |
| [ Governance Standard](governance/GOVERNANCE_STANDARD.md) | 治理标准 |

---

## 四、形式化验证

| 文档 | 说明 |
|------|------|
| [Proof 注册表索引](proof/REGISTRY_INDEX.md) | 所有 Proof 文件索引 |
| [工具链安装](proof/TOOLCHAIN_INSTALLATION.md) | TLA+ / Formulog 安装 |

---

## 五、变更历史

| 版本 | 日期 | 说明 |
|------|------|------|
| 3.1 | 2026-05-15 | 更新为 v3.2.0 Beta，添加 v3.2.0 完整文档链 |
| 3.0 | 2026-05-05 | 更新为 v3.1.0 Alpha，添加 v3.1.0 文档 |
| 2.1 | 2026-04-22 | 更新为 v2.9.0，添加 v2.9.0 文档入口 |
| 2.0 | 2026-04-17 | 更新为 v2.8.0，清理过时版本链接 |
| 1.0 | 2026-03-04 | 初始版本，整合所有文档索引 |

---

*本文档由 hermes-z6g4 维护*