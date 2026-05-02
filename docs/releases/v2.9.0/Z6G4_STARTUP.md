# Z6G4 OpenCode 启动指南 (紧急修正)

## 当前问题
Z6G4 当前在错误的分支（main），落后 develop/v2.9.0 50+ 提交。需要切换到正确的开发分支。

## 正确的启动流程

```bash
# 1. 切换到正确目录
cd /home/liying/dev/sqlrustgo

# 2. 拉取最新代码
git fetch origin develop/v2.9.0

# 3. 切换到开发分支
git checkout develop/v2.9.0

# 4. 或者创建新分支开发
git checkout -b feature/g01-r-gate-extend origin/develop/v2.9.0
```

## 你的任务 (来自 Issue #116, #117)

Phase G: 可信任治理体系
- G-01: R8 门禁 - SQL 兼容性不退化 (< 40.8%)
- G-02: R9 门禁 - 每个函数必须有测试用例
- G-03: R10 门禁 - 审计日志完整性
- G-04: 证明注册表系统升级
- G-05: AV10 形式化模型检查

Phase S: SQL 可证明性 (来自 Issue #117)
- S-01: 选择形式化证明框架
- S-02: 构建 5 条核心不变量
- S-03: 实现自动化证明检查
- S-04: 证明报告生成
- S-05: 集成到 CI

## 开发规范

1. 每次提交 PR 前运行:
   cargo test --all-features
   cargo clippy --all-features
   cargo fmt --check

2. PR 描述必须引用 Issue (Closes #116 等)

3. 进度报告更新到 Issue 评论

## 文档位置
- 开发计划: docs/releases/v2.9.0/DEVELOPMENT_PLAN.md
- 协作架构: docs/releases/v2.9.0/ORCHESTRATION.md