# SQLRustGo 协作流程规范

## Bug 处理流程

### 发现 Bug

当 `docs-testing` 分支在测试中发现 baseline 或其他分支的 bug 时：

1. **创建 Issue** - 详细描述问题
2. **指派给对应开发者**
3. **等待修复** - 核心代码只能由功能分支开发者修改

### Issue 模板

```markdown
## Bug 描述
[清晰描述问题]

## 复现步骤
1. ...
2. ...

## 预期行为
[应该怎样]

## 实际行为
[实际怎样]

## 发现环境
- 分支: docs-testing
- 相关模块: [index-executor / network-protocol / baseline]
```

## 代码修改权限

| 分支 | 可修改内容 | 不可修改 |
|------|-----------|----------|
| feature/index-executor | 索引、执行器代码 | 网络协议代码 |
| feature/network-protocol | 网络协议代码 | 索引、执行器代码 |
| feature/docs-testing | **仅测试和文档** | 核心代码 |

## 使用 Git Worktree 隔离开发

### 为什么使用 Worktree

- 避免来回切换分支
- 保持工作区干净
- 可以同时在多个分支工作

### 创建 Worktree 示例

```bash
# 在 index-executor 分支创建 worktree
git worktree add ../sqlrustgo-index -b feature/index-executor

# 在 network-protocol 分支创建 worktree
git worktree add ../sqlrustgo-network -b feature/network-protocol

# 在 docs-testing 分支创建 worktree
git worktree add ../sqlrustgo-docs -b feature/docs-testing
```

### 切换工作目录

```bash
# 进入 index-executor worktree
cd ../sqlrustgo-index

# 进入 network-protocol worktree
cd ../sqlrustgo-network

# 进入 docs-testing worktree
cd ../sqlrustgo-docs
```

## PR 合并规则

### 合并到 baseline

1. 功能开发完成
2. 创建 PR: `feature/xxx` → `baseline`
3. 需要**其他分支**至少 1 人评审
4. CI 通过后合并

### 评审要求

| PR 作者 | 评审者 |
|---------|--------|
| index-executor | network-protocol 或 docs-testing |
| network-protocol | index-executor 或 docs-testing |
| docs-testing | index-executor 或 network-protocol |

## 沟通渠道

1. **GitHub Issue** - Bug 报告、功能请求
2. **PR 评论** - 代码评审讨论
3. **微信/消息** - 紧急问题同步
