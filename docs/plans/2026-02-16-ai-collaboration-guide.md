# AI 协作开发指南 - 基于 Issue 的工作流

> 本文档为学生提供使用 AI 工具进行协作开发的完整指南，基于 SQLRustGo 项目的实践经验。

---

## 目录

1. [AI 工具链介绍](#1-ai-工具链介绍)
2. [GitHub Issue 使用规范](#2-github-issue-使用规范)
3. [与 AI 沟通的技巧](#3-与-ai-沟通的技巧)
4. [标准开发流程](#4-标准开发流程)
5. [分支管理策略](#5-分支管理策略)
6. [代码审查流程](#6-代码审查流程)
7. [常见问题解决](#7-常见问题解决)

---

## 1. AI 工具链介绍

### 核心工具

| 工具 | 用途 | 使用场景 |
|:-----|:-----|:---------|
| **Claude Code** | 主 AI 助手 | 代码编写、调试、文档 |
| **GitHub** | 代码托管 | 版本控制、PR 管理 |
| **GitHub CLI** | 命令行工具 | gh pr, gh issue, gh repo |
| **Rust/Cargo** | 构建系统 | 编译、测试、打包 |

### Claude Code 技能系统

项目定义了多个专业技能（Skills）：

```bash
# 可用技能列表
superpowers:brainstorming      # 功能设计讨论
superpowers:tdd-workflow       # TDD 开发流程
superpowers:code-review        # 代码审查
superpowers:verification       # 完成前验证
superpowers:using-git-worktrees # 隔离开发环境
```

**使用方式**：
```
# 在对话中告诉 AI 使用某个技能
"使用 brainstorming 技能帮我设计 JOIN 查询的实现"
"使用 tdd-workflow 技能来实现 ORDER BY 功能"
```

---

## 2. GitHub Issue 使用规范

### Issue 创建流程

#### 步骤 1：明确任务类型

```
[bug]      - 功能缺陷或错误
[feature]  - 新功能请求
[enhancement] - 现有功能改进
[question] - 问题咨询
[docs]     - 文档相关
```

#### 步骤 2：编写规范标题

```
✓ [bug] CREATE TABLE 语句解析失败
✗ 创建表出错了
✗ 帮我看看这个 bug
```

#### 步骤 3：详细描述模板

```markdown
## 问题描述
清晰、简洁地描述问题

## 复现步骤（针对 bug）
1. 执行命令：...
2. 预期结果：...
3. 实际结果：...

## 环境信息
- OS: ...
- Rust 版本: ...
- 项目版本: ...

## 尝试过的解决方案
- 方案 A：...
- 方案 B：...

## 附加信息
截图、错误日志、相关代码片段
```

#### 步骤 4：使用标签

在 GitHub Issue 页面添加标签：
- `bug` - 红色
- `enhancement` - 蓝色
- `good first issue` - 绿色（适合新手）
- `help wanted` - 需要帮助

### Issue 示例

**示例 1：Bug 报告**
```markdown
## Bug: INSERT 语句中文本值未正确存储

### 复现步骤
1. 运行 `cargo run --bin sqlrustgo`
2. 执行 `CREATE TABLE users (id INT, name TEXT);`
3. 执行 `INSERT INTO users VALUES (1, '张三');`
4. 执行 `SELECT * FROM users;`

### 预期结果
显示 id=1, name='张三'

### 实际结果
显示 id=1, name=''
```

**示例 2：功能请求**
```markdown
## 功能请求: 支持 ORDER BY 子句

### 需求描述
在 SELECT 查询中支持 ORDER BY 子句进行排序

### 期望语法
SELECT * FROM table ORDER BY column [ASC|DESC]

### 优先级
高 - 这是课程项目必需功能

### 相关 Issue
无
```

---

## 3. 与 AI 沟通的技巧

### 有效沟通原则

#### ✅ 有效提问

```markdown
"在 src/parser/mod.rs 中实现 ORDER BY 解析功能，需要支持 ASC 和 DESC 关键字。请使用 tdd-workflow 技能开发，先写测试再实现代码。"
```

```markdown
"修复 src/storage/buffer_pool.rs 中的内存泄漏问题。使用 verification 技能验证修复后无内存泄漏。"
```

#### ❌ 无效提问

```markdown
"帮我改进代码"              # 太模糊
"实现一个数据库"            # 范围太大
"怎么运行测试？"            # 伸手党
```

### 提供上下文

```markdown
# 好的上下文示例
"在 B+ Tree 索引模块 (src/storage/bplus_tree/) 中添加唯一索引支持。现有实现只支持普通索引，需要修改 node 结构添加唯一性约束。约束：使用 B+ Tree 的 key 唯一性保证。"
```

### 指定约束条件

```markdown
"实现 INSERT 功能，约束：
1. 必须通过 WAL 实现持久化
2. 需要支持事务回滚
3. 遵循已有的代码风格（使用 thiserror）
4. 测试覆盖率需要达到 80%+"
```

### 检查 AI 的工作

```markdown
# 要求验证
"完成实现后，请：
1. 运行 cargo test 确认所有测试通过
2. 运行 cargo clippy 确认无警告
3. 解释你的实现方案"
```

---

## 4. 标准开发流程

### 完整工作流

```
┌─────────────────────────────────────────────────────────────┐
│                    开发流程总览                              │
├─────────────────────────────────────────────────────────────┤
│  1. 创建 Issue     →    明确任务目标和验收标准              │
│         ↓                                                    │
│  2. 设计讨论       →    使用 brainstorming 技能            │
│         ↓                                                    │
│  3. TDD 开发       →    使用 tdd-workflow 技能             │
│         ↓                                                    │
│  4. 代码审查       →    使用 code-review 技能              │
│         ↓                                                    │
│  5. 完成验证       →    使用 verification 技能            │
│         ↓                                                    │
│  6. 创建 PR       →    链接 Issue，请求审查                │
│         ↓                                                    │
│  7. 合并代码       →    审核通过后合并到 baseline          │
└─────────────────────────────────────────────────────────────┘
```

### Step 1: 创建 Issue

```bash
# 使用 GitHub CLI 创建
gh issue create --title "[feature] 添加 ORDER BY 支持" \
  --body "## 需求\n在 SELECT 查询中支持 ORDER BY 子句\n\n## 验收标准\n1. 支持 ORDER BY column ASC\n2. 支持 ORDER BY column DESC\n3. 支持多列排序"
```

### Step 2: 设计讨论

```markdown
# 对话示例

用户: 我需要实现 ORDER BY 功能，请使用 brainstorming 技能帮我设计。

AI: (启动 brainstorming 技能)

# 接下来会：
# 1. 询问需求细节
# 2. 讨论实现方案
# 3. 给出 2-3 个方案及其利弊
# 4. 提供建议的方案
```

### Step 3: TDD 开发

```markdown
# 对话示例

用户: 请使用 tdd-workflow 技能实现 ORDER BY 解析功能。

AI: (启动 TDD 工作流)

# 接下来会：
# 1. 创建测试文件
# 2. 运行测试（RED - 失败）
# 3. 实现最小代码（GREEN - 通过）
# 4. 重构优化（IMPROVE）
# 5. 验证覆盖率
```

### Step 4: 代码审查

```markdown
# 对话示例

用户: 代码已实现完成，请使用 code-review 技能进行审查。

AI: (启动代码审查)

# 会检查：
# 1. 代码逻辑正确性
# 2. 错误处理
# 3. 性能考虑
# 4. 代码风格
# 5. 安全性
```

### Step 5: 创建 PR

```bash
# 1. 推送分支
git add .
git commit -m "feat: 实现 ORDER BY 解析功能"
git push origin feature/order-by

# 2. 创建 PR
gh pr create --base baseline --head feature/order-by \
  --title "feat: 添加 ORDER BY 支持" \
  --body "## 描述\n实现 SELECT 查询的 ORDER BY 子句\n\n## 相关 Issue\nCloses #12\n\n## 测试计划\n- [x] 单元测试\n- [x] 集成测试\n- [x] 手动测试"
```

---

## 5. 分支管理策略

### 分支结构

```
                    PR 审核
                        │
    ┌───────────────────┼───────────────────┐
    │                   │                   │
    ▼                   ▼                   ▼
┌─────────┐      ┌─────────────┐      ┌─────────────┐
│feature/ │      │ feature/    │      │ feature/    │
│ order-by│      │ index-opt   │      │ docs-readme │
└────┬────┘      └──────┬──────┘      └──────┬──────┘
     │                   │                   │
     └───────────────────┼───────────────────┘
                        ▼
                 ┌─────────────┐
                 │  baseline   │ ← 合并点
                 └──────┬──────┘
                        │ PR (maintainer 审批)
                        ▼
                 ┌─────────────┐
                 │    main    │ ← 锁定
                 └─────────────┘
```

### 分支命名规范

| 类型 | 示例 | 说明 |
|:-----|:-----|:-----|
| feature | feature/order-by | 新功能 |
| bugfix | bugfix/insert-crash | Bug 修复 |
| docs | docs/readme-update | 文档更新 |
| refactor | refactor/parser-clean | 代码重构 |
| test | test/integration-add | 测试添加 |

### 分支操作命令

```bash
# 创建功能分支（基于 baseline）
git checkout baseline
git pull origin baseline
git checkout -b feature/xxx

# 保持分支更新
git fetch origin
git rebase origin/baseline

# 推送分支
git push -u origin feature/xxx

# 删除分支（合并后）
git branch -d feature/xxx
git push origin --delete feature/xxx
```

---

## 6. 代码审查流程

### 审查要点清单

```markdown
## 代码审查清单

### 功能正确性
- [ ] 代码逻辑是否正确？
- [ ] 是否覆盖所有边界情况？
- [ ] 错误处理是否完善？

### 代码质量
- [ ] 是否有重复代码？
- [ ] 函数是否过长？
- [ ] 命名是否清晰？

### 性能考虑
- [ ] 是否有不必要的内存分配？
- [ ] 循环是否高效？
- [ ] 是否有 O(n²) 问题？

### 安全问题
- [ ] 是否有 SQL 注入风险？
- [ ] 是否有内存安全问题？
- [ ] 敏感数据是否泄露？

### 测试覆盖
- [ ] 是否有单元测试？
- [ ] 测试是否覆盖主要逻辑？
- [ ] 边界条件是否有测试？
```

### 审查反馈示例

```markdown
## 审查意见

### CRITICAL
- 第 45 行：缺少空指针检查，可能导致 panic

### HIGH
- 第 78-90 行：重复代码，应提取为函数

### MEDIUM
- 建议添加日志记录关键操作

### SUGGESTION
- 可以使用 `?` 运算符简化错误处理
```

---

## 7. 常见问题解决

### 问题 1：合并冲突

```bash
# 查看冲突文件
git status

# 解决冲突
git checkout --theirs src/xxx.rs   # 使用对方版本
git checkout --ours src/xxx.rs      # 使用我方版本
# 或手动编辑解决

# 标记已解决
git add .
git commit -m "merge: 解决与 baseline 的冲突"
```

### 问题 2：测试失败

```bash
# 运行单个测试
cargo test test_name --all-features -- --nocapture

# 查看详细错误
cargo test --all-features 2>&1 | tail -50

# 使用 AI 调试
"运行测试失败，请使用 debugging 技能帮我分析错误原因。"
```

### 问题 3：分支落后

```bash
# 拉取最新并变基
git fetch origin
git rebase origin/baseline

# 如果有冲突，解决后继续
git add .
git rebase --continue
```

### 问题 4：PR 无法合并

```bash
# 检查 PR 状态
gh pr view <number> --json state,mergeable,statusCheckRollup

# 可能原因：
# 1. 有冲突 → 解决冲突
# 2. CI 失败 → 修复测试
# 3. 需要审查 → 等待审查
```

---

## 附录：常用命令速查

### Git 操作

```bash
# 分支
git checkout -b feature/xxx          # 创建分支
git branch -d feature/xxx            # 删除分支
git push origin --delete feature/xxx # 删除远程分支

# 提交
git add .                            # 添加所有修改
git commit -m "feat: 添加功能"       # 提交
git commit --amend                   # 修改最后提交

# 更新
git fetch origin                     # 获取更新
git pull --rebase origin baseline   # 拉取并变基
```

### 项目操作

```bash
# 构建和测试
cargo build --all-features           # 构建
cargo test --all-features           # 测试
cargo clippy --all-features -- -D warnings  # 检查

# 运行
cargo run --bin sqlrustgo           # 运行 REPL
```

### GitHub 操作

```bash
# Issue
gh issue create                     # 创建 Issue
gh issue list                       # 列出 Issue
gh issue view <number>              # 查看 Issue

# PR
gh pr create                        # 创建 PR
gh pr list                         # 列出 PR
gh pr view <number>                # 查看 PR
gh pr merge <number>               # 合并 PR
```

---

## 相关资源

- [SQLRustGo 仓库](https://github.com/yinglichina8848/sqlrustgo)
- [对话记录](./对话记录.md)
- [ROADMAP](./ROADMAP.md)
- [CHANGELOG](./CHANGELOG.md)
- [GitHub 文档](https://docs.github.com)
- [Claude Code 文档](https://docs.anthropic.com)

---

> 本指南基于 SQLRustGo 项目实践编写，适合数据库原理和软件工程课程教学使用。
