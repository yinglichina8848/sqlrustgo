# 分支管理策略

> 版本：v1.0
> 日期：2026-02-16

---

## 一、分支架构

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         分支架构                                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   main (保护分支)                                                            │
│   │                                                                           │
│   │   - 稳定的、可用于教学演示和工业生产的版本                              │
│   │   - 只接受从 baseline 合并                                              │
│   │   - 需要 PR 审查                                                       │
│   │                                                                           │
│   ↑                                                                           │
│   │                                                                           │
│   baseline (合并点)                                                          │
│   │                                                                           │
│   │   - 每个版本的正式发布版                                                │
│   │   - 来自 Alpha/Beta 分支合并                                           │
│   │   - 包含完整的版本功能                                                 │
│   │                                                                           │
│   ↑                              ↑                                           │
│   │                              │                                           │
│   │    ┌──────────────┐    ┌──────────────┐                               │
│   │    │ Alpha        │    │ Beta         │                               │
│   │    │ 分支         │    │ 分支         │                               │
│   │    └──────┬───────┘    └──────┬───────┘                               │
│   │           │                    │                                        │
│   │           ↓                    ↓                                        │
│   │    feature/v1.x.x-alpha   feature/v1.x.x-beta                          │
│   │           ↓                    ↓                                        │
│   │    文档补全 + 错误修复     自动测试 + 性能分析 + CI/CD                │
│   │                                                                           │
│   │                                                                       │
│   │                                                                       │
│   │    ┌──────────────────────────────────────────────────────┐            │
│   │    │                   功能分支                           │            │
│   │    │                                                       │            │
│   │    │  feature/v1.1.0-aggregate   feature/v1.1.0-orderby │            │
│   │    │         ↓                           ↓                │            │
│   │    │    聚合函数实现                 ORDER BY 实现        │            │
│   │    │                                                       │            │
│   │    └──────────────────────────────────────────────────────┘            │
│   │                                                                       │
│   └───────────────────────────────────────────────────────────────────────  │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 二、各分支说明

### 2.1 main 分支（主分支）

| 属性 | 说明 |
|:-----|:-----|
| 用途 | 稳定版本，用于教学演示和工业生产 |
| 保护 | ✅ 禁止直接推送，需要 PR 审查 |
| 合并来源 | 只能从 baseline 合并 |
| 标签 | v1.0.0, v1.1.0, v2.0.0 等正式版本标签 |

### 2.2 baseline 分支（基线分支）

| 属性 | 说明 |
|:-----|:-----|
| 用途 | 每个版本的正式发布版 |
| 保护 | ✅ 禁止直接推送，需要 PR 审查 |
| 合并来源 | 从 Alpha 或 Beta 分支合并 |
| 生命周期 | 每个版本一个 baseline |

### 2.3 Alpha 分支

| 属性 | 说明 |
|:-----|:-----|
| 用途 | 文档补全 + 错误修复 |
| 命名 | `feature/vX.Y.Z-alpha` |
| 合并目标 | baseline |
| 进入条件 | 功能开发完成 |

**任务**：
- [ ] API 文档补全
- [ ] 用户手册更新
- [ ] 代码错误修复
- [ ] 测试覆盖率 >= 90%

### 2.4 Beta 分支

| 属性 | 说明 |
|:-----|:-----|
| 用途 | 自动安装、测试和性能分析，CI/CD 流程完整 |
| 命名 | `feature/vX.Y.Z-beta` |
| 合并目标 | baseline |
| 进入条件 | Alpha 分支通过审查 |

**任务**：
- [ ] CI/CD 流水线完善
- [ ] 基准测试建立
- [ ] 多平台验证
- [ ] 安全扫描

### 2.5 功能分支

| 属性 | 说明 |
|:-----|:-----|
| 用途 | 单个功能开发 |
| 命名 | `feature/vX.Y.Z-<feature-name>` |
| 合并目标 | 直接合并到 baseline 或通过 Alpha/Beta |
| 协作方式 | 可并行开发，每个功能一个分支 |

