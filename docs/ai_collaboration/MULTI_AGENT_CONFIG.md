# 多 Agent 配置说明

> **版本**: 1.0  
> **制定日期**: 2026-03-04  
> **适用范围**: SQLRustGo 项目  
> **文档类型**: 配置规范

---

## 目录

1. [概述](#一概述)
2. [环境准备](#二环境准备)
3. [目录结构配置](#三目录结构配置)
4. [Git 配置](#四git-配置)
5. [GitHub Token 配置](#五github-token-配置)
6. [GPG 签名配置](#六gpg-签名配置)
7. [身份切换脚本](#七身份切换脚本)
8. [验证与测试](#八验证与测试)
9. [常见问题](#九常见问题)

---

## 一、概述

### 1.1 配置目标

本文档描述如何配置单机 4 AI Agent 多身份隔离开发环境。

### 1.2 配置清单

| 配置项 | 说明 |
|--------|------|
| 目录结构 | 4 个独立工作目录 |
| Git 配置 |4 套独立 local config|
|GitHub 令牌| 4 个独立 PAT |
| GPG 签名 | 4 套独立签名密钥 |

### 1.3 架构图

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          多 Agent 配置架构                                    │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   /Users/liying/workspace                                                   │
│   │                                                                          │
│   ├── dev/                          # 开发目录                              │
│   │   ├── heartopen/                # heartopen 环境                        │
│   │   │   └── sqlrustgo/            # 项目目录                              │
│   │   │       └── .git/config       # Git 配置                              │
│   │   └── openheart/                # openheart 环境                        │
│   │       └── sqlrustgo/                                                     │
│   │                                                                          │
│   ├── maintainer/                   # 审核目录                              │
│   │   └── sqlrustgo/                                                         │
│   │                                                                          │
│   ├── yinglichina/                  # 发布目录                              │
│   │   └── sqlrustgo/                                                         │
│   │                                                                          │
│   └── identities/                   # 身份材料目录                          │
│       ├── heartopen/                                                         │
│       │   ├── PAT.txt               # GitHub Token                          │
│       │   └── gpg.key               # GPG 密钥                              │
│       ├── openheart/                                                         │
│       ├── maintainer/                                                        │
│       └── yinglichina8848/                                                   │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 二、环境准备

### 2.1 系统要求

| 要求 | 说明 |
|------|------|
| 操作系统 |macOS/Linux|
| Git | ≥ 2.30 |
|GitHub CLI| ≥ 2.0 |
| GPG | ≥ 2.2 |

### 2.2 安装依赖

```bash
# macOS
brew install git gh gnupg

# Linux (Ubuntu/Debian)
sudo apt install git gh gnupg

# Linux (CentOS/RHEL)
sudo yum install git gh gnupg2
```

### 2.3 验证安装

```bash
git --version
gh --version
gpg --version
```

---

## 三、目录结构配置

### 3.1 创建目录结构

```bash
# 创建基础目录
mkdir -p ~/workspace/dev/heartopen/sqlrustgo
mkdir -p ~/workspace/dev/openheart/sqlrustgo
mkdir -p ~/workspace/maintainer/sqlrustgo
mkdir -p ~/workspace/yinglichina/sqlrustgo
mkdir -p ~/workspace/identities/{heartopen,openheart,maintainer,yinglichina8848}

# 克隆项目到各目录
cd ~/workspace/dev/heartopen && git clone https://github.com/org/sqlrustgo.git
cd ~/workspace/dev/openheart && git clone https://github.com/org/sqlrustgo.git
cd ~/workspace/maintainer && git clone https://github.com/org/sqlrustgo.git
cd ~/workspace/yinglichina && git clone https://github.com/org/sqlrustgo.git
```

### 3.2 目录权限设置

```bash
# 设置 identities 目录权限
chmod 700 ~/workspace/identities
chmod 700 ~/workspace/identities/*
chmod 600 ~/workspace/identities/*/PAT.txt
chmod 600 ~/workspace/identities/*/gpg.key
```

---

## 四、Git 配置

### 4.1 heartopen Git 配置

```bash
cd ~/workspace/dev/heartopen/sqlrustgo

# 配置用户信息
git config user.name "heartopen"
git config user.email "heartopen@guizhouminzuuniversity.edu.cn"

# 配置 GPG 签名
git config user.signingkey <HEARTOPEN_GPG_KEY_ID>
git config commit.gpgsign true

# 配置默认分支
git config init.defaultBranch main

# 验证配置
git config --local --list | grep user
```

### 4.2 openheart Git部署

```bash
cd ~/workspace/dev/openheart/sqlrustgo

# 配置用户信息
git config user.name "openheart"
git config user.email "openheart@guizhouminzuuniversity.edu.cn"

# 配置 GPG 签名
git config user.signingkey <OPENHEART_GPG_KEY_ID>
git config commit.gpgsign true

# 验证配置
git config --local --list | grep user
```

### 4.3 maintainer Git 配置

```bash
cd ~/workspace/maintainer/sqlrustgo

# 配置用户信息
git config user.name "maintainer"
git config user.email "maintainer@guizhouminzuuniversity.edu.cn"

# 配置 GPG 签名
git config user.signingkey <MAINTAINER_GPG_KEY_ID>
git config commit.gpgsign true

# 验证配置
git config --local --list | grep user
```

### 4.4 yinglichina8848 Git 配置

```bash
cd ~/workspace/yinglichina/sqlrustgo

# 配置用户信息
git config user.name "yinglichina8848"
git config user.email "yinglichina8848@guizhouminzuuniversity.edu.cn"

# 配置 GPG 签名
git config user.signingkey <YINGLICHINA_GPG_KEY_ID>
git config commit.gpgsign true

# 验证配置
git config --local --list | grep user
```

### 4.5 禁止全局配置

```bash
# 检查是否有全局配置
git config --global --get user.name

# 如果有，删除全局配置
git config --global --unset user.name
git config --global --unset user.email
```

---

## 五、GitHub Token 配置

### 5.1 创建 PAT

1.登录GitHub→设置→开发者设置→个人访问令牌→令牌（经典）
2. 点击 "Generate new token (classic)"
3. 设置 Token 名称和权限：
   - `repo` (完整仓库访问)
- `workflow`（GitHub 操作）
- `write:packages` (包发布)
4. 生成并保存 Token

### 5.2 存储 PAT

```bash
# 存储 heartopen PAT
echo "ghp_xxxxxxxxxxxxxxxxxxxx" > ~/workspace/identities/heartopen/PAT.txt
chmod 600 ~/workspace/identities/heartopen/PAT.txt

# 存储 openheart PAT
echo "ghp_yyyyyyyyyyyyyyyyyyyy" > ~/workspace/identities/openheart/PAT.txt
chmod 600 ~/workspace/identities/openheart/PAT.txt

# 存储 maintainer PAT
echo "ghp_zzzzzzzzzzzzzzzzzzzz" > ~/workspace/identities/maintainer/PAT.txt
chmod 600 ~/workspace/identities/maintainer/PAT.txt

# 存储 yinglichina8848 PAT
echo "ghp_wwwwwwwwwwwwwwwwwwwww" > ~/workspace/identities/yinglichina8848/PAT.txt
chmod 600 ~/workspace/identities/yinglichina8848/PAT.txt
```

### 5.3 使用 PAT

```bash
# 设置环境变量
export GITHUB_TOKEN=$(cat ~/workspace/identities/heartopen/PAT.txt)

# 验证身份
gh api user --jq .login
```

### 5.4 禁止 gh auth login

```bash
# ❌ 禁止使用
gh auth login

# 如果已经登录，退出
gh auth logout
```

---

## 六、GPG 签名配置

### 6.1 生成 GPG 密钥

```bash
# 为每个身份生成 GPG 密钥
gpg --full-generate-key

# 选择：
# (1) RSA and RSA
# 密钥长度：4096
# 过期时间：0 (永不过期)
# 姓名：heartopen
# 邮箱：heartopen@guizhouminzuuniversity.edu.cn
```

### 6.2 查看 GPG 密钥

```bash
# 列出密钥
gpg --list-secret-keys --keyid-format=long

# 输出示例：
# sec   rsa4096/3AA5C34371567BD2 2026-03-04 [SC]
# uid                 [ultimate] heartopen <heartopen@guizhouminzuuniversity.edu.cn>
# ssb   rsa4096/B42B068F7B2766A4 2026-03-04 [E]
```

### 6.3 导出公钥

```bash
# 导出公钥
gpg --armor --export 3AA5C34371567BD2 > ~/workspace/identities/heartopen/gpg.key

# 上传到 GitHub
# Settings → SSH and GPG keys → New GPG key
```

### 6.4 配置 Git 使用 GPG

```bash
# 在项目目录配置
cd ~/workspace/dev/heartopen/sqlrustgo
git config user.signingkey 3AA5C34371567BD2
git config commit.gpgsign true

# 配置 GPG 程序路径 (macOS)
git config gpg.program /usr/local/bin/gpg
```

### 6.5 测试 GPG 签名

```bash
# 测试签名
echo "test" | gpg --clearsign

# 测试 Git 签名提交
git commit --allow-empty -m "test gpg signing"
git log --show-signature
```

---

## 七、身份切换脚本

### 7.1 切换脚本

```bash
#!/bin/bash
# ~/workspace/switch-identity.sh

set -e

IDENTITY=$1

if [ -z "$IDENTITY" ]; then
    echo "Usage: switch-identity.sh <identity>"
    echo "Identities: heartopen, openheart, maintainer, yinglichina8848"
    exit 1
fi

case $IDENTITY in
    heartopen)
        WORK_DIR="$HOME/workspace/dev/heartopen/sqlrustgo"
        ;;
    openheart)
        WORK_DIR="$HOME/workspace/dev/openheart/sqlrustgo"
        ;;
    maintainer)
        WORK_DIR="$HOME/workspace/maintainer/sqlrustgo"
        ;;
    yinglichina8848)
        WORK_DIR="$HOME/workspace/yinglichina/sqlrustgo"
        ;;
    *)
        echo "Unknown identity: $IDENTITY"
        exit 1
        ;;
esac

TOKEN_FILE="$HOME/workspace/identities/$IDENTITY/PAT.txt"

if [ ! -f "$TOKEN_FILE" ]; then
    echo "Token file not found: $TOKEN_FILE"
    exit 1
fi

# 设置 GITHUB_TOKEN
export GITHUB_TOKEN=$(cat "$TOKEN_FILE")

# 切换目录
cd "$WORK_DIR"

# 验证身份
echo "=========================================="
echo "Identity: $(git config user.name)"
echo "Email: $(git config user.email)"
echo "GitHub: $(gh api user --jq .login)"
echo "Directory: $(pwd)"
echo "=========================================="

# 启动新 shell
exec $SHELL
```

### 7.2 使用切换脚本

```bash
# 添加执行权限
chmod +x ~/workspace/switch-identity.sh

# 切换到 heartopen
source ~/workspace/switch-identity.sh heartopen

# 切换到 maintainer
source ~/workspace/switch-identity.sh maintainer
```

### 7.3 快捷别名

```bash
# 添加到 ~/.zshrc 或 ~/.bashrc
alias heartopen='source ~/workspace/switch-identity.sh heartopen'
alias openheart='source ~/workspace/switch-identity.sh openheart'
alias maintainer='source ~/workspace/switch-identity.sh maintainer'
alias yinglichina='source ~/workspace/switch-identity.sh yinglichina8848'
```

---

## 八、验证与测试

### 8.1 验证清单

```bash
#!/bin/bash
# verify-config.sh

echo "=== 验证多 Agent 配置 ==="

# 验证目录结构
echo "1. 验证目录结构..."
for dir in heartopen openheart maintainer yinglichina8848; do
    if [ -d "$HOME/workspace/identities/$dir" ]; then
        echo "  ✅ $dir 目录存在"
    else
        echo "  ❌ $dir 目录不存在"
    fi
done

# 验证 PAT 文件
echo "2. 验证 PAT 文件..."
for identity in heartopen openheart maintainer yinglichina8848; do
    if [ -f "$HOME/workspace/identities/$identity/PAT.txt" ]; then
        echo "  ✅ $identity PAT 文件存在"
    else
        echo "  ❌ $identity PAT 文件不存在"
    fi
done

# 验证 Git 配置
echo "3. 验证 Git 配置..."
for dir in dev/heartopen/sqlrustgo dev/openheart/sqlrustgo maintainer/sqlrustgo yinglichina/sqlrustgo; do
    if [ -d "$HOME/workspace/$dir" ]; then
        cd "$HOME/workspace/$dir"
        user=$(git config user.name)
        email=$(git config user.email)
        echo "  ✅ $dir: $user <$email>"
    fi
done

# 验证 GitHub 身份
echo "4. 验证 GitHub 身份..."
for identity in heartopen openheart maintainer yinglichina8848; do
    export GITHUB_TOKEN=$(cat "$HOME/workspace/identities/$identity/PAT.txt" 2>/dev/null)
    if [ -n "$GITHUB_TOKEN" ]; then
        login=$(gh api user --jq .login 2>/dev/null)
        if [ -n "$login" ]; then
            echo "  ✅ $identity: GitHub login = $login"
        else
            echo "  ❌ $identity: GitHub 验证失败"
        fi
    fi
done

echo "=== 验证完成 ==="
```

### 8.2 测试提交流程

```bash
# 切换到 heartopen
source ~/workspace/switch-identity.sh heartopen

# 创建测试分支
git checkout -b test/multi-agent-config

# 创建测试提交
echo "test" > test.txt
git add test.txt
git commit -m "test: multi-agent config test"

# 验证提交签名
git log --show-signature -1

# 清理测试
git checkout main
git branch -D test/multi-agent-config
```

---

## 九、常见问题

### 9.1 GPG 签名失败

**问题**: `gpg: signing failed: Inappropriate ioctl for device`

**解决方案**:
```bash
# 添加到 ~/.zshrc 或 ~/.bashrc
export GPG_TTY=$(tty)
```

### 9.2 Token 权限不足

**问题**: `gh: Permission denied`

**解决方案**:
```bash
# 检查 Token 权限
gh api user --jq '.permissions'

# 确保 Token 有以下权限：
# - repo
# - workflow
# - write:packages
```

### 9.3 身份混淆

**问题**: 提交使用了错误的身份

**解决方案**:
```bash
# 检查当前身份
git config user.name
gh api user --jq .login

# 修正最后一次提交
git commit --amend --reset-author --no-edit
```

### 9.4 目录权限问题

**问题**: `Permission denied: identities/`

**解决方案**:
```bash
# 修复权限
chmod 700 ~/workspace/identities
chmod 700 ~/workspace/identities/*
chmod 600 ~/workspace/identities/*/PAT.txt
```

---

## 附录

### A. 配置文件模板

#### .git/config 模板

```ini
[core]
    repositoryformatversion = 0
    filemode = true
    bare = false
    logallrefupdates = true
[user]
    name = heartopen
    email = heartopen@guizhouminzuuniversity.edu.cn
    signingkey = 3AA5C34371567BD2
[commit]
    gpgsign = true
[gpg]
    program = /usr/local/bin/gpg
```

### B. 相关文档

| 文档 | 路径 | 说明 |
|------|------|------|
|AI Agent 提示词| [AI_AGENT_PROMPTS.md](./AI_AGENT_PROMPTS.md) | 4 Agent 提示词体系 |
| 多身份隔离开发模式 | [../MULTI_IDENTITY_DEVELOPMENT_MODEL.md](../MULTI_IDENTITY_DEVELOPMENT_MODEL.md) | 完整配置规范 |
| 权限模型 | [../GIT_PERMISSION_MODEL.md](../GIT_PERMISSION_MODEL.md) | 2.0 权限模型 |

### C. 变更历史

| 版本 | 日期 | 说明 |
|------|------|------|
| 1.0 | 2026-03-04 | 初始版本 |

---

*本文档由 yinglichina8848 制定*
