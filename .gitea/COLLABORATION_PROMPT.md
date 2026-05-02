# SQLRustGo 协同开发规范 (v2.9.0)

## 项目概览

SQLRustGo 是一个用 Rust 实现的 SQL 数据库引擎，当前版本 v2.8.0。
代码仓库: `ssh://git@192.168.0.252:222/openclaw/sqlrustgo.git`

## CI/CD 环境

CI 通过 Gitea Actions 运行，runner 标签 `[hp-z6g4]`。
所有 push 到 `develop/v2.8.0` 分支的代码必须通过 CI 流水线。

**禁止** `git commit --allow-empty` 伪造 CI 触发。

## Git 提交规范

### 用户身份（强制）

提交必须使用以下身份之一，否则 pre-commit hook 拒绝提交：

| 用户 | 邮箱 | 用途 |
|------|------|------|
| openheart | openheart@gaoyuanyiyao.com | 正式开发 |
| CI Bot | ci@example.com | CI 自动提交 |

其他邮箱的提交会被 pre-commit hook 拦截并报错：
`❌ Wrong Git identity!`

### Commit Message 格式

```
<type>: <简短描述>

[可选正文]

[可选 footer]
```

Type: `feat` / `fix` / `test` / `refactor` / `chore` / `docs` / `ci`

## 治理规则 (Governance v2.8.0)

所有代码必须通过 Pull Request 合入 develop/v2.8.0。
PR 必须满足以下规则：

- **R1**: 测试不可变性 — 测试不能因新代码而变灰或被删除，只能扩充
- **R2**: Ignore 防护 — 任何 `#[ignore]` 标记必须有对应 Issue 记录阻塞原因
- **R3**: Proof 真实性 — proof 目录下任何 .json 文件必须包含实际执行证据（测试输出、benchmark 数据）
- **R4**: 全量执行 — 代码变更必须运行 `cargo test --all`，不能只跑部分测试
- **R5**: Baseline 验证 — 合入前必须验证 baseline（当前: 226 PASS, 0 FAIL, 0 IGNORED）未被破坏
- **R6**: 测试数量单调性 — 总测试数只能增加或不变，不能减少
- **R7**: CI 完整性 — CI 流水线必须全绿才能合入
- **R8**: SQL兼容性 — SQL Corpus 通过率必须 ≥80% (`cargo test -p sql-corpus`)
- **R9**: 性能基线 — 性能基准测试无显著退化 (benchmark 结果在 baseline ±10% 范围内)
- **R10**: 形式化证明 — proof 目录下必须包含有效的证明文件 (.json 包含实际证据)

## 内存约束

测试宁可缩小数据集（100x）也不要 ignore 测试。数据集大小应体现在测试名称中（如 `test_ivfpq_10k_performance` 表示 10K 向量）。

## 核心原则

**数据库必须"绝对可信"**，不是"能跑就行"。

## 参考文档

- 治理规则: `docs/system/governance.md`
- 版本状态: `docs/versions/v2.8.0/verification_report.json`
- Proof 记录: `docs/proof/*.json`
- Blocker 列表: `docs/system/blockers.md`
- GBrain 知识库: `ssh://git@192.168.0.252:222/openclaw/ai-brain.git`
- LLM-Wiki 文档: `ssh://git@192.168.0.252:222/openclaw/sqlrustgo-wiki.git`