---

## 三、版本发布流程

```
功能开发 → 功能分支
    ↓
功能分支合并 → Alpha 分支
    ↓
Alpha 阶段：文档补全 + 错误修复
    ↓
Alpha 审查通过 → Beta 分支
    ↓
Beta 阶段：自动测试 + 性能分析 + CI/CD
    ↓
Beta 审查通过 → baseline (正式版本)
    ↓
baseline 审查通过 → main (稳定版本)
    ↓
打标签发布：vX.Y.Z
```

---

## 四、当前分支状态

### 4.1 分支列表

| 分支 | 状态 | 用途 |
|:-----|:-----|:-----|
| `main` | 保护 | 稳定版本 |
| `baseline` | 保护 | 当前基线版本 |
| `feature/v1.0.0-evaluation` | 开发中 | 评估分支 → 将转为 baseline |
| `feature/v1.0.0-alpha` | 新建 | Alpha 分支 |
| `feature/v1.0.0-beta` | 新建 | Beta 分支 |

### 4.2 v1.0.0 发布流程

```
feature/v1.0.0-evaluation (当前)
    ↓ [评估完成]
feature/v1.0.0-alpha (文档 + 错误修复)
    ↓ [Alpha 通过]
feature/v1.0.0-beta (CI + 性能测试)
    ↓ [Beta 通过]
baseline (v1.0.0 正式版)
    ↓ [合并]
main (稳定版)
    ↓ [打标签]
v1.0.0 Release
```

---

## 五、分支命名规范

```
feature/<version>-<type>

示例：
- feature/v1.1.0-aggregate    # 聚合函数功能
- feature/v1.1.0-orderby      # ORDER BY 功能
- feature/v1.0.0-alpha        # Alpha 阶段
- feature/v1.0.0-beta         # Beta 阶段
- feature/v1.0.0-evaluation   # 评估分支
```

---

## 六、协作示例

### 6.1 单功能开发流程

```bash
# 1. 从 baseline 创建功能分支
git checkout -b feature/v1.1.0-aggregate baseline

# 2. 开发功能
# ... 实现代码 ...

# 3. 提交并推送到远程
git push -u origin feature/v1.1.0-aggregate

# 4. 创建 PR，合并到 baseline
```

### 6.2 多分支并行开发

```bash
# 实例 A: 开发聚合函数
git checkout -b feature/v1.1.0-aggregate baseline
# ... 工作 ...

# 实例 B: 开发 ORDER BY
git checkout -b feature/v1.1.0-orderby baseline
# ... 工作 ...

# 实例 C: 处理 Alpha 文档
git checkout feature/v1.0.0-alpha
# ... 工作 ...
```

---

## 七、保护分支设置

### 7.1 main 分支

- ✅ 禁止直接推送
- ✅ 需要 PR 审查
- ✅ 需要 1 人审查通过

### 7.2 baseline 分支

- ✅ 禁止直接推送
- ✅ 需要 PR 审查
- ✅ 需要 1 人审查通过
- ✅ 状态检查通过（CI）

---

## 八、附录

### 8.1 快速命令参考

```bash
# 创建功能分支
git checkout -b feature/v1.x.x-featureName baseline

# 创建 Alpha 分支
git checkout -b feature/v1.x.x-alpha baseline

# 创建 Beta 分支
git checkout -b feature/v1.x.x-beta baseline

# 合并到 baseline
git checkout baseline
git merge feature/v1.x.x-featureName
git push origin baseline

# 合并到 main
git checkout main
git merge baseline
git push origin main
```

### 8.2 相关文档

- [版本演化规划](./2026-02-16-version-evolution-plan.md)

---

> 本文档定义 SQLRustGo 分支管理策略，确保版本发布的规范性和稳定性。
