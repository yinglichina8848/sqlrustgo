# v2.7.0 文档索引

> **版本**: v2.7.0
> **代号**: Enterprise Resilience
> **状态**: ✅ GA 已发布 (2026-04-22)
> **最后更新**: 2026-04-22

---

## 版本概述

v2.7.0 是 SQLRustGo 迈向 **企业级韧性 (Enterprise Resilience)** 的关键版本。本版本重点实现 WAL 崩溃恢复、外键稳定性增强、备份恢复机制、审计证据链等企业级功能。

### 版本目标

| 目标 | 说明 |
|------|------|
| 企业级可靠性 | WAL 崩溃恢复、72h 稳定性验证 |
| 数据完整性 | FK 稳定性、审计证据链 |
| 运维能力 | 备份恢复、统一搜索 API、GMP Top10 |
| 性能优化 | 性能回归修复、混合排序 |

### 核心功能

| 功能 | 状态 | PR |
|------|------|-----|
| WAL 崩溃恢复 (T-01) | ✅ | - |
| FK 稳定性增强 (T-02) | ✅ | - |
| 备份恢复演练 (T-03) | ✅ | - |
| qmd-bridge 统一检索层 (T-04) | ✅ | #1713 |
| 统一检索 API (T-05) | ✅ | #1714 |
| 混合检索重排 (T-06) | ✅ | #1714 |
| GMP Top 10 审核查询 (T-07) | ✅ | #1714 |
| 审计证据链 (T-08) | ✅ | #1718 |

---

## 文档清单

### 必选文档

| 文档 | 说明 | 状态 |
|------|------|------|
| [README.md](./README.md) | 文档索引 | ✅ |
| [CHANGELOG.md](./CHANGELOG.md) | 变更日志 | ✅ |
| [RELEASE_NOTES.md](./RELEASE_NOTES.md) | 发布说明 | ✅ |
| [MIGRATION_GUIDE.md](./MIGRATION_GUIDE.md) | 升级指南 | ✅ |
| [DEPLOYMENT_GUIDE.md](./DEPLOYMENT_GUIDE.md) | 部署指南 | ✅ |
| [DEVELOPMENT_GUIDE.md](./DEVELOPMENT_GUIDE.md) | 开发指南 | ✅ |
| [TEST_PLAN.md](./TEST_PLAN.md) | 测试计划 | ✅ |
| [TEST_MANUAL.md](./TEST_MANUAL.md) | 测试手册 | ✅ |
| [EVALUATION_REPORT.md](./EVALUATION_REPORT.md) | 版本评估报告 | ✅ |
| [DOCUMENT_AUDIT.md](./DOCUMENT_AUDIT.md) | 文档审计报告 | ✅ |
| [FEATURE_MATRIX.md](./FEATURE_MATRIX.md) | 功能矩阵 | ✅ |
| [COVERAGE_REPORT.md](./COVERAGE_REPORT.md) | 覆盖率报告 | ✅ |
| [SECURITY_ANALYSIS.md](./SECURITY_ANALYSIS.md) | 安全分析 | ✅ |
| [SECURITY_REPORT.md](./SECURITY_REPORT.md) | 安全报告 | ✅ |
| [PERFORMANCE_TARGETS.md](./PERFORMANCE_TARGETS.md) | 性能目标 | ✅ |
| [PERFORMANCE_REPORT.md](./PERFORMANCE_REPORT.md) | 性能报告 | ✅ |
| [QUICK_START.md](./QUICK_START.md) | 快速开始 | ✅ |
| [INSTALL.md](./INSTALL.md) | 安装说明 | ✅ |
| [API_DOCUMENTATION.md](./API_DOCUMENTATION.md) | API 文档 | ✅ |
| [RELEASE_GATE_CHECKLIST.md](./RELEASE_GATE_CHECKLIST.md) | 门禁检查清单 | ✅ |

### OO 架构文档

| 文档 | 说明 | 状态 |
|------|------|------|
| [oo/README.md](./oo/README.md) | OO 文档索引 | ✅ |
| [oo/architecture/ARCHITECTURE_V2.7.md](./oo/architecture/ARCHITECTURE_V2.7.md) | 架构设计 | ✅ |
| [oo/modules/README.md](./oo/modules/README.md) | 模块设计索引 | ✅ |
| [oo/reports/PERFORMANCE_ANALYSIS.md](./oo/reports/PERFORMANCE_ANALYSIS.md) | 性能分析 | ✅ |
| [oo/reports/SQL92_COMPLIANCE.md](./oo/reports/SQL92_COMPLIANCE.md) | SQL 合规报告 | ✅ |
| [oo/user-guide/USER_MANUAL.md](./oo/user-guide/USER_MANUAL.md) | 用户手册 | ✅ |

### 测试报告

| 文档 | 说明 | 状态 |
|------|------|------|
| [report/GA_TEST_REPORT.md](./report/GA_TEST_REPORT.md) | GA 测试报告 | ✅ |
| [report/RC_TEST_REPORT.md](./report/RC_TEST_REPORT.md) | RC 测试报告 | ✅ |
| [report/BETA_TEST_REPORT.md](./report/BETA_TEST_REPORT.md) | Beta 测试报告 | ✅ |

### 设计文档

| 文档 | 说明 | 状态 |
|------|------|------|
| [qmd-bridge-design.md](./qmd-bridge-design.md) | QMD Bridge 设计 | ✅ |
| [gmp-top10-scenarios.md](./gmp-top10-scenarios.md) | GMP Top10 场景 | ✅ |
| [STABILITY_REPORT.md](./STABILITY_REPORT.md) | 稳定性报告 | ✅ |
| [ARCHITECTURE_DECISIONS.md](./ARCHITECTURE_DECISIONS.md) | 架构决策 | ✅ |

---

## 发布信息

| 信息 | 内容 |
|------|------|
| 版本 | v2.7.0 |
| 代号 | Enterprise Resilience |
| 发布日期 | 2026-04-22 |
| Git Tag | v2.7.0 |
| 主分支 | main |
| 开发分支 | develop/v2.7.0 |

### 发布 PR

| PR | 说明 |
|-----|------|
| #1729 | chore: v2.7.0 GA release |
| #1718 | feat(gmp): evidence module T-08 |
| #1714 | feat: unified search API T-04/T-05/T-06 |
| #1713 | feat: qmd-bridge T-04 |

---

## 相关文档

1. [v2.6.0 文档索引](../v2.6.0/README.md)
2. [长期路线图](../LONG_TERM_ROADMAP.md)
3. [版本演化计划](../VERSION_ROADMAP.md)
4. [GMP 开发计划](../../gmp-audit-db-development-plan.md)

---

*文档索引 v2.7.0*
*最后更新: 2026-04-22*
