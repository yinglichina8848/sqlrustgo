# v1.2.0 版本阶段切换声明

## 变更日期

2026-03-07

## 变更概述

SQLRustGo v1.2.0 已完成 Craft 阶段，正式进入 Alpha 阶段。

## 主要变更

### 1. 目录架构重构

- ✅ 从单体架构迁移到 crates workspace 模块化架构
- ✅ 10 个核心 crate: catalog, common, executor, optimizer, parser, planner, server, storage, transaction, types

### 2. 分支结构规范化

| 旧分支 | 新分支 | 状态 |
|--------|--------|------|
| develop-v1.2.0 | alpha/v1.2.0 | ✅ Alpha 阶段 |
| develop/v1.2.0 | develop/v1.2.0 | ✅ 开发分支 |
| develop-v1.2.0 | develop-v1.2.0 | ❌ 冻结 |

### 3. 分支保护

已为以下分支配置保护规则：
- alpha/v1.2.0 - 必须通过 PR 合并
- beta/v1.1.0 - 冻结
- release/v1.1.0 - 冻结
- develop/v1.2.0 - 开发分支保护
- develop/v1.3.0 - 冻结
- main - 保护

### 4. CI 状态

- ⚠️ 由于企业 GitHub 安全策略限制外部 Actions，CI 暂时跳过
- 修复方案已准备，需要企业管理员在 GitHub Enterprise 设置中允许外部 Actions

## 版本阶段

| 阶段 | 版本 | 状态 |
|------|------|------|
| Draft | v1.0.x | ✅ 已完成 |
| Alpha | v1.1.x | ✅ 已完成 |
| Craft | v1.2.0 | ✅ 已完成 |
| Alpha | v1.2.0 | 🔄 进行中 |
| Beta | v1.2.x | ⏳ 待启动 |

## 后续计划

1. **Alpha 阶段** (当前)
   - 完善核心功能
   - 集成测试
   - 性能优化

2. **Beta 阶段**
   - 稳定性测试
   - 文档完善
   - API 冻结

3. **RC 阶段**
   - Bug 修复
   - 性能调优
   - 发布准备

## 注意事项

- Alpha 阶段 API 可能会有较大变更
- 建议在生产环境使用已发布的稳定版本
- 欢迎贡献者和测试者参与 Alpha 测试

## 联系方式

- GitHub Issues: https://github.com/minzuuniversity/sqlrustgo/issues
- 项目文档: [docs/](docs/)

---

**签名**: SQLRustGo 团队
**日期**: 2026-03-07
