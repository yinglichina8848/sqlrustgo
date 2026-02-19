# PR 审核与合并流程

> 版本：v1.0
> 日期：2026-02-16
> 适用范围：v1.0.0 及后续版本

---

## 一、工作流程总览

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        PR 审核与合并流程                                    │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   1. 创建功能分支                                                            │
│       ↓                                                                     │
│   2. 本地开发 + 测试                                                          │
│       ↓                                                                     │
│   3. 推送分支 + 创建 PR                                                      │
│       ↓                                                                     │
│   4. CI 检查 (自动)                                                          │
│       ↓                                                                     │
│   5. 代码审查 (人工)                                                         │
│       ↓                                                                     │
│   6. 修改 + 重新审查                                                         │
│       ↓                                                                     │
│   7. 合并到目标分支                                                          │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 二、分支与 PR 对应关系

### 2.1 功能分支 → Alpha/Beta

| 源分支 | 目标分支 | 场景 |
|:-------|:---------|:-----|
| `feature/v1.x.x-*` | `feature/v1.x.x-alpha` | 功能开发完成，进入 Alpha |
| `feature/v1.x.x-alpha` | `feature/v1.x.x-beta` | Alpha 完成，进入 Beta |
| `feature/v1.x.x-beta` | `baseline` | Beta 完成，发布正式版 |

### 2.2 baseline → main

| 源分支 | 目标分支 | 场景 |
|:-------|:---------|:-----|
| `baseline` | `main` | 正式版本发布 |

---

## 三、PR 创建流程

### 3.1 功能开发 PR

```bash
# 1. 确保在最新 baseline 上
git checkout baseline
git pull origin baseline

# 2. 创建功能分支
git checkout -b feature/v1.0.1-unwrap-fix baseline

# 3. 开发并提交
git add .
git commit -m "fix: replace unwrap with proper error handling"

# 4. 推送并创建 PR
git push -u origin feature/v1.0.1-unwrap-fix
gh pr create --title "fix: replace unwrap in executor" \
  --body "$(cat <<'EOF'
## 描述
替换 executor 模块中的 unwrap，使用正确的错误处理

## 变更内容
- 替换了 24 处 unwrap
- 添加了 TableNotFound 错误处理

## 测试
- [x] cargo test 通过
- [x] cargo clippy 无警告

## 相关 Issue
Closes #9
EOF
)"
```

### 3.2 Alpha/Beta 合并 PR

```bash
# Alpha 合并到 Beta
gh pr create --title "v1.0.0-alpha: merge to beta" \
  --body "$(cat <<'EOF'
## 阶段：Alpha → Beta

### Alpha 完成清单
- [x] 测试覆盖率 >= 90%
- [x] 所有测试通过
- [x] clippy 无警告
- [x] API 文档完整

### Beta 任务
- [ ] CI/CD 完善
- [ ] 基准测试
- [ ] 多平台验证
EOF
)"
```

---

## 四、CI 检查清单（自动）

### 4.1 PR 自动检查项

创建 PR 后自动触发：

```yaml
# .github/workflows/ci.yml
name: CI

on:
  pull_request:
    branches: [baseline, main, feature/*-alpha, feature/*-beta]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Build
        run: cargo build --all-features

      - name: Test
        run: cargo test --all-features

      - name: Clippy
        run: cargo clippy --all-features -- -D warnings

      - name: Format Check
        run: cargo fmt --check --all
```

### 4.2 必须通过的检查

| 检查项 | 命令 | 状态要求 |
|:-------|:-----|:---------|
| 编译 | `cargo build --all-features` | ✅ 通过 |
| 测试 | `cargo test --all-features` | ✅ 全部通过 |
| Clippy | `cargo clippy -- -D warnings` | ✅ 无警告 |
| 格式 | `cargo fmt --check` | ✅ 格式正确 |

---

## 五、代码审查流程

### 5.1 审查要点

#### 功能正确性
- [ ] 代码逻辑正确实现需求
- [ ] 边界条件有处理
- [ ] 错误路径有覆盖
- [ ] 单元测试 >= 90% 覆盖

#### 代码质量
- [ ] 无重复代码
- [ ] 函数长度 < 50 行
- [ ] 命名清晰有意义
- [ ] 复杂的逻辑有注释

#### 安全与性能
- [ ] 无安全漏洞
- [ ] 无明显性能问题
- [ ] 资源正确释放

### 5.2 审查反馈格式

