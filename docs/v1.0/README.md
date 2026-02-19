# SQLRustGo v1.0 文档目录

> 版本：v1.0.0-beta
> 日期：2026-02-19
> 状态：Beta 阶段（稳定性验证）

---

## 一、v1.0 版本概述

SQLRustGo v1.0 是一个用 Rust 从零实现的 SQL-92 子集兼容的关系型数据库系统。

### 1.1 核心特性

| 特性 | 说明 |
|------|------|
| **SQL 支持** | SELECT, INSERT, UPDATE, DELETE, CREATE TABLE, DROP TABLE |
| **存储引擎** | Buffer Pool + FileStorage 持久化存储 |
| **索引结构** | B+ Tree 索引支持 |
| **事务机制** | Write-Ahead Log (WAL) 保障事务安全 |
| **网络协议** | MySQL 风格协议支持 TCP 连接 |
| **交互式 REPL** | 支持交互式 SQL 命令行 |

### 1.2 技术栈

- Rust Edition 2024
- Tokio 异步运行时
- thiserror 错误处理
- serde 序列化

### 1.3 快速开始

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

### 1.4 架构概览

```
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

### 1.5 项目结构

```
sqlrustgo/
├── Cargo.toml
├── README.md
├── .github/workflows/
├── docs/
│   ├── architecture.md       ← 架构设计文档
│   ├── v1.0/                 ← v1.0 文档
│   └── v2.0/                 ← v2.0 规划
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

---

## 二、版本阶段

### 2.1 Alpha 阶段（已完成）

- 目标：功能开发完成
- 质量门禁：测试覆盖率 > 70%
- 详细文档：[alpha/README.md](alpha/README.md)

### 2.2 Beta 阶段（当前）

- 目标：稳定性验证、性能测试
- 质量门禁：测试覆盖率 > 85%
- 网络功能完善

### 2.3 Release 阶段（计划中）

- 目标：发布候选版本
- 质量门禁：无 blocker bug，文档完整

---

## 三、文档目录结构

```
docs/v1.0/
├── README.md                   # 本文档
├── alpha/                      # Alpha 阶段文档
│   └── README.md              # Alpha 目标与质量门禁
├── 草稿/                       # 早期草稿文档
├── 草稿计划/                   # 早期计划文档
├── 评估改进/                   # 评估报告
├── dev/                        # 开发文档
├── meeting-notes/              # 会议记录
├── 对话记录.md                 # 关键对话记录
├── 小龙虾的群聊记录.md
└── 飞书-龙虾群聊记录.md
```

---

## 四、文档分类

### 4.1 Alpha 阶段文档

| 文档 | 说明 |
|:-----|:-----|
| [alpha/README.md](alpha/README.md) | Alpha 阶段目标、质量门禁、验收口径 |

### 4.2 草稿文档

早期开发过程中的原始文档，保留了项目演进的完整轨迹：

| 文档 | 说明 |
|:-----|:-----|
| [阶段性工作报告.md](草稿/阶段性工作报告.md) | 开发历程总结 |
| [改进计划.md](草稿/改进计划.md) | 改进计划 |
| [2026-02-13-sqlcc-rust-redesign-design.md](草稿/2026-02-13-sqlcc-rust-redesign-design.md) | 设计文档 |

### 4.3 草稿计划

开发过程中制定的各类计划文档：

| 文档 | 说明 |
|:-----|:-----|
| [2026-02-16-ai-collaboration-guide.md](草稿计划/2026-02-16-ai-collaboration-guide.md) | AI 工具协作开发指南 |
| [2026-02-16-branch-strategy.md](草稿计划/2026-02-16-branch-strategy.md) | 分支管理策略 |
| [2026-02-16-version-evolution-plan.md](草稿计划/2026-02-16-version-evolution-plan.md) | 版本演化规划 |
| [2026-02-16-test-coverage-impl-plan.md](草稿计划/2026-02-16-test-coverage-impl-plan.md) | 测试覆盖率提升计划 |

### 4.4 评估改进

项目各维度的评估报告：

| 文档 | 说明 |
|:-----|:-----|
| [综合改进计划.md](评估改进/综合改进计划.md) | 问题汇总、改进措施 |
| [01-AI协作开发评估.md](评估改进/01-AI协作开发评估.md) | AI 协作评估 |
| [02-TDD开发流程评估.md](评估改进/02-TDD开发流程评估.md) | TDD 流程评估 |
| [03-代码审查评估.md](评估改进/03-代码审查评估.md) | 代码审查评估 |

### 4.5 对话记录

项目开发过程的真实记录：

| 文档 | 说明 |
|:-----|:-----|
| [对话记录.md](对话记录.md) | 项目创建过程中的关键对话 |
| [飞书-龙虾群聊记录.md](飞书-龙虾群聊记录.md) | 飞书群聊记录 |

---

## 五、阅读建议

### 新用户 / 学生

```
1. README.md (项目根目录) → 了解项目背景
2. alpha/README.md → 了解当前阶段目标
3. 草稿/阶段性工作报告.md → 了解项目历程
```

### 贡献者

```
1. 草稿计划/2026-02-16-ai-collaboration-guide.md → 学习 AI 辅助开发
2. 草稿计划/2026-02-16-branch-strategy.md → 理解分支策略
3. dev/DEVELOP.md → 开发环境和规范
```

---

## 六、版本状态

| 指标 | 当前值 | 目标值 |
|:-----|:-------|:-------|
| 测试覆盖率 | ~82% | 85% (Beta) / 90% (Release) |
| 功能完整度 | 基础 SQL | SQL-92 子集 |
| 文档完整度 | 进行中 | 完整 |

---

*本文档由 TRAE (GLM-5.0) 创建和维护*
