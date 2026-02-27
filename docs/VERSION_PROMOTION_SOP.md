# SQLRustGo 版本推进标准操作流程（SOP）

> 版本：v1.2 (更新分支命名规范)
> 日期：2026-02-20
> 来源：ChatGPT 建议 + 项目实际情况 + TRAE 修正

---

## 1. 目标

规范版本从 alpha → beta → release → baseline 的推进流程，确保：

- 功能开发可控
- 稳定性逐级验证
- 基线分支始终可发布
- 可回溯、可审计、可恢复

---

## 2. 分支职责定义

| 分支 | 角色 | 是否允许新功能 | 是否允许修 Bug |
|:-----|:-----|:---------------|:---------------|
| **alpha** | 当前功能开发阶段 | ✅ | ✅ |
| **beta** | 稳定性验证阶段 | ⚠️ 仅已规划功能 | ✅ |
| **release** | 发布候选阶段 | ❌ | 仅阻断性 Bug |
| **release/vX.Y.Z** | 正式稳定版本 | ❌ | 仅 hotfix |
| **baseline** | 历史参考版本 | ❌ | 仅 hotfix |

> **注**：Beta 阶段允许实现"Alpha 阶段已规划的功能"，禁止"新增未规划功能"
> **注**：从 v1.0.0 开始，正式稳定版本使用 `release/vX.Y.Z` 格式命名，`baseline` 分支保留作为历史参考

---

## 3. 版本推进总流程

```
alpha → beta → release → release/vX.Y.Z
```

**禁止跳级合并。**

---

## 4. Alpha 阶段流程

### 4.1 进入 Alpha 阶段

从最新的正式版本分支切出：

```bash
git checkout release/vX.Y.Z
git checkout -b feature/vX+1.Y.Z-alpha
git push origin feature/vX+1.Y.Z-alpha
```

### 4.2 Alpha 阶段允许

- 新功能开发
- 结构重构
- API 调整
- 单元测试补充

### 4.3 Alpha 完成标准

必须满足：

- 所有 alpha 目标 Issue 关闭
- 所有功能合入 alpha
- CI 全部通过
- 无 blocker 级 Bug

---

## 5. Alpha → Beta 推进

### 5.1 冻结 Alpha

```bash
git checkout feature/vX.Y.Z-alpha
git tag vX.Y.Z-alpha
git push origin vX.Y.Z-alpha
```

之后：

- 不再允许新功能合入 alpha
- alpha 进入冻结状态

### 5.2 合并到 Beta

```bash
git checkout feature/vX.Y.Z-beta
git merge --no-ff feature/vX.Y.Z-alpha
git push origin feature/vX.Y.Z-beta
```

打 Beta Tag：

```bash
git tag vX.Y.Z-beta
git push origin vX.Y.Z-beta
```

---

## 6. Beta 阶段规则

Beta 阶段允许：

- 修复 Bug
- 性能优化
- 文档修正
- **已规划功能的实现**（Alpha 阶段已列入计划的功能）

**禁止：**

- 新增未规划功能
- 大规模结构性重构

### 6.1 已规划功能 vs 新增功能

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        Beta 阶段功能判断标准                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   ✅ 允许（已规划功能）                                                       │
│   ├── Alpha 阶段 Issue/计划中已列出的功能                                    │
│   ├── 综合改进计划中已定义的 P0/P1 任务                                      │
│   └── 对版本核心价值有本质影响的功能                                         │
│                                                                              │
│   ❌ 禁止（新增功能）                                                         │
│   ├── Beta 阶段临时提出的新功能                                              │
│   ├── 与当前版本目标无关的功能                                               │
│   └── 可推迟到下一版本的功能                                                 │
│                                                                              │
│   示例（v1.0.0-beta）：                                                      │
│   ├── ✅ 聚合函数 - Alpha 已规划，SQL 核心功能                               │
│   ├── ✅ 错误处理改进 - 质量保证                                             │
│   ├── ✅ 覆盖率提升 - 质量指标                                               │
│   └── ❌ 客户端/服务器分离 - 推迟到 v2.0                                     │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 6.2 Beta 结束标准

- 无严重 Bug
- 所有测试通过
- 验证报告完成

---

## 7. Beta → Release 推进

```bash
git checkout release/vX.Y.Z
git merge --no-ff feature/vX.Y.Z-beta
git push origin release/vX.Y.Z
```

打 RC Tag：

```bash
git tag vX.Y.Z-rc.1
git push origin vX.Y.Z-rc.1
```

---

## 8. Release 阶段规则

只允许：

- 阻断性 Bug 修复

每修复一次：

```bash
git tag vX.Y.Z-rc.N
```

---

## 9. Release → release/vX.Y.Z（正式版本发布）

当确认可正式发布：

```bash
git checkout -b release/vX.Y.Z
git merge --no-ff release/vX.Y.Z-rc
# 或者从最新的 RC 标签创建
# git checkout vX.Y.Z-rc.N
# git checkout -b release/vX.Y.Z
git tag vX.Y.Z
git push origin release/vX.Y.Z --tags
```

这一步称为：**正式版本发布**

---

## 10. Bug 回流机制（非常重要）

如果在 beta 或 release 修复了 Bug：

**必须回流到 alpha**，确保下一轮版本包含修复。

```bash
git checkout feature/vX.Y.Z-alpha
git merge feature/vX.Y.Z-beta
```

或

```bash
git merge release/vX.Y.Z
```

---

## 11. 下一轮开发

正式版本发布后：

```bash
git checkout release/vX.Y.Z
git checkout -b feature/vX.Y+1.0-alpha
```

开始下一版本周期。

---

## 12. 紧急 Hotfix 流程

如果正式版本分支出现严重线上问题：

```bash
git checkout release/vX.Y.Z
git checkout -b hotfix-X.Y.Z+1
```

修复后：

```bash
git checkout release/vX.Y.Z
git merge --no-ff hotfix-X.Y.Z+1
git tag vX.Y.Z+1
git push origin release/vX.Y.Z --tags
```

然后必须同步回：

```bash
git checkout feature/vX.Y+1.0-alpha
git merge release/vX.Y.Z
```

---

## 13. 版本命名规范

| 阶段 | 示例 |
|:-----|:-----|
| Alpha | v1.0.0-alpha |
| Beta | v1.0.0-beta |
| RC | v1.0.0-rc.1 |
| 正式版 | v1.0.0 |

---

## 14. 不允许的操作

```
❌ alpha 直接合 baseline
❌ beta 跳过 release 合 baseline
❌ baseline 接受新功能
❌ 在 beta 阶段新增未规划功能
❌ 在 beta 阶段做大规模重构
```

---

## 15. 核心原则总结

1. **分支职责单一**
2. **基线只能向上推进**
3. **每个阶段必须打 Tag**
4. **Bug 修复必须回流**
5. **baseline 永远保持可发布**

---

## 16. 生命周期示意图

```
release/vX.Y.Z
   ↑
release
   ↑
beta
   ↑
alpha
```

---

## ✅ 最终定义

| 分支 | 定义 |
|:-----|:-----|
| **release/vX.Y.Z** | 唯一正式可发布版本 |
| **release** | 发布候选 |
| **beta** | 稳定验证 |
| **alpha** | 功能开发 |
| **baseline** | 历史参考版本 |

---

*本文档基于 ChatGPT 建议整理，适配 SQLRustGo 项目实际情况*
