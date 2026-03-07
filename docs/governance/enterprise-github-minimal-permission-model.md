# 最小权限 GitHub 组织模型

## 🎯 目标

- **Owner 不参与日常开发**
- **Developer 无法绕过规则**
- **发布只能通过 CI**
- **AI 无法利用高权限账号**

## 🧱 账号与角色分层

### 👑 A. Owner（冷账户）

**用途：**
- 修改仓库设置
- 配置规则
- 紧急恢复

**限制：**
- 不提交代码
- 不创建 tag
- 不发布 release
- 不参与 PR

**操作建议：**
- 开启 2FA
- 不在本地 git 配置该账号
- 不在 IDE 登录
- 日常保持登出

### 👨‍💻 B. Maintainer（日常使用）

**权限：**
- 可以 merge PR
- 不能绕过 branch protection
- 不能删除 tag
- 不能修改保护规则

**必须：**
- 所有提交通过 PR
- 无直接 push main 权限

### 👩‍💻 C. Developer（未来扩展）

- 只能提交 PR
- 不能 merge

### 🤖 D. CI Bot（可选高级）

**权限：**
- 仅用于创建 tag / release
- 无 UI 登录

## 🔒 企业级规则配置矩阵

### main 分支

**必须开启：**
- 需要拉取请求
- 需要批准（≥1）
- 需要状态检查
- 需要对话解决
- 需要签名提交
- 需要线性历史
- 包括管理员
- 禁用强制推送
- 禁用删除

### rc/*

同 main。

＃＃＃ 发布/*

同 main。

### 🏷 Tag 保护

**规则：**
- v*

**必须：**
- 禁止删除
- 禁止重建
- 包括管理员

## 🛡️ 权限边界

| 操作 | Owner |维护者|开发商| CI Bot |
|------|-------|------------|-----------|--------|
| 修改仓库设置 | ✅ | ❌ | ❌ | ❌ |
| 修改权限 | ✅ | ❌ | ❌ | ❌ |
| 配置分支保护 | ✅ | ❌ | ❌ | ❌ |
|直接 push main| ❌ | ❌ | ❌ | ❌ |
|合并公关| ✅ | ✅ | ❌ | ❌ |
| 提交 PR | ✅ | ✅ | ✅ | ❌ |
| 创建 tag | ❌ | ❌ | ❌ | ✅ |
|发布 release| ❌ | ❌ | ❌ | ✅ |
| 删除 tag | ❌ | ❌ | ❌ | ❌ |

## 📋 身份迁移步骤

### 1. 创建新的 Maintainer 账号

1. 在 GitHub 上创建新账号
2. 邀请到仓库
3. 分配 Maintainer 权限

### 2. 配置本地开发环境

```bash
# 配置新账号
git config --global user.name "Maintainer Name"
git config --global user.email "maintainer@example.com"

# 生成新的 SSH 密钥
ssh-keygen -t ed25519 -C "maintainer@example.com"

# 添加到 ssh-agent
ssh-add ~/.ssh/id_ed25519

# 添加到 GitHub
# 复制 ~/.ssh/id_ed25519.pub 内容到 GitHub 设置
```

### 3. Owner 账号进入冷冻模式

1. 退出所有浏览器登录
2. 移除本地 git 配置
3. 仅在需要时登录

## 🚨 安全最佳实践

### 防止 AI 利用高权限账号

1. **不要**在 IDE 中登录 Owner 账号
2. **不要**在浏览器中保持 Owner 账号登录
3. **不要**将 Owner 账号的凭证存储在任何工具中
4. **不要**使用 Owner 账号运行任何自动化脚本

### 防止权限提升

1. 定期审查权限配置
2. 使用 GitHub Audit Log 监控权限变更
3. 实施最小权限原则
4. 定期轮换凭证

## 📊 成熟度评估

### 100% 成熟度标准

- [ ] Owner 不能直接 push main
- [ ] Owner 不能删除 tag
- [ ] 发布必须 CI 触发
- [ ] 所有 commit 必须 PR
- [ ] main 只能 fast-forward
- [ ] release 分支不可变
- [ ] 构建产物带 Hash

## 🔍 验证方法

参见 `enterprise-permission-validation-checklist.md` 中的详细验证步骤。