```markdown
## 审查意见

### CRITICAL (必须修复)
- **文件:行号**: 问题描述
- 建议修复方案

### HIGH (建议修复)
- **文件:行号**: 问题描述
- 建议修复方案

### MEDIUM (可选修复)
- **文件:行号**: 问题描述
- 建议修复方案

### SUGGESTION (建议)
- 改进建议
```

### 5.3 审查人员

| 分支 | 审查要求 |
|:-----|:---------|
| baseline | 至少 1 人审查 |
| main | 至少 1 人审查 + CI 通过 |
| Alpha/Beta | 至少 1 人审查 |

---

## 六、合并流程

### 6.1 合并条件

| 目标分支 | 合并条件 |
|:---------|:---------|
| `feature/v1.x.x-alpha` | 1. CI 通过<br>2. 1 人审查通过<br>3. 测试覆盖 >= 90% |
| `feature/v1.x.x-beta` | 1. Alpha 合并完成<br>2. CI 通过<br>3. 1 人审查通过 |
| `baseline` | 1. Beta 合并完成<br>2. CI 通过<br>3. 1 人审查通过 |
| `main` | 1. baseline 合并完成<br>2. CI 通过<br>3. 版本标签创建 |

### 6.2 合并命令

```bash
# 1. 切换到目标分支
git checkout baseline

# 2. 拉取最新
git pull origin baseline

# 3. 合并功能分支
git merge feature/v1.0.1-xxx

# 4. 解决冲突（如果有）

# 5. 推送
git push origin baseline
```

### 6.3 合并方式

推荐使用 **Squash Merge**（压缩合并）：

```bash
# 通过 GitHub PR 界面选择 Squash Merge
# 或命令行
gh pr merge <pr-number> --squash --delete-branch
```

---

## 七、v1.0.0 当前流程

### 7.1 当前状态

```
feature/v1.0.0-evaluation (当前)
    ↓
feature/v1.0.0-alpha (待创建)
    ↓
feature/v1.0.0-beta (待创建)
    ↓
baseline (v1.0.0)
    ↓
main (稳定版)
```

### 7.2 立即需要修复的问题

**CI 检查失败**：
- ❌ clippy 有错误（1个严重 + 28个警告）
- 需要修复后 才能进入 Alpha

**修复清单**：
- [ ] 修复 `to_string` 方法覆盖 `Display` 的问题
- [ ] 修复 28 个 clippy 警告

### 7.3 PR 创建示例

当前评估分支完成后，创建 PR 到 Alpha：

```bash
# 创建 PR
gh pr create --title "v1.0.0: evaluation complete - merge to alpha" \
  --body "$(cat <<'EOF'
## 概述
v1.0.0 评估完成，准备进入 Alpha 阶段

## 评估报告
- 7 个维度评估报告
- 综合改进计划
- 版本演化规划
- 分支管理策略

## 待修复
- [ ] clippy 错误修复

## 阶段目标
- [ ] 测试覆盖率 >= 90%
- [ ] clippy 无警告
- [ ] API 文档完整
EOF
)"
```

---

## 八、审查检查清单模板

### 8.1 PR 作者检查清单

```markdown
- [ ] 本地测试全部通过
- [ ] clippy 无警告
- [ ] 格式正确
- [ ] 代码已注释（复杂逻辑）
- [ ] 有必要的单元测试
- [ ] 更新了相关文档
- [ ] 提交信息符合规范
```

### 8.2 审查者检查清单

```markdown
- [ ] 代码逻辑正确
- [ ] 边界条件处理
- [ ] 错误处理完善
- [ ] 测试覆盖足够
- [ ] 无安全漏洞
- [ ] 性能合理
- [ ] 文档同步更新
```

---

## 九、快速命令参考

```bash
# 创建功能分支
git checkout -b feature/v1.x.x-featureName baseline

# 推送并创建 PR
git push -u origin feature/v1.x.x-featureName
gh pr create --title "feat: description" --body "..."

# 查看 PR 状态
gh pr list
gh pr view <number>

# 审查 PR
gh pr review <number> --approve
gh pr review <number> --request-changes --body "..."

# 合并 PR
gh pr merge <number> --squash --delete-branch

# 查看 CI 状态
gh run list
```

---

## 十、相关文档

- [分支管理策略](./2026-02-16-branch-strategy.md)
- [版本演化规划](./2026-02-16-version-evolution-plan.md)

---

> 本流程确保代码质量，每个 PR 都经过自动检查和人工审查。
