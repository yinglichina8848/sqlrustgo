---
name: "code-reviewer"
description: "Reviews code changes, runs checks, analyzes code quality, and merges PRs. Invoke when user asks for code review, PR review, or to merge a PR."
---

# Code Reviewer

This skill provides comprehensive code review and PR management capabilities.

## Features

### 1. 查看PR详情 (View PR Details)
- Get PR information including title, description, author, status
- View file changes and diffs
- Check PR comments and reviews
- View CI/CD status

### 2. 运行检查 (Run Checks)
- Run project linters (cargo clippy, rustfmt, etc.)
- Execute test suites
- Run security checks
- Verify code formatting

### 3. 代码分析 (Code Analysis)
- Analyze code changes for potential issues
- Identify code smells
- Check for common bugs
- Review code complexity
- Verify best practices adherence

### 4. 合并PR (Merge PR)
- Squash and merge
- Create merge commit
- Rebase and merge
- Handle merge conflicts

## Usage

### Manual Trigger
When user asks:
- "Review this PR"
- "审核这个PR"
- "Check this pull request"
- "Merge this PR"
- "合并PR"
- Or any similar request

### Automatic Trigger
When:
- A new PR is created
- New commits are pushed to an existing PR
- A PR is marked as ready for review

## Workflow

1. **Fetch PR Information**
   ```bash
   gh pr view <pr-number> --json title,body,state,author,files,comments
   ```

2. **Run Checks**
   ```bash
   cargo clippy -- -D warnings
   cargo fmt --check
   cargo test
   ```

3. **Review Changes**
   - Analyze the diff
   - Check for issues
   - Add review comments if needed

4. **Merge (if approved)**
   ```bash
   gh pr merge <pr-number> --admin --squash
   ```

## Configuration

The skill can be configured with:
- Preferred merge method (squash/rebase/merge)
- Required checks before merge
- Auto-approve patterns
- Review comment templates
