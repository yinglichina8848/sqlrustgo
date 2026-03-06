# SQLRustGo GitHub 分支管理方案

> **版本**: 1.0
> **制定日期**: 2026-03-06
> **制定人**: yinglichina8848
> **适用阶段**: Draft 阶段

---

## 一、当前分支状态

### 1.1 现有分支

```
main                    # 稳定版本 (v1.0.0)
develop-v1.2.0          # 当前开发分支
fix/v1.2.0-index-v2     # 当前修复分支
docs/v1.3.0-*           # 文档分支
```

### 1.2 问题分析

| 问题 | 影响 | 优先级 |
|------|------|--------|
| 分支命名不统一 | 管理混乱 | 🔴 高 |
| 缺少 develop 主分支 | 版本管理困难 | 🔴 高 |
| 缺少 release 分支 | 维护困难 | 🟡 中 |
| 分支保护不完善 | 代码安全风险 | 🔴 高 |

---

## 二、目标分支结构

### 2.1 完整分支架构

```
main ─────────────────────────────────────────────────────►
  │
  │  (merge from release/*)
  │
  ├── release/1.0 ────────────────────────────────────────►
  │   │
  │   └── v1.0.1, v1.0.2, ... (patch)
  │
  ├── release/1.1 ────────────────────────────────────────►
  │   │
  │   └── v1.1.1, v1.1.2, ... (patch)
  │
  ├── develop ────────────────────────────────────────────►
  │   │
  │   └── (下一版本开发)
  │
  ├── develop-1.2.0 ──────────────────────────────────────►
  │   │
  │   ├── refactor/directory-*   (目录重构)
  │   ├── feature/v1.2.0-*       (功能开发)
  │   ├── fix/v1.2.0-*           (Bug 修复)
  │   └── docs/v1.2.0-*          (文档更新)
  │
  └── develop-1.3.0 ──────────────────────────────────────►
      │
      └── (下一版本预研)
```

### 2.2 分支类型定义

| 分支类型 | 命名规则 | 生命周期 | 用途 |
|----------|----------|----------|------|
| `main` | `main` | 永久 | 稳定版本 |
| `develop` | `develop` | 永久 | 下一版本开发 |
| `develop-x.y.z` | `develop-{version}` | 版本周期 | 版本开发 |
| `release/x.y` | `release/{major.minor}` | 长期 | 版本维护 |
| `feature/*` | `feature/{version}-{name}` | 短期 | 功能开发 |
| `fix/*` | `fix/{version}-{name}` | 短期 | Bug 修复 |
| `refactor/*` | `refactor/{name}` | 短期 | 重构 |
| `docs/*` | `docs/{version}-{name}` | 短期 | 文档 |

---

## 三、分支创建规则

### 3.1 版本开发分支

```bash
# 创建新版本开发分支
git checkout main
git checkout -b develop-1.2.0
git push origin develop-1.2.0
```

### 3.2 功能分支

```bash
# 从版本分支创建功能分支
git checkout develop-1.2.0
git checkout -b feature/v1.2.0-cascades
git push origin feature/v1.2.0-cascades
```

### 3.3 修复分支

```bash
# 从版本分支创建修复分支
git checkout develop-1.2.0
git checkout -b fix/v1.2.0-page-bug
git push origin fix/v1.2.0-page-bug
```

### 3.4 重构分支

```bash
# 从版本分支创建重构分支
git checkout develop-1.2.0
git checkout -b refactor/directory-phase1
git push origin refactor/directory-phase1
```

---

## 四、分支合并规则

### 4.1 PR 目标分支

| 源分支 | 目标分支 | 说明 |
|--------|----------|------|
| `feature/v1.2.0-*` | `develop-1.2.0` | 功能合并到版本开发 |
| `fix/v1.2.0-*` | `develop-1.2.0` | 修复合并到版本开发 |
| `refactor/*` | `develop-1.2.0` | 重构合并到版本开发 |
| `docs/v1.2.0-*` | `develop-1.2.0` | 文档合并到版本开发 |
| `develop-1.2.0` | `main` | 版本发布时合并 |

