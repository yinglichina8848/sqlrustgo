# SQLRustGo 版本生命周期

> **版本**: 1.0
> **更新日期**: 2026-03-07
> **维护人**: yinglichina8848

---

## 一、版本阶段模型

SQLRustGo 使用标准的软件发布生命周期：

```
Draft → Alpha → Beta → RC → GA
```

该模型适用于 SOLO Coder + AI 协作开发环境。

---

## 二、阶段详解

### 2.1 Draft 阶段

**阶段说明**: 设计与架构阶段

**特点**:
- 架构设计
- 文档设计
- 技术验证
- 原型实现

**规则**:
- 主要工作在 `develop-vX.Y.Z`
- 不发布 Release
- 不产生稳定 API

**Git 实现**:
```
develop/v1.2.0
```

**产出**:
- 设计文档
- 架构图
- 原型代码

**门禁要求**:
- 编译通过

---

### 2.2 Alpha 阶段

**阶段说明**: 功能开发阶段

**特点**:
- 核心功能实现
- API 可能变化
- 功能不完整

**规则**:
- 继续在 `develop-vX.Y.Z` 开发
- 使用 Tag 标记版本

**Tag 示例**:
```
v1.2.0-alpha1
v1.2.0-alpha2
v1.2.0-alpha3
```

**门禁要求**:
- 编译通过
- 测试通过率 ≥ 80%

---

### 2.3 Beta 阶段

**阶段说明**: 功能基本完成

**特点**:
- 功能冻结
- API 基本稳定
- 重点修复 Bug

**规则**:
- 不再接受新功能
- 只允许 Bug Fix

**Tag 示例**:
```
v1.2.0-beta1
v1.2.0-beta2
```

**门禁要求**:
- 编译通过
- 测试通过率 ≥ 95%
- Clippy 零警告

---

### 2.4 RC 阶段 (Release Candidate)

**阶段说明**: 候选发布版本

**特点**:
- 只允许严重 Bug 修复
- 文档完善
- 性能优化

**规则**:
- 禁止新功能
- CI 必须全部通过

**Tag 示例**:
```
v1.2.0-rc1
v1.2.0-rc2
```

**门禁要求**:
- 测试 100% 通过
- CI 全绿

---

### 2.5 GA 阶段 (General Availability)

**正式发布版本**

**Tag**:
```
v1.2.0
```

**流程**:
```
develop/v1.2.0
    │
    └── GA
        ├── Tag v1.2.0
        ├── Merge → main
        └── Create release/1.2
```

**门禁要求**:
- 所有问题关闭
- 发布审批通过

---

## 三、阶段转换检查点

### 3.1 草案 → 阿尔法

| 检查项 | 要求 | 状态 |
|--------|------|------|
| 架构设计完成 | ✅ | |
| 接口定义完整 | ✅ | |
| 编译通过 | ✅ | |

### 3.2 阿尔法 → 贝塔

| 检查项 | 要求 | 状态 |
|--------|------|------|
| 核心功能完成 | 测试 ≥ 80% | |
| 无 P0 Bug | ✅ | |
| 功能评审通过 | ✅ | |

### 3.3 测试版 → RC

| 检查项 | 要求 | 状态 |
|--------|------|------|
| 功能冻结 | ✅ | |
| 测试 ≥ 95% | ✅ | |
| Clippy 零警告 | ✅ | |

### 3.4 RC → GA

| 检查项 | 要求 | 状态 |
|--------|------|------|
| 测试 100% | ✅ | |
| CI 全绿 | ✅ | |
| 无开放 Bug | ✅ | |
| 发布审批 | ✅ | |

---

## 四、Tag 命名规范

### 4.1 格式

```
v{MAJOR}.{MINOR}.{PATCH}-{phase}{number}
```

### 4.2 示例

| 阶段 | Tag 格式 | 示例 |
|------|----------|------|
| Alpha |vX.Y.Z-alpha{N}|v1.2.0-alpha1|
| Beta |vX.Y.Z-beta{N}| v1.2.0-beta1 |
| RC |vX.Y.Z-rc{N}| v1.2.0-rc1 |
| GA | vX.Y.Z | v1.2.0 |
| Patch |vX.Y.Z-补丁{N}|v1.2.1-补丁1|

---

## 五、Milestone 使用

每个版本使用 GitHub Milestone 管理:

```
v1.2.0 Milestone
    │
    ├── Issues
    │   ├── Feature Request
    │   ├── Bug Report
    │   └── Documentation
    │
    └── Pull Requests
        ├── PR for Feature A
        ├── PR for Feature B
        └── PR for Bug Fix
```

---

## 六、版本号规则

遵循语义化版本 (Semantic Versioning):

```
MAJOR.MINOR.PATCH
1  . 2  . 0

MAJOR: 破坏性变更
MINOR: 新功能 (向后兼容)
PATCH: Bug 修复 (向后兼容)
```

---

## 七、相关文档

| 文档 | 说明 |
|------|------|
| [BRANCH_GOVERNANCE.md](../BRANCH_GOVERNANCE.md) | 分支治理 |
| [RELEASE_POLICY.md](./RELEASE_POLICY.md) | 发布策略 |
| [CONTRIBUTING.md](../CONTRIBUTING.md) | 贡献指南 |

---

## 八、变更历史

| 版本 | 日期 | 说明 |
|------|------|------|
| 1.0 | 2026-03-07 | 初始版本 |

---

*本文档由 yinglichina8848 维护*
