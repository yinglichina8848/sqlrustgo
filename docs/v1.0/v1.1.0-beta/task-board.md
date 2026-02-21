# v1.1.0-Beta 任务看板

## 当前状态

| 任务 | 负责人 | 状态 | PR |
|------|--------|------|-----|
| 聚合函数 (COUNT/SUM/AVG/MIN/MAX) | opencode | ✅ 完成 | #31 (待创建) |
| unwrap 错误处理 | claude code | ✅ 完成 | #29 |
| Network 覆盖率提升 | claude code | ✅ 完成 | #30 |
| fmt + clippy 修复 | claude code | ✅ 完成 | #28 |
| 文档完善 | opencode | 🔄 进行中 | - |

## 待处理任务

### P0 - 必须完成

| 任务 | 负责人 | 状态 |
|------|--------|------|
| 合并 PR #28/#29/#30 到 beta | codex-cli | ⏳ 待审核 |
| 创建聚合函数 PR | opencode | ⏳ 待创建 |
| 覆盖率验证 (≥80%) | 待定 | ⏳ 待验证 |

### 文档任务

| 任务 | 负责人 | 状态 |
|------|--------|------|
| 任务看板与文档索引 | opencode | ✅ 完成 |
| PR 风险/回滚/验证摘要 | opencode | 🔄 进行中 |
| 阶段日报模板 | opencode | ⏳ 待开始 |
| 执行手册(学生版) | opencode | ⏳ 待开始 |
| 执行手册(助教版) | opencode | ⏳ 待开始 |

## PR 状态汇总

### 已创建 PR

| # | 标题 | 状态 | 审核 |
|---|------|------|------|
| 30 | test: network coverage 90.94% | OPEN | 高小药审核通过 |
| 29 | refactor: unwrap error handling | OPEN | 高小药审核通过 |
| 28 | fix: fmt + clippy | OPEN | 高小药审核通过 |

### 待创建 PR

| # | 标题 | 状态 |
|---|------|------|
| 31 | feat: aggregate functions | 待创建 |

## 门禁状态

| 检查项 | 状态 |
|--------|------|
| cargo build | ✅ |
| cargo test | ✅ |
| cargo clippy | ⚠️ 需验证 |
| cargo fmt | ✅ |
| coverage (80%) | ⚠️ 需验证 |

## 风险记录

| PR | 风险级别 | 回滚方案 |
|----|----------|----------|
| #28 fmt+clippy | 低 | 回退到上一版本 |
| #29 unwrap处理 | 中 | 回退提交 |
| #30 coverage | 低 | 回退测试代码 |
| #31 aggregate | 高 | 保持 alpha 分支备份 |

## 更新日志

- 2026-02-20: 初始化任务看板
