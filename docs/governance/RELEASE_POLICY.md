# SQLRustGo 发布策略

> **版本**: 2.1
> **更新日期**: 2026-05-05
> **维护人**: hermes-z6g4

> **SSOT 声明**: 门禁定义以 `gate_spec.md` 为唯一权威。本文档的阶段/门禁概述引用 gate_spec.md，不自行定义检查项。

---

## 一、版本号规范

### 1.1 语义化版本 (Semantic Versioning)

```
MAJOR.MINOR.PATCH[-prerelease][+build]
示例: v1.2.0, v1.2.0-alpha1, v1.2.0-beta1+build.123
```

| 组件 | 变化条件 | 示例 |
|------|----------|------|
| **MAJOR** | 不兼容的 API 变更 | 1.0.0 → 2.0.0 |
| **MINOR** | 向后兼容的新功能 | 1.2.0 → 1.3.0 |
| **PATCH** | 向后兼容的 Bug 修复 | 1.2.0 → 1.2.1 |
|**预发布**| 预发布版本 |阿尔法、贝塔、RC|
| **build** | 构建元数据 | +build.123 |

### 1.2 内部版本号

SQLRustGo 内部使用 `version` 字段管理版本:

```toml
[package]
version = "1.2.0"
```

---

## 二、版本阶段与门禁模型

### 2.1 四级门禁模型 (A/B/R/G)

v2.9.0+ 采用四级门禁模型，确保每个发布阶段的质量：

```
A-Gate → B-Gate → R-Gate → G-Gate
 (α入口)  (β入口)  (RC入口)  (GA入口)
```

|| 门禁 | 名称 | 目标 | 覆盖率目标 |
||------|------|------|-----------|
|| **A-Gate** | Alpha Gate | 开发完成 | ≥50% |
|| **B-Gate** | Beta Gate | 功能冻结 | ≥75% |
|| **R-Gate** | RC Gate | 发布候选 | ≥75% |
|| **G-Gate** | GA Gate | 正式发布 | ≥85% |

### 2.2 阶段与门禁对应

|| 阶段 | 状态 | 门禁 | 目标用户 |
||------|------|------|----------|
|| **Draft** | 架构设计 | 无 | 开发团队 |
|| **Alpha** | 功能开发 | A-Gate | 早期测试者 |
|| **Beta** | Bug 修复 | B-Gate | 测试用户 |
|| **RC** | 候选发布 | R-Gate (R1-R10) | 预览用户 |
|| **GA** | 正式发布 | G-Gate | 生产用户 |

### 2.3 R-Gate 内部检查项 (R1-R10)

> **详见**: [gate_spec.md](./gate_spec.md) — 第 28-41 行 R1-R10 定义（唯一权威）

本文档不重复定义 R1-R10 的具体命令和阈值。R5 使用 `cargo llvm-cov`，R7 包含 R7a-R7d 子检查，R9 必须执行性能回归判定，R10 要求 proof 文件含 `tool_output` 字段。

### 2.4 阶段转换规则

```
Draft → Alpha → Beta → RC → GA
  │       │       │      │
  │       │       │      └─ 需通过 G-Gate
  │       │       └──────── 需通过 R-Gate (R1-R10)
  │       └──────────────── 需通过 B-Gate
  └───────────────────────── 需通过 A-Gate
```

详见: [gate_spec.md](./gate_spec.md) | [RELEASE_LIFECYCLE.md](./RELEASE_LIFECYCLE.md)

---

## 三、发布流程

### 3.1 发布检查点 (门禁)

|| 检查点 | 触发条件 | 审核人 |
||--------|----------|--------|
|| **A-Gate** | 进入 Alpha 前 | 架构委员会 |
|| **B-Gate** | 进入 Beta 前 | 维护者 |
|| **R-Gate** | 进入 RC 前 | 维护者 + 1 审阅者 |
|| **G-Gate** | 进入 GA 前 | 完整评审 |

### 3.2 门禁检查清单

