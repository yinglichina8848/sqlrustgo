# Alpha 阶段文档

## 文档导航

### 入门指南

- **[文档阅读指南](../文档阅读指南.md)** - 项目文档结构和阅读路径指南
- **[项目演进说明](../项目演进说明.md)** - 从零开始的 AI 协作开发完整流程

### 原始对话记录

- **[TRAE-文档改进和补全对话](../TRAE-文档改进和补全对话.md)** - 文档分析、组织和 AI 方法论
- **[对话记录](../对话记录.md)** - 项目创建过程中的关键对话
- **[飞书群聊记录](../飞书-龙虾群聊记录.md)** - 多 Agent 协作的实时沟通

## 阶段目标

完成 SQLRustGo v1.0.0-alpha 版本开发，实现一个轻量级 SQL 数据库引擎的核心功能。

### 功能目标

- [x] 基础 SQL 解析 (SELECT, INSERT, UPDATE, DELETE)
- [x] B+ Tree 索引
- [x] Volcano 执行引擎
- [x] 事务管理 (BEGIN, COMMIT, ROLLBACK)
- [x] WAL 日志
- [x] MySQL 协议基础 (握手包、OK包、ERR包)
- [x] ORDER BY 解析支持
- [x] LIMIT 解析支持

### 版本信息

- 版本号: v1.0.0-alpha
- 分支: feature/v1.0.0-alpha

---

## 质量门禁

所有 Pull Request 必须通过以下检查才能合并：

| 检查项 | 命令 | 状态要求 |
|:-------|:-----|:---------|
| 编译 | `cargo build --all-features` | 通过 |
| 测试 | `cargo test --all-features` | 全部通过 |
| Clippy | `cargo clippy --all-features -- -D warnings` | 无警告 |
| 格式化 | `cargo fmt --check` | 通过 |
| 覆盖率 | `cargo tarpaulin --all-features` | ≥ 80% |

### 当前状态 (2026-02-18)

| 检查项 | 状态 | 备注 |
|:-------|:-----|:-----|
| 编译 | ❌ 待修复 | - |
| 测试 | ✅ 通过 | 13 个测试 |
| Clippy | ❌ 19 errors | 格式问题 |
| 格式化 | ❌ 需修复 | 多处格式问题 |
| 覆盖率 | ⚠️ 未知 | 需运行 tarpaulin |

---

## 验收口径

### 功能验收

1. **SQL 解析**: 支持标准 SQL 语法的 SELECT/INSERT/UPDATE/DELETE
2. **索引**: B+ Tree 索引支持 PRIMARY KEY 和 INDEX 语句
3. **事务**: 支持 ACID 事务 (BEGIN/COMMIT/ROLLBACK)
4. **持久化**: WAL 日志确保数据不丢失
5. **网络协议**: MySQL 协议兼容，可通过 mysql client 连接

### 代码质量验收

1. **无 unwrap 滥用**: 关键路径必须使用合理的错误处理
2. **测试覆盖**: 核心模块覆盖率 ≥ 80%
3. **文档**: 关键 API 有文档注释
4. **无安全漏洞**: 通过安全扫描

### 发布验收

- [ ] 所有门禁检查通过
- [ ] 功能测试覆盖所有声明的功能
- [ ] 无已知 Critical/Bug 级别问题
- [ ] Release Notes 已准备

---

## 任务分配

| 角色 | GitHub ID | 任务 |
|:-----|:-----------|:-----|
| claude code | sonaheartopen | 测试增强 |
| opencode | sonaopenheart | 文档与流程证据 |
| codex-cli | yinglichina8848 | 验收与门禁 |
| OpenClaw | 高小原 | 调度与决策 |
