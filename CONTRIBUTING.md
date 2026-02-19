# 贡献指南

感谢你对 SQLRustGo 项目的兴趣！

## 一、PR 命名规范（强约束版）

### 1. 标准格式

```
<type>(<scope>): <summary>
```

### 2. 示例

```
feat(auth): implement basic authentication
fix(executor): resolve unwrap panic in pipeline
refactor(network): replace unwrap with proper error handling
perf(parser): optimize token scanning
test(parser): increase coverage to 85%
chore(ci): update workflow for coverage report
docs(readme): update installation instructions
```

### 3. type 规范（必须限制）

| type | 含义 | 是否进入 Release Note |
|------|------|----------------------|
| feat | 新功能 | ✅ |
| fix | Bug 修复 | ✅ |
| perf | 性能优化 | ✅ |
| refactor | 结构重构 | ⚠️ 可选 |
| test | 测试改进 | ❌ |
| chore | 构建/CI | ❌ |
| docs | 文档 | ❌ |

### 4. scope 规范（必须来自模块）

只允许以下 scope：

- `parser` - 词法/语法分析
- `executor` - 执行引擎
- `planner` - 查询规划
- `network` - 网络协议
- `auth` - 认证授权
- `storage` - 存储引擎
- `optimizer` - 优化器
- `ci` - CI/CD
- `docs` - 文档

### 5. 禁止的 PR 标题

❌ 以下标题一律拒绝：

```
fix bug
update
refactor
improve code
WIP
```

## 二、Commit 语义化规范

### 1. 格式

```
type(scope): short summary

(optional body)

(optional footer)
```

### 2. 示例

```
fix(executor): remove unwrap in result handling

Replace unwrap() with proper error propagation using Result.
This prevents panic during network failure.

Closes #45
```

### 3. 关键规则

- 一次 commit 只做一件事
- 不允许多个无关修改混在一起
- PR 可以有多个 commit，但每个 commit 必须独立可理解
- 必须关联相关 Issue

## 三、Beta 阶段 PR 收敛策略

### 允许的 PR 类型

Beta 阶段只允许三类 PR：

1. **fix** - Bug 修复
2. **perf** - 性能优化
3. **refactor** - 低风险重构

### 禁止的 PR 类型

❌ 以下类型一律拒绝：

- 大型新功能
- 架构推翻
- 大规模 API 变更
- 破坏性改动

### PR 审核标准（Beta 阶段）

- ✅ 必须有测试
- ✅ 不允许 unwrap
- ✅ 不允许 panic
- ✅ 覆盖率不能下降
- ✅ benchmark 不能明显退化

## 四、分支策略

### 当前活跃分支

```
main                    # 稳定发布分支
baseline                # 开发基线分支
feature/v1.0.0-beta     # Beta 开发主干
```

### 分支规则

- Beta 作为唯一活跃开发主干
- 所有 feature 分支从 beta 拉
- PR 只合并到 beta
- Beta 稳定后合并到 main

## 五、版本号策略

### Beta 阶段

```
v1.0.0-beta.1
v1.0.0-beta.2
v1.0.0-beta.3
```

### 稳定后

```
v1.0.0-rc.1
v1.0.0
```

## 六、开发流程

### 报告 Bug

1. 搜索现有 Issue 确认是否已报告
2. 创建新 Issue，包含:
   - 清晰的标题
   - 复现步骤
   - 预期行为 vs 实际行为
   - 环境信息 (OS, Rust 版本)

### 提出新功能

1. 创建 Issue 描述功能
2. 说明使用场景
3. 讨论确认后开始实现

### 提交代码

1. Fork 项目
2. 从 `feature/v1.0.0-beta` 创建特性分支
3. 遵循代码规范
4. 添加测试
5. 提交 PR（遵循命名规范）

## 七、代码审查

所有 PR 需要:

- 至少 1 位审查者通过
- CI 检查全部通过
- 无合并冲突
- 符合 PR 命名规范
- 符合 Beta 阶段收敛策略

## 八、行为准则

- 尊重他人
- 建设性讨论
- 欢迎新手贡献

---

**注意**: 不符合规范的 PR 将被拒绝，请务必遵守以上规范。
