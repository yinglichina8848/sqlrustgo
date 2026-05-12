# SQLRustGo 文档索引

> **最后更新**: 2026-05-12
> **当前版本**: v3.1.0 (Alpha → Beta 阶段)

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

### v3.1.0 (当前版本: Alpha → Beta 阶段)

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