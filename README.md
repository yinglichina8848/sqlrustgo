# SQLRustGo

<<<<<<< HEAD
<p align="center">
  <img src="https://img.shields.io/badge/Rust-1.85+-dea584?style=flat-square&logo=rust" alt="Rust">
  <img src="https://img.shields.io/badge/version-1.1.0--draft-blue" alt="Version">
  <img src="https://img.shields.io/badge/coverage-94.18%25-brightgreen" alt="Coverage">
  <img src="https://img.shields.io/badge/maturity-L3%20Product%20Ready-green" alt="Maturity">
  <img src="https://img.shields.io/badge/license-MIT-green" alt="License">
</p>
=======
Rust 实现的关系型数据库系统，支持 SQL-92 子集。
>>>>>>> origin/main

## 核心特性

<<<<<<< HEAD
---

## 写在前面

你好，欢迎来到 SQLRustGo！

这是一个**教学演示型项目**，也是一个关于 AI 与软件工程相遇的故事。

如果你好奇：一个老师如何用 AI 辅助工具，在短短几天内从零开始构建一个关系型数据库系统？这个项目就是答案。

**完整故事**：[项目诞生记](docs/项目诞生记.md) —— 从 DeepSeek 探索到 Mac mini 重生，见证 AI 时代的软件工程新可能。

---

## 给不同读者的建议

### 🎯 你是谁？

| 读者类型 | 建议阅读路径 | 预计时间 |
|:---------|:-------------|:---------|
| **外行/好奇者** | [项目诞生记](docs/项目诞生记.md) → 本文档 | 15 分钟 |
| **学生/初学者** | [文档阅读指南](docs/文档阅读指南.md) → [v1.0 文档](docs/v1.0/README.md) | 1-2 小时 |
| **开发者** | [开发文档](docs/v1.0/dev/DEVELOP.md) → [AI 协作指南](docs/v1.0/草稿计划/2026-02-16-ai-collaboration-guide.md) | 2-3 小时 |
| **架构师** | [v2.0 白皮书](docs/v2.0/WHITEPAPER.md) → [插件架构设计](docs/v2.0/架构设计/PLUGIN_ARCHITECTURE.md) | 3-4 小时 |
| **管理者** | [版本推进流程](docs/VERSION_PROMOTION_SOP.md) → [AI 协作治理](docs/AI增强软件工程/AI_AGENT_COLLAB_GOVERNANCE.md) | 1 小时 |

---

## 项目简介

SQLRustGo 是一个用 Rust 从零实现的 SQL-92 子集兼容的关系型数据库系统。

### 这个项目适合谁？

- **学生**：学习数据库内核实现，理解 SQL 解析、存储引擎、事务机制
- **开发者**：研究 Rust 系统编程，学习 AI 辅助开发流程
- **教师**：参考 AI 时代的软件工程教学方法
- **研究者**：探索多 Agent 协作、SDD/TDD 开发实践

### 核心特性

| 特性 | 说明 |
|------|------|
| **SQL 支持** | SELECT, INSERT, UPDATE, DELETE, CREATE TABLE, DROP TABLE |
| **存储引擎** | Buffer Pool + FileStorage 持久化存储 |
| **索引结构** | B+ Tree 索引支持 |
| **事务机制** | Write-Ahead Log (WAL) 保障事务安全 |
| **网络协议** | MySQL 风格协议支持 TCP 连接 |
| **交互式 REPL** | 支持交互式 SQL 命令行 |

---

## 版本状态

| 版本 | 状态 | 说明 |
|:-----|:-----|:-----|
| **v1.1.0-draft** | 当前 | L3 产品级，架构升级版本 |
| v1.0.0 | 已发布 | 稳定版本 |
| v1.2.0 | 计划中 | 性能优化阶段 |
| v2.0.0 | 规划中 | 分布式架构版本 |

---
=======
- **SQL-92 支持**: 支持 SELECT, INSERT, UPDATE, DELETE, CREATE TABLE, DROP TABLE 等常用语句
- **事务支持**: ACID 事务，通过 Write-Ahead Log (WAL) 实现
- **存储引擎**: Page 管理 + BufferPool 缓存 + B+ Tree 索引
- **网络支持**: TCP 服务器，支持多客户端连接
- **REPL 交互**: 交互式命令行界面
>>>>>>> origin/main

