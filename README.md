# SQLRustGo

<p align="center">
  <img src="https://img.shields.io/badge/Rust-1.75+-dea584?style=flat-square&logo=rust" alt="Rust">
  <img src="https://img.shields.io/badge/version-1.0.0--beta-blue" alt="Version">
  <img src="https://img.shields.io/badge/coverage-82%25-green" alt="Coverage">
  <img src="https://img.shields.io/badge/license-MIT-green" alt="License">
</p>

> **一个用 Rust 从零实现的 SQL-92 子集兼容的关系型数据库系统**
> 
> 专为学习和研究数据库内核设计，同时具备生产级别的代码质量

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
| **v1.0.0-beta** | 当前 | 稳定性验证阶段 |
| v1.0.0-alpha | 已完成 | 功能开发完成 |
| v1.0.0 | 计划中 | 正式发布版本 |
| v2.0.0 | 规划中 | 架构演进版本 |

---

## 快速开始

```bash
# 克隆项目
git clone https://github.com/minzuuniversity/sqlrustgo.git
cd sqlrustgo

# 构建
cargo build --all-features

# 运行测试
cargo test --all-features

# 启动 REPL
cargo run --bin sqlrustgo
```

---

## 项目演进时间线

```
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
2026-02-19    Beta 阶段（当前）
              ├── 稳定性验证
              ├── 性能测试
              └── v2.0 架构规划
```

---

## 文档导航

### 按版本浏览

| 版本 | 文档入口 | 说明 |
|:-----|:---------|:-----|
| **v1.0** | [docs/v1.0/README.md](docs/v1.0/README.md) | 当前开发版本，Beta 阶段 |
| **v2.0** | [docs/v2.0/README.md](docs/v2.0/README.md) | 架构演进规划 |

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

```
docs/
├── 项目诞生记.md                ← 项目故事（推荐先读）
├── 文档阅读指南.md              ← 文档导航
├── 项目演进说明.md              ← 项目演进历史
│
├── VERSION_PROMOTION_SOP.md     ← 版本推进流程
├── VERSION_FLOW_DIAGRAM.md      ← 版本流转图
├── AI增强软件工程/              ← AI 协作治理（已移入）
│   ├── AI_AGENT_COLLAB_GOVERNANCE.md
│   └── AI协作开发教程.md
│
├── v1.0/                        ← v1.0 版本文档
│   ├── README.md               ← v1.0 文档索引
│   ├── alpha/                  ← Alpha 阶段
│   ├── 草稿/                   ← 早期草稿
│   ├── 草稿计划/               ← 早期计划
│   ├── 评估改进/               ← 评估报告
│   └── ...
│
└── v2.0/                        ← v2.0 规划文档
    ├── README.md               ← v2.0 文档索引
    ├── WHITEPAPER.md           ← 白皮书
    ├── 成熟度评估/             ← 成熟度模型
    ├── 架构设计/               ← 架构设计
    ├── 性能优化/               ← 性能优化
    └── 重构计划/               ← 重构计划
```

---

## 测试覆盖

| 指标 | 当前值 | Beta 目标 | Release 目标 |
|:-----|:-------|:----------|:-------------|
| 行覆盖率 | 82.24% | 85% | 90% |
| 函数覆盖率 | 84.73% | 87% | 92% |
| 测试数量 | 118+ | 130+ | 150+ |

---

## 技术栈

- **语言**：Rust Edition 2024
- **异步运行时**：Tokio
- **错误处理**：thiserror
- **序列化**：serde

---

## 参与贡献

欢迎提交 PR！请参考：

- [AI 协作开发指南](docs/v1.0/草稿计划/2026-02-16-ai-collaboration-guide.md)
- [分支管理策略](docs/v1.0/草稿计划/2026-02-16-branch-strategy.md)
- [PR 工作流程](docs/v1.0/草稿计划/2026-02-16-pr-workflow.md)

---

## 联系方式

- GitHub: https://github.com/minzuuniversity/sqlrustgo
- Issues: https://github.com/minzuuniversity/sqlrustgo/issues

---

*如果你对这个项目感兴趣，欢迎 star、fork、pr。让我们一起探索 AI 时代软件工程的新可能。*
