# Branching Model

> 分支策略

---

## 主分支

| 分支 | 说明 |
|------|------|
| `main` | 生产版本 (v1.0.0) |
| `baseline` | 基准分支 |
| `feature/v1.0.0-beta` | 当前开发主干 |

---

## 开发流程

```
从 beta 拉分支
    ↓
feature/xxx
    ↓
PR → beta
    ↓
合并
    ↓
删除 feature/xxx
```

---

## 分支命名规范

### 功能分支

```
feature/<description>
```

示例：
- `feature/auth-impl`
- `feature/coverage-improvement`

### 修复分支

```
fix/<description>
```

示例：
- `fix/unwrap-panic`
- `fix/parser-error`

### 热修复分支

```
hotfix/<version>-<description>
```

示例：
- `hotfix/1.0.1-critical-bug`

---

## 发布后分支模型

```
main          ← 1.x 稳定维护
develop       ← 2.0 架构升级
feature/*     ← 从 develop 拉
hotfix/*      ← 从 main 拉
```

---

## 规则

1. **禁止直接 push 到主分支**
2. **功能分支合并后立即删除**
3. **PR 保留，分支删除**
4. **所有变更必须通过 PR**

---

## GitHub 设置建议

开启 `Automatically delete head branches`：

```
Settings → General → Pull Requests
```

PR 合并后自动删除远程分支。
