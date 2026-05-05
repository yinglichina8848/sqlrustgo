# Z440 OpenCode 启动指南

## 正确的启动流程

```bash
# 找到你的代码目录
cd /home/liying/dev/sqlrustgo

# 拉取最新代码
git fetch origin develop/v2.9.0

# 切换到开发分支
git checkout develop/v2.9.0

# 或者创建新分支开发
git checkout -b feature/c01-corpus-80pct origin/develop/v2.9.0
```

## 你的任务 (来自 Issue #118, #119)

Phase C: SQL 兼容性 (Issue #118)
- C-01: SQL Corpus 40.8% → 80%
- C-02: CTE (Common Table Expressions)
- C-03: JSON/JSONB 支持
- C-04: 窗口函数 (ROW_NUMBER, RANK, etc)
- C-05: 复杂 GROUP BY
- C-06: 嵌套查询优化

Phase D: 分布式增强 (Issue #119)
- D-01: 半同步复制
- D-02: 并行复制
- D-03: 2PC 分布式事务
- D-04: 自动故障转移

## 开发规范

1. 每次提交 PR 前运行:
   cargo test --all-features
   cargo clippy --all-features  
   cargo fmt --check

2. PR 描述必须引用 Issue (Closes #118, #119)

3. 进度报告更新到 Issue 评论

## 文档位置
- 开发计划: docs/releases/v2.9.0/DEVELOPMENT_PLAN.md
- 协作架构: docs/releases/v2.9.0/ORCHESTRATION.md