# SQLRustGo 版本生命周期

> **版本**: 2.1
> **更新日期**: 2026-05-05
> **维护人**: hermes-z6g4

> **SSOT 声明**: 门禁定义以 `gate_spec.md` 为唯一权威。本文档的阶段门禁描述引用 gate_spec.md，不自行定义检查项。

---

## 一、版本阶段模型

SQLRustGo 使用四级门禁发布生命周期：

```
A-Gate → B-Gate → R-Gate → G-Gate
 (α入口)  (β入口)  (RC入口)  (GA入口)
```

该模型适用于 SOLO Coder + AI 协作开发环境，确保每个发布阶段的质量门槛。

---

## 二、阶段详解

### 2.1 A-Gate (Alpha Gate)

**阶段说明**: 开发完成阶段

**特点**:
- 核心功能实现
- 架构设计完成
- 技术验证通过
- 原型可运行

**规则**:
- 主要工作在 `develop/vX.Y.Z`
- 不发布正式 Release
- API 可能变化

**分支示例**:
```
develop/v2.9.0
```

**产出**:
- 设计文档
- 架构图
- 可运行原型

**门禁要求**:
- 编译通过
- 测试通过率 ≥ 80%
- 格式化检查通过

---

### 2.2 B-Gate (Beta Gate)

**阶段说明**: 功能冻结阶段

**特点**:
- 功能开发完成
- 进入冻结期
- API 基本稳定
- 重点修复 Bug

**规则**:
- 不再接受新功能
- 只允许 Bug Fix
- 使用 Tag 标记版本

**分支/Tag 示例**:
```
alpha/v2.9.0
v2.9.0-alpha1
v2.9.0-alpha2
```

**门禁要求**:
- 编译通过 (release 模式)
- 测试通过率 ≥ 90%
- Clippy 零警告
- 覆盖率 ≥ 75%

---

### 2.3 R-Gate (RC Gate)

**阶段说明**: 发布候选阶段

**特点**:
- 只允许严重 Bug 修复
- 文档完善
- 性能优化
- 功能完全冻结

**规则**:
- 禁止新功能
- CI 必须全部通过
- R1-R10 十项检查全部通过

**分支/Tag 示例**:
```
beta/v2.9.0
v2.9.0-rc1
v2.9.0-rc2
```

**R1-R10 检查项**:

> **详见**: [gate_spec.md](./gate_spec.md) 第 28-41 行（唯一权威）

R5 使用 `cargo llvm-cov`，R7 包含 R7a-R7d 子检查，R9 必须执行回归判定，R10 要求 proof 含 `tool_output`。

**门禁要求**:
- R1-R10 全部通过
- 测试 100% 通过
- CI 全绿
- 覆盖率 ≥ 75%

---

### 2.4 G-Gate (GA Gate)

**正式发布版本**

**阶段说明**: 正式发布阶段

**Tag**:
```
v2.9.0
```

**流程**:
```
develop/v2.9.0
    │
    └── G-Gate
        ├── Tag v2.9.0
        ├── Merge → main
        └── Create release/2.9
```

**门禁要求**:
- 所有 R1-R10 检查通过
- 覆盖率 ≥ 85%
- 所有问题关闭
- 发布审批通过

---

## 三、阶段转换检查点

### 3.1 A-Gate → B-Gate

| 检查项 | 要求 | 状态 |
|--------|------|------|
| 架构设计完成 | ✅ | |
| 接口定义完整 | ✅ | |
| 编译通过 | ✅ | |
| 测试通过率 ≥ 80% | ✅ | |

### 3.2 B-Gate → R-Gate

| 检查项 | 要求 | 状态 |
|--------|------|------|
| 功能冻结 | ✅ | |
| 无 P0/P1 Bug | ✅ | |
| 测试通过率 ≥ 90% | ✅ | |
| Clippy 零警告 | ✅ | |
| 覆盖率 ≥ 75% | ✅ | |

### 3.3 R-Gate → G-Gate

| 检查项 | 要求 | 状态 |
|--------|------|------|
| R1-R10 全部通过 | ✅ | |
| 测试 100% | ✅ | |
| CI 全绿 | ✅ | |
| 覆盖率 ≥ 85% | ✅ | |
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
| Alpha | vX.Y.Z-alpha{N} | v2.9.0-alpha1 |
| Beta | vX.Y.Z-alpha{N} | v2.9.0-alpha3 |
| RC | vX.Y.Z-rc{N} | v2.9.0-rc1 |
| GA | vX.Y.Z | v2.9.0 |
| Patch | vX.Y.Z-patch{N} | v2.9.1-patch1 |

---

## 五、版本号规则

遵循语义化版本 (Semantic Versioning):

```
MAJOR.MINOR.PATCH
2  . 9  . 0

MAJOR: 破坏性变更
MINOR: 新功能 (向后兼容)
PATCH: Bug 修复 (向后兼容)
```

---

## 六、相关文档

| 文档 | 说明 |
|------|------|
| [gate_spec.md](./gate_spec.md) | 门禁规范详细说明 |
| [RELEASE_POLICY.md](./RELEASE_POLICY.md) | 发布策略 |
| [RELEASE_GATE_CHECKLIST.md](./RELEASE_GATE_CHECKLIST.md) | 门禁检查清单 |
| [GA_RELEASE_TIMELINE.md](./GA_RELEASE_TIMELINE.md) | GA 发布 timeline |

---

## 七、变更历史

| 版本 | 日期 | 说明 |
|------|------|------|
| 2.1 | 2026-05-05 | 对齐 v2.9.0: R1-R10 表替换为引用 gate_spec.md(唯一权威), 添加 SSOT 声明 |
| 2.0 | 2026-05-05 | 替换 Draft/Alpha/Beta/RC/GA 为 A-Gate→B-Gate→R-Gate→G-Gate 模型 |
| 1.0 | 2026-03-07 | 初始版本 |

---

*本文档由 hermes-z6g4 维护。门禁权威来源: gate_spec.md*