### 4.2 合并条件

| 阶段 | CI 要求 | 审核人数 | 额外条件 |
|------|---------|----------|----------|
| Draft | 编译通过 | 1 | 无 |
| Alpha | 测试 ≥ 80% | 1 | 功能测试 |
| Beta | 测试 ≥ 95% | 2 | 回归测试 |
| RC | 测试 100% | 2 | 发布审批 |

---

## 五、Draft 阶段重构分支计划

### 5.1 目录重构分支

```
develop-1.2.0 (Draft)
    │
    ├── refactor/directory-phase1
    │   │
    │   └── 创建 Workspace 结构
    │       ├── 创建 crates/ 目录
    │       ├── 创建各 crate Cargo.toml
    │       └── 创建根 Cargo.toml (workspace)
    │
    ├── refactor/directory-phase2
    │   │
    │   └── 迁移模块
    │       ├── common crate
    │       ├── parser crate
    │       ├── catalog crate
    │       ├── storage crate
    │       ├── transaction crate
    │       ├── planner crate
    │       ├── optimizer crate
    │       ├── executor crate
    │       └── server crate
    │
    ├── refactor/directory-phase3
    │   │
    │   └── 更新导入
    │       ├── 批量替换 use crate::
    │       └── 修复导入错误
    │
    └── refactor/directory-phase4
        │
        └── 验证修复
            ├── 编译检查
            ├── 测试检查
            └── Clippy 检查
```

### 5.2 分支时间表

| 分支 | 创建日期 | 合并日期 | 负责人 |
|------|----------|----------|--------|
| `refactor/directory-phase1` | Day 1 | Day 2 | openheart |
| `refactor/directory-phase2` | Day 3 | Day 7 | openheart, heartopen |
| `refactor/directory-phase3` | Day 8 | Day 10 | heartopen |
| `refactor/directory-phase4` | Day 11 | Day 12 | maintainer |

---

## 六、分支保护配置

### 6.1 main 分支

```json
{
  "required_status_checks": {
    "strict": true,
    "contexts": ["build", "test", "lint", "security-scan"]
  },
  "required_pull_request_reviews": {
    "required_approving_review_count": 2
  },
  "enforce_admins": true,
  "allow_force_pushes": false,
  "allow_deletions": false
}
```

### 6.2 develop-1.2.0 分支

```json
{
  "required_status_checks": {
    "contexts": ["build", "test"]
  },
  "required_pull_request_reviews": {
    "required_approving_review_count": 1
  },
  "allow_force_pushes": false,
  "allow_deletions": false
}
```

---

## 七、分支清理规则

### 7.1 合并后清理

```bash
# PR 合并后自动删除分支
git branch -d feature/v1.2.0-cascades
git push origin --delete feature/v1.2.0-cascades
```

### 7.2 过期分支清理

| 分支类型 | 过期时间 | 处理方式 |
|----------|----------|----------|
| `feature/*` | 合并后立即 | 自动删除 |
| `fix/*` | 合并后立即 | 自动删除 |
| `refactor/*` | 合并后立即 | 自动删除 |
| `develop-x.y.z` | GA 发布后 | 转为 release/x.y |

---

## 八、相关文档

| 文档 | 说明 |
|------|------|
| [BRANCH_GOVERNANCE.md](../../BRANCH_GOVERNANCE.md) | 分支治理规范 |
| [RELEASE_GOVERNANCE.md](../../RELEASE_GOVERNANCE.md) | 版本治理模型 |
| [DIRECTORY_REFACTORING_PLAN.md](./DIRECTORY_REFACTORING_PLAN.md) | 目录重构计划 |

---

## 九、变更历史

| 版本 | 日期 | 说明 |
|------|------|------|
| 1.0 | 2026-03-06 | 初始版本，定义 Draft 阶段分支管理方案 |

---

*本文档由 yinglichina8848 制定*
