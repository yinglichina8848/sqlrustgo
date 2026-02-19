# Contributing Guide

感谢你对本项目的贡献！

本项目是一个数据库内核级 Rust 工程。为了保证稳定性和可维护性，请遵循以下规范。

---

## 1. 分支策略

主分支：

- `main` → 生产版本
- `feature/v1.0.0-beta` → 当前开发主干

开发流程：

1. 从 `feature/v1.0.0-beta` 创建功能分支
2. 提交 PR 到 `feature/v1.0.0-beta`
3. PR 合并后删除功能分支

禁止直接 push 到主分支。

---

## 2. PR 命名规范（必须遵守）

格式：

```
type(scope): summary
```

示例：

- `feat(auth): implement basic authentication`
- `fix(executor): remove unwrap panic`
- `perf(parser): optimize tokenizer`
- `refactor(network): improve error handling`

### type 允许值

- feat - 新功能
- fix - Bug 修复
- perf - 性能优化
- refactor - 结构重构
- test - 测试改进
- docs - 文档
- chore - 构建/CI

### scope 允许值

- parser - 词法/语法分析
- executor - 执行引擎
- planner - 查询规划
- optimizer - 优化器
- storage - 存储引擎
- network - 网络协议
- auth - 认证授权
- ci - CI/CD

---

## 3. Commit Message 规范

格式：

```
type(scope): short summary

(optional body)
```

规则：

- 一次 commit 只做一件事
- 不允许使用 `update`, `fix bug` 这种模糊信息
- 不允许包含无关改动

---

## 4. 代码规范

- 禁止使用 `unwrap()`（除测试）
- 禁止 `panic!`（除测试）
- 必须使用 `Result` 传播错误
- 所有新增逻辑必须有测试

运行以下命令必须通过：

```bash
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test
```

---

## 5. 覆盖率要求

- 覆盖率不得下降
- 核心模块（parser / executor / planner）必须 ≥ 80%

---

## 6. Benchmark 要求

性能相关 PR 必须提供 benchmark 对比。

```bash
cargo bench
```

---

## 7. Beta 阶段限制

当前为 v1.0.0-beta 阶段：

允许：
- fix - Bug 修复
- perf - 性能优化
- 小规模 refactor

禁止：
- 重大架构变更
- API 破坏性修改
- 大型新功能

---

## 8. v1.0 冻结原则

v1.0 定义：

✅ 可运行
✅ 不崩溃
✅ 有测试
✅ 有文档
✅ 有版本
✅ 有 CI
✅ 有 Release Note
✅ 符合工程规范

从现在开始：

❌ 禁止：新功能、性能优化、架构重构、API 变更、新模块

✅ 只允许：panic 修复、unwrap 移除、测试补充、文档完善、CI 修复

---

感谢贡献 ❤️