## 快速开始

```bash
# 构建
cargo build --all-features

# 运行测试
cargo test --all-features

# 启动 REPL
cargo run --bin sqlrustgo

# Lint 检查
cargo clippy --all-features -- -D warnings
```

## 架构概览

```
<<<<<<< HEAD
2026-02-13    项目启动，从 SQLCC 重构为 SQLRustGo
    │
    ▼
2026-02-14    完成核心架构设计
    │
    ▼
2026-02-16    Alpha 阶段：功能开发
    │         ├── SQL Parser 实现
    │         ├── 存储引擎实现
    │         ├── 事务支持 (WAL)
    │         └── 网络协议支持
    │
    ▼
2026-02-18    Alpha 完成，进入 Beta
    │         ├── 测试覆盖率 82%
    │         ├── 文档整理
    │         └── 版本推进流程建立
    │
    ▼
2026-02-20    v1.0.0 Release Candidate
    │         ├── 稳定性验证
    │         └── 安全审计
    │
    ▼
2026-03-03    v1.1.0 Draft 发布 (当前)
              ├── L2 → L3 成熟度升级
              ├── LogicalPlan/PhysicalPlan 分离
              ├── ExecutionEngine 插件化
              ├── HashJoin 实现
              ├── 测试覆盖率 94.18%
              └── Client-Server 架构
```

---

## 文档导航

### 按版本浏览

| 版本 | 文档入口 | 说明 |
|:-----|:---------|:-----|
| **v1.1.0** | [docs/releases/v1.1.0/](docs/releases/v1.1.0/) | 当前版本，L3 产品级 |
| **v1.0** | [docs/v1.0/README.md](docs/v1.0/README.md) | 稳定版本 |
| **v2.0** | [docs/v2.0/README.md](docs/v2.0/README.md) | 架构演进规划 |

### v1.1.0 文档导航

| 分类 | 文档 | 说明 |
|:-----|:-----|:-----|
| **发布文档** | [Release Notes](docs/releases/v1.1.0/RELEASE_NOTES.md) | 版本发布说明 |
| | [CHANGELOG](CHANGELOG.md) | 变更日志 |
| | [门禁检查清单](docs/releases/v1.1.0/RELEASE_GATE_CHECKLIST.md) | 发布门禁 |
| **技术文档** | [API 文档](docs/releases/v1.1.0/API_DOCUMENTATION.md) | API 参考 |
| | [升级指南](docs/releases/v1.1.0/UPGRADE_GUIDE.md) | v1.0.0 → v1.1.0 |
| | [性能报告](docs/releases/v1.1.0/PERFORMANCE_REPORT.md) | 性能测试分析 |
| **规范文档** | [日志规范](docs/releases/v1.1.0/LOGGING_SPECIFICATION.md) | 日志格式规范 |
| | [监控规范](docs/releases/v1.1.0/MONITORING_SPECIFICATION.md) | 性能监控规范 |
| | [健康检查规范](docs/releases/v1.1.0/HEALTH_CHECK_SPECIFICATION.md) | 健康检查端点 |

### v1.0 文档导航

| 分类 | 文档 | 说明 |
|:-----|:-----|:-----|
| **阶段文档** | [Alpha 阶段](docs/v1.0/alpha/README.md) | Alpha 目标与质量门禁 |
| **草稿计划** | [AI 协作指南](docs/v1.0/草稿计划/2026-02-16-ai-collaboration-guide.md) | AI 工具协作开发指南 |
| | [分支策略](docs/v1.0/草稿计划/2026-02-16-branch-strategy.md) | 分支管理策略 |
| **评估改进** | [综合改进计划](docs/v1.0/评估改进/综合改进计划.md) | 问题汇总、改进措施 |
| **对话记录** | [对话记录](docs/v1.0/对话记录.md) | 项目创建过程中的关键对话 |

### v2.0 规划导航

