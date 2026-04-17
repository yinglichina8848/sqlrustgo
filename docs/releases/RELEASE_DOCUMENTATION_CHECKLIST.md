# 版本发布文档清单

> **版本**: 1.0
> **最后更新**: 2026-04-18

---

## 概述

本文档定义 SQLRustGo 每个版本发布时必须提供的文档类别和清单。

---

## 一、必需文档 (A类 - 必须提供)

### 1.1 版本信息文档

| 文档 | 文件名 | 说明 | 状态 |
|------|---------|------|------|
| 版本计划 | `VERSION_PLAN.md` | 版本目标、里程碑、功能列表 | ⏳ |
| 发布说明 | `RELEASE_NOTES.md` | 新增功能、已知问题、升级指南 | ⏳ |
| 变更日志 | `CHANGELOG.md` | 根目录变更日志 | ⏳ |

### 1.2 测试文档

| 文档 | 文件名 | 说明 | 状态 |
|------|---------|------|------|
| 门禁检查清单 | `RELEASE_GATE_CHECKLIST.md` | 发布前必须通过的检查项 | ⏳ |
| 测试计划 | `TEST_PLAN.md` | 测试策略、覆盖率目标 | ⏳ |
| 集成测试计划 | `INTEGRATION_TEST_PLAN.md` | 集成测试方案 | ⏳ |

### 1.3 功能文档

| 文档 | 文件名 | 说明 | 状态 |
|------|---------|------|------|
| 功能集成状态 | `INTEGRATION_STATUS.md` | 功能模块集成进度 | ⏳ |
| 功能矩阵 | `FEATURE_MATRIX.md` | 功能完整矩阵 | ⏳ |
| 性能目标 | `PERFORMANCE_TARGETS.md` | 性能基准目标 | ⏳ |

---

## 二、推荐文档 (B类 - 应该提供)

### 2.1 用户文档

| 文档 | 文件名 | 说明 | 状态 |
|------|---------|------|------|
| 用户手册 | `USER_MANUAL.md` | 使用说明 | ⏳ |
| 升级指南 | `UPGRADE_GUIDE.md` | 从上一版本升级步骤 | ⏳ |
| 部署指南 | `DEPLOYMENT_GUIDE.md` | 部署配置 | ⏳ |

### 2.2 开发文档

| 文档 | 文件名 | 说明 | 状态 |
|------|---------|------|------|
| API 文档 | `API_DOCUMENTATION.md` | 接口说明 | ⏳ |
| 开发计划 | `DEVELOPMENT_PLAN.md` | 开发安排 | ⏳ |
| 架构决策 | `ARCHITECTURE_DECISIONS.md` | ADR 记录 | ⏳ |

### 2.3 报告文档

| 文档 | 文件名 | 说明 | 状态 |
|------|---------|------|------|
| 性能报告 | `PERFORMANCE_REPORT.md` | 性能测试结果 | ⏳ |
| 安全报告 | `SECURITY_REPORT.md` | 安全审计 | ⏳ |
| 覆盖率报告 | `COVERAGE_REPORT.md` | 测试覆盖率 | ⏳ |

---

## 三、特定版本文档

### 3.1 Alpha 版本

最低要求:
- [x] VERSION_PLAN.md
- [x] RELEASE_GATE_CHECKLIST.md
- [x] TEST_PLAN.md
- [x] CHANGELOG.md

### 3.2 Beta 版本

最低要求 +:
- [x] RELEASE_NOTES.md
- [x] INTEGRATION_STATUS.md
- [x] FEATURE_MATRIX.md
- [x] PERFORMANCE_TARGETS.md

### 3.3 RC (Release Candidate) 版本

Beta 要求 +:
- [x] UPGRADE_GUIDE.md
- [x] USER_MANUAL.md
- [x] 安全性检查

### 3.4 GA (General Availability) 版本

RC 要求 +:
- [x] API_DOCUMENTATION.md
- [x] PERFORMANCE_REPORT.md
- [x] SECURITY_REPORT.md
- [x] COVERAGE_REPORT.md

---

## 四、文档模板

### 4.1 VERSION_PLAN.md 模板

```markdown
# v{X}.{Y}.{Z} 版本计划

> **版本**: v{X}.{Y}.{Z}
> **代号**: 
> **阶段**: Alpha/Beta/RC/GA

## 版本目标

## 里程碑

## 功能列表

### P0 功能

### P1 功能

### P2 功能

## 测试策略

## 风险评估
```

### 4.2 RELEASE_NOTES.md 模板

```markdown
# v{X}.{Y}.{Z} Release Notes

> **版本**: v{X}.{Y}.{Z}
> **发布日期**: 

## 版本概述

## 新增功能

### 功能 A

### 功能 B

## 重大变更

## 已知问题

## 升级指南

## 相关链接
```

### 4.3 FEATURE_MATRIX.md 模板

```markdown
# SQLRustGo v{X}.{Y}.{Z} 功能矩阵

## 1. SQL 功能

| 功能 | 状态 | 备注 |
|------|------|------|
| SELECT | ✅ | |

## 2. 存储引擎

## 3. API 接口
```

---

## 五、检查清单

### 发布前检查

- [ ] 所有必需文档已创建
- [ ] 文档格式符合模板
- [ ] 文档无语法错误
- [ ] 链接有效
- [ ] 版本号正确
- [ ] 日期正确

### 文档质量检查

- [ ] 内容完整
- [ ] 示例可运行
- [ ] 无占位符
- [ ] 无 TODO
- [ ] 术语一致

---

## 六、版本历史

| 版本 | 日期 | 说明 |
|------|------|------|
| 1.0 | 2026-04-18 | 初始版本 |

---

*本文档由 SQLRustGo Team 维护*
