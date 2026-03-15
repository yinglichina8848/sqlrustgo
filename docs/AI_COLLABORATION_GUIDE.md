# SQLRustGo AI 协作开发指南

> **版本**: v1.0
> **适用范围**: v1.3.0 开发周期
> **更新日期**: 2026-03-13

---

## 一、协作原则

### 1.1 核心原则

| 原则 | 说明 |
|------|------|
| **任务独立** | 每个 AI 负责一个独立任务，避免冲突 |
| **本地验证** | 代码提交前必须在本地通过所有测试 |
| **及时同步** | 定期同步主分支变更到工作分支 |
| **清晰沟通** | 在 Issue 中更新进度和遇到的问题 |

### 1.2 禁止事项

| 禁止 | 说明 |
|------|------|
| ❌ 未经测试提交 | 禁止提交未在本地验证的代码 |
| ❌ 破坏性变更 | 禁止修改核心 API 除非必要 |
| ❌ 强制推送 | 禁止 force push 到非自己分支 |
| ❌ 抢夺任务 | 禁止领取已被其他 AI 领取的任务 |

---

## 二、任务领取

### 2.1 领取流程

```
1. 选择任务
      ↓
2. 在 Issue 下评论 "我会领取 E-001"
      ↓
3. 将自己 assign 到 Issue
      ↓
4. 创建工作分支
      ↓
5. 本地开发 + 测试
      ↓
6. 提交 PR
      ↓
7. 等待 code review
      ↓
8. 合并
```

### 2.2 分支命名规范

| 类型 | 格式 | 示例 |
|------|------|------|
| 功能开发 | `feat/E-xxx-description` | `feat/E-001-executor-trait` |
| Bug 修复 | `fix/E-xxx-bug-description` | `fix/E-002-scan-null` |
| 测试 | `test/E-xxx-description` | `test/E-003-projection` |
| 文档 | `docs/E-xxx-description` | `docs/E-001-add-doc` |

### 2.3 任务状态

| 状态 | 标记 | 说明 |
|------|------|------|
| 待领取 | 🟡 | 无人领取 |
| 进行中 | 🟢 | 已领取，正在开发 |
| 待审核 | 🔵 | PR 已提交，等待 review |
| 已完成 | ✅ | 已合并 |

---

## 三、代码提交

### 3.1 Commit 规范

使用 Conventional Commits 格式：

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

**类型 (type)**:

| 类型 | 说明 |
|------|------|
| feat | 新功能 |
| fix | Bug 修复 |
| test | 测试 |
| docs | 文档 |
| refactor | 重构 |
| chore | 构建/工具 |

**示例**:

```bash
git commit -m "feat(executor): 实现 Volcano Executor trait"
git commit -m "fix(storage): 修复缓冲池内存泄漏"
git commit -m "test(optimizer): 添加常量折叠测试"
```

### 3.2 PR 标题格式

```
<type>(<module>): <description>
```

**示例**:
```
feat(executor): 实现 Volcano Executor trait
fix(storage): 修复缓冲池内存泄漏  
test(optimizer): 添加常量折叠测试
```

### 3.3 PR 描述模板

```markdown
## 关联 Issue
- 关联: #463
- 任务: E-001

## 变更内容
- 新增 Executor trait 定义
- 实现 TableScan 算子
- 添加相关测试

## 测试结果
- [x] cargo test --workspace 通过
- [x] cargo clippy 无警告
- [x] cargo fmt 通过
- [x] 覆盖率检查通过

## 覆盖率变化
- executor: 72% → 75%

## AI 信息 (可选)
- AI 助手: Claude Code
- 模型: MiniMax-M2.1
```

---

## 四、本地验证

### 4.1 必须通过的检查

在提交 PR 前，必须在本地执行以下命令并全部通过：

```bash
# 1. 编译检查
cargo build --workspace

# 2. 测试
cargo test --workspace

# 3. Clippy (零警告)
cargo clippy --workspace -- -D warnings

# 4. 格式检查
cargo fmt --all -- --check

# 5. 覆盖率 (可选，建议运行)
cargo tarpaulin --workspace --all-features
```

### 4.2 覆盖率要求

| 模块 | 最低要求 |
|------|----------|
| 整体 | ≥65% |
| executor | ≥60% |
| planner | ≥60% |
| optimizer | ≥40% |

### 4.3 分支同步

在提交 PR 前，同步最新变更：

```bash
# 1. 切回主分支
git checkout develop/v1.3.0

# 2. 拉取最新
git pull origin develop/v1.3.0

# 3. 切回工作分支
git checkout feat/E-xxx

# 4. 合并主分支
git merge develop/v1.3.0

# 5. 解决冲突（如有）
# 6. 重新测试
cargo test --workspace
```

---

## 五、代码审查

### 5.1 Review 要求

| 项目 | 要求 |
|------|------|
| 最少 Review 人数 | 1 人 |
| Review 通过条件 | 无 blocking comments |
| 合并前必须 | CI 全部通过 |

### 5.2 Review 关注点

审查者应关注：

1. **逻辑正确性** - 代码逻辑是否正确
2. **测试覆盖** - 是否有足够的测试
3. **性能影响** - 是否有性能问题
4. **代码风格** - 是否符合项目规范
5. **安全性** - 是否有安全漏洞

### 5.3 Review 流程

```
提交 PR
    ↓
CI 检查 (自动)
    ↓
Code Review (人工)
    ↓
修改 (如需要)
    ↓
再次 Review
    ↓
合并
```

---

## 六、问题处理

### 6.1 遇到困难

如果在开发中遇到困难：

1. 在 Issue 下评论描述问题
2. 标记 @相关人员
3. 寻求帮助

### 6.2 任务无法完成

如果任务无法完成：

1. 提前在 Issue 中说明
2. 说明原因和已完成的进度
3. 释放任务供其他 AI 领取

### 6.3 冲突处理

如果与其他 AI 的任务冲突：

1. 协商分工
2. 如无法协商，在 Issue 中报告
3. 由维护者协调

---

## 七、签名要求

### 7.1 签名政策

**不强制要求签名**

但建议在 PR 描述中注明：

```markdown
## AI 信息 (可选)
- AI 助手: Claude Code / DeepSeek / ChatGPT / OpenCode
- 模型版本: xxx
```

### 7.2 贡献归属

所有 AI 贡献都会在 PR 和 commit 历史中记录。

---

## 八、联系与支持

| 渠道 | 用途 |
|------|------|
| Issue 评论 | 任务进度、问题求助 |
| GitHub Discussion | 通用讨论 |
| PR Review | 代码审查 |

---

## 九、相关文档

| 文档 | 位置 |
|------|------|
| 版本路线图 | `docs/releases/VERSION_ROADMAP.md` |
| 开发计划 | `docs/releases/v1.3.0/DEVELOPMENT_PLAN.md` |
| 门禁检查 | `docs/releases/v1.3.0/RELEASE_GATE_CHECKLIST.md` |
| 主任务 Issue | #463 |

---

**文档状态**: 正式版  
**创建日期**: 2026-03-13
