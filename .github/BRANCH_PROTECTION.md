# SQLRustGo GitHub Branch Protection 配置指南

> **版本**: 1.0
> **制定日期**: 2026-03-06
> **制定人**: yinglichina8848

---

## 一、保护策略概览

| 分支 | 保护级别 | 审核人数 | CI 要求 | 签名要求 |
|------|----------|----------|---------|----------|
| `main` | 🔴 最高 | 2 | 全部通过 | 必须 |
|__代码0__| 🔴 高 | 2 | 全部通过 | 必须 |
|__代码0__| 🟡 中等 | 1 | 编译+测试 | 推荐 |
|__代码0__| 🟡 中等 | 1 | 编译+测试 | 推荐 |
|__代码0__| 🟢 低 | 可选 | 无 | 推荐 |

---

## 二、main 分支配置

### 2.1 保护规则

```yaml
# GitHub Settings → Branches → Branch protection rules
Branch name pattern: main

☑ Require a pull request before merging
  ☑ Require approvals: 2
  ☑ Dismiss stale pull request approvals when new commits are pushed
  ☑ Require review from Code Owners
  ☑ Require conversation resolution before merging

☑ Require status checks to pass before merging
  ☑ Require branches to be up to date before merging
  Status checks:
    - build
    - test
    - lint (clippy)
    - security-scan
    - coverage

☑ Require linear history

☑ Include administrators

☑ Restrict who can push to matching branches
  (仅允许特定人员)

☑ Allow force pushes: NO

☑ Allow deletions: NO

☑ Require signed commits
```

### 2.2 GitHub CLI 配置命令

```bash
# 使用 gh cli 设置分支保护
gh api -X PUT repos/{owner}/{repo}/branches/main/protection \
  -H "Accept: application/vnd.github+json" \
  -F required_pull_request_reviews='{"required_approving_review_count":2,"dismiss_stale_reviews":true,"require_code_owner_reviews":true}' \
  -F required_status_checks='{"strict":true,"contexts":["build","test","lint","security-scan"]}' \
  -F enforce_admins=true \
  -F required_linear_history=true \
  -F allow_force_pushes=false \
  -F allow_deletions=false
```

---

## 三、release 分支配置

### 3.1 保护规则

```yaml
# GitHub Settings → Branches → Branch protection rules
Branch name pattern: release/*

☑ Require a pull request before merging
  ☑ Require approvals: 2
  ☑ Dismiss stale pull request approvals when new commits are pushed
  ☑ Require review from Code Owners

☑ Require status checks to pass before merging
  ☑ Require branches to be up to date before merging
  Status checks:
    - build
    - test
    - lint
    - security-scan

☑ Require linear history

☑ Include administrators

☑ Allow force pushes: NO

☑ Allow deletions: NO

☑ Require signed commits
```

### 3.2 GitHub CLI 配置命令

```bash
gh api -X PUT repos/{owner}/{repo}/branches/release%2F/protection \
  -H "Accept: application/vnd.github+json" \
  -F required_pull_request_reviews='{"required_approving_review_count":2,"dismiss_stale_reviews":true}' \
  -F required_status_checks='{"strict":true,"contexts":["build","test","lint","security-scan"]}' \
  -F enforce_admins=true \
  -F required_linear_history=true \
  -F allow_force_pushes=false \
  -F allow_deletions=false
```

---

## 四、develop 分支配置

### 4.1 保护规则

```yaml
# GitHub Settings → Branches → Branch protection rules
Branch name pattern: develop*

☑ Require a pull request before merging
  ☑ Require approvals: 1
  ☑ Dismiss stale pull request approvals when new commits are pushed

☑ Require status checks to pass before merging
  Status checks:
    - build
    - test

☑ Allow force pushes: NO

☑ Allow deletions: NO
```

### 4.2 GitHub CLI 配置命令

```bash
gh api -X PUT repos/{owner}/{repo}/branches/develop/protection \
  -H "Accept: application/vnd.github+json" \
  -F required_pull_request_reviews='{"required_approving_review_count":1,"dismiss_stale_reviews":true}' \
  -F required_status_checks='{"strict":false,"contexts":["build","test"]}' \
  -F allow_force_pushes=false \
  -F allow_deletions=false
```

