# 企业级权限验证清单

## 🎯 验证目标

确保所有权限规则和分支保护措施正确生效，防止任何绕过行为。

## 🔍 验证方法

### 测试 1：绕过 main 分支保护

**步骤：**
1. `git checkout main`
2. `git commit --allow-empty -m "test bypass"`
3. `git push origin main`

**预期结果：**
- 被拒绝，提示需要通过 PR

### 测试 2：force push 保护

**步骤：**
1. `git checkout main`
2. `git commit --allow-empty -m "test force push"`
3. `git push --force origin main`

**预期结果：**
- 被拒绝，提示 force push 被禁用

### 测试 3：删除 tag 保护

**步骤：**
1. `git tag v1.0.0-test`
2. `git push origin v1.0.0-test`
3. `git tag -d v1.0.0-test`
4. `git push origin :refs/tags/v1.0.0-test`

**预期结果：**
- 被拒绝，提示 tag 删除被禁用

### 测试 4：Owner 账号绕过测试

**步骤：**
1. 使用 Owner 账号登录
2. 重复测试 1-3

**预期结果：**
- 所有操作均被拒绝（Include administrators 生效）

### 测试 5：GitHub UI 直接编辑测试

**步骤：**
1. 登录 GitHub
2. 尝试直接编辑 main 分支的文件

**预期结果：**
- 不允许直接 commit，强制创建 PR

### 测试 6：PR 绕过测试

**步骤：**
1. 创建 PR 到 main 分支
2. 不等待 CI 完成，尝试 merge

**预期结果：**
- 被拒绝，提示需要 CI 检查通过

### 测试 7：PR 审批绕过测试

**步骤：**
1. 创建 PR 到 main 分支
2. CI 完成后，不等待审批，尝试 merge

**预期结果：**
- 被拒绝，提示需要审批

### 测试 8：发布流程验证

**步骤：**
1. 手动创建 tag `v1.0.1-test`
2. `git push origin v1.0.1-test`

**预期结果：**
- 触发 release-validation workflow
- 如果 CI 失败，release 不会创建

### 测试 9：分支删除保护

**步骤：**
1. 在 GitHub UI 尝试删除 main 分支

**预期结果：**
- 被拒绝，提示分支删除被禁用

### 测试 10：签名提交验证

**步骤：**
1. 创建未签名的 commit
2. 尝试 push 到 main 分支

**预期结果：**
- 被拒绝，提示需要签名提交

## 🧨 不可绕过验证流程

### 场景 A：试图篡改已发布版本

**步骤：**
1. 切换到 main 分支
2. 修改 README.md
3. 尝试强推
4. 尝试删除现有 tag
5. 尝试重建相同 tag

**预期结果：**
- 全部失败

### 场景 B：绕过 PR 流程

**步骤：**
1. 在 GitHub UI 编辑 main 文件
2. 尝试直接提交

**预期结果：**
- 不允许直接提交，强制创建 PR

### 场景 C：篡改 release artifact

**步骤：**
1. 手动编辑 GitHub Release
2. 尝试修改发布内容

**预期结果：**
- 不能在未通过 CI 情况下修改发布

## 📊 验证结果记录

| 测试项 | 预期结果 | 实际结果 | 状态 | 备注 |
|--------|----------|----------|------|------|
| 绕过 main 分支 | 被拒绝 | | | |
| force push 保护 | 被拒绝 | | | |
| 删除 tag 保护 | 被拒绝 | | | |
| Owner 绕过测试 | 被拒绝 | | | |
| GitHub UI 直接编辑 | 强制 PR | | | |
| PR CI 绕过测试 | 被拒绝 | | | |
| PR 审批绕过测试 | 被拒绝 | | | |
| 发布流程验证 | CI 触发 | | | |
| 分支删除保护 | 被拒绝 | | | |
| 签名提交验证 | 被拒绝 | | | |

## 🎯 100% 成熟度判定标准

必须满足以下所有条件：

| 条件 | 必须 | 状态 |
|------|------|------|
| Owner 不能直接 push main | ✅ | |
| Owner 不能删除 tag | ✅ | |
| 发布必须 CI 触发 | ✅ | |
| 所有 commit 必须 PR | ✅ | |
| main 只能 fast-forward | ✅ | |
| release 分支不可变 | ✅ | |
| 构建产物带 Hash | ✅ | |

## 🔒 失败处理

如果任何测试失败：

1. 检查 GitHub 分支保护规则配置
2. 确保 "Include administrators" 已开启
3. 确保所有必要的保护选项已启用
4. 重新运行测试

## 📋 定期验证

建议：
- 每次修改权限规则后进行验证
- 每月进行一次完整验证
- 每次发布前进行验证

## 🚨 紧急情况处理

如果需要紧急绕过规则：

1. 仅限 Owner 账号操作
2. 必须在 GitHub UI 中临时禁用特定规则
3. 操作完成后立即重新启用
4. 记录操作原因和时间

## 🔍 常见问题排查

### 问题：规则仍可被绕过

**可能原因：**
- "Include administrators" 未开启
- 规则配置不完整
- GitHub API 缓存延迟

**解决方案：**
- 检查并启用 "Include administrators"
- 重新配置所有必要规则
- 等待 5-10 分钟后重试

### 问题：CI 不触发

**可能原因：**
- workflow 配置错误
- 权限不足
- GitHub Actions 服务问题

**解决方案：**
- 检查 workflow 文件配置
- 确保 CI Bot 权限正确
- 查看 GitHub Actions 状态页面