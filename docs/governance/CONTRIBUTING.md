# SQLRustGo 贡献指南

> **版本**: 1.0
> **更新日期**: 2026-03-07
> **维护人**: yinglichina8848

---

## 一、项目欢迎

SQLRustGo 是一个开源的 Rust SQL 数据库执行引擎，我们欢迎所有形式的贡献，包括但不限于:

- 代码提交
- Bug 报告
- 文档改进
- 问题解答
- 测试用例
- 性能优化
- AI 协作贡献

---

## 二、行为准则

### 2.1 基本原则

- **尊重**: 尊重所有贡献者
- **包容**: 欢迎不同背景的参与者
- **专业**: 保持专业和建设性的沟通
- **负责**: 对自己的行为负责

### 2.2 不可接受的行为

- 人身攻击或侮辱性言论
- 公开或私下骚扰
- 泄露他人隐私信息
- 其他不道德或不专业的行为

---

## 三、贡献流程

### 3.1 贡献类型

| 类型 | 描述 | 审核要求 |
|------|------|----------|
| **Bug 修复** | 修复已知问题 | 1 人 Review |
| **新功能** | 添加新功能 | 1 人 Review + CI 通过 |
| **重构** | 代码优化 | 1 人 Review |
| **文档** | 文档更新 | 1 人 Review |
| **热修复** | 紧急修复 | 2 人 Review |

### 3.2 开发流程

```
1. Fork 仓库
   └── https://github.com/yinglichina/sqlrustgo

2. 克隆本地
   git clone https://github.com/<your>/sqlrustgo

3. 创建功能分支
   git checkout -b feature/v1.2.0-your-feature

4. 开发并测试
   cargo build
   cargo test

5. 提交更改
   git add .
   git commit -m "feat: add new feature"

6. 推送分支
   git push origin feature/v1.2.0-your-feature

7. 创建 PR
   └── GitHub 上创建 Pull Request
```

---

## 四、分支命名规范

### 4.1 分支类型前缀

| 前缀 | 用途 | 示例 |
|------|------|------|
| `feature/` | 新功能 | feature/v1.2.0-cascades |
| `fix/` | Bug 修复 | fix/v1.2.0-index-bug |
| `refactor/` | 重构 | refactor/v1.2.0-storage |
| `docs/` | 文档更新 | docs/v1.2.0-api-doc |
| `hotfix/` | 热修复 | hotfix/v1.2.1-security |

### 4.2 命名规则

- 使用 kebab-case: `feature/v1.2.0-my-new-feature`
- 包含版本号: `feature/v1.2.0-xxx`
- 简洁明了: 描述性名称, 不超过 50 字符

详见: [BRANCH_GOVERNANCE.md](../BRANCH_GOVERNANCE.md)

---

## 五、提交规范

### 5.1 提交信息格式

```
<type>(<scope>): <subject>

[optional body]

[optional footer]
```

### 5.2 Type 类型

| Type | 说明 | 示例 |
|------|------|------|
| `feat` | 新功能 | feat: add cascades optimizer |
| `fix` | Bug 修复 | fix: resolve index out of bounds |
| `refactor` | 重构 | refactor: optimize query planner |
| `docs` | 文档 | docs: update API documentation |
| `test` | 测试 | test: add integration tests |
| `chore` | 维护 | chore: update dependencies |
| `perf` | 性能 | perf: improve join algorithm |

### 5.3 提交示例

```bash
feat(planner): add hash join operator

- Implement hash join algorithm
- Add hash join to physical plan
- Add unit tests for hash join

Closes #123
```

---

## 六、代码规范

### 6.1 Rust 代码风格

遵循 Rust 官方代码风格:

```bash
cargo fmt
cargo clippy
```

### 6.2 代码检查

提交前必须通过:

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

### 6.3 文档要求

- 公共 API 必须添加文档注释
- 复杂逻辑需要代码注释
- 更新相关文档

---

## 七、Pull Request 规范

### 7.1 PR 标题格式

```
<type>(<scope>): <description>
```

### 7.2 PR 描述模板

```markdown
## 描述
<!-- 简要描述这个 PR 做了什么 -->

## 变更类型
- [ ] Bug 修复
- [ ] 新功能
- [ ] 重构
- [ ] 文档更新
- [ ] 测试

## 测试
<!-- 描述如何测试这个变更 -->

##  Checklist
- [ ] 代码遵循项目规范
- [ ] 已添加/更新测试
- [ ] 已更新文档
- [ ] CI 检查通过
```

### 7.3 审核标准

PR 必须满足以下条件才能合并:

- [ ] 代码审查通过
- [ ] 所有 CI 检查通过
- [ ] 测试覆盖率未下降
- [ ] 无冲突分支

---

## 八、AI 协作贡献

### 8.1 AI 贡献者

AI 贡献者 (如 GitHub Copilot) 产生的代码同样需要:

- 人工审查确认
- 测试验证
- 符合代码规范

### 8.2 AI 协作规则

详见: [AI_COLLABORATION.md](./AI_COLLABORATION.md)

---

## 九、问题反馈

### 9.1 Bug 报告

使用 GitHub Issues 报告 Bug, 请包含:

- 问题描述
- 复现步骤
- 期望行为
- 实际行为
- 环境信息 (Rust 版本, 操作系统等)

### 9.2 功能请求

使用 GitHub Issues 提出功能请求, 请包含:

- 功能描述
- 使用场景
- 可能的实现方案

---

## 十、许可证

通过贡献代码, 您同意将您的贡献以 MIT 许可证发布。

---

## 十一、相关文档

| 文档 | 说明 |
|------|------|
| [BRANCH_GOVERNANCE.md](../BRANCH_GOVERNANCE.md) | 分支治理 |
| [RELEASE_LIFECYCLE.md](./RELEASE_LIFECYCLE.md) | 版本生命周期 |
| [RELEASE_POLICY.md](./RELEASE_POLICY.md) | 发布策略 |
| [AI_COLLABORATION.md](./AI_COLLABORATION.md) | AI 协作规则 |

---

## 十二、变更历史

| 版本 | 日期 | 说明 |
|------|------|------|
| 1.0 | 2026-03-07 | 初始版本 |

---

*本文档由 yinglichina8848 维护*
