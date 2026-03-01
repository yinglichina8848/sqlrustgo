# GitHub 多账号配置指南

## 概述

本文档记录了如何在单机环境下配置多个 GitHub 账号，实现身份隔离和自动化协作开发。

## 系统架构

### 硬件环境
- MacMini（主力）
- MacBookPro（移动）
- Ubuntu（服务器）

### 账号规划
| 账号 | 角色 | 权限 | 用途 |
|------|------|------|------|
| yinglichina8848 | Owner（人类） | Admin | main 分支控制、最终发布 |
| sonaopenheart | AI Dev A | Write | 开发、创建 PR |
| sonaheartopen | AI Review B | Write | Review、合并 develop |

### AI-CLI 工具分配
| 身份 | 主要工具 | 备注 |
|------|----------|------|
| yinglichina8848 | Codex CLI + Claude Code | 最强交互能力，适合复杂设计 |
| sonaopenheart | OpenCode + Claude Code | 自动生成代码、写 feature |
| sonaheartopen | Codex CLI + OpenCode | 自动 review + QA 检查 |

## 核心原则

### 不要"人切换身份"，要"流程切换身份"

GitHub 不允许自审自合，本质约束是：
- 同一个 PR 不能由同一个账号完成创建 + 批准

但 GitHub 没有限制：
- 同一台机器使用多个身份
- 自动化工具代替账号操作
- 多系统使用不同 token

解决方案：
- 每个系统预置两个 Git 身份 + 两套 GitHub Token
- 目录即身份，每个身份绑定独立工作空间

## 配置步骤

### 第一阶段：生成 SSH Key

```bash
# 为 sonaopenheart 生成 key
ssh-keygen -t ed25519 -C "sonaopenheart@ai" -f ~/.ssh/id_ed25519_openheart

# 为 sonaheartopen 生成 key
ssh-keygen -t ed25519 -C "sonaheartopen@ai" -f ~/.ssh/id_ed25519_heartopen

# 为 yinglichina8848 生成 key（如果还没有）
ssh-keygen -t ed25519 -C "yinglichina@main" -f ~/.ssh/id_ed25519
```

### 第二阶段：上传公钥到 GitHub

登录对应 GitHub 账号：
- Settings → SSH and GPG Keys → New SSH key
- 分别添加对应的公钥

### 第三阶段：配置 SSH config

编辑 `~/.ssh/config`：

```
Host github-openheart
  HostName github.com
  User git
  IdentityFile ~/.ssh/id_ed25519_openheart

Host github-heartopen
  HostName github.com
  User git
  IdentityFile ~/.ssh/id_ed25519_heartopen

Host github-yingli
  HostName github.com
  User git
  IdentityFile ~/.ssh/id_ed25519
```

测试连接：

```bash
ssh -T git@github-openheart
ssh -T git@github-heartopen
ssh -T git@github-yingli
```

### 第四阶段：创建工作目录

```bash
mkdir -p ~/workspace/openheart
mkdir -p ~/workspace/heartopen
mkdir -p ~/workspace/yinglichina
```

### 第五阶段：Clone 仓库

```bash
# openheart（开发账号）
cd ~/workspace/openheart
git clone git@github-openheart:ORG/sqlrustgo.git
cd sqlrustgo
git config user.name "sonaopenheart"
git config user.email "sonaopenheart@example.com"

# heartopen（测试账号）
cd ~/workspace/heartopen
git clone git@github-heartopen:ORG/sqlrustgo.git
cd sqlrustgo
git config user.name "sonaheartopen"
git config user.email "sonaheartopen@example.com"

# yinglichina（人类控制）
cd ~/workspace/yinglichina
git clone git@github-yingli:ORG/sqlrustgo.git
cd sqlrustgo
git config user.name "yinglichina8848"
git config user.email "yinglichina@example.com"
```

### 第六阶段：隔离 gh CLI

在每个目录分别执行：

```bash
# 在 openheart 目录
export GH_CONFIG_DIR=./.gh
gh auth login
# 登录 sonaopenheart

# 在 heartopen 目录
export GH_CONFIG_DIR=./.gh
gh auth login
# 登录 sonaheartopen

# 在 yinglichina 目录
export GH_CONFIG_DIR=./.gh
gh auth login
# 登录 yinglichina8848
```

## 工作流程

### 开发流程（openheart）

```bash
cd ~/workspace/openheart/sqlrustgo
git checkout -b feature-x
# 编码...
git add .
git commit -m "feat: implement feature x"
git push
gh pr create --base develop --title "Feature X" --body "Description"
```

### Review 流程（heartopen）

```bash
cd ~/workspace/heartopen/sqlrustgo
git fetch
gh pr list
gh pr checkout <PR号>
# 测试...
gh pr review --approve
gh pr merge --squash
```

### Owner 流程（yinglichina）

```bash
cd ~/workspace/yinglichina/sqlrustgo
git pull
# 确认合并结果
```

## 分支保护规则

### main 分支
- Require pull request
- Require 1 approval
- Restrict who can push → 只选 yinglichina8848

### develop 分支
- Require pull request
- Require 1 approval
- 不限制 push 人

## 常见问题排查

### SSH 认证失败

1. 检查 config 是否生效：
```bash
ssh -G github-openheart
```

2. 检查权限：
```bash
chmod 700 ~/.ssh
chmod 600 ~/.ssh/id_ed25519_*
chmod 644 ~/.ssh/*.pub
chmod 600 ~/.ssh/config
```

3. 清空 SSH agent：
```bash
ssh-add -D
```

4. 直接测试 key：
```bash
ssh -i ~/.ssh/id_ed25519_openheart -T git@github.com
```

### PR 创建失败

1. 确认 remote 指向正确：
```bash
git remote -v
```

2. 确认默认分支：
```bash
git branch -r
```

3. 如果有 fork remote 干扰：
```bash
git remote remove fork
```

### gh CLI 串号

确保每次操作前设置正确的配置目录：
```bash
export GH_CONFIG_DIR=./.gh
```

或者固化到 `.env` 文件：
```bash
echo 'export GH_CONFIG_DIR=./.gh' >> .env
```

## 架构优势

1. **身份物理隔离** - 不同目录绑定不同 SSH 和 gh 配置
2. **符合 GitHub 规则** - 不同账号创建和审核 PR
3. **单系统可闭环** - 一台机器完成完整开发流程
4. **三系统可并行** - 多机器可同时开发不同分支
5. **适合自动化** - OpenClaw 只需选择目录即可切换身份

## 最终目录结构

```
~/workspace/
   openheart/
       sqlrustgo/        ← 开发（sonaopenheart）
   heartopen/
       sqlrustgo/        ← Review（sonaheartopen）
   yinglichina/
       sqlrustgo/        ← 人类控制（yinglichina8848）
```

## 参考资源

- [GitHub SSH 配置文档](https://docs.github.com/en/authentication/connecting-to-github-with-ssh)
- [GitHub CLI 文档](https://cli.github.com/manual/)
- [OpenClaw 文档](https://docs.openclaw.ai/)
