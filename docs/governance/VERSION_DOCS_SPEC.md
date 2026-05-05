# 版本文档规范 (VERSION_DOCS_SPEC)

> **版本**: 1.0
> **创建日期**: 2026-05-05
> **维护人**: hermes-z6g4
> **适用范围**: v2.9.0+

---

## 一、目的

定义 SQLRustGo 每个发布版本必须遵守的文档标准，解决以下问题：

- 文档结构、格式、内容要求不统一
- 版本号、日期、阶段状态缺失或混乱
- 文档间引用使用绝对路径导致死链
- 审计时无法快速判断必选文档是否齐全

---

## 二、元数据要求

每个版本文档头部必须包含以下元数据：

```markdown
> **版本**: <主版本号>.<次版本号>.<补丁版本号>
> **更新日期**: YYYY-MM-DD
> **维护人**: <GitHub/Gitea 用户名>
> **阶段状态**: Draft | Alpha | Beta | RC | GA
```

**示例**:
```markdown
> **版本**: 2.9.0
> **更新日期**: 2026-05-05
> **维护人**: hermes-z6g4
> **阶段状态**: RC
```

---

## 三、命名规范

| 文档类型 | 文件名 | 位置 |
|----------|--------|------|
| 发布说明 | `RELEASE_NOTES.md` | `docs/releases/vX.Y.Z/` |
| 变更日志 | `CHANGELOG.md` | `docs/releases/vX.Y.Z/` |
| 升级指南 | `MIGRATION_GUIDE.md` | `docs/releases/vX.Y.Z/` |
| 部署指南 | `DEPLOYMENT_GUIDE.md` | `docs/releases/vX.Y.Z/` |
| 开发指南 | `DEVELOPMENT_GUIDE.md` | `docs/releases/vX.Y.Z/` |
| 测试计划 | `TEST_PLAN.md` | `docs/releases/vX.Y.Z/` |
| 测试手册 | `TEST_MANUAL.md` | `docs/releases/vX.Y.Z/` |
| 覆盖率报告 | `COVERAGE_REPORT.md` | `docs/releases/vX.Y.Z/` |
| 安全分析 | `SECURITY_ANALYSIS.md` | `docs/releases/vX.Y.Z/` |
| 性能目标 | `PERFORMANCE_TARGETS.md` | `docs/releases/vX.Y.Z/` |
| RC 状态报告 | `RC_STATUS_YYYYMMDD.md` | `docs/releases/vX.Y.Z/` |
| 门禁豁免记录 | `GATE_EXEMPTIONS.md` | `docs/governance/` |
| OO 文档索引 | `oo/README.md` | `docs/releases/vX.Y.Z/oo/` |

---

## 四、最小文档集

版本发布前，以下文档必须存在且通过 `DOCUMENT_COMPLETENESS_CHECK.md` 检查：

### 必选文档 (Required)

| 文档 | 条件 | 说明 |
|------|------|------|
| `RELEASE_NOTES.md` | RC 后 | 新功能、变更、已知问题 |
| `CHANGELOG.md` | RC 后 | 逐条变更记录 |
| `MIGRATION_GUIDE.md` | RC 后（有 breaking changes 时） | 升级注意事项 |
| `COVERAGE_REPORT.md` | RC 后 | 覆盖率数据 |
| `SECURITY_ANALYSIS.md` | RC 后 | 安全分析结论 |
| `TEST_PLAN.md` | Alpha 后 | 测试计划 |
| RC Gate 报告 | GA 前 | R1-R10 全部检查项通过证据 |

### 可选文档 (Optional)

| 文档 | 说明 |
|------|------|
| `PERFORMANCE_TARGETS.md` | 性能基准和目标 |
| `DEPLOYMENT_GUIDE.md` | 部署指南 |
| `DEVELOPMENT_GUIDE.md` | 开发指南 |

---

## 五、链接规范

- **必须使用相对路径**: `../gate_spec.md`，`./oo/README.md`
- **禁止使用绝对路径**: `http://192.168.0.252:3000/...`
- **跨版本引用**: 明确标注版本号，如 `gate_spec.md (v1.2)`

---

## 六、内容模板

### RELEASE_NOTES.md

```markdown
# vX.Y.Z Release Notes

> **版本**: X.Y.Z
> **日期**: YYYY-MM-DD
> **阶段**: GA

## 新功能
- ...

## 变更
- ...

## Bug 修复
- ...

## 已知问题
- ...

## 升级注意
- ...
```

### CHANGELOG.md

```markdown
# Changelog

## [X.Y.Z] - YYYY-MM-DD

### Added
### Changed
### Deprecated
### Removed
### Fixed
### Security
```

---

## 七、版本一致性检查

每次 RC 门禁通过前，执行以下一致性检查：

| 检查项 | 方法 |
|--------|------|
| 文档版本号 = 当前版本 | `grep -r "版本>: X.Y.Z" docs/releases/vX.Y.Z/` |
| 无遗留旧版本号 | `grep -r "v1.8.0\|v2.7.0" docs/releases/v2.9.0/`（应为 0 结果） |
| 日期格式正确 | `YYYY-MM-DD` 格式，非中文日期 |
| 阶段状态正确 | 不得出现 "Draft" 状态的 RC/GA 文档 |

---

## 八、变更历史

| 版本 | 日期 | 说明 |
|------|------|------|
| 1.0 | 2026-05-05 | 初始版本，定义元数据、命名、最小文档集、链接规范、内容模板 |