| 分类 | 文档 | 说明 |
|:-----|:-----|:-----|
| **白皮书** | [2.0 白皮书](docs/v2.0/WHITEPAPER.md) | SQLRustGo 2.0 架构愿景 |
| **成熟度** | [成熟度模型](docs/v2.0/成熟度评估/MATURITY_MODEL.md) | L0-L4 定义 |
| **架构** | [插件架构](docs/v2.0/架构设计/PLUGIN_ARCHITECTURE.md) | 插件化执行架构 |
| **性能** | [向量化执行](docs/v2.0/性能优化/VECTORIZED_EXECUTION.md) | 向量化执行模型 |
| **重构** | [L3 升级计划](docs/v2.0/重构计划/L3_UPGRADE_PLAN.md) | L2→L3 升级路径 |

---

## 文档体系
=======
┌─────────────────────────────────────┐
│           main.rs (REPL)             │
├─────────────────────────────────────┤
│           executor/                 │  ← 查询执行
├─────────────────────────────────────┤
│           parser/                    │  ← SQL → AST
│           lexer/                    │  ← SQL → Tokens
├─────────────────────────────────────┤
│           storage/                   │  ← Page, BufferPool, B+ Tree
├─────────────────────────────────────┤
│         transaction/                 │  ← WAL, TxManager
├─────────────────────────────────────┤
│           network/                   │  ← TCP Server/Client
├─────────────────────────────────────┤
│           types/                     │  ← Value, SqlError
└─────────────────────────────────────┘
```

## 项目结构
>>>>>>> origin/main

```
sqlrustgo/
├── Cargo.toml
├── README.md
├── .github/workflows/
├── docs/
│   └── architecture.md       ← 架构设计文档
├── src/
│   ├── main.rs              ← REPL 入口
│   ├── lib.rs               ← 库入口
│   ├── lexer/               ← 词法分析
│   ├── parser/              ← 语法分析
│   ├── executor/            ← 查询执行
│   ├── storage/             ← 存储层
│   ├── transaction/         ← 事务管理
│   ├── network/             ← 网络通信
│   └── types/               ← 类型系统
└── tests/                   ← 集成测试
```

## 功能特性

- ✅ SQL-92 子集支持 (SELECT, INSERT, UPDATE, DELETE)
- ✅ 存储引擎 (Buffer Pool, FileStorage)
- ✅ B+ Tree 索引
- ✅ 事务支持 (WAL)
- ✅ 网络协议支持

## 文档

- [架构设计](docs/architecture.md)
- [设计文档](docs/2026-02-13-sqlcc-rust-redesign-design.md)
- [实施计划](docs/2026-02-13-sqlcc-rust-impl-plan.md)

## 测试覆盖

<<<<<<< HEAD
<<<<<<< HEAD
=======
>>>>>>> origin/develop-v1.2.0
| 指标 | 当前值 | v1.1.0 目标 | 状态 |
|:-----|:-------|:------------|:-----|
| 行覆盖率 | 94.18% | ≥90% | ✅ 达标 |
| 函数覆盖率 | 93.81% | ≥85% | ✅ 达标 |
| 区域覆盖率 | 93.54% | ≥85% | ✅ 达标 |
| 测试数量 | 120+ | 100+ | ✅ 达标 |

---
=======
- 测试数量: 118+ 个
- 行覆盖率: 82.24%
- 函数覆盖率: 84.73%
>>>>>>> origin/main

## 技术栈

- Rust Edition 2024
- Tokio 异步运行时
- thiserror 错误处理
- serde 序列化

## 参与贡献

欢迎提交 PR！请参考 GitHub 上的 Issue 和 PR 列表。

---

## 分支结构 (v1.2.0 Alpha)

```
main                    - 最终发布版本
alpha/v1.2.0           - Alpha 开发版本 (当前)
beta/v1.1.0            - Beta 测试版本 (冻结)
release/v1.1.0         - 发布版本 (冻结)
develop/v1.2.0         - 开发分支 (当前)
```

详细分支规范见 [BRANCH_GOVERNANCE.md](docs/BRANCH_GOVERNANCE.md)

## 版本阶段状态

| 阶段 | 版本 | 状态 |
|------|------|------|
| Draft | v1.0.x | ✅ 已完成 |
| Alpha | v1.1.x | ✅ 已完成 |
| Craft | v1.2.0 | ✅ 已完成 |
| Alpha | v1.2.0 | 🔄 进行中 |
| Beta | v1.2.x | ⏳ 待启动 |

**注意**: 项目正处于 Alpha 阶段，API 可能有变更。
