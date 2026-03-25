# v1.1.0-Beta 执行手册（助教版）

## 概述

本手册面向助教（TA），指导如何审核 PR、验证门禁、记录证据链。

**版本**: v1.1.0-beta
**目标读者**: 助教 / 审核人员

---

## 1. PR 审核流程

### 1.1 获取 PR 列表

```bash
gh pr list
```

### 1.2 查看 PR 详情

```bash
gh pr view <PR_NUMBER>
gh pr view <PR_NUMBER> --comments
```

### 1.3 审核检查清单

| # | 检查项 | 说明 |
|---|--------|------|
| 1 | **代码质量** | 无明显错误、逻辑合理 |
| 2 | **测试覆盖** | 新功能有对应测试 |
| 3 | **Clippy 检查** | `cargo clippy --all-features -- -D warnings` |
| 4 | **格式化检查** | `cargo fmt --check` |
| 5 | **门禁验证** | 所有检查通过才能合并 |

---

## 2. 门禁检查（Gatekeeper）

### 2.1 必须通过的检查

```bash
# 1. 编译检查
cargo build --all-features

# 2. 测试检查
cargo test --all-features

# 3. Clippy 检查（零警告）
cargo clippy --all-features -- -D warnings

# 4. 格式化检查
cargo fmt --check
```

### 2.2 覆盖率检查（可选但推荐）

```bash
cargo llvm-cov --all-features
```

**目标**: ≥ 80%

### 2.3 门禁状态标记

| 状态 | 含义 |
|------|------|
| ✅ 通过 | 所有检查项通过 |
| ⚠️ 警告 | 有非关键问题 |
| ❌ 失败 | 有阻断性问题 |

---

## 3. PR 证据链记录

### 3.1 证据链模板

每个合并的 PR 必须记录以下信息：

```markdown
## PR #XXX: <标题>

### 基本信息
| 项目 | 内容 |
|------|------|
| 分支 | <源分支> → <目标分支> |
| 作者 | <GitHub用户名> |
| 状态 | MERGED/CLOSED |
| 审核人 | <审核人> |

### 变更内容
- <变更描述1>
- <变更描述2>

### 验证证据
| 检查项 | 结果 |
|--------|------|
| cargo build | ✅/❌ |
| cargo test | ✅/❌ |
| cargo clippy | ✅/❌ |
| cargo fmt | ✅/❌ |
| coverage | XX% |

### 风险评估
- **风险级别**: 低/中/高
- **影响范围**: <范围>
- **回滚方案**: <方案>
```

### 3.2 风险评估标准

| 风险级别 | 标准 |
|----------|------|
| **低** | 仅测试代码、文档修改 |
| **中** | 生产代码修改、错误处理改进 |
| **高** | 核心功能变更、架构调整 |

### 3.3 回滚方案

| 风险级别 | 回滚方式 |
|----------|----------|
| 低 | 回退单个提交 |
| 中 | 回退多个提交或 PR |
| 高 | 保持分支备份、逐步合并 |

---

## 4. 审核示例

### 4.1 PR #29: unwrap 错误处理

**基本信息**
| 项目 | 内容 |
|------|------|
| 分支 | feature/unwrap-error-handling → feature/v1.1.0-beta |
| 作者 | yinglichina8848 |
| 状态 | MERGED |
| 审核人 | 高小药 |

**变更内容**
- executor/mod.rs: 6 处 unwrap 替换
- parser/mod.rs: 1 处 unwrap 替换
- transaction/manager.rs: 6 处 unwrap 替换
- transaction/wal.rs: 3 处 unwrap 替换

**验证证据**
| 检查项 | 结果 |
|--------|------|
| cargo build | ✅ 通过 |
| cargo test | ✅ 297 通过 |
| cargo clippy | ✅ 零警告 |
| cargo fmt | ✅ 通过 |

**风险评估**
- **风险级别**: 中
- **影响范围**: 生产代码错误处理
- **回滚方案**: 回退对应提交

---

### 4.2 PR #30: Network 覆盖率提升

**基本信息**
| 项目 | 内容 |
|------|------|
| 分支 | feature/network-coverage-improvement → feature/v1.1.0-beta |
| 作者 | yinglichina8848 |
| 状态 | MERGED |
| 审核人 | 高小药 |

**变更内容**
- 添加 13 个集成测试
- 覆盖率: 75.85% → 90.94%

**验证证据**
| 检查项 | 结果 |
|--------|------|
| cargo build | ✅ 通过 |
| cargo test | ✅ 297 通过 |
| cargo clippy | ✅ 零警告 |
| cargo fmt | ✅ 通过 |
| coverage | ✅ 90.94% |

**风险评估**
- **风险级别**: 低
- **影响范围**: 仅测试代码
- **回滚方案**: 回退测试代码提交

---

## 5. 审核意见模板

### 5.1 批准合并

```markdown
## 审核意见 - [审核人]

### PR #XXX: <标题>

**状态**: ✅ **批准合并**

#### 优点
1. <优点1>
2. <优点2>

#### 变更详情
- <模块>: <变更数量> 处修改

#### 验证结果
| 检查项 | 结果 |
|--------|------|
| cargo build | ✅ |
| cargo test | ✅ |
| cargo clippy | ✅ |
| cargo fmt | ✅ |

---
*审核结论: ✅ 批准合并*
```

### 5.2 有条件批准

```markdown
## 审核意见 - [审核人]

### PR #XXX: <标题>

**状态**: ⚠️ **有条件批准**

#### 优点
1. <优点>

#### 需要修复
1. <问题1> - <修复建议>
2. <问题2> - <修复建议>

#### 建议
- <建议1>

---
*审核结论: ⚠️ 修复后合并*
```

### 5.3 拒绝合并

```markdown
## 审核意见 - [审核人]

### PR #XXX: <标题>

**状态**: ❌ **拒绝合并**

#### 问题
1. <阻断性问题1>
2. <阻断性问题2>

#### 建议
- <修复建议>

---
*审核结论: ❌ 拒绝合并*
```

---

## 6. 常见审核问题

### 6.1 Clippy 失败

**问题**: `error: clippy failed`

**解决**: 本地运行修复
```bash
cargo clippy --all-features -- -D warnings
# 修复显示的问题
```

### 6.2 测试失败

**问题**: 部分测试未通过

**解决**:
1. 查看测试输出定位问题
2. 检查是否与主分支有冲突
3. 确保本地测试全部通过后再提交

### 6.3 覆盖率不足

**问题**: 覆盖率 < 80%

**解决**:
1. 添加更多测试用例
2. 使用 `cargo llvm-cov` 查看未覆盖的代码行

---

## 7. 附录

### 7.1 相关文档

- [任务看板](./task-board.md) - Beta 阶段任务追踪
- [PR 证据链](./pr-evidence.md) - 已合并 PR 记录
- [阶段日报模板](./daily-template.md) - 每日报告模板

### 7.2 GitHub CLI 常用命令

```bash
# 查看 PR 列表
gh pr list

# 查看 PR 详情
gh pr view <PR_NUMBER>

# 查看 PR 评论
gh pr view <PR_NUMBER> --comments

# 合并 PR
gh pr merge <PR_NUMBER>

# 关闭 PR
gh pr close <PR_NUMBER>
```

---

*手册版本: v1.0*
*最后更新: 2026-02-20*
