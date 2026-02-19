# Commit Standard

> Commit 语义化规范

---

## Commit 格式

```
type(scope): short summary

(optional body)

(optional footer)
```

---

## 示例

### 基本格式

```
fix(executor): remove unwrap in result handling
```

### 带正文

```
fix(executor): remove unwrap in result handling

Replace unwrap() with proper error propagation using Result.
This prevents panic during network failure.
```

### 带关联 Issue

```
fix(executor): remove unwrap in result handling

Replace unwrap() with proper error propagation using Result.

Closes #45
```

---

## Type 规范

| Type | 说明 |
|------|------|
| feat | 新功能 |
| fix | Bug 修复 |
| perf | 性能优化 |
| refactor | 代码重构 |
| test | 测试相关 |
| docs | 文档更新 |
| chore | 构建/依赖 |
| ci | CI 配置 |

---

## Scope 规范

```
parser | executor | planner | optimizer | storage | network | auth | ci
```

---

## 规则

### 必须

- 一次 commit 只做一件事
- 使用英文
- 首字母小写
- 不以句号结尾

### 禁止

```
❌ update
❌ fix bug
❌ minor change
❌ WIP
❌ 修复问题
```

---

## Commit Lint

建议启用 conventional commits 校验：

```yaml
# .github/workflows/commitlint.yml
name: Commit Lint
on: [pull_request]
jobs:
  commitlint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      - uses: wagoid/commitlint-github-action@v5
```

---

## 良好示例

```
feat(storage): add basic page manager
fix(parser): resolve identifier parsing bug
docs(readme): clarify build steps
refactor(exec): simplify executor pipeline
test(network): add connection timeout tests
perf(optimizer): improve cost estimation
```

---

## 不良示例

```
❌ update code
❌ fix
❌ changes
❌ 添加功能
❌ Fix bug in parser
```