G-Gate 检查项详见 [gate_spec.md](./gate_spec.md) 第五章 G-Gate。

除 gate_spec.md 定义的门禁外，GA 发布还需满足：

- [ ] 所有 R1-R10 检查通过（证据见 `artifacts/gate/vX.Y.Z/`）
- [ ] 门禁豁免/延期记录已录入 [GATE_EXEMPTIONS.md](./GATE_EXEMPTIONS.md)
- [ ] 文档齐全：API 文档、CHANGELOG、RELEASE_NOTES
- [ ] 版本号已在 `Cargo.toml` 和所有文档中更新
- [ ] Proof files 全部包含 `tool_output` 字段
- [ ] 性能基准 `perf_baselines/vX.Y.Z_baseline.json` 已建立（标注 PROVISIONAL 如适用）
- [ ] CI 检查通过且产物归档至 `artifacts/`

### 3.3 发布执行步骤

```
1. 创建 Tag
   git tag -a v2.9.0 -m "Release v2.9.0"

2. 推送 Tag
   git push gitea v2.9.0

3. 创建 Gitea Release
   - 填写 Release Notes
   - 附加构建产物
   - 标记为 Pre-release (非 GA)

4. 合并到 main
   git checkout main
   git merge release/2.9
   git push gitea main

5. 创建维护分支
   git checkout -b release/2.9
   git push gitea release/2.9
```

---

## 四、版本维护

### 4.1 维护周期

| 版本 | 维护状态 | 维护期限 |
|------|----------|----------|
| **GA** | 活跃 | 发布后 12 个月 |
| **维护中** | Bug 修复 | 发布后 6 个月 |
| **废弃** | 安全更新 | 发布后 3 个月 |

### 4.2 热修复流程

```
发现 Bug
    │
    ├── 严重 (Security/Critical)
    │   └── 创建 hotfix/vX.Y.Z-name
    │
    ├── 普通
    │   └── 创建 fix/vX.Y.Z-name
    │
    └── 优化
        └── 纳入下一版本
```

### 4.3 版本降级策略

当发现严重问题时:

1. **立即停止分发**: 撤回当前版本
2. **评估影响范围**: 确定受影响的用户
3. **发布补丁版本**: vX.Y.Z → vX.Y.Z+1
4. **发布安全公告**: 说明问题及解决方案

---

## 五、回滚机制

### 5.1 回滚触发条件

- [ ] 严重安全漏洞
- [ ] 数据丢失或损坏
- [ ] 核心功能完全失效
- [ ] 性能下降 > 50%

### 5.2 回滚执行

```bash
# 回滚到上一个 Tag
git revert HEAD
git push gitea main --force

# 或者切换到上一个稳定版本
git checkout v2.9.0
```

---

## 六、相关文档

|| 文档 | 说明 |
||------|------|
|| [gate_spec.md](./gate_spec.md) | A/B/R/G 门禁规范详细说明 |
|| [RELEASE_LIFECYCLE.md](./RELEASE_LIFECYCLE.md) | 版本生命周期 |
|| [RELEASE_GATE_CHECKLIST.md](./RELEASE_GATE_CHECKLIST.md) | 门禁检查清单 |
|| [GATE_CI_CD.md](./GATE_CI_CD.md) | CI/CD 自动化 |
|| [AI_COLLABORATION.md](./AI_COLLABORATION.md) | AI 协作规则 |

---

## 七、变更历史

| 版本 | 日期 | 说明 |
|------|------|------|
| 2.1 | 2026-05-05 | 对齐 v2.9.0: 移除 R1-R10 重复定义(引用 gate_spec.md), 消除 tarpaulin 引用, G-Gate 检查项引用 gate_spec.md |
| 2.0 | 2026-05-05 | 对齐 v2.9.0 门禁模型：A/B/R/G 四级门禁，R1-R10 检查项 |
| 1.0 | 2026-03-07 | 初始版本 |

---

*本文档由 hermes-z6g4 维护。门禁权威来源: gate_spec.md*
