# PR Rules

> PR 命名规范与审核规则

---

## PR 标题格式

```
<type>(<scope>): <summary>
```

### 示例

```
feat(auth): implement basic authentication
fix(executor): resolve unwrap panic in pipeline
perf(parser): optimize token scanning
refactor(network): replace unwrap with proper error handling
test(parser): increase coverage to 85%
chore(ci): update workflow for coverage report
docs(readme): clarify build steps
```

---

## Type 规范

| Type | 含义 | 进入 Release Note |
|------|------|-------------------|
| feat | 新功能 | ✅ |
| fix | Bug 修复 | ✅ |
| perf | 性能优化 | ✅ |
| refactor | 结构重构 | ⚠️ 可选 |
| test | 测试改进 | ❌ |
| docs | 文档 | ❌ |
| chore | 构建/CI | ❌ |
| ci | CI 修改 | ❌ |

---

## Scope 规范

只允许以下模块：

| Scope | 模块 |
|-------|------|
| parser | SQL 解析器 |
| executor | 执行引擎 |
| planner | 查询规划器 |
| optimizer | 优化器 |
| storage | 存储引擎 |
| network | 网络层 |
| auth | 认证模块 |
| ci | CI 配置 |

---

## 禁止的 PR 标题

```
❌ fix bug
❌ update
❌ refactor
❌ improve code
❌ minor change
```

这些都必须拒绝。

---

## PR 审核标准

### Beta 阶段要求

- 必须有测试
- 不允许 unwrap
- 不允许 panic
- 覆盖率不能下降
- benchmark 不能明显退化

### PR 大小限制

- 单 PR 改动 ≤ 400 行
- ≤ 1 个模块
- 超过拒绝

---

## Beta 阶段 PR 收敛策略

### 允许

- fix：Bug 修复
- perf：性能优化
- refactor（低风险）

### 禁止

- ❌ 大型新功能
- ❌ 架构推翻
- ❌ 大规模 API 变更

---

## PR 流程

```
1. 从 beta 创建功能分支
2. 提交 PR 到 beta
3. CI 检查必须全绿
4. Code Review 通过
5. 合并
6. 删除功能分支
```

---

## 三个必答问题

任何 PR 必须回答：

1. 是否影响 public API？
2. 是否引入技术债？
3. 是否可在当前版本合理存在？
