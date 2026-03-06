# SQLRustGo 发布策略

> **版本**: 1.0
> **更新日期**: 2026-03-07
> **维护人**: yinglichina8848

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
| **prerelease** | 预发布版本 | alpha, beta, rc |
| **build** | 构建元数据 | +build.123 |

### 1.2 内部版本号

SQLRustGo 内部使用 `version` 字段管理版本:

```toml
[package]
version = "1.2.0"
```

---

## 二、版本阶段

### 2.1 阶段定义

| 阶段 | 状态 | 目标用户 | 稳定性 |
|------|------|----------|--------|
| **Draft** | 架构设计 | 开发团队 | 无保证 |
| **Alpha** | 功能开发 | 早期测试者 | 低 |
| **Beta** | Bug 修复 | 测试用户 | 中 |
| **RC** | 候选发布 | 预览用户 | 高 |
| **GA** | 正式发布 | 生产用户 | 100% |

### 2.2 阶段转换规则

```
Draft → Alpha → Beta → RC → GA
  │       │       │      │
  │       │       │      └─ 需通过 Release Gate
  │       │       └──────── 需通过 Beta Gate
  │       └──────────────── 需通过 Alpha Gate
  └───────────────────────── 架构评审通过
```

详见: [RELEASE_LIFECYCLE.md](./RELEASE_LIFECYCLE.md)

---

## 三、发布流程

### 3.1 发布检查点

| 检查点 | 触发条件 | 审核人 |
|--------|----------|--------|
| **Draft Gate** | 进入 Alpha 前 | 架构委员会 |
| **Alpha Gate** | 进入 Beta 前 | Maintainer |
| **Beta Gate** | 进入 RC 前 | Maintainer + 1 Reviewer |
| **Release Gate** | 进入 GA 前 | 完整评审 |

### 3.2 发布前检查清单

#### Release Gate 检查项:

- [ ] 所有 CI 检查通过
- [ ] 单元测试覆盖率 ≥ 80%
- [ ] 集成测试全部通过
- [ ] 文档齐全 (API, CHANGELOG)
- [ ] 安全扫描无高危漏洞
- [ ] 性能基准测试达标
- [ ] 向后兼容性检查通过
- [ ] 版本号已更新
- [ ] CHANGELOG.md 已更新

### 3.3 发布执行步骤

```
1. 创建 Tag
   git tag -a v1.2.0 -m "Release v1.2.0"

2. 推送 Tag
   git push origin v1.2.0

3. 创建 GitHub Release
   - 填写 Release Notes
   - 附加构建产物
   - 标记为 Pre-release (非 GA)

4. 合并到 main
   git checkout main
   git merge release/1.2
   git push origin main

5. 创建维护分支
   git checkout -b release/1.2
   git push origin release/1.2
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
git push origin main --force

# 或者切换到上一个稳定版本
git checkout v1.1.0
```

---

## 六、相关文档

| 文档 | 说明 |
|------|------|
| [BRANCH_GOVERNANCE.md](../BRANCH_GOVERNANCE.md) | 分支治理 |
| [RELEASE_LIFECYCLE.md](./RELEASE_LIFECYCLE.md) | 版本生命周期 |
| [CONTRIBUTING.md](../CONTRIBUTING.md) | 贡献指南 |
| [AI_COLLABORATION.md](./AI_COLLABORATION.md) | AI 协作规则 |

---

## 七、变更历史

| 版本 | 日期 | 说明 |
|------|------|------|
| 1.0 | 2026-03-07 | 初始版本 |

---

*本文档由 yinglichina8848 维护*
