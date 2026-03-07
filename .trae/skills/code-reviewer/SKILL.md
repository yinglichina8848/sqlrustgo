---
名称：“代码审查员”
描述：“审查代码更改、运行检查、分析代码质量并合并 PR。当用户请求代码审查、PR 审查或合并 PR 时调用。”
---

# 代码审查员

该技能提供全面的代码审查和 PR 管理能力。

＃＃ 特征

### 1. 查看PR详情 (View PR Details)
- 获取 PR 信息，包括标题、描述、作者、状态
- 查看文件更改和差异
- 检查公关评论和评论
- 查看 CI/CD 状态

### 2. 运行检查 (Run Checks)
- 运行项目 linter（cargo Clippy、rustfmt 等）
- 执行测试套件
- 运行安全检查
- 验证代码格式

### 3. 代码分析 (Code Analysis)
- 分析代码更改是否存在潜在问题
- 识别代码气味
- 检查常见错误
- 检查代码复杂性
- 验证最佳实践的遵守情况

### 4. 合并PR (Merge PR)
- 挤压并合并
- 创建合并提交
- 变基和合并
- 处理合并冲突

## Usage

### 手动触发
当用户询问：
- “查看此公关”
- "审核这个PR"
-“检查此拉取请求”
- “合并此 PR”
- "合并PR"
- 或任何类似的要求

### 自动触发
When:
- 创建了一个新的 PR
- 新的提交被推送到现有的 PR
- PR 被标记为可供审核

## 工作流程

1. **获取公关信息**
   ```bash
   gh pr view <pr-number> --json title,body,state,author,files,comments
   ```

2. **运行检查**
   ```bash
   cargo clippy -- -D warnings
   cargo fmt --check
   cargo test
   ```

3. **查看更改**
- 分析差异
- 检查问题
- 如果需要添加评论评论

4. **合并（如果获得批准）**
   ```bash
   gh pr merge <pr-number> --admin --squash
   ```

＃＃ 配置

该技能可以配置为：
- 首选合并方法（squash/rebase/merge）
- 合并前所需的检查
- 自动批准模式
- 审核评论模板
