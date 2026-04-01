# 分支保护配置

## 概述

为了保护 `develop/v1.9.0` 和 `release/v1.9.0` 分支，需要在 GitHub 仓库设置中配置分支保护规则。

## 手动设置步骤

### 1. 进入仓库设置

1. 打开 https://github.com/minzuuniversity/sqlrustgo/settings/branches
2. 点击 "Add branch protection rule"

### 2. 配置 develop/v1.9.0

填写以下配置：

- **Branch name pattern**: `develop/v1.9.0`
- ✅ Require a pull request before merging
- ✅ Require approvals (1 approval required)
- ✅ Require review from code owners
- ✅ Dismiss stale reviews when new commits are pushed
- ✅ Allow force pushes: **NO**
- ✅ Allow deletions: **NO**
- ✅ Require status checks to pass before merging
  - 添加 `ci/check` 状态检查

### 3. 配置 release/v1.9.0

填写以下配置：

- **Branch name pattern**: `release/v1.9.0`
- ✅ Require a pull request before merging
- ✅ Require approvals (1 approval required)
- ✅ Require review from code owners
- ✅ Dismiss stale reviews when new commits are pushed
- ✅ Allow force pushes: **NO**
- ✅ Allow deletions: **NO**
- ✅ Require status checks to pass before merging
  - 添加 `ci/check` 状态检查

## 预期效果

配置完成后：
- ❌ 不允许直接推送到这两个分支
- ❌ 不允许 force push
- ❌ 不允许删除分支
- ✅ 必须创建 PR 才能合并
- ✅ 必须有 1 位审核人员批准才能合并

## 替代方案

如果无法访问仓库设置，可以联系仓库管理员进行配置。

## 相关文档

- [GitHub 文档: 管理分支保护规则](https://docs.github.com/en/github/administering-a-repository/managing-protected-branches)