---

## 五、CODEOWNERS 配置

### 5.1 文件位置

__代码0__

### 5.2 配置内容

```
# SQLRustGo CODEOWNERS

# 默认所有者
* @yinglichina8848

# 核心代码需要架构师审核
/src/ @yinglichina8848 @maintainer

# 存储引擎需要存储专家审核
/src/storage/ @yinglichina8848

# 优化器需要优化专家审核
/src/optimizer/ @yinglichina8848

# CI/CD 配置需要 DevOps 审核
/.github/ @yinglichina8848

# 文档需要文档负责人审核
/docs/ @maintainer

# 发布相关需要发布负责人审核
/RELEASE* @yinglichina8848
/VERSION @yinglichina8848
/Cargo.toml @yinglichina8848
```

---

## 六、CI 状态检查配置

### 6.1 必需检查项

| 检查项 | 工作流 | 说明 |
|--------|--------|------|
| `build` | ci.yml | 编译检查 |
| `test` | ci.yml | 测试检查 |
| `lint` | ci.yml |Clippy 检查|
|__代码0__| ci.yml | 安全扫描 |
|__代码0__| ci.yml | 覆盖率检查 |

### 6.2 CI 工作流配置

```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [main, develop, 'develop-*', 'release/*']
  pull_request:
    branches: [main, develop, 'develop-*', 'release/*']

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Build
        run: cargo build --all

  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Test
        run: cargo test --all

  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Clippy
        run: cargo clippy --all-targets -- -D warnings
      - name: Format
        run: cargo fmt --check

  security-scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Security Audit
        run: cargo audit

  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Coverage
        run: |
          cargo install cargo-llvm-cov
          cargo llvm-cov --all-features --lcov --output-path lcov.info
```

---

## 七、签名提交配置

### 7.1 GPG 签名设置

```bash
# 生成 GPG 密钥
gpg --full-generate-key

# 查看密钥
gpg --list-secret-keys --keyid-format=long

# 导出公钥
gpg --armor --export <key-id>

# 配置 Git 使用签名
git config --global user.signingkey <key-id>
git config --global commit.gpgsign true

# 将公钥添加到 GitHub
# Settings → SSH and GPG keys → New GPG key
```

### 7.2 SSH 签名设置 (推荐)

```bash
# 生成 SSH 签名密钥
ssh-keygen -t ed25519 -C "signing" -f ~/.ssh/signing_key

# 配置 Git 使用 SSH 签名
git config --global gpg.format ssh
git config --global user.signingkey ~/.ssh/signing_key.pub
git config --global commit.gpgsign true

# 将公钥添加到 GitHub
# Settings → SSH and GPG keys → New SSH key (Signing Key)
```

---

## 八、违规处理

### 8.1 常见违规场景

| 场景 | 原因 | 解决方案 |
|------|------|----------|
| 直接 Push 失败 | 分支保护生效 | 创建 PR |
| CI 检查失败 | 测试/编译未通过 | 修复问题后重新提交 |
| 审核未通过 | 缺少审核 | 等待审核 |
| 签名验证失败 | 未签名提交 | 配置签名后重新提交 |

### 8.2 紧急修复流程

```bash
# 仅限管理员，紧急情况下临时禁用保护
# Settings → Branches → Edit → 临时取消 "Include administrators"

# 修复后立即恢复保护
```

---

## 九、相关文档

| 文档 | 说明 |
|------|------|
| [RELEASE_GOVERNANCE.md](./RELEASE_GOVERNANCE.md) | 版本治理模型 |
| [ARCHITECTURE_RULES.md](./ARCHITECTURE_RULES.md) | AI 协作安全规则 |
| [BRANCH_GOVERNANCE.md](./BRANCH_GOVERNANCE.md) | 分支治理规范 |

---

## 十、变更历史

| 版本 | 日期 | 说明 |
|------|------|------|
| 1.0 | 2026-03-06 | 初始版本，定义分支保护配置 |

---

*本文档由 yinglichina8848 制定*
