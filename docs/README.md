# SQLRustGo 文档索引

> **最后更新**: 2026-05-05
> **当前版本**: v2.9.0 (Alpha → Beta 过渡中)

---

## 一、文档目录结构

```
docs/
├── releases/                    # 版本发布文档
│   ├── v2.9.0/                  # v2.9.0 (当前开发版本)
│   ├── v2.8.0/                  # v2.8.0 (GA)
│   ├── v2.7.0/                  # v2.7.0 (GA)
│   ├── v2.6.0/                  # v2.6.0
│   ├── v2.0/                    # v2.0 规划文档
│   └── ...
│
├── governance/                  # 治理文档
│   ├── gate_spec.md            # 门禁规范 (A/B/R/G)
│   └── GATE_CI_CD.md          # CI/CD 流水线
│
├── proof/                       # 形式化验证
│   ├── REGISTRY_INDEX.md       # Proof 注册表索引
│   └── TOOLCHAIN_INSTALLATION.md
│
├── benchmark/                   # 性能基准
└── ...
```

---

## 二、版本发布文档

### v2.9.0 (当前版本: Alpha → Beta 过渡)

| 文档 | 说明 |
|------|------|
| [综合说明](releases/v2.9.0/README.md) | v2.9.0 完整文档 |
| [发布说明](releases/v2.9.0/RELEASE_NOTES.md) | 版本发布说明 |
| [变更日志](releases/v2.9.0/CHANGELOG.md) | 详细变更记录 |
| [功能矩阵](releases/v2.9.0/FEATURE_MATRIX.md) | 功能实现状态 |
| [集成状态](releases/v2.9.0/INTEGRATION_STATUS.md) | 功能集成跟踪 |
| [测试策略](releases/v2.9.0/TEST_STRATEGY.md) | 测试目标与阶段安排 |
| [测试状态](releases/v2.9.0/TEST_STATUS_20260503.md) | 测试执行报告 |
| [Proof 覆盖](releases/v2.9.0/PROOF_COVERAGE.md) | 形式化验证覆盖 |
| [覆盖率报告](releases/v2.9.0/COVERAGE_REPORT.md) | 覆盖率分析 |
| [Beta 门禁报告](releases/v2.9.0/BETA_GATE_REPORT_20260504.md) | Beta 门禁检查 |
| [门禁清单](releases/v2.9.0/RELEASE_GATE_CHECKLIST.md) | 发布门禁 |
| [版本计划](releases/v2.9.0/VERSION_PLAN.md) | 版本计划 |
| [开发计划](releases/v2.9.0/DEVELOPMENT_PLAN.md) | 开发计划 |
| [性能目标](releases/v2.9.0/PERFORMANCE_TARGETS.md) | 性能目标 |
| [安全报告](releases/v2.9.0/SECURITY_REPORT.md) | 安全分析 |
| [分布式设计](releases/v2.9.0/DISTRIBUTED_DESIGN.md) | 分布式架构设计 |
| [编排文档](releases/v2.9.0/ORCHESTRATION.md) | 多平台编排 |
| [工具链 CI/CD](releases/v2.9.0/TOOLCHAIN_CICD_GUIDE.md) | CI/CD 指南 |
| [快速开始](releases/v2.9.0/QUICK_START.md) | 快速入门指南 |
| [用户手册](releases/v2.9.0/USER_MANUAL.md) | SQL 语法与功能详解 |
| [客户端连接](releases/v2.9.0/CLIENT_CONNECTION.md) | 连接方式详解 |
| [迁移指南](releases/v2.9.0/MIGRATION_GUIDE.md) | 从 v2.8.0 升级 |

### v2.8.0 (已发布: GA)

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
| [门禁规范](governance/gate_spec.md) | A/B/R/G 四级门禁规格 |
| [CI/CD 流水线](governance/GATE_CI_CD.md) | 门禁 CI/CD 实现 |

---

## 四、形式化验证

| 文档 | 说明 |
|------|------|
| [Proof 注册表索引](proof/REGISTRY_INDEX.md) | 所有 Proof 文件索引 |
| [工具链安装](proof/TOOLCHAIN_INSTALLATION.md) | TLA+ / Formulog 安装 |
| [Phase S 验证流程](proof/PHASE_S_VERIFICATION_WORKFLOW.md) | S 系列验证流程 |

---

## 五、变更历史

| 版本 | 日期 | 说明 |
|------|------|------|
| 3.0 | 2026-05-05 | 更新为 v2.9.0，添加 v2.9.0 完整文档链 |
| 2.1 | 2026-04-22 | 更新为 v2.7.0 GA，添加 v2.7.0 文档入口 |
| 2.0 | 2026-04-17 | 更新为 v2.6.0，清理过时版本链接 |
| 1.1 | 2026-03-05 | 新增 v1.3.0 计划，重组教学材料目录 |
| 1.0 | 2026-03-04 | 初始版本，整合所有文档索引 |

---

*本文档由 yinglichina8848 维护*