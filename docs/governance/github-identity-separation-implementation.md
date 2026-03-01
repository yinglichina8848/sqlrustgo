# GitHub 身份拆分实施方案

## 🎯 目标

将当前高权限账号拆分为多个职责明确的账号，实现最小权限原则，防止AI或工具利用高权限账号绕过规则。

## 🧱 身份拆分模型

| 身份 | 类型 | 权限级别 | 用途 |
|------|------|----------|------|
| Owner | 冷账户 | 最高 | 仅用于配置和紧急操作 |
| Maintainer | 日常使用 | 中等 | 开发和PR合并 |
| Developer | 未来扩展 | 低 | 仅提交PR |
| CI Bot | 自动化 | 特殊 | 仅用于CI/CD操作 |

## 🔄 实施步骤

### 1. 准备工作

**前置条件：**
- 现有 GitHub 账号（Owner）：`yinglichina8848`
- 新 GitHub 账号（Maintainer）：待创建

**工具准备：**
- Git 客户端
- SSH 密钥生成工具
- 文本编辑器

### 2. 创建新的 Maintainer 账号

**步骤：**
1. 访问 [GitHub 注册页面](https://github.com/join)
2. 创建新账号（例如：`yinglichina-dev`）
3. 验证邮箱
4. 开启 2FA 认证

### 3. 邀请新账号到仓库

**步骤：**
1. 使用 Owner 账号登录 GitHub
2. 进入仓库设置 → 协作者和团队
3. 邀请新账号 `yinglichina-dev`
4. 分配 **Maintainer** 权限
5. 接受邀请（新账号邮箱）

### 4. 配置新账号的本地环境

**步骤：**

#### 4.1 生成 SSH 密钥

```bash
# 为新账号生成 SSH 密钥
ssh-keygen -t ed25519 -C "maintainer@example.com" -f ~/.ssh/id_ed25519_maintainer

# 添加到 ssh-agent
ssh-add ~/.ssh/id_ed25519_maintainer

# 查看公钥内容（需要复制到 GitHub）
cat ~/.ssh/id_ed25519_maintainer.pub
```

#### 4.2 添加 SSH 密钥到 GitHub

1. 使用新账号登录 GitHub
2. 进入 settings → SSH and GPG keys
3. 点击 "New SSH key"
4. 粘贴公钥内容
5. 保存

#### 4.3 配置 Git 多账号

**创建或编辑 ~/.ssh/config 文件：**

```ssh-config
# Owner 账号
Host github.com-owner
  HostName github.com
  User git
  IdentityFile ~/.ssh/id_ed25519_owner

# Maintainer 账号
Host github.com-maintainer
  HostName github.com
  User git
  IdentityFile ~/.ssh/id_ed25519_maintainer
```

**配置本地仓库：**

```bash
# 进入仓库目录
cd /path/to/sqlrustgo

# 查看当前配置
git config --list

# 配置仓库级别的用户信息（使用 Maintainer 账号）
git config user.name "Maintainer Name"
git config user.email "maintainer@example.com"

# 更新远程 URL 使用新的 SSH 主机
# 查看当前远程 URL
git remote -v

# 更新为 Maintainer 账号的 SSH URL
git remote set-url origin git@github.com-maintainer:minzuuniversity/sqlrustgo.git
```

### 5. Owner 账号冷冻操作

**步骤：**
1. 移除本地 Git 配置中的 Owner 信息
2. 退出所有浏览器中的 Owner 账号登录
3. 不在 IDE 中登录 Owner 账号
4. 仅在需要时使用 Incognito 模式登录 Owner 账号

### 6. CI Bot 配置（可选）

**步骤：**
1. 创建新的 GitHub 账号（例如：`sqlrustgo-ci`）
2. 生成机器专用 SSH 密钥
3. 添加为仓库协作者，分配 **Maintainer** 权限
4. 在 GitHub Actions secrets 中配置该账号的凭证
5. 更新 CI/CD 配置使用该账号

## 📋 配置文件模板

### SSH 配置模板

```ssh-config
# GitHub 多账号配置

# Owner 账号（冷账户）
Host github.com-owner
  HostName github.com
  User git
  IdentityFile ~/.ssh/id_ed25519_owner
  IdentitiesOnly yes

# Maintainer 账号（日常使用）
Host github.com-maintainer
  HostName github.com
  User git
  IdentityFile ~/.ssh/id_ed25519_maintainer
  IdentitiesOnly yes

# CI Bot 账号（自动化）
Host github.com-ci
  HostName github.com
  User git
  IdentityFile ~/.ssh/id_ed25519_ci
  IdentitiesOnly yes
```

### Git 配置模板

**全局配置（默认使用 Maintainer 账号）：**

```bash
git config --global user.name "Maintainer Name"
git config --global user.email "maintainer@example.com"
```

**仓库级配置（如需覆盖）：**

```bash
# 在特定仓库中
cd /path/to/repo
git config user.name "Specific User"
git config user.email "specific@example.com"
```

## 🚨 迁移注意事项

### 1. PR 和 Issue 处理

- **PR 作者**：新的提交将显示为 Maintainer 账号
- **Issue 评论**：使用对应账号登录评论
- **审批操作**：使用 Maintainer 账号进行 PR 审批

### 2. 凭证管理

- **不要**在任何工具中存储 Owner 账号的凭证
- **不要**在浏览器中保持 Owner 账号登录
- **不要**使用 Owner 账号运行自动化脚本
- **定期**轮换所有账号的 SSH 密钥

### 3. 权限验证

- 迁移完成后运行权限验证脚本
- 确认 Maintainer 账号无法绕过规则
- 确认 Owner 账号操作受到限制

### 4. 紧急情况处理

**如果需要紧急使用 Owner 权限：**
1. 使用 Incognito 模式登录 Owner 账号
2. 执行必要操作
3. 立即退出登录
4. 记录操作原因和时间

## 🔍 验证方法

### 验证 1：Maintainer 账号权限

**步骤：**
1. 使用 Maintainer 账号尝试直接 push main
2. 预期：被拒绝

### 验证 2：Owner 账号冷冻状态

**步骤：**
1. 检查本地 Git 配置
2. 检查浏览器登录状态
3. 确认 IDE 中无 Owner 账号登录

### 验证 3：权限边界

**步骤：**
1. 运行权限验证脚本
2. 确认所有测试通过
3. 确认规则无法被绕过

## 📊 迁移进度跟踪

| 步骤 | 状态 | 完成时间 | 负责人 |
|------|------|----------|--------|
| 创建 Maintainer 账号 | | | |
| 邀请到仓库 | | | |
| 配置 SSH 密钥 | | | |
| 更新本地配置 | | | |
| Owner 账号冷冻 | | | |
| 权限验证 | | | |
| CI Bot 配置 | | | |

## 🎯 成功标准

- [ ] Maintainer 账号可正常提交 PR
- [ ] Maintainer 账号无法直接 push main
- [ ] Owner 账号已从本地环境移除
- [ ] 所有权限验证测试通过
- [ ] CI/CD 流程正常运行

## 🔒 安全最佳实践

1. **定期审查**：每月审查账号权限配置
2. **密钥轮换**：每季度轮换 SSH 密钥
3. **审计日志**：启用 GitHub Audit Log
4. **访问控制**：遵循最小权限原则
5. **培训**：团队成员了解身份使用规范

## 📚 参考文档

- [GitHub 分支保护规则文档](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/defining-the-mergeability-of-pull-requests/about-protected-branches)
- [GitHub SSH 密钥配置](https://docs.github.com/en/authentication/connecting-to-github-with-ssh)
- [企业级权限验证清单](enterprise-permission-validation-checklist.md)
- [最小权限 GitHub 组织模型](enterprise-github-minimal-permission-model.md)