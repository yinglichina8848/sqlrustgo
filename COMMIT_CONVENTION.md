# Commit Message 规范

> 版本：v1.0
> 日期：2026-02-18
> 规范：Conventional Commits

---

## 1. 基础格式

```
<type>(scope): <description>

[optional body]

[optional footer(s)]
```

---

## 2. 类型规范

| type | 含义 | 示例 |
|:-----|:-----|:-----|
| `feat` | 新功能 | feat(parser): add aggregate function support |
| `fix` | 修复 bug | fix(executor): handle null pointer panic |
| `perf` | 性能优化 | perf(storage): optimize index scan |
| `refactor` | 重构 | refactor(core): decouple planner module |
| `docs` | 文档 | docs: update README with installation guide |
| `test` | 测试 | test(network): add mock tcp stream tests |
| `chore` | 维护 | chore: update dependencies |
| `ci` | CI 修改 | ci: add coverage report workflow |
| `build` | 构建相关 | build: optimize release build flags |
| `style` | 代码风格 | style: fix formatting issues |

---

## 3. Scope 规范

| scope | 模块 |
|:------|:-----|
| `parser` | SQL 解析器 |
| `lexer` | 词法分析器 |
| `executor` | 执行引擎 |
| `storage` | 存储引擎 |
| `network` | 网络层 |
| `transaction` | 事务管理 |
| `bplus` | B+ Tree 索引 |
| `core` | 核心模块 |
| `docs` | 文档 |
| `ci` | CI/CD |

---

## 4. 示例

### 新功能

```
feat(parser): support nested SELECT statements

- Add subquery parsing
- Support EXISTS clause
- Add integration tests

Closes #12
```

### Bug 修复

```
fix(executor): handle null pointer panic in aggregate

The executor was panicking when processing NULL values
in aggregate functions. This fix adds proper NULL handling.

Fixes #15
```

### 性能优化

```
perf(storage): optimize B+ Tree node split

- Reduce memory allocation during split
- Add benchmark tests
- 15% improvement in insert performance
```

### 重构

```
refactor(core): decouple planner from executor

Break circular dependency between planner and executor
modules. This improves testability and maintainability.
```

---

## 5. 重大变更

### 格式 1：类型后加 `!`

```
feat!: redesign execution engine

BREAKING CHANGE: The execution engine has been completely
redesigned. All custom executors need to implement the
new Executor trait.
```

### 格式 2：Footer 中声明

```
refactor(parser): migrate to new AST structure

The AST structure has been changed to support more
SQL features.

BREAKING CHANGE: AST node types have been renamed
- SelectStmt -> SelectStatement
- InsertStmt -> InsertStatement
```

---

## 6. 多行提交

```
feat(executor): implement aggregate functions

Implement COUNT, SUM, AVG, MIN, MAX aggregate functions.

Changes:
- Add AggregateExecutor
- Support GROUP BY clause
- Add NULL handling
- Add integration tests

Test coverage: 92%

Closes #18
```

---

## 7. 关联 Issue

```
feat(parser): add ORDER BY support

Closes #5
```

```
fix(storage): resolve index corruption bug

Fixes #10
```

```
feat(executor): implement aggregate functions

Implements #18
```

---

## 8. AI Agent 提交规范

AI Agent 提交时应包含身份标识：

```
feat(parser): add LIMIT clause support

Implemented by: TRAE (GLM-5.0)
GitHub: yinglichina
```

或：

```
docs: add version promotion SOP

Created by TRAE (GLM-5.0) - 人类李哥控制
```

---

## 9. 检查清单

提交前检查：

- [ ] 类型正确
- [ ] Scope 准确
- [ ] 描述清晰
- [ ] 关联 Issue（如有）
- [ ] 无拼写错误
- [ ] 符合团队规范

---

## 10. 工具支持

### git commit 模板

```bash
# 设置提交模板
git config commit.template .git/commit-template
```

### commitlint 配置

```json
{
  "extends": ["@commitlint/config-conventional"],
  "rules": {
    "type-enum": [2, "always", [
      "feat", "fix", "perf", "refactor",
      "docs", "test", "chore", "ci", "build", "style"
    ]]
  }
}
```

---

*本文档由 TRAE (GLM-5.0) 创建*
