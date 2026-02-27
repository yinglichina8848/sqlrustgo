# RC 分支保护规则设置指南

## 1. 手动创建 PR 步骤

1. **打开 GitHub 仓库**：
   https://github.com/minzuuniversity/sqlrustgo

2. **点击 "Pull requests" 标签**

3. **点击 "New pull request" 按钮**

4. **设置分支**：
   - **base**: `rc/v1.0.0-1`
   - **compare**: `feature/2.0-engineering-setup-rc`

5. **填写 PR 信息**：
   - **Title**: "Setup engineering system rules for 2.0 development"
   - **Description**:
     ```
     This PR sets up the engineering system rules for 2.0 development, including:
     
     - GitHub Actions workflows for:
       - Automatic RC branch creation
       - Release promotion
       - RC branch protection
       - Release notes generation
       - Merge queue
       - Matrix testing
     
     - Dependency management configuration:
       - Dependabot setup
       - Renovate configuration
     
     - Documentation:
       - Architecture evolution from 1.0 to 4.0
       - Release normalization process
       - Engineering automation system design
     ```

6. **点击 "Create pull request" 按钮**

## 2. RC 分支保护规则设置

### 2.1 通过 GitHub UI 设置

1. **打开仓库设置**：
   https://github.com/minzuuniversity/sqlrustgo/settings/branches

2. **点击 "Add branch protection rule"**

3. **设置分支规则**：
   - **Branch name pattern**: `rc/*`

4. **启用保护选项**：
   - ✅ **Require a pull request before merging**
   - ✅ **Require approvals** (至少 1 个批准)
   - ✅ **Require status checks to pass before merging**
   - ✅ **Require branches to be up to date before merging**
   - ✅ **Restrict who can push to matching branches** (可选)
   - ✅ **Do not allow bypassing the above settings**
   - ✅ **Do not allow force pushes**
   - ✅ **Do not allow deletions**

5. **点击 "Create" 按钮**

### 2.2 分支保护规则说明

| 规则 | 说明 | 原因 |
|------|------|------|
| **Require PR** | 禁止直接提交，必须通过 PR | 确保代码经过审核 |
| **Require approvals** | 至少 1 个非作者的批准 | 避免自我审核 |
| **Require status checks** | 确保 CI 通过 | 保证代码质量 |
| **Up to date before merging** | 确保分支基于最新代码 | 减少合并冲突 |
| **Restrict pushes** | 限制直接推送权限 | 强化分支保护 |
| **No bypass** | 不允许绕过规则 | 确保规则强制执行 |
| **No force pushes** | 禁止强制推送 | 保护分支历史 |
| **No deletions** | 禁止删除分支 | 防止意外删除 |

## 3. 后续步骤

1. **审核 PR**：团队成员审核 `feature/2.0-engineering-setup-rc` → `rc/v1.0.0-1` 的 PR

2. **合并 PR**：审核通过后合并到 rc 分支

3. **验证保护规则**：尝试直接推送代码到 rc 分支，确认规则生效

4. **通知团队**：告知团队所有对 rc 分支的更改都必须通过 PR 提交

## 4. 版本管理流程

### 4.1 分支流转

```
feature/* → rc/* → release/* → main
```

### 4.2 注意事项

- **禁止直接提交**：任何对 rc 分支的更改都必须通过 PR
- **审核要求**：PR 必须至少有 1 个非作者的批准
- **CI 要求**：所有状态检查必须通过
- **版本推进**：使用自动化工作流从 rc 推进到 release

通过这些设置，我们可以确保 rc 分支的代码质量和稳定性，为 2.0 版本的开发做好准备。