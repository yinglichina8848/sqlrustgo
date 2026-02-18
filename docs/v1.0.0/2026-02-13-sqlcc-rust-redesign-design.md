# SQLCC Rust 重构项目设计文档

> **项目代号**: SQLCC-Rust-Redesign
> **版本**: v1.0.0
> **日期**: 2026-02-13
> **状态**: 设计评审中

---

## 1. 项目概述

### 1.1 背景与目标

SQLCC 是一个声称支持 SQL-92 标准的 C++20 数据库系统，但存在以下严重问题：

- **代码质量**: AI 生成代码，存在大量编译错误和逻辑缺陷
- **架构问题**: 模式设计不合理，模块耦合严重
- **无法交付**: 整体无法编译通过，测试无法运行
- **文档缺失**: 设计决策不清晰，难以维护和演进

**核心目标**:

| 目标 | 指标 |
|------|------|
| 编译状态 | 100% 编译通过，无错误 |
| 测试覆盖 | >= 80% |
| 性能 | 较原系统提升 >= 50% |
| 文档 | SDD + 用户手册 + 开发演进记录 |
| 可扩展 | 预留 SQL-99/SQL-2023 扩展接口 |

### 1.2 教学定位

本项目作为**数据库原理 + 软件工程导论**课程的实践载体，通过以下方式配合教学：

- **渐进式开发**: 每周任务对应课程知识点
- **AI辅助编程**: Claude Code、GitHub Copilot 等工具的规范使用
- **代码即文档**: 详细注释 + 设计文档
- **协作学习**: ISSUE + PR 工作流 + Code Review
- **演进记录**: 完整的过程文档供后续学生学习

### 1.3 AI工具链定位

> **核心理念**: AI是副驾驶，不是驾驶员

本课程将系统性地教授 **AI辅助软件工程** 的方法论，重点是 **多Agent协同工作**：

| 工具类别 | 工具 | 教学定位 | 使用阶段 |
|:---------|:-----|:--------:|:--------:|
| **多Agent协作** | OpenClaw | 主多Agent框架，执行复杂任务 | 全流程 |
| **规范控制** | OpenSpec | SDD/TDD 规范控制 CLI | 设计/测试 |
| **Agent团队** | Agent Team | Claude Code 多Agent并行 | 开发/审查 |
| **AI编程助手** | Claude Code | 主AI工具，全流程使用 | 全程 |
| **代码补全** | GitHub Copilot | 实时补全，语法提示 | 编码阶段 |
| **AI审查** | CodeRabbit AI | 自动化代码审查 | PR阶段 |
| **IDE** | VS Code / Zed | 主开发环境 | 全程 |
| **CLI工具** | uv / ripgrep / fd | 高效命令行操作 | 全程 |
| **CI/CD** | GitHub Actions | 自动化流水线 | 集成阶段 |
| **文档工程** | mdBook / rustdoc | 自动化文档生成 | 文档阶段 |

#### 多Agent协同架构

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                           多 Agent 协同工作流                                          │
├─────────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                      │
│   用户需求                                                                         │
│       │                                                                           │
│       ▼                                                                           │
│   ┌─────────────────────────────────────────────────────────────────────────────┐   │
│   │                        OpenSpec 规范控制层                                    │   │
│   │   ────────────────────────────────────────────────────────────────────────  │   │
│   │   · SDD (软件设计描述) 规范验证                                             │   │
│   │   · TDD (测试驱动开发) 规范验证                                              │   │
│   │   · 接口契约检查                                                             │   │
│   │   · 代码风格规范                                                              │   │
│   └─────────────────────────────────────────────────────────────────────────────┘   │
│       │                                                                           │
│       ▼                                                                           │
│   ┌─────────────────────────────────────────────────────────────────────────────┐   │
│   │                        OpenClaw 多Agent执行层                                   │   │
│   │   ────────────────────────────────────────────────────────────────────────  │   │
│   │                                                                            │   │
│   │   ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │   │
│   │   │  Architect  │  │  Developer  │  │   Tester    │  │  Reviewer   │     │   │
│   │   │   Agent    │──►│   Agent    │──►│   Agent    │──►│   Agent    │     │   │
│   │   └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘     │   │
│   │                                                                            │   │
│   └─────────────────────────────────────────────────────────────────────────────┘   │
│       │                                                                           │
│       ▼                                                                           │
│   ┌─────────────────────────────────────────────────────────────────────────────┐   │
│   │                        Agent Team 并行协调层                                    │   │
│   │   ────────────────────────────────────────────────────────────────────────  │   │
│   │   · 任务分发                                                                  │   │
│   │   · 结果聚合                                                                  │   │
│   │   · 冲突解决                                                                  │   │
│   │   · 进度追踪                                                                  │   │
│   └─────────────────────────────────────────────────────────────────────────────┘   │
│       │                                                                           │
│       ▼                                                                           │
│   人类审批 / 合并到主分支                                                          │
│                                                                                      │
└─────────────────────────────────────────────────────────────────────────────────────────┘
```

#### AI使用原则

```
✓ AI 可以做的：
  · 生成样板代码、测试用例
  · 解释复杂代码逻辑
  · 辅助调试、定位问题
  · 生成文档、注释
  · 代码重构建议

✗ AI 不能做的（必须人工确认）：
  · 架构设计决策（需要人类专家判断）
  · 安全敏感代码（密码、加密）
  · 性能关键路径（需要 profiling 验证）
  · 业务逻辑正确性（需要人类理解需求）
```

#### 多Agent工具使用模式

| 工具 | 模式 | 命令示例 | 使用场景 |
|:-----|:-----|:--------|:---------|
| **OpenClaw** | 多Agent执行 | `/openclaw --task parser` | 执行复杂多步骤任务 |
| **OpenSpec** | 规范验证 | `/openspec validate sdd` | 验证设计规范 |
| **Agent Team** | 并行开发 | `/agent-team --parallel` | 多任务并行处理 |
| **Claude Code** | 任务执行 | `/task 实现BufferPool` | 单一任务执行 |

---

## 2. 技术架构决策

### 2.1 技术栈选择

#### 语言: Rust

**理由**:

1. **内存安全**: 消除 C++ 内存安全问题（野指针、内存泄漏）
2. **性能**: 与 C++ 持平，适合高性能数据库场景
3. **现代包管理**: Cargo 比 Bazel 更简洁、更适合教学
4. **生态系统**: Apache Arrow、DataFusion 等成熟库
5. **教学价值**: 学习现代系统编程语言

**配套措施**:

- 《Rust 快速入门教程》（配套数据库场景 + AI辅助）
- 每行代码都有详细注释
- Rust 安全模式数据库应用专版
- AI生成的代码审查清单

#### 构建系统: Cargo

**理由**:

1. **简单易用**: 比 Bazel 更适合教学
2. **依赖管理**: Cargo.toml 声明式依赖
3. **工作空间**: 原生支持多模块项目
4. **测试集成**: `cargo test` 一键测试

#### 开发工具链

```bash
# ============================================================
# 学期初一次性配置 - 开发工具链
# ============================================================

# Rust 工具链
rustup install stable
rustup component add rust-src rust-analysis rust-analyzer

# Cargo 增强工具
cargo install cargo-watch     # 文件监控自动构建
cargo install cargo-expand    # 宏展开查看
cargo install cargo-flamegraph # 性能火焰图
cargo install cargo-audit     # 安全审计
cargo install cargo-deny      # 依赖检查
cargo install cargo-nextest   # 更快测试

# CLI 增强工具
brew install ripgrep         # 高效搜索 (rg)
brew install fd              # 文件查找 (fd)
brew install bat             # 高亮 cat (bat)
brew install exa            # 增强 ls (exa)
brew install fzf             # 模糊搜索
brew install lazygit         # Git TUI
brew install htop btop       # 系统监控
brew install jq yq           # JSON/YAML 处理
brew install docker kubectl  # 容器 & K8s

# Git 工具
brew install git-delta       # 增强 diff
brew install gh             # GitHub CLI
```

#### VS Code 必备配置

```json
{
  "editor.formatOnSave": true,
  "editor.rust-analyzer.checkOnSave.command": "clippy",
  "github.copilot.enable": { "rust": true, "markdown": true }
}
```

#### SQL Parser: ANTLR4 或 rust-arctic

| 方案 | 优点 | 缺点 |
|------|------|------|
| ANTLR4 | 成熟的语法树生成，SQL 语法规则丰富 | 需要安装 Java 运行时 |
| rust-arctic | 纯 Rust 实现，无外部依赖 | 语法规则较少 |

**推荐**: ANTLR4（利用现有 SQL 语法规则资源）

#### 执行引擎: Volcano Model

**迭代器模式**:

```
TableScan ──► Filter ──► Project ──► HashJoin ──► Sort ──► Limit
   │            │           │           │           │         │
   └────────────┴───────────┴───────────┴───────────┴─────────┘
                         next() 调用链
```

**优点**:

1. 简洁统一: 所有算子实现相同接口
2. 易于优化: 便于代价模型选择执行策略
3. 并行友好: 易于实现并行执行

#### 存储格式: Apache Arrow + Parquet

**理由**:

1. **列式存储**: OLAP 友好，向量化执行基础
2. **生态系统**: 与 DataFusion、Polars 兼容
3. **类型系统**: 丰富的类型支持
4. **文件格式**: Parquet 高效压缩

#### 事务控制: MVCC + 2PL

| 组件 | 职责 |
|------|------|
| MVCC | 多版本并发控制，读写不阻塞 |
| 2PL | 两阶段锁，防止写写冲突 |
| WAL | Write-Ahead Log，恢复保证 |

#### 网络协议: gRPC + MySQL 兼容层

| 协议 | 用途 |
|------|------|
| gRPC | 内部服务通信、云原生接口 |
| MySQL Wire | 客户端兼容性（mysql-client 可直接连接） |

### 2.2 架构分层

```
┌─────────────────────────────────────────────────────────────────┐
│                     API / CLI 层                                 │
├────────────────┬──────────────────┬─────────────────────────────┤
│   MySQL Wire   │     gRPC API     │        REST API              │
│   Protocol     │   (管理接口)      │       (监控/运维)            │
└────────────────┴──────────────────┴─────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────────┐
│                    查询处理层 (Query Processing)                 │
├─────────────────────────────────────────────────────────────────┤
│   ┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────────┐  │
│   │ Parser  │───►│Analyzer │───►│Optimizer│───►│  Executor   │  │
│   └─────────┘    └─────────┘    └─────────┘    │(Volcano)    │  │
│        │            │             │              └─────────────┘  │
│        │            │             │                     │          │
│   SQL 语法树     语义分析      执行计划               算子树执行     │
└─────────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────────┐
│                    存储引擎层 (Storage Engine)                    │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌────────────────┐   │
│  │Catalog   │  │BufferPool│  │ Index    │  │  Storage       │   │
│  │Manager   │  │Manager   │  │ Manager  │  │  (Arrow/Parquet)│   │
│  └──────────┘  └──────────┘  └──────────┘  └────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────────┐
│                   事务与并发控制层                                 │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌────────────────┐   │
│  │Lock      │  │MVCC      │  │WAL       │  │Transaction     │   │
│  │Manager   │  │Manager   │  │Manager   │  │Manager         │   │
│  └──────────┘  └──────────┘  └──────────┘  └────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────────┐
│                    可观测性层 (Observability)                     │
├─────────────────────────────────────────────────────────────────┤
│   OpenTelemetry │ Prometheus Metrics │ Distributed Tracing      │
└─────────────────────────────────────────────────────────────────┘
```

### 2.3 可扩展架构设计

#### SQL 功能扩展点

```
┌─────────────────────────────────────────────────────────────────┐
│                      扩展接口层                                   │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │  FunctionRegistry│  │TypeSystem      │ │DialectParser    │ │
│  │  (函数扩展)       │  │(类型扩展)       │ │(方言扩展)        │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │ExpressionBuilder│  │OptimizerRule   │ │PhysicalOperator │ │
│  │(表达式构建)      │  │(优化规则)       │ │(物理算子)        │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────────┐
│                    SQL 标准支持路线图                              │
├─────────────────────────────────────────────────────────────────┤
│  v1.0 (3月)     │ v2.0 (期中)    │ v3.0 (期末)   │ v4.0 (未来)   │
│  ──────────────┼────────────────┼────────────────┼─────────────│
│  SQL-92 子集    │ SQL-99        │ SQL-2003      │ SQL-2023      │
│  - CREATE TABLE │ - 窗口函数     │ - 递归 CTE    │ - 向量搜索    │
│  - SELECT       │ - CTE         │ - JSON        │ - 图查询      │
│  - INSERT       │ - GROUPING    │ - XML         │ - 多模态      │
│  - UPDATE       │ - ROLLUP      │               │               │
│  - DELETE       │               │               │               │
└─────────────────────────────────────────────────────────────────┘
```

#### 云原生扩展点

```
┌─────────────────────────────────────────────────────────────────┐
│                    容器化/云原生准备                               │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   [配置外部化]        [健康检查]         [指标暴露]               │
│   ───────────        ─────────        ─────────                │
│   env/config.yaml    /health/live     /metrics                 │
│   --config CLI       /health/ready    Prometheus                │
│                                                                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   [存储外部化]        [服务发现]         [可观测性]                │
│   ───────────        ─────────        ─────────                │
│   S3/云存储          K8s Service      OpenTelemetry             │
│   PV/PVC             DNS               Tracing                   │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## 3. 模块详细设计

### 3.1 目录结构

```
sqlcc-rust/
├── Cargo.toml                  # 工作空间配置
├── Cargo.lock                  # 依赖锁定
├── rust-toolchain.toml         # Rust 版本声明
│
├── apps/
│   ├── sqlcc-server/           # 服务器主程序
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.rs
│   │
│   ├── sqlcc-cli/             # CLI 客户端
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.rs
│   │
│   └── sqlcc-bench/           # 性能测试工具
│       ├── Cargo.toml
│       └── src/
│           └── main.rs
│
├── crates/
│   ├── sqlcc-core/            # 核心接口
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       └── error.rs
│   │
│   ├── sqlcc-parser/          # SQL 解析器
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── lexer.rs
│   │   │   ├── parser.rs
│   │   │   └── ast/
│   │   │       ├── mod.rs
│   │   │       ├── ddl.rs
│   │   │       ├── dml.rs
│   │   │       └── expr.rs
│   │   └── sql/
│   │       └── sql92.g4      # ANTLR4 语法文件
│   │
│   ├── sqlcc-analyzer/        # 语义分析器
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── binder.rs
│   │       └── resolver.rs
│   │
│   ├── sqlcc-optimizer/       # 查询优化器
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── planner.rs
│   │       └── rules/
│   │           ├── mod.rs
│   │           ├── predicate_pushdown.rs
│   │           └── projection_pushdown.rs
│   │
│   ├── sqlcc-executor/        # 执行引擎
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── volcano/
│   │       │   ├── mod.rs
│   │       │   ├── table_scan.rs
│   │       │   ├── filter.rs
│   │       │   ├── project.rs
│   │       │   ├── hash_join.rs
│   │       │   └── sort.rs
│   │       └── row_batch.rs
│   │
│   ├── sqlcc-storage/         # 存储引擎
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── buffer_pool.rs
│   │       ├── catalog.rs
│   │       ├── page.rs
│   │       ├── table/
│   │       │   ├── mod.rs
│   │       │   ├── row_format.rs
│   │       │   └── arrow_format.rs
│   │       └── index/
│   │           ├── mod.rs
│   │           └── bplus_tree.rs
│   │
│   ├── sqlcc-transaction/     # 事务管理
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── mvcc.rs
│   │       ├── lock_manager.rs
│   │       └── wal.rs
│   │
│   ├── sqlcc-network/         # 网络层
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── mysql/
│   │       │   ├── mod.rs
│   │       │   ├── protocol.rs
│   │       │   └── handler.rs
│   │       └── grpc/
│   │           ├── mod.rs
│   │           └── service.rs
│   │
│   └── sqlcc-proto/           # protobuf 定义
│       ├── Cargo.toml
│       └── src/
│           └── sqlcc.proto
│
├── docs/
│   ├── sdd/                    # 软件设计文档
│   │   ├── architecture.md
│   │   ├── module-design.md
│   │   └── interface-spec.md
│   │
│   ├── api/                    # API 文档
│   │   ├── rust-api/
│   │   └── grpc-api/
│   │
│   ├── user-guide/             # 用户手册
│   │   ├── getting-started.md
│   │   ├── sql-reference.md
│   │   └── administration.md
│   │
│   ├── tutorials/              # 教程
│   │   ├── rust-quick-start.md
│   │   ├── sqlcc-architecture.md
│   │   └── contribution-guide.md
│   │
│   └── development/            # 开发文档
│       ├── coding-standards.md
│       ├── testing-guide.md
│       └── release-process.md
│
├── tests/
│   ├── integration/            # 集成测试
│   │   └── src/
│   │       └── lib.rs
│   ├── e2e/                    # 端到端测试
│   │   └── src/
│   │       └── lib.rs
│   └── performance/            # 性能测试
│       └── src/
│           └── lib.rs
│
├── scripts/
│   ├── build.sh
│   ├── test.sh
│   ├── coverage.sh
│   └── docker-build.sh
│
├── Dockerfile
├── docker-compose.yml
├── k8s/
│   ├── deployment.yaml
│   └── service.yaml
│
├── .github/
│   └── workflows/
│       ├── ci.yml
│       └── cd.yml
│
├── README.md
├── CONTRIBUTING.md
└── LICENSE
```

### 3.2 核心模块设计

#### 3.2.1 SQL Parser (sqlcc-parser)

**职责**:

1. 词法分析 (Lexical Analysis)
2. 语法分析 (Syntax Analysis)
3. AST 生成

**接口设计**:

```rust
// sqlcc-parser/src/lib.rs

use crate::ast::{Statement, Expr, DataType};

pub struct Parser<'a> {
    /// 词法分析器
    lexer: Lexer<'a>,
    /// 当前 token
    current_token: Token,
    /// 预读 token
    peek_token: Token,
}

impl<'a> Parser<'a> {
    /// 解析单个 SQL 语句
    pub fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        match self.current_token {
            Token::CREATE => self.parse_create(),
            Token::SELECT => self.parse_select(),
            Token::INSERT => self.parse_insert(),
            Token::UPDATE => self.parse_update(),
            Token::DELETE => self.parse_delete(),
            _ => Err(ParseError::unexpected_token(&self.current_token)),
        }
    }

    /// 解析 SELECT 语句
    pub fn parse_select(&mut self) -> Result<Statement, ParseError> {
        // 1. 消费 SELECT
        self.next_token();

        // 2. 解析投影列
        let projections = self.parse_select_list()?;

        // 3. 解析 FROM 子句
        let from = self.parse_from_clause()?;

        // 4. 解析 WHERE 子句 (可选)
        let filter = if self.consume_if(Token::WHERE) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        // 5. 解析 GROUP BY 子句 (可选)
        let group_by = if self.consume_if(Token::GROUP) {
            self.expect(Token::BY)?;
            self.parse_group_by()?
        } else {
            None
        };

        // 6. 解析 HAVING 子句 (可选)
        let having = if self.consume_if(Token::HAVING) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        // 7. 解析 ORDER BY 子句 (可选)
        let order_by = if self.consume_if(Token::ORDER) {
            self.expect(Token::BY)?;
            self.parse_order_by()?
        } else {
            None
        };

        // 8. 解析 LIMIT 子句 (可选)
        let limit = self.parse_limit()?;

        Ok(Statement::Select {
            projections,
            from,
            filter,
            group_by,
            having,
            order_by,
            limit,
        })
    }

    /// 解析 CREATE TABLE 语句
    pub fn parse_create_table(&mut self) -> Result<Statement, ParseError> {
        // 1. 消费 CREATE
        self.next_token();

        // 2. 确认 TABLE
        self.expect(Token::TABLE)?;

        // 3. 解析表名
        let table_name = self.parse_table_name()?;

        // 4. 解析列定义
        self.expect(Token::LPAREN)?;
        let columns = self.parse_column_definitions()?;
        self.expect(Token::RPAREN)?;

        // 5. 解析表约束 (可选)
        let constraints = self.parse_table_constraints()?;

        Ok(Statement::CreateTable {
            name: table_name,
            columns,
            constraints,
        })
    }
}

/// SQL 抽象语法树
pub mod ast {
    /// SQL 语句
    pub enum Statement {
        /// CREATE TABLE, CREATE INDEX, etc.
        Create(CreateStatement),
        /// SELECT 语句
        Select(SelectStatement),
        /// INSERT 语句
        Insert(InsertStatement),
        /// UPDATE 语句
        Update(UpdateStatement),
        /// DELETE 语句
        Delete(DeleteStatement),
        /// 其他 DDL/DML 语句
        Other(Box<dyn StatementExt>),
    }

    /// SELECT 语句
    pub struct SelectStatement {
        /// 投影列
        pub projections: Vec<Projection>,
        /// FROM 子句
        pub from: Option<TableReference>,
        /// WHERE 条件
        pub filter: Option<Expr>,
        /// GROUP BY 子句
        pub group_by: Option<GroupByClause>,
        /// HAVING 条件
        pub having: Option<Expr>,
        /// ORDER BY 子句
        pub order_by: Option<OrderByClause>,
        /// LIMIT 子句
        pub limit: Option<LimitClause>,
    }

    /// CREATE 语句基类
    pub struct CreateStatement {
        /// 创建对象类型
        pub object_type: ObjectType,
        /// 对象名称
        pub name: ObjectName,
        /// 列定义 (仅 TABLE)
        pub columns: Vec<ColumnDef>,
        /// 表约束
        pub constraints: Vec<TableConstraint>,
    }

    /// 表达式
    pub enum Expr {
        /// 列引用
        Column(ColumnRef),
        /// 字面量
        Literal(Literal),
        /// 函数调用
        Function(FunctionCall),
        /// 二元运算
        BinaryOp {
            /// 运算符
            op: BinaryOperator,
            /// 左操作数
            left: Box<Expr>,
            /// 右操作数
            right: Box<Expr>,
        },
        /// 一元运算
        UnaryOp {
            /// 运算符
            op: UnaryOperator,
            /// 操作数
            expr: Box<Expr>,
        },
        /// 子查询
        Subquery(Box<SelectStatement>),
        /// CASE 表达式
        Case {
            /// 条件分支
            branches: Vec<(Expr, Expr)>,
            /// 默认分支
            else_expr: Option<Box<Expr>>,
        },
    }
}

/// 词法分析器
pub struct Lexer<'a> {
    /// 输入字符串
    input: &'a str,
    /// 当前位置
    position: usize,
    /// 读取位置
    read_position: usize,
    /// 当前字符
    ch: char,
}

impl<'a> Lexer<'a> {
    /// 从输入创建词法分析器
    pub fn new(input: &'a str) -> Self {
        let mut lexer = Self {
            input,
            position: 0,
            read_position: 0,
            ch: '\0',
        };
        lexer.read_char();
        lexer
    }

    /// 读取下一个字符
    fn read_char(&mut self) {
        if self.read_position >= self.input.len() {
            self.ch = '\0';
        } else {
            self.ch = self.input[self.read_position..].chars().next().unwrap();
        }
        self.position = self.read_position;
        self.read_position += self.ch.len_utf8();
    }

    /// 跳过空白字符
    fn skip_whitespace(&mut self) {
        while self.ch.is_whitespace() {
            self.read_char();
        }
    }

    /// 读取标识符或关键字
    fn read_identifier(&mut self) -> String {
        let start = self.position;
        while self.ch.is_alphanumeric() || self.ch == '_' {
            self.read_char();
        }
        self.input[start..self.position].to_string()
    }

    /// 读取数字
    fn read_number(&mut self) -> Result<Literal, ParseError> {
        let start = self.position;
        while self.ch.is_ascii_digit() || self.ch == '.' {
            self.read_char();
        }

        let num_str = &self.input[start..self.position];

        // 尝试解析为整数
        if let Ok(int_val) = num_str.parse::<i64>() {
            Ok(Literal::Integer(int_val))
        } else {
            // 解析为浮点数
            num_str.parse::<f64>()
                .map(Literal::Float)
                .map_err(|_| ParseError::invalid_number(num_str))
        }
    }

    /// 读取字符串
    fn read_string(&mut self) -> Result<Literal, ParseError> {
        let quote = self.ch;
        self.read_char(); // 消费开引号

        let start = self.position;
        while self.ch != quote && self.ch != '\0' {
            self.read_char();
        }

        let str_val = &self.input[start..self.position];

        // 消费闭引号
        if self.ch != '\0' {
            self.read_char();
        }

        Ok(Literal::String(str_val.to_string()))
    }

    /// 获取下一个 Token
    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        let token = match self.ch {
            // 单字符 Token
            '*' => Token::ASTERISK,
            '(' => Token::LPAREN,
            ')' => Token::RPAREN,
            ',' => Token::COMMA,
            ';' => Token::SEMICOLON,
            '.' => Token::DOT,
            '+' => Token::PLUS,
            '-' => Token::MINUS,
            '/' => Token::SLASH,
            // 多字符 Token
            '=' if self.peek_char() == '=' => {
                self.read_char();
                Token::EQ
            }
            '!' if self.peek_char() == '=' => {
                self.read_char();
                Token::NEQ
            }
            '<' if self.peek_char() == '=' => {
                self.read_char();
                Token::LTE
            }
            '<' if self.peek_char() == '>' => {
                self.read_char();
                Token::NEQ
            }
            '<' => Token::LT,
            '>' if self.peek_char() == '=' => {
                self.read_char();
                Token::GTE
            }
            '>' => Token::GT,
            // 字符串
            '\'' | '"' => {
                let lit = self.read_string()?;
                return Token::StringLiteral(lit);
            }
            // 数字
            '0'..='9' => {
                return self.read_number().unwrap_or(Token::ILLEGAL);
            }
            // 标识符或关键字
            'a'..='z' | 'A'..='Z' | '_' => {
                let ident = self.read_identifier();
                // 检查是否为关键字
                if let Some(keyword) = Keyword::from_str(&ident) {
                    keyword.to_token()
                } else {
                    Token::Identifier(ident)
                }
            }
            // 结束
            '\0' => Token::EOF,
            // 非法字符
            _ => Token::ILLEGAL,
        };

        self.read_char();
        token
    }

    /// 预览下一个字符
    fn peek_char(&self) -> char {
        if self.read_position >= self.input.len() {
            '\0'
        } else {
            self.input[self.read_position..].chars().next().unwrap()
        }
    }
}
```

#### 3.2.2 Analyzer (sqlcc-analyzer)

**职责**:

1. 符号表管理
2. 类型检查
3. 权限验证

**接口设计**:

```rust
// sqlcc-analyzer/src/lib.rs

use sqlcc_parser::ast::{Statement, Expr, DataType};
use crate::binder::Binder;
use crate::resolver::Resolver;

/// 语义分析器
pub struct Analyzer {
    /// 符号绑定器
    binder: Binder,
    /// 类型解析器
    resolver: Resolver,
}

impl Analyzer {
    /// 创建新的分析器
    pub fn new() -> Self {
        Self {
            binder: Binder::new(),
            resolver: Resolver::new(),
        }
    }

    /// 分析 SQL 语句
    pub fn analyze(&mut self, stmt: Statement) -> Result<AnalyzedStatement, AnalysisError> {
        match stmt {
            Statement::Select(select) => self.analyze_select(select),
            Statement::CreateTable(create) => self.analyze_create_table(create),
            Statement::Insert(insert) => self.analyze_insert(insert),
            Statement::Update(update) => self.analyze_update(update),
            Statement::Delete(delete) => self.analyze_delete(delete),
        }
    }

    /// 分析 SELECT 语句
    fn analyze_select(&mut self, select: SelectStatement) -> Result<AnalyzedStatement, AnalysisError> {
        // 1. 绑定 FROM 子句中的表
        self.binder.bind_tables(&select.from)?;

        // 2. 解析列引用
        self.resolver.resolve_select_list(&select.projections)?;

        // 3. 解析 WHERE 表达式
        if let Some(filter) = &select.filter {
            self.resolver.resolve_expression(filter)?;
        }

        // 4. 解析 GROUP BY 表达式
        if let Some(group_by) = &select.group_by {
            self.resolver.resolve_group_by(group_by)?;
        }

        // 5. 解析 HAVING 表达式
        if let Some(having) = &select.having {
            self.resolver.resolve_expression(having)?;

            // 检查 HAVING 中只能使用聚合函数或 GROUP BY 列
            self.resolver.validate_having_clause(having)?;
        }

        // 6. 类型检查
        self.resolver.type_check_select(&select)?;

        // 7. 生成逻辑计划
        let logical_plan = LogicalPlan::Select {
            projections: select.projections,
            from: select.from,
            filter: select.filter,
            group_by: select.group_by,
            having: select.having,
            order_by: select.order_by,
            limit: select.limit,
        };

        Ok(AnalyzedStatement {
            logical_plan,
            output_schema: self.resolver.get_output_schema()?,
        })
    }

    /// 分析 CREATE TABLE 语句
    fn analyze_create_table(&mut self, create: CreateStatement) -> Result<AnalyzedStatement, AnalysisError> {
        // 1. 检查表是否已存在
        if self.binder.table_exists(&create.name) {
            return Err(AnalysisError::table_already_exists(&create.name));
        }

        // 2. 检查列名重复
        let column_names: Vec<_> = create.columns.iter().map(|c| c.name.clone()).collect();
        let duplicates = find_duplicates(&column_names);
        if !duplicates.is_empty() {
            return Err(AnalysisError::duplicate_column_names(duplicates));
        }

        // 3. 类型检查
        for column in &create.columns {
            self.validate_column_type(&column.data_type)?;
        }

        // 4. 检查约束有效性
        self.validate_constraints(&create.columns, &create.constraints)?;

        // 5. 生成逻辑计划
        let logical_plan = LogicalPlan::CreateTable {
            name: create.name,
            columns: create.columns,
            constraints: create.constraints,
        };

        Ok(AnalyzedStatement {
            logical_plan,
            output_schema: Schema::empty(),
        })
    }

    /// 分析 INSERT 语句
    fn analyze_insert(&mut self, insert: InsertStatement) -> Result<AnalyzedStatement, AnalysisError> {
        // 1. 解析目标表
        let table_ref = self.binder.resolve_table(&insert.table_name)?;

        // 2. 匹配列
        match &insert.source {
            InsertSource::Values(values) => {
                // VALUES 语法: INSERT INTO t (c1, c2) VALUES (v1, v2)
                self.validate_insert_columns(&insert.columns, &table_ref.schema)?;

                // 验证值数量匹配
                for row in values {
                    if row.len() != insert.columns.len() {
                        return Err(AnalysisError::column_count_mismatch(
                            insert.columns.len(),
                            row.len(),
                        ));
                    }

                    // 类型检查
                    for (col, val) in insert.columns.iter().zip(row.iter()) {
                        let col_type = table_ref.schema.get_column_type(col)?;
                        self.validate_value_type(val, col_type)?;
                    }
                }
            }
            InsertSource::Query(query) => {
                // INSERT INTO t SELECT ...
                self.analyze(*query)?;

                // 检查源查询输出与目标表列兼容
                // TODO: 实现类型兼容检查
            }
        }

        let logical_plan = LogicalPlan::Insert {
            table: insert.table_name,
            columns: insert.columns,
            source: insert.source,
        };

        Ok(AnalyzedStatement {
            logical_plan,
            output_schema: Schema::empty(),
        })
    }
}

/// 符号绑定器
pub struct Binder {
    /// 当前作用域的表
    tables: HashMap<String, TableRef>,
    /// 作用域栈
    scope_stack: Vec<Scope>,
}

impl Binder {
    /// 创建新的绑定器
    pub fn new() -> Self {
        Self {
            tables: HashMap::new(),
            scope_stack: Vec::new(),
        }
    }

    /// 绑定表引用
    pub fn bind_tables(&mut self, from: &Option<TableReference>) -> Result<(), AnalysisError> {
        match from {
            Some(TableReference::Table { name, alias }) => {
                let table_ref = self.resolve_table(name)?;
                let alias = alias.as_ref().unwrap_or(name);
                self.tables.insert(alias.clone(), table_ref);
            }
            Some(TableReference::Subquery { alias, .. }) => {
                // 子查询作为派生表
                // TODO: 实现子查询绑定
            }
            None => {
                // 没有 FROM 子句，如 SELECT 1 + 1
            }
        }
        Ok(())
    }

    /// 解析表引用
    pub fn resolve_table(&self, name: &str) -> Result<TableRef, AnalysisError> {
        self.tables.get(name)
            .cloned()
            .ok_or_else(|| AnalysisError::table_not_found(name))
    }

    /// 检查表是否存在
    pub fn table_exists(&self, name: &str) -> bool {
        self.tables.contains_key(name)
    }

    /// 解析列引用
    pub fn resolve_column(&self, column: &ColumnRef) -> Result<ColumnRef, AnalysisError> {
        // 1. 首先检查是否是 qualified 列 (table.column)
        if let Some(table_name) = &column.relation {
            if let Some(table) = self.tables.get(table_name) {
                let col = table.schema.get_column(&column.name)?;
                return Ok(ColumnRef {
                    relation: Some(table_name.clone()),
                    name: col.name,
                    data_type: col.data_type.clone(),
                });
            }
            return Err(AnalysisError::table_not_found(table_name));
        }

        // 2. 未限定的列名，需要搜索所有表
        let mut matches: Vec<ColumnRef> = Vec::new();
        for (table_name, table) in &self.tables {
            if let Ok(col) = table.schema.get_column(&column.name) {
                matches.push(ColumnRef {
                    relation: Some(table_name.clone()),
                    name: col.name,
                    data_type: col.data_type.clone(),
                });
            }
        }

        // 3. 检查歧义
        match matches.len() {
            0 => Err(AnalysisError::column_not_found(&column.name)),
            1 => Ok(matches.remove(0)),
            _ => Err(AnalysisError::ambiguous_column(&column.name, matches.len())),
        }
    }
}

/// 类型解析器
pub struct Resolver {
    /// 当前作用域的表达式类型
    expr_types: HashMap<ExprId, DataType>,
    /// 输出模式
    output_schema: Option<Schema>,
}

impl Resolver {
    /// 创建新的解析器
    pub fn new() -> Self {
        Self {
            expr_types: HashMap::new(),
            output_schema: None,
        }
    }

    /// 解析表达式
    pub fn resolve_expression(&mut self, expr: &Expr) -> Result<ExprId, AnalysisError> {
        match expr {
            Expr::Column(col_ref) => {
                // 列引用已在 Binder 中解析
                Ok(ExprId::new())
            }
            Expr::Literal(lit) => {
                let expr_id = ExprId::new();
                let data_type = self.resolve_literal_type(lit)?;
                self.expr_types.insert(expr_id, data_type);
                Ok(expr_id)
            }
            Expr::Function(func) => {
                // 解析函数参数
                for arg in &func.args {
                    self.resolve_expression(arg)?;
                }
                // TODO: 解析函数返回类型
                Ok(ExprId::new())
            }
            Expr::BinaryOp { op, left, right } => {
                let left_id = self.resolve_expression(left)?;
                let right_id = self.resolve_expression(right)?;

                // 类型检查
                let left_type = self.expr_types.get(&left_id).cloned();
                let right_type = self.expr_types.get(&right_id).cloned();
                let result_type = self.binary_op_result_type(op, left_type, right_type)?;

                let expr_id = ExprId::new();
                self.expr_types.insert(expr_id, result_type);
                Ok(expr_id)
            }
            _ => Ok(ExprId::new()),
        }
    }

    /// SELECT 列表类型检查
    fn type_check_select(&self, select: &SelectStatement) -> Result<(), AnalysisError> {
        // 1. 检查 SELECT 列表与 WHERE 条件的类型兼容性
        // 2. 检查 GROUP BY 与 SELECT 列表的一致性
        // 3. 检查聚合函数的使用合法性

        // 如果有 GROUP BY，SELECT 列表中的非聚合列必须在 GROUP BY 中
        if let Some(group_by) = &select.group_by {
            for proj in &select.projections {
                if let ProjectionExpr::Expr { expr, alias: _ } = proj {
                    if !self.is_aggregate_expr(expr) && !self.is_in_group_by(expr, group_by) {
                        return Err(AnalysisError::column_must_in_group_by(
                            format!("{:?}", expr)
                        ));
                    }
                }
            }
        }

        // 检查聚合函数参数类型
        // TODO: 实现详细的聚合函数类型检查

        Ok(())
    }

    /// 获取输出模式
    pub fn get_output_schema(&self) -> Result<Schema, AnalysisError> {
        self.output_schema
            .clone()
            .ok_or_else(|| AnalysisError::internal_error("output schema not set"))
    }
}

/// 分析错误
#[derive(Debug, thiserror::Error)]
pub enum AnalysisError {
    #[error("table not found: {0}")]
    TableNotFound(String),

    #[error("column not found: {0}")]
    ColumnNotFound(String),

    #[error("ambiguous column: {0} (matches {1} tables)")]
    AmbiguousColumn(String, usize),

    #[error("table already exists: {0}")]
    TableAlreadyExists(String),

    #[error("duplicate column names: {:?}", .0)]
    DuplicateColumnNames(Vec<String>),

    #[error("type mismatch: expected {expected}, found {found}")]
    TypeMismatch { expected: String, found: String },

    #[error("invalid value for column {column}: {message}")]
    InvalidColumnValue { column: String, message: String },

    #[error("column count mismatch: expected {expected}, found {found}")]
    ColumnCountMismatch { expected: usize, found: usize },

    #[error("column must appear in GROUP BY or be used in an aggregate function: {0}")]
    ColumnMustInGroupBy(String),

    #[error("aggregate function is not allowed in WHERE clause")]
    AggregateInWhere,

    #[error("internal error: {0}")]
    InternalError(String),
}

/// 分析后的语句
pub struct AnalyzedStatement {
    /// 逻辑计划
    pub logical_plan: LogicalPlan,
    /// 输出模式
    pub output_schema: Schema,
}
```

#### 3.2.3 Optimizer (sqlcc-optimizer)

**职责**:

1. 逻辑优化 (谓词下推、投影下推、常量折叠)
2. 物理计划生成
3. 代价估算

**接口设计**:

```rust
// sqlcc-optimizer/src/lib.rs

use sqlcc_analyzer::{AnalyzedStatement, LogicalPlan, Schema};

/// 查询优化器
pub struct Optimizer {
    /// 优化规则列表
    rules: Vec<Box<dyn OptimizationRule>>,
    /// 代价模型
    cost_model: CostModel,
}

impl Optimizer {
    /// 创建优化器
    pub fn new() -> Self {
        Self {
            rules: vec![
                Box::new(PredicatePushdown),
                Box::new(ProjectionPushdown),
                Box::new(ConstantFolding),
                Box::new(FilterCombine),
            ],
            cost_model: CostModel::default(),
        }
    }

    /// 优化逻辑计划
    pub fn optimize(&mut self, plan: LogicalPlan) -> Result<PhysicalPlan, OptimizerError> {
        // 1. 应用逻辑优化规则
        let mut optimized_plan = plan;
        for rule in &self.rules {
            optimized_plan = rule.optimize(optimized_plan)?;
        }

        // 2. 生成物理计划
        self.create_physical_plan(optimized_plan)
    }

    /// 创建物理计划
    fn create_physical_plan(&self, plan: LogicalPlan) -> Result<PhysicalPlan, OptimizerError> {
        match plan {
            LogicalPlan::TableScan { table_name, projection, filter } => {
                Ok(PhysicalPlan::TableScan {
                    table_name,
                    projection,
                    filter,
                })
            }
            LogicalPlan::Filter { input, predicate } => {
                let input_plan = self.create_physical_plan(*input)?;
                Ok(PhysicalPlan::Filter {
                    input: Box::new(input_plan),
                    predicate,
                })
            }
            LogicalPlan::Project { input, expressions } => {
                let input_plan = self.create_physical_plan(*input)?;
                Ok(PhysicalPlan::Project {
                    input: Box::new(input_plan),
                    expressions,
                })
            }
            LogicalPlan::HashJoin { left, right, join_type, condition } => {
                let left_plan = self.create_physical_plan(*left)?;
                let right_plan = self.create_physical_plan(*right)?;

                // 估算代价，选择构建/探查侧
                let (build_side, probe_side) = self.select_join_sides(&left_plan, &right_plan)?;

                Ok(PhysicalPlan::HashJoin {
                    build_side: Box::new(build_side),
                    probe_side: Box::new(probe_side),
                    join_type,
                    condition,
                })
            }
            LogicalPlan::Sort { input, order_by } => {
                let input_plan = self.create_physical_plan(*input)?;
                Ok(PhysicalPlan::Sort {
                    input: Box::new(input_plan),
                    order_by,
                })
            }
            LogicalPlan::Limit { input, limit, offset } => {
                let input_plan = self.create_physical_plan(*input)?;
                Ok(PhysicalPlan::Limit {
                    input: Box::new(input_plan),
                    limit,
                    offset,
                })
            }
        }
    }

    /// 选择 Join 的构建侧和探查侧
    fn select_join_sides(
        &self,
        left: &PhysicalPlan,
        right: &PhysicalPlan,
    ) -> Result<(PhysicalPlan, PhysicalPlan), OptimizerError> {
        let left_cost = self.cost_model.estimate_cost(left)?;
        let right_cost = self.cost_model.estimate_cost(right)?;

        // 选择较小的输入作为构建侧
        if left_cost < right_cost {
            Ok((left.clone(), right.clone()))
        } else {
            Ok((right.clone(), left.clone()))
        }
    }
}

/// 优化规则 trait
pub trait OptimizationRule {
    fn optimize(&self, plan: LogicalPlan) -> Result<LogicalPlan, OptimizerError>;
}

/// 谓词下推规则
pub struct PredicatePushdown;

impl OptimizationRule for PredicatePushdown {
    fn optimize(&mut self, plan: LogicalPlan) -> Result<LogicalPlan, OptimizerError> {
        match plan {
            // Filter 下推到 TableScan
            LogicalPlan::Filter {
                input: box LogicalPlan::Filter { input, predicate: pred1 },
                predicate: pred2,
            } => {
                // 合并两个 Filter
                let combined_pred = Expr::BinaryOp {
                    op: BinaryOperator::And,
                    left: Box::new(pred1),
                    right: Box::new(pred2),
                };
                self.optimize(LogicalPlan::Filter {
                    input,
                    predicate: combined_pred,
                })
            }
            LogicalPlan::Filter {
                input: box LogicalPlan::TableScan { table_name, projection, filter: None },
                predicate,
            } => {
                // 将 Filter 下推到 TableScan
                let new_filter = if let Some(existing) = filter {
                    Expr::BinaryOp {
                        op: BinaryOperator::And,
                        left: Box::new(existing),
                        right: Box::new(predicate),
                    }
                } else {
                    predicate
                };
                Ok(LogicalPlan::TableScan {
                    table_name,
                    projection,
                    filter: Some(new_filter),
                })
            }
            // 处理嵌套的 Filter
            LogicalPlan::Filter {
                input,
                predicate,
            } => {
                let new_input = self.optimize(*input)?;
                Ok(LogicalPlan::Filter {
                    input: Box::new(new_input),
                    predicate,
                })
            }
            _ => Ok(plan),
        }
    }
}

/// 投影下推规则
pub struct ProjectionPushdown;

impl OptimizationRule for ProjectionPushdown {
    fn optimize(&mut self, plan: LogicalPlan) -> Result<LogicalPlan, OptimizerError> {
        // TODO: 实现投影下推
        // 1. 收集每个节点实际需要的列
        // 2. 下推 Projection 到 TableScan
        // 3. 移除不必要的列
        Ok(plan)
    }
}

/// 代价模型
pub struct CostModel {
    /// 表统计信息
    stats: HashMap<String, TableStats>,
}

impl CostModel {
    /// 估算计划代价
    pub fn estimate_cost(&self, plan: &PhysicalPlan) -> Result<Cost, OptimizerError> {
        match plan {
            PhysicalPlan::TableScan { table_name, projection, filter: _ } => {
                let stats = self.stats.get(table_name)
                    .ok_or_else(|| OptimizerError::no_stats(table_name.clone()))?;
                let row_count = stats.row_count;
                let columns = projection.len();
                Ok(Cost {
                    cpu_cost: row_count * columns,
                    io_cost: (row_count * columns * std::mem::size_of::<u64>()) as f64 / 8192.0,
                })
            }
            PhysicalPlan::Filter { input, .. } => {
                let input_cost = self.estimate_cost(input)?;
                // Filter 增加 10% 的 CPU 代价
                Ok(Cost {
                    cpu_cost: input_cost.cpu_cost * 1.1,
                    io_cost: input_cost.io_cost,
                })
            }
            PhysicalPlan::Project { input, expressions } => {
                let input_cost = self.estimate_cost(input)?;
                // Project 增加 CPU 代价
                let expr_count = expressions.len();
                Ok(Cost {
                    cpu_cost: input_cost.cpu_cost + expr_count as f64,
                    io_cost: input_cost.io_cost,
                })
            }
            PhysicalPlan::HashJoin { build_side, probe_side, .. } => {
                let build_cost = self.estimate_cost(build_side)?;
                let probe_cost = self.estimate_cost(probe_side)?;
                // Hash Join 代价 = 构建哈希表 + 探查
                Ok(Cost {
                    cpu_cost: build_cost.cpu_cost + probe_cost.cpu_cost + build_cost.io_cost,
                    io_cost: build_cost.io_cost,
                })
            }
            PhysicalPlan::Sort { input, .. } => {
                let input_cost = self.estimate_cost(input)?;
                // 排序代价 O(n log n)
                Ok(Cost {
                    cpu_cost: input_cost.cpu_cost * input_cost.cpu_cost.log2(),
                    io_cost: input_cost.io_cost,
                })
            }
            PhysicalPlan::Limit { input, .. } => {
                self.estimate_cost(input)
            }
        }
    }
}

/// 代价统计
#[derive(Clone, Debug)]
pub struct Cost {
    /// CPU 代价
    pub cpu_cost: f64,
    /// I/O 代价 (页数)
    pub io_cost: f64,
}

impl std::ops::Add for Cost {
    type Output = Cost;

    fn add(self, other: Cost) -> Cost {
        Cost {
            cpu_cost: self.cpu_cost + other.cpu_cost,
            io_cost: self.io_cost + other.io_cost,
        }
    }
}

/// 表统计信息
pub struct TableStats {
    /// 行数
    pub row_count: f64,
    /// 列数
    pub column_count: usize,
    /// 每行平均字节数
    pub avg_row_size: f64,
    /// 页面数
    pub page_count: f64,
}

/// 优化器错误
#[derive(Debug, thiserror::Error)]
pub enum OptimizerError {
    #[error("no statistics for table: {0}")]
    NoStats(String),

    #[error("optimization rule failed: {0}")]
    RuleFailed(String),

    #[error("internal error: {0}")]
    InternalError(String),
}
```

#### 3.2.4 Executor (sqlcc-executor)

**职责**: Volcano 迭代器模型执行

```rust
// sqlcc-executor/src/volcano/mod.rs

use sqlcc_optimizer::PhysicalPlan;
use crate::RowBatch;

/// 算子 trait - Volcano 模型核心
pub trait PhysicalOperator {
    /// 获取输出模式
    fn schema(&self) -> &Schema;

    /// 打开算子（初始化资源）
    fn open(&mut self) -> Result<(), ExecutionError>;

    /// 获取下一行
    fn next(&mut self) -> Result<Option<Row>, ExecutionError>;

    /// 关闭算子（释放资源）
    fn close(&mut self) -> Result<(), ExecutionError>;
}

/// 批量算子 - 向量化执行优化
pub trait BatchOperator {
    /// 获取输出批次模式
    fn schema(&self) -> &Schema;

    /// 打开算子
    fn open(&mut self) -> Result<(), ExecutionError>;

    /// 获取下一个批次
    fn next_batch(&mut self) -> Result<Option<RowBatch>, ExecutionError>;

    /// 关闭算子
    fn close(&mut self) -> Result<(), ExecutionError>;
}

/// 表扫描算子
pub struct TableScanOperator {
    /// 表名
    table_name: String,
    /// 投影列
    projection: Vec<String>,
    /// 过滤条件
    filter: Option<Expr>,
    /// 存储访问器
    storage: Arc<dyn StorageAccess>,
    /// 迭代器状态
    iterator: Box<dyn TableIterator>,
}

impl PhysicalOperator for TableScanOperator {
    fn schema(&self) -> &Schema {
        // 返回投影列的模式
        &self.schema
    }

    fn open(&mut self) -> Result<(), ExecutionError> {
        // 初始化表迭代器
        self.iterator = self.storage.create_iterator(
            self.projection.clone(),
            self.filter.clone(),
        )?;
        Ok(())
    }

    fn next(&mut self) -> Result<Option<Row>, ExecutionError> {
        self.iterator.next()
    }

    fn close(&mut self) -> Result<(), ExecutionError> {
        self.iterator.close()
    }
}

/// 过滤算子
pub struct FilterOperator {
    /// 输入算子
    input: Box<dyn PhysicalOperator>,
    /// 过滤条件
    predicate: Expr,
    /// 输出模式
    schema: Schema,
}

impl PhysicalOperator for FilterOperator {
    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn open(&mut self) -> Result<(), ExecutionError> {
        self.input.open()
    }

    fn next(&mut self) -> Result<Option<Row>, ExecutionError> {
        loop {
            match self.input.next()? {
                Some(row) => {
                    // 评估过滤条件
                    if self.evaluate_predicate(&row)? {
                        return Ok(Some(row));
                    }
                }
                None => return Ok(None),
            }
        }
    }

    fn close(&mut self) -> Result<(), ExecutionError> {
        self.input.close()
    }
}

/// 投影算子
pub struct ProjectOperator {
    /// 输入算子
    input: Box<dyn PhysicalOperator>,
    /// 投影表达式
    expressions: Vec<Expr>,
    /// 输出模式
    schema: Schema,
}

impl PhysicalOperator for ProjectOperator {
    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn open(&mut self) -> Result<(), ExecutionError> {
        self.input.open()
    }

    fn next(&mut self) -> Result<Option<Row>, ExecutionError> {
        match self.input.next()? {
            Some(input_row) => {
                // 计算投影表达式
                let output_values = self.evaluate_expressions(&input_row)?;
                Ok(Some(Row::new(output_values)))
            }
            None => Ok(None),
        }
    }

    fn close(&mut self) -> Result<(), ExecutionError> {
        self.input.close()
    }
}

/// Hash Join 算子
pub struct HashJoinOperator {
    /// 构建侧
    build_side: Box<dyn PhysicalOperator>,
    /// 探查侧
    probe_side: Box<dyn PhysicalOperator>,
    /// Join 类型
    join_type: JoinType,
    /// Join 条件
    condition: Expr,
    /// 哈希表
    hash_table: HashMap<HashValue, Vec<Row>>,
    /// 输出模式
    schema: Schema,
}

impl HashJoinOperator {
    /// 构建哈希键
    fn build_hash_key(&self, row: &Row, expr: &Expr) -> Result<HashValue, ExecutionError> {
        // 评估表达式获取键值
        let key_value = self.evaluate_expression(row, expr)?;
        Ok(self.hash_function(&key_value))
    }

    /// 哈希函数
    fn hash_function(&self, value: &Value) -> HashValue {
        match value {
            Value::Int64(n) => (*n as u64).hash(),
            Value::String(s) => s.hash(),
            Value::Float64(f) => (f.to_bits()).hash(),
            Value::Null => 0,
            _ => 0,
        }
    }
}

impl PhysicalOperator for HashJoinOperator {
    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn open(&mut self) -> Result<(), ExecutionError> {
        // 1. 打开构建侧
        self.build_side.open()?;

        // 2. 构建哈希表
        self.hash_table.clear();
        while let Some(row) = self.build_side.next()? {
            let key = self.build_hash_key(&row, &self.condition)?;
            self.hash_table.entry(key).or_insert_with(Vec::new).push(row);
        }

        // 3. 打开探查侧
        self.probe_side.open()
    }

    fn next(&mut self) -> Result<Option<Row>, ExecutionError> {
        loop {
            match self.probe_side.next()? {
                Some(probe_row) => {
                    let key = self.build_hash_key(&probe_row, &self.condition)?;

                    if let Some(build_rows) = self.hash_table.get(&key) {
                        // 找到匹配，构建输出行
                        return Ok(Some(self.join_rows(&probe_row, &build_rows[0])));
                    }
                }
                None => return Ok(None),
            }
        }
    }

    fn close(&mut self) -> Result<(), ExecutionError> {
        self.build_side.close()?;
        self.probe_side.close()
    }
}

/// 执行器
pub struct Executor {
    /// 物理计划
    plan: PhysicalPlan,
    /// 当前算子根节点
    root: Box<dyn PhysicalOperator>,
}

impl Executor {
    /// 创建执行器
    pub fn new(plan: PhysicalPlan) -> Result<Self, ExecutionError> {
        let root = Self::create_operator(&plan)?;
        Ok(Self { plan, root })
    }

    /// 创建算子树
    fn create_operator(plan: &PhysicalPlan) -> Result<Box<dyn PhysicalOperator>, ExecutionError> {
        match plan {
            PhysicalPlan::TableScan { table_name, projection, filter } => {
                let op = TableScanOperator::new(
                    table_name.clone(),
                    projection.clone(),
                    filter.clone(),
                )?;
                Ok(Box::new(op))
            }
            PhysicalPlan::Filter { input, predicate } => {
                let input_op = Self::create_operator(input)?;
                let op = FilterOperator::new(input_op, predicate.clone());
                Ok(Box::new(op))
            }
            PhysicalPlan::Project { input, expressions } => {
                let input_op = Self::create_operator(input)?;
                let op = ProjectOperator::new(input_op, expressions.clone());
                Ok(Box::new(op))
            }
            PhysicalPlan::HashJoin { build_side, probe_side, join_type, condition } => {
                // 递归创建两侧算子
                let build_op = Self::create_operator(build_side)?;
                let probe_op = Self::create_operator(probe_side)?;
                let op = HashJoinOperator::new(
                    build_op,
                    probe_op,
                    *join_type,
                    condition.clone(),
                );
                Ok(Box::new(op))
            }
            PhysicalPlan::Sort { input, order_by } => {
                let input_op = Self::create_operator(input)?;
                let op = SortOperator::new(input_op, order_by.clone());
                Ok(Box::new(op))
            }
            PhysicalPlan::Limit { input, limit, offset } => {
                let input_op = Self::create_operator(input)?;
                let op = LimitOperator::new(input_op, *limit, *offset);
                Ok(Box::new(op))
            }
        }
    }

    /// 执行查询
    pub fn execute(&mut self) -> Result<Option<Row>, ExecutionError> {
        self.root.open()?;
        self.root.next()
    }

    /// 执行并返回所有行
    pub fn execute_all(&mut self) -> Result<Vec<Row>, ExecutionError> {
        self.root.open()?;

        let mut results = Vec::new();
        while let Some(row) = self.root.next()? {
            results.push(row);
        }

        self.root.close()?;
        Ok(results)
    }
}
```

#### 3.2.5 Storage (sqlcc-storage)

**职责**:

1. 页面管理
2. Buffer Pool
3. B+ 树索引
4. Catalog

```rust
// sqlcc-storage/src/lib.rs

/// 存储引擎
pub struct StorageEngine {
    /// 缓冲池
    buffer_pool: BufferPool,
    /// 磁盘管理器
    disk_manager: DiskManager,
    /// 编目管理器
    catalog: Catalog,
    /// 索引管理器
    index_manager: IndexManager,
}

/// 表存储格式 - 行式存储
pub struct RowFormat {
    /// 固定宽度列
    fixed_width_columns: Vec<FixedWidthColumn>,
    /// 变长列起始偏移
    var_length_offsets: Vec<ColumnOffset>,
    /// 变长数据区域
    var_length_data: Vec<u8>,
}

impl RowFormat {
    /// 编码一行数据
    pub fn encode(&self, values: &[Value]) -> Result<Vec<u8>, StorageError> {
        // 1. 计算变长列的偏移
        let mut offsets = Vec::new();
        let mut offset = 0u32;

        for (i, value) in values.iter().enumerate() {
            if self.fixed_width_columns[i].is_fixed_width() {
                continue;
            }
            offsets.push(offset);
            offset += value.serialized_size() as u32;
        }

        // 2. 编码固定宽度列
        let mut row_data = Vec::with_capacity(4096);
        for (i, column) in self.fixed_width_columns.iter().enumerate() {
            if let Some(value) = values.get(i) {
                column.serialize(value, &mut row_data)?;
            }
        }

        // 3. 写入变长列偏移
        for col_offset in &offsets {
            row_data.extend_from_slice(&col_offset.to_le_bytes());
        }

        // 4. 编码变长列
        for (i, value) in values.iter().enumerate() {
            if !self.fixed_width_columns[i].is_fixed_width() {
                let mut var_data = Vec::new();
                value.serialize(&mut var_data)?;
                row_data.extend_from_slice(&var_data);
            }
        }

        Ok(row_data)
    }

    /// 解码一行数据
    pub fn decode(&self, data: &[u8]) -> Result<Vec<Value>, StorageError> {
        let mut values = Vec::new();
        let fixed_width_end = self.calculate_fixed_width_end();

        // 1. 读取固定宽度列
        let mut pos = 0;
        for column in &self.fixed_width_columns {
            let col_data = &data[pos..pos + column.size()];
            values.push(column.deserialize(col_data)?);
            pos += column.size();
        }

        // 2. 读取变长列偏移
        let offset_base = pos;
        let var_length_indices: Vec<usize> = self.fixed_width_columns
            .iter()
            .enumerate()
            .filter(|(_, col)| !col.is_fixed_width())
            .map(|(i, _)| i)
            .collect();

        // 3. 读取变长列
        for (idx, var_idx) in var_length_indices.iter().enumerate() {
            let offset_start = offset_base + var_idx * std::mem::size_of::<u32>();
            let offset = u32::from_le_bytes(
                data[offset_start..offset_start + 4].try_into().unwrap()
            ) as usize;

            let offset_end = if idx + 1 < var_length_indices.len() {
                let next_var_idx = var_length_indices[idx + 1];
                offset_base + next_var_idx * std::mem::size_of::<u32>()
            } else {
                data.len()
            };
            let var_data = &data[offset..offset_end];
            values.push(Value::deserialize(var_data)?);
        }

        Ok(values)
    }
}

/// 缓冲池 (Buffer Pool)
pub struct BufferPool {
    /// 页面缓存
    pages: ShardedHashMap<PageId, PageFrame>,
    /// 替换策略
    replacer: Box<dyn PageReplacer>,
    /// 磁盘管理器
    disk: Arc<DiskManager>,
    /// 锁管理器
    locks: RwLock<HashMap<PageId, PageLock>>,
    /// 配置
    config: BufferPoolConfig,
}

impl BufferPool {
    /// 获取页面
    pub fn get(&self, page_id: PageId) -> Result<Arc<PageFrame>, StorageError> {
        // 1. 检查页面是否在缓存中
        {
            let read_guard = self.locks.read().unwrap();
            if let Some(page) = self.pages.get(&page_id) {
                // 更新访问时间
                page.update_access_time();
                // 移动到 LRU 头部
                self.replacer.record_access(page_id);
                return Ok(Arc::clone(page));
            }
        }

        // 2. 从磁盘加载
        let page = self.load_page(page_id)?;

        // 3. 尝试插入缓存
        self.insert_page(page_id, page)?;

        Ok(Arc::clone(self.pages.get(&page_id).unwrap()))
    }

    /// 加载页面
    fn load_page(&self, page_id: PageId) -> Result<Arc<PageFrame>, StorageError> {
        // 1. 分配新页面帧
        let frame = Arc::new(PageFrame::new(page_id));

        // 2. 从磁盘读取
        self.disk.read_page(page_id, &mut frame.data)?;

        // 3. 设置页面状态
        {
            let mut state = frame.state.lock().unwrap();
            state.status = PageStatus::Clean;
        }

        Ok(frame)
    }

    /// 插入页面
    fn insert_page(&self, page_id: PageId, page: Arc<PageFrame>) -> Result<(), StorageError> {
        // 1. 检查是否有空闲帧
        let frame_id = if self.pages.len() < self.config.max_size {
            Some(self.alloc_frame()?)
        } else {
            self.replacer.evict()?
        };

        // 2. 如果需要淘汰
        if let Some(evicted_id) = frame_id {
            self.evict_page(evicted_id)?;
        }

        // 3. 插入页面
        let mut write_guard = self.locks.write().unwrap();
        self.pages.insert(page_id, page);
        write_guard.insert(page_id, PageLock::new());

        Ok(())
    }

    /// 淘汰页面
    fn evict_page(&self, page_id: PageId) -> Result<(), StorageError> {
        if let Some(page) = self.pages.get(&page_id) {
            let state = page.state.lock().unwrap();

            // 只允许淘汰脏页或未被pin的页面
            if state.pin_count > 0 {
                return Err(StorageError::page_pinned(page_id));
            }

            // 如果是脏页，写回磁盘
            if state.status == PageStatus::Dirty {
                self.disk.write_page(page_id, &page.data)?;
                state.status = PageStatus::Clean;
            }

            // 移除页面
            drop(state);
            self.pages.remove(&page_id);
        }
        Ok(())
    }
}

/// B+ 树索引
pub struct BPlusTree<K: BTreeKey, V: BTreeValue> {
    /// 根节点 ID
    root_id: Option<PageId>,
    /// 比较函数
    comparator: Box<dyn Fn(&K, &K) -> Ordering>,
    /// 存储
    storage: Arc<StorageEngine>,
}

impl<K: BTreeKey + Clone, V: BTreeValue> BPlusTree<K, V> {
    /// 搜索
    pub fn search(&self, key: &K) -> Result<Option<V>, StorageError> {
        // 1. 从根节点开始搜索
        let mut node_id = self.root_id.ok_or(StorageError::empty_index())?;
        let mut node = self.load_node(node_id)?;

        // 2. 沿路径搜索到叶子节点
        while !node.is_leaf() {
            let child_id = node.search_child(key, &self.comparator)?;
            node_id = child_id;
            node = self.load_node(child_id)?;
        }

        // 3. 在叶子节点中查找
        node.as_leaf_mut()?.find(key, &self.comparator)
    }

    /// 插入
    pub fn insert(&mut self, key: K, value: V) -> Result<(), StorageError> {
        // 1. 如果树为空，创建根节点
        if self.root_id.is_none() {
            let root = LeafNode::new();
            self.root_id = Some(self.alloc_node(root)?);
        }

        // 2. 查找插入位置
        let mut path = Vec::new();
        let mut node_id = self.root_id.unwrap();
        let mut node = self.load_node(node_id)?;

        while !node.is_leaf() {
            path.push(node_id);
            node_id = node.search_child(&key, &self.comparator)?;
            node = self.load_node(node_id)?;
        }

        // 3. 插入到叶子节点
        let leaf = node.as_leaf_mut()?;
        if leaf.is_full() {
            // 4. 分裂叶子节点
            self.split_leaf(leaf, path)?;
        }
        leaf.insert(key, value)?;

        Ok(())
    }

    /// 分裂叶子节点
    fn split_leaf(&mut self, leaf: &mut LeafNode<K, V>, path: Vec<PageId>) -> Result<(), StorageError> {
        // 1. 创建新叶子节点
        let new_leaf = LeafNode::new();
        let new_leaf_id = self.alloc_node(new_leaf)?;

        // 2. 移动后半部分数据到新节点
        let mid = leaf.len() / 2;
        let split_key = leaf.keys[mid].clone();
        leaf.split_to(new_leaf_id, mid)?;

        // 3. 插入新节点到父节点
        if let Some(parent_id) = path.last() {
            let mut parent = self.load_node(*parent_id)?;
            parent.as_internal_mut()?.insert_child(split_key, new_leaf_id);
        } else {
            // 4. 需要创建新的根节点
            let new_root = InternalNode::new();
            let new_root_id = self.alloc_node(new_root)?;
            new_root.as_internal_mut()?.insert_child(split_key, self.root_id.unwrap());
            new_root.as_internal_mut()?.insert_child(split_key.clone(), new_leaf_id);
            self.root_id = Some(new_root_id);
        }

        Ok(())
    }
}
```

#### 3.2.6 Transaction (sqlcc-transaction)

**职责**: MVCC + 2PL + WAL

```rust
// sqlcc-transaction/src/lib.rs

/// 事务管理器
pub struct TransactionManager {
    /// 活跃事务表
    active_transactions: DashMap<TransactionId, Transaction>,
    /// 锁管理器
    lock_manager: LockManager,
    /// 版本存储
    version_store: VersionStore,
    /// WAL 管理器
    wal: WALManager,
    /// 事务 ID 生成器
    txn_id_generator: AtomicU64,
}

/// 事务
pub struct Transaction {
    /// 事务 ID
    id: TransactionId,
    /// 事务状态
    state: AtomicCell<TransactionState>,
    /// 开始时间戳
    start_timestamp: Timestamp,
    /// 隔离级别
    isolation_level: IsolationLevel,
    /// 保存点列表
    savepoints: Vec<Savepoint>,
    /// 写集
    write_set: WriteSet,
    /// 读集
    read_set: ReadSet,
}

/// MVCC 版本存储
pub struct VersionStore {
    /// 数据页版本
    page_versions: DashMap<PageId, Vec<PageVersion>>,
    /// 元组版本
    tuple_versions: DashMap<TupleId, Vec<TupleVersion>>,
}

impl VersionStore {
    /// 获取读取可见版本
    pub fn get_read_version(
        &self,
        tuple_id: TupleId,
        txn: &Transaction,
    ) -> Result<Option<TupleVersion>, TransactionError> {
        let versions = self.tuple_versions.get(&tuple_id);

        match txn.isolation_level() {
            IsolationLevel::ReadCommitted => {
                // 读取最新已提交版本
                if let Some(vers) = versions {
                    for version in vers.iter().rev() {
                        if version.commit_timestamp <= txn.read_snapshot().as_of() {
                            return Ok(Some(version.clone()));
                        }
                    }
                }
                Ok(None)
            }
            IsolationLevel::RepeatableRead => {
                // 读取事务开始时的最新已提交版本
                if let Some(vers) = versions {
                    for version in vers.iter().rev() {
                        if version.commit_timestamp < txn.start_timestamp() {
                            return Ok(Some(version.clone()));
                        }
                    }
                }
                Ok(None)
            }
            _ => unimplemented!(),
        }
    }

    /// 添加新版本
    pub fn insert_version(
        &self,
        tuple_id: TupleId,
        version: TupleVersion,
    ) {
        let mut versions = self.tuple_versions.entry(tuple_id)
            .or_insert_with(Vec::new);
        versions.push(version);
    }
}

/// 锁管理器 - 2PL 实现
pub struct LockManager {
    /// 锁表
    lock_table: DashMap<LockableResource, ResourceLock>,
    /// 等待队列
    wait_queue: DashMap<TransactionId, WaitInfo>,
    /// 死锁检测器
    deadlock_detector: DeadlockDetector,
}

impl LockManager {
    /// 获取共享锁 (S 锁)
    pub fn acquire_shared(
        &self,
        txn_id: TransactionId,
        resource: &LockableResource,
        timeout: Duration,
    ) -> Result<LockGrant, LockError> {
        let lock = self.lock_table.entry(resource.clone())
            .or_insert_with(|| ResourceLock::new(resource.clone()));

        let mut guard = lock.write().unwrap();

        // 1. 检查是否可以立即获取
        if guard.is_compatible(txn_id, LockMode::Shared) {
            guard.grant_lock(txn_id, LockMode::Shared);
            return Ok(LockGrant::new(txn_id, LockMode::Shared));
        }

        // 2. 进入等待队列
        let wait_info = WaitInfo::new(txn_id, LockMode::Shared, timeout);
        guard.wait_queue.push_back(wait_info.clone());

        // 3. 等待锁授予
        drop(guard);
        self.wait_for_grant(wait_info)
    }

    /// 获取排他锁 (X 锁)
    pub fn acquire_exclusive(
        &self,
        txn_id: TransactionId,
        resource: &LockableResource,
        timeout: Duration,
    ) -> Result<LockGrant, LockError> {
        let lock = self.lock_table.entry(resource.clone())
            .or_insert_with(|| ResourceLock::new(resource.clone()));

        let mut guard = lock.write().unwrap();

        // 检查是否可以获取 X 锁
        if guard.can_acquire_exclusive(txn_id) {
            guard.grant_lock(txn_id, LockMode::Exclusive);
            return Ok(LockGrant::new(txn_id, LockMode::Exclusive));
        }

        // 需要等待
        let wait_info = WaitInfo::new(txn_id, LockMode::Exclusive, timeout);
        guard.wait_queue.push_back(wait_info.clone());

        drop(guard);
        self.wait_for_grant(wait_info)
    }

    /// 释放锁
    pub fn release(&self, txn_id: TransactionId, resource: &LockableResource) {
        if let Some(lock) = self.lock_table.get(resource) {
            let mut guard = lock.write().unwrap();
            guard.release(txn_id);

            // 尝试授予等待队列中的下一个事务
            self.try_grant_next(&mut guard);
        }
    }
}

/// WAL 管理器
pub struct WALManager {
    /// 日志文件
    log_file: File,
    /// 日志缓冲区
    buffer: Vec<u8>,
    /// 刷盘策略
    flush_policy: FlushPolicy,
    /// 检查点信息
    checkpoint_info: CheckpointInfo,
    /// LSN 序列
    lsn: AtomicU64,
}

impl WALManager {
    /// 写入日志记录
    pub fn write_record(&self, record: WALRecord) -> Result<u64, WALError> {
        // 1. 序列化记录
        let mut serialized = Vec::new();
        self.serialize_record(&record, &mut serialized)?;

        // 2. 写入缓冲区
        let lsn = self.lsn.fetch_add(1, Ordering::SeqCst);
        let record_header = WALRecordHeader {
            lsn,
            txn_id: record.txn_id,
            record_type: record.record_type,
            size: serialized.len() as u32,
        };

        self.buffer.extend_from_slice(&self.serialize_header(&record_header));
        self.buffer.extend_from_slice(&serialized);

        // 3. 检查是否需要刷盘
        if self.should_flush() {
            self.flush()?;
        }

        Ok(lsn)
    }

    /// 刷新到磁盘
    pub fn flush(&self) -> Result<(), WALError> {
        use std::io::Write;

        // 1. 获取日志文件偏移
        let offset = self.log_file.seek(std::io::SeekFrom::End(0))?;

        // 2. 写入日志
        self.log_file.write_all(&self.buffer)?;

        // 3. 更新检查点
        self.checkpoint_info.last_flushed_lsn = self.lsn.load(Ordering::SeqCst);
        self.checkpoint_info.file_offset = offset;

        // 4. 清空缓冲区
        self.buffer.clear();

        // 5. 强制刷盘
        self.log_file.flush()?;
        unsafe {
            libc::fsync(self.log_file.as_raw_fd());
        }

        Ok(())
    }

    /// 恢复
    pub fn recover(&self) -> Result<Vec<TransactionAction>, WALError> {
        let mut actions = Vec::new();

        // 1. 读取检查点
        let checkpoint = self.load_checkpoint()?;

        // 2. 分析阶段 - 重做所有已提交事务
        self.redo(checkpoint.last_flushed_lsn)?;

        // 3. 撤销阶段 - 回滚未提交事务
        self.undo()?;

        Ok(actions)
    }
}
```

#### 3.2.7 Network (sqlcc-network)

**职责**: MySQL 协议兼容 + gRPC

```rust
// sqlcc-network/src/mysql/protocol.rs

/// MySQL 协议处理器
pub struct MySQLProtocolHandler {
    /// 连接状态
    state: ConnectionState,
    /// 能力标志
    capabilities: CapabilityFlags,
    /// 字符集
    charset: Charset,
    /// 认证信息
    auth: AuthInfo,
    /// SQL 执行器
    executor: Arc<QueryExecutor>,
}

impl MySQLProtocolHandler {
    /// 处理认证握手
    pub fn handle_handshake(&mut self, packet: &HandshakeV10Packet) -> Result<(), NetworkError> {
        // 1. 发送握手包
        let handshake = self.create_handshake_packet();
        self.send_packet(&handshake)?;

        // 2. 读取认证响应
        let auth_response = self.read_auth_response()?;

        // 3. 验证密码
        if !self.verify_password(&auth_response)? {
            return Err(NetworkError::auth_failed("invalid password"));
        }

        // 4. 发送 OK 包
        let ok_packet = self.create_ok_packet(0, 0, 0);
        self.send_packet(&ok_packet)?;

        self.state = ConnectionState::Authenticated;
        Ok(())
    }

    /// 处理查询命令
    pub fn handle_query(&mut self, packet: &QueryCommandPacket) -> Result<(), NetworkError> {
        // 1. 解析 SQL
        let sql = String::from_utf8(packet.sql.to_vec())
            .map_err(|_| NetworkError::invalid_packet("invalid utf-8"))?;

        // 2. 执行查询
        let result = self.executor.execute(&sql)?;

        // 3. 发送结果
        match result {
            QueryResult::Rows(rows, schema) => {
                self.send_resultset(&rows, &schema)?;
            }
            QueryResult::AffectedRows(count) => {
                self.send_ok_packet(count, 0, 0)?;
            }
            QueryResult::Error(message, code) => {
                self.send_error_packet(code, &message)?;
            }
        }

        Ok(())
    }

    /// 发送结果集
    fn send_resultset(&self, rows: &[Row], schema: &Schema) -> Result<(), NetworkError> {
        // 1. 发送列计数
        let column_count = ColumnCountPacket {
            count: schema.columns.len() as u64,
        };
        self.send_packet(&column_count)?;

        // 2. 发送每个列的定义
        for column in &schema.columns {
            let column_def = ColumnDefinitionPacket {
                catalog: "def".to_string(),
                schema: self.current_database().unwrap_or_default(),
                table: column.table.clone(),
                origin_table: column.table.clone(),
                column_name: column.name.clone(),
                origin_column_name: column.name.clone(),
                charset: self.charset as u16,
                column_length: column.data_type.max_display_size(),
                column_type: column.data_type.to_mysql_type(),
                flags: ColumnFlags::from_data_type(&column.data_type),
                decimals: 0,
            };
            self.send_packet(&column_def)?;
        }

        // 3. 发送 EOF
        self.send_packet(&EOFPacket::default())?;

        // 4. 发送行数据
        for row in rows {
            let row_packet = self.encode_row(row, schema)?;
            self.send_packet(&row_packet)?;
        }

        // 5. 发送 EOF
        self.send_packet(&EOFPacket::default())?;

        Ok(())
    }
}

/// MySQL 数据包
#[derive(Debug)]
pub struct MySQLPacket {
    /// 序列号
    sequence_id: u8,
    /// 负载
    payload: Vec<u8>,
}

impl MySQLPacket {
    /// 从字节流解析
    pub fn parse(stream: &mut BytesStream) -> Result<Self, NetworkError> {
        // 1. 读取包长度 (3 字节)
        let length = stream.read_u24()?;

        // 2. 读取序列号 (1 字节)
        let sequence_id = stream.read_u8()?;

        // 3. 读取负载
        let payload = stream.read_bytes(length)?;

        Ok(Self { sequence_id, payload })
    }

    /// 写入字节流
    pub fn write(&self, stream: &mut BytesStream) -> Result<(), NetworkError> {
        stream.write_u24(self.payload.len() as u32);
        stream.write_u8(self.sequence_id);
        stream.write_bytes(&self.payload);
        Ok(())
    }
}
```

---

## 4. 开发流程与规范

### 4.1 PR-ISSUE 工作流

```
┌─────────────────────────────────────────────────────────────────┐
│                        ISSUE 生命周期                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   [创建 ISSUE]                                                   │
│       │                                                          │
│       ▼                                                          │
│   [分配给开发者] ──► [确认需求] ──► [设计评审] ──► [开始实现]    │
│       │                      │              │                    │
│       │                      ▼              ▼                    │
│       │                 [需求变更]      [创建分支]               │
│       │                      │              │                    │
│       │                      │              ▼                    │
│       │                      │         [实现代码]                │
│       │                      │              │                    │
│       │                      │              ▼                    │
│       │                      │        [编写测试]                  │
│       │                      │              │                    │
│       │                      │              ▼                    │
│       │                      │       [提交 PR]                   │
│       │                      │              │                    │
│       │                      │              ▼                    │
│       │                      │       [代码审查]                  │
│       │                      │              │                    │
│       │                      │              ▼                    │
│       │                      │       [CI 通过]                   │
│       │                      │              │                    │
│       │                      │              ▼                    │
│       │                      │       [合并到主分支]              │
│       │                      │              │                    │
│       └──────────────────────┴──────────────┘                    │
│                                                              │
└─────────────────────────────────────────────────────────────────┘
```

### 4.2 SDD-TDD 流程

```
┌─────────────────────────────────────────────────────────────────┐
│                    SDD-TDD 开发流程                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────┐                                                  │
│  │  编写 SDD   │  设计文档 (Software Design Description)          │
│  └──────┬──────┘                                                  │
│         │                                                         │
│         ▼                                                         │
│  ┌─────────────┐                                                  │
│  │  接口评审   │  接口设计评审 + 团队确认                          │
│  └──────┬──────┘                                                  │
│         │                                                         │
│         ▼                                                         │
│  ┌─────────────┐                                                  │
│  │  写测试用例  │  先写测试 (Test-Driven Development)             │
│  └──────┬──────┘                                                  │
│         │                                                         │
│         ▼                                                         │
│  ┌─────────────┐                                                  │
│  │  实现代码   │  满足测试用例                                     │
│  └──────┬──────┘                                                  │
│         │                                                         │
│         ▼                                                         │
│  ┌─────────────┐                                                  │
│  │  CI 验证    │  编译 + 测试 + 覆盖率检查                         │
│  └──────┬──────┘                                                  │
│         │                                                         │
│         ▼                                                         │
│  ┌─────────────┐                                                  │
│  │  代码审查   │  PR 审查                                          │
│  └─────────────┘                                                  │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 4.3 CI/CD 流程

```yaml
# .github/workflows/ci.yml

name: CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          profile: minimal
          components: rustfmt, clippy

      - name: Cache dependencies
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/bin/
            ~/.cargo/registry/index/
            ~/.cargo/registry/cache/
            target/
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.toml') }}

      - name: Check format
        run: cargo fmt -- --check

      - name: Check clippy
        run: cargo clippy --all-features -- -D warnings

      - name: Build
        run: cargo build --all-features

      - name: Run tests
        run: cargo test --all-features

      - name: Generate coverage
        run: cargo tarpaulin --out Xml

      - name: Upload coverage
        uses: codecov/codecov-action@v3
        with:
          files: ./target/tarpaulin.xml
```

---

## 5. 16周上机任务规划（数据库原理 + 软件工程导论 + AI工具）

> **三主线设计**:
> - **主线一**: 数据库原理（基于教材《数据库系统原理与开发实践》）
> - **主线二**: 软件工程导论（真实公司最佳实践）
> - **主线三**: AI辅助开发工具（Claude Code / Copilot / CLI）
> - **项目载体**: SQLCC-Rust 数据库系统重构

### 5.1 专题划分与时间线（三主线对照）

```
┌─────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                              16 周 AI驱动开发学习路径                                             │
├─────────────────────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                                  │
│  开学前2周 [AI工具准备]                                                                          │
│  ◀────────────────────── Claude Code / Copilot / IDE 配置 ───────────────────────────────────▶│
│                                                                                                  │
│  ╔═══════════════════════════════════════════════════════════════════════════════════════════════╗ │
│  ║  专题一：AI辅助工程化基础 (第1-2周)                                                         ║ │
│  ║  AI工具链配置 + Git Flow + CI/CD + Claude Code 入门                                         ║ │
│  ╚═══════════════════════════════════════════════════════════════════════════════════════════════╝ │
│                                    │                                                                │
│                                    ▼                                                                │
│  ╔═══════════════════════════════════════════════════════════════════════════════════════════════╗ │
│  ║  专题二：AI辅助SQL解析器 (第3-4周)                                                           ║ │
│  ║  Claude Code 生成Lexer/Parser + TDD测试 + Copilot 补全                                        ║ │
│  ╚═══════════════════════════════════════════════════════════════════════════════════════════════╝ │
│                                    │                                                                │
│                                    ▼                                                                │
│  ╔═══════════════════════════════════════════════════════════════════════════════════════════════╗ │
│  ║  专题三：AI辅助存储引擎 (第5-6周)                                                             ║ │
│  ║  AI生成Page/BufferPool + 性能分析 + Debug技巧                                                  ║ │
│  ╚═══════════════════════════════════════════════════════════════════════════════════════════════╝ │
│                                    │                                                                │
│                                    ▼                                                                │
│  ╔═══════════════════════════════════════════════════════════════════════════════════════════════╗ │
│  ║  专题四：AI辅助性能优化 (第7-8周) ████████████████ 期中前 ████████████████                  ║ │
│  ║  AI Profiling + 优化决策 + 基准测试 + CodeRabbit AI 审查                                      ║ │
│  ╚═══════════════════════════════════════════════════════════════════════════════════════════════╝ │
│                                    │                                                                │
│                                    ▼                                                                │
│  ╔═══════════════════════════════════════════════════════════════════════════════════════════════╗ │
│  ║                              期中汇报演示 (第8周末)                                          ║ │
│  ╚═══════════════════════════════════════════════════════════════════════════════════════════════╝ │
│                                    │                                                                │
│                                    ▼                                                                │
│  ╔═══════════════════════════════════════════════════════════════════════════════════════════════╗ │
│  ║  专题五：AI辅助重构优化 (第9-10周)                                                            ║ │
│  ║  Claude Code 重构 + 设计模式 + AI代码审查                                                     ║ │
│  ╚═══════════════════════════════════════════════════════════════════════════════════════════════╝ │
│                                    │                                                                │
│                                    ▼                                                                │
│  ╔═══════════════════════════════════════════════════════════════════════════════════════════════╗ │
│  ║  专题六：AI辅助事务实现 (第11-12周)                                                           ║ │
│  ║  AI生成事务代码 + 并发测试 + 调试技巧                                                         ║ │
│  ╚═══════════════════════════════════════════════════════════════════════════════════════════════╝ │
│                                    │                                                                │
│                                    ▼                                                                │
│  ╔═══════════════════════════════════════════════════════════════════════════════════════════════╗ │
│  ║  专题七：AI辅助可观测性 (第13-14周)                                                           ║ │
│  ║  AI生成监控代码 + 日志规范 + 告警配置                                                          ║ │
│  ╚═══════════════════════════════════════════════════════════════════════════════════════════════╝ │
│                                    │                                                                │
│                                    ▼                                                                │
│  ╔═══════════════════════════════════════════════════════════════════════════════════════════════╗ │
│  ║  专题八：AI辅助集成交付 (第15-16周)                                                           ║ │
│  ║  AI生成测试/部署 + 文档自动化 + 演示准备                                                       ║ │
│  ╚═══════════════════════════════════════════════════════════════════════════════════════════════╝ │
│                                                                                                  │
└─────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

### 5.2 每周AI工具使用清单

| 周次 | OpenClaw | OpenSpec | Agent Team | Claude Code | Copilot | CLI | 其他 |
|:----:|:---------:|:--------:|:----------:|:-----------:|:-------:|:---:|:----:|
| 1-2 | 项目配置 | SDD规范 | 任务分发 | 项目配置 | 代码补全 | uv, rg | IDE配置 |
| 3-4 | Parser开发 | TDD验证 | 并行测试 | Parser生成 | 语法补全 | bat, fd | 测试框架 |
| 5-6 | 存储开发 | 规范检查 | 冲突解决 | 存储生成 | 模式补全 | htop | GDB调试 |
| 7-8 | 优化分析 | 性能规范 | 进度追踪 | 优化分析 | 优化补全 | flamegraph | 基准测试 |
| 9-10 | 重构任务 | 重构规范 | 结果聚合 | 重构建议 | 重构补全 | lazygit |复杂度分析 |
| 11-12 | 事务开发 | ACID规范 | 并发协调 | 事务生成 | 并发补全 | tmux | 压力测试 |
| 13-14 | 监控开发 | 监控规范 | 告警配置 | 监控生成 | 日志补全 | kubectl | Grafana |
| 15-16 | 部署任务 | 发布规范 | 交付确认 | 部署脚本 | 文档补全 | docker | K9s |

### 5.3 教材章节与知识点对照表

| 周次 | 数据库原理 | 软件工程 | AI工具 | 实践任务 |
|:----:|:----------:|:--------:|:------:|:--------:|
| 1-2 | 第1-2章 | 版本控制、CI/CD | OpenClaw + OpenSpec | 项目骨架 |
| 3-4 | 第3章 | TDD、编译原理 | Agent Team | SQL Parser |
| 5-6 | 第4-6章 | 面向对象、SOLID | 冲突解决 | 存储引擎 |
| 7-8 | 第7章 | Profiling、质量 | 进度追踪 | 性能优化 |
| 9-10 | 附录A | 重构、设计模式 | 结果聚合 | 架构重构 |
| 11-12 | 第8章 | 并发、需求分析 | 并发协调 | 事务管理 |
| 13-14 | 第9章 | DevOps、监控 | 告警配置 | 监控运维 |
| 15-16 | 第10章 | 容器化、文档 | 交付确认 | 集成交付 |
| 9-10 | 附录A：实践指南 | 重构、设计模式 | 架构重构 |
| 11-12 | 第8章：事务处理 | 需求分析、并发控制 | 事务管理 |
| 13-14 | 第9章：网络通信 | DevOps、可观测性 | 监控运维 |
| 15-16 | 第10章：云安全 | 项目管理、文档 | 集成交付 |

---

## 6. 专题详细规划（三主线融合）

### 6.1 专题一：AI辅助工程化基础

**周次**: 开学前2周 + 第1-2周

**AI工具教学目标**:
1. 掌握 Claude Code 基本使用方法
2. 配置 GitHub Copilot
3. 搭建高效开发环境（IDE + CLI）
4. 理解 Git Flow 工作流

| 周次 | 主题 | AI工具实践 | 交付物 |
|:----:|:-----|:-----------|:--------:|
| 开学前 | 环境准备 | Claude Code 配置项目骨架 | Rust环境、IDE配置 |
| 第1周 | 项目结构 | Copilot 生成Cargo配置 | 项目骨架完成 |
| 第2周 | CI/CD | GitHub Actions 流水线 | CI绿色运行 |

**Claude Code 示例**:

```bash
/claude-code
> 帮我创建一个 Rust 数据库项目的初始结构，包括：
> 1. Cargo.toml 工作空间配置
> 2. sqlcc-parser 子 crate
> 3. sqlcc-storage 子 crate
> 4. sqlcc-executor 子 crate
> 5. 基础的 CI/CD 配置
```

**验收标准**:
- Claude Code 能正常工作
- GitHub Copilot 能提供代码补全
- CI 流水线绿色
- 每个 crate 有 README

### 6.2 专题二：AI辅助SQL解析器

**周次**: 第3-4周

**AI工具教学目标**:
1. 使用 Claude Code 生成 Lexer/Parser
2. TDD 模式开发
3. AI 生成测试用例
4. Copilot 辅助代码补全

| 周次 | 主题 | 教材章节 | AI实践 | 交付物 |
|:----:|:-----|:--------:|:-------|:--------:|
| 第3周 | 词法分析 | 第3章 | Claude生成Lexer | Token定义 |
| 第4周 | 语法分析 | 第3-5章 | TDD开发Parser | AST结构 |

**TDD with Claude Code**:

```bash
/claude-code
/test-driven
> 为 sqlcc-parser 的词法分析器编写测试：
>
> 测试用例：
> 1. 关键字识别（SELECT, FROM, WHERE等）
> 2. 标识符解析
> 3. 数字解析
> 4. 字符串解析
> 5. 运算符解析
>
> 预期：每个测试先失败，再通过
```

**验收标准**:
- Parser 能解析 SQL-92 DDL/DML
- 测试覆盖率 >= 80%
- 提交 5+ 个 PR

### 6.3 专题三：AI辅助存储引擎

**周次**: 第5-6周

**AI工具教学目标**:
1. Claude Code 生成存储代码
2. AI 辅助调试（GDB + AI）
3. 性能监控工具

| 周次 | 主题 | 教材章节 | AI实践 | 交付物 |
|:----:|:-----|:--------:|:-------|:--------:|
| 第5周 | 页面管理 | 第4-6章 | AI生成Page结构 | Page读写 |
| 第6周 | BufferPool | 第6章 | AI生成LRU | 缓存实现 |

**AI辅助调试示例**:

```bash
/claude-code
> 我的 BufferPool 在并发测试时出现 data race，请帮我：
>
> 1. 分析可能的问题点（在 buffer_pool.rs:156 附近）
> 2. 建议修复方案
> 3. 生成修复后的代码
> 4. 建议如何添加线程安全测试
```

**验收标准**:
- BufferPool 线程安全
- 能用 htop/btop 监控性能
- 完成性能基准

### 6.4 专题四：AI辅助性能优化（期中前重点）

**周次**: 第7-8周

**AI工具教学目标**:
1. AI Profiling 分析
2. 性能优化决策
3. CodeRabbit AI 审查

| 周次 | 主题 | 教材章节 | AI实践 | 交付物 |
|:----:|:-----|:--------:|:-------|:--------:|
| 第7周 | Profiling | 第7章 | flamegraph分析 | 瓶颈报告 |
| 第8周 | 优化 | 第7章 | AI辅助优化 | 性能提升 |

**AI性能优化示例**:

```bash
/claude-code
> 使用 cargo flamegraph 生成了火焰图，发现：
> - calculate_checksum 占用 40% CPU
> - page_lookup 频繁调用
>
> 请帮我：
> 1. 解释火焰图中的热点
> 2. 提出优化建议
> 3. 生成优化后的代码
> 4. 给出性能提升的预估
```

**期中汇报检查清单**:
- SQL Parser + 存储引擎演示
- 性能基准报告（优化前后对比）
- 代码质量指标
- AI工具使用总结
- 提交 3 个以上 PR

### 6.3 专题三：存储引擎核心

**数据库主线**:
- 教材章节：第4章《DBMS总体架构设计》+ 第6章《存储引擎与执行器》
- 教学目标：理解DBMS三层架构
- 核心内容：页面管理、缓冲池、存储格式

**软件工程主线**:
- 面向对象设计原则（SOLID）
- 缓存策略设计
- 并发安全

| 周次 | 日期 | 主题 | 教材内容 | 实践任务 |
|:----:|:----:|:-----|:--------:|:--------:|
| 第5周 | 03-30~04-05 | 页面管理 | 第4章1-2节 | Page结构 |
| 第6周 | 04-06~04-12 | 缓冲池 | 第6章3-4节 | BufferPool |

**验收标准**:
- Page读写正确
- BufferPool 命中率可观测
- 至少1个设计模式应用

### 6.4 专题四：性能优化专题（期中前重点）

**数据库主线**:
- 教材章节：第7章《索引系统与查询优化》
- 教学目标：理解查询优化原理
- 核心内容：代价模型、谓词下推、索引选择

**软件工程主线**:
- Profiling工具链
- 性能基准测试
- 优化决策方法论

| 周次 | 日期 | 主题 | 教材内容 | 实践任务 |
|:----:|:----:|:-----|:--------:|:--------:|
| 第7周 | 04-13~04-19 | Profiling | 第7章1-2节 | perf/flamegraph |
| 第8周 | 04-20~04-26 | 优化实现 | 第7章3-5节 | 谓词下推 |

**期中汇报内容**:
- SQL Parser + 存储引擎演示
- 性能分析报告
- 代码质量指标

### 6.5 专题五：AI辅助重构与架构优化

**周次**: 第9-10周

**AI工具教学目标**:
1. Claude Code 重构建议
2. AI 辅助设计模式
3. CodeRabbit 代码审查

| 周次 | 主题 | AI实践 | 交付物 |
|:----:|:-----|:-------|:--------:|
| 第9周 | 技术债务 | Claude分析 | 重构计划 |
| 第10周 | 重构实践 | Copilot补全 | 优化代码 |

**AI重构示例**:

```bash
/claude-code
/refactor
> 分析 sqlcc-parser 中的重复代码，识别可以提取的模式：
>
> 约束：
> - 保持公开接口不变
> - 添加测试确保功能正确
> - 使用 Rust 最佳实践
>
> 建议的重构：
> 1. 提取公共表达式解析器
> 2. 使用 trait 统一接口
> 3. 减少重复的匹配逻辑
```

### 6.6 专题六：AI辅助事务实现

**周次**: 第11-12周

**AI工具教学目标**:
1. Claude Code 生成事务代码
2. AI 辅助并发测试
3. 压力测试

| 周次 | 主题 | AI实践 | 交付物 |
|:----:|:-----|:-------|:--------:|
| 第11周 | ACID基础 | AI生成2PL | 锁实现 |
| 第12周 | MVCC | AI生成MVCC | 版本管理 |

**AI辅助事务测试**:

```bash
/claude-code
> 为事务管理器生成并发测试用例：
>
> 测试场景：
> 1. 两个事务同时更新同一行（应检测死锁）
> 2. 读已提交隔离级别测试
> 3. 可重复读隔离级别测试
> 4. 崩溃恢复测试
>
> 使用 rust-criterion 生成基准测试
```

### 6.7 专题七：AI辅助可观测性

**周次**: 第13-14周

**AI工具教学目标**:
1. AI 生成监控代码
2. 日志规范
3. 告警配置

| 周次 | 主题 | AI实践 | 交付物 |
|:----:|:-----|:-------|:--------:|
| 第13周 | 指标监控 | Prometheus埋点 | /metrics端点 |
| 第14周 | 日志追踪 | 结构化日志 | 日志系统 |

**AI生成监控代码示例**:

```bash
/claude-code
> 为 sqlcc-storage 添加 Prometheus 指标：
>
> 需要的指标：
> 1. buffer_pool_hits_total - BufferPool 命中次数
> 2. buffer_pool_misses_total - BufferPool 未命中次数
> 3. page_read_total - 页面读取次数
> 4. page_write_total - 页面写入次数
> 5. active_transactions - 活跃事务数
>
> 使用 prometheus-client Rust 库
```

### 6.8 专题八：AI辅助集成交付

**周次**: 第15-16周

**AI工具教学目标**:
1. AI 生成部署脚本
2. 文档自动化
3. 演示准备

| 周次 | 主题 | AI实践 | 交付物 |
|:----:|:-----|:-------|:--------:|
| 第15周 | 测试集成 | AI生成E2E测试 | 测试套件 |
| 第16周 | 部署交付 | Docker/K8s | 期末演示 |

---

## 7. 软件工程核心技能训练点

### 7.1 版本控制与协作

| 技能 | 实践任务 | 验收标准 |
|:----:|:--------:|:--------:|
| Git Flow | 功能分支开发 | 正确使用分支 |
| PR 审查 | 代码审查实践 | 审查意见有建设性 |
| 版本标签 | 发布管理 | 语义化版本 |

### 7.2 自动化流水线

| 技能 | 实践任务 | 验收标准 |
|:----:|:--------:|:--------:|
| CI 构建 | GitHub Actions | 每次提交自动构建 |
| 代码质量 | Clippy/Fmt | 零警告通过 |
| 覆盖率 | Tarpaulin | >= 80% |

### 7.3 测试策略

| 技能 | 实践任务 | 验收标准 |
|:----:|:--------:|:--------:|
| 单元测试 | 每个函数测试 | 分支覆盖 >= 80% |
| 集成测试 | 模块间测试 | 接口测试通过 |

---

## 8. 评分标准（AI辅助软件工程）

| 类别 | 权重 | 评分细则 |
|:----:|:----:|:--------:|
| **代码质量** | 25% | 编译(8%) + 测试(8%) + 规范(9%) |
| **功能完成** | 20% | 实现功能完整性 |
| **多Agent协同** | 15% | OpenClaw(5%) + OpenSpec(5%) + Agent Team(5%) |
| **AI工具使用** | 15% | Claude Code(8%) + Copilot(4%) + CLI(3%) |
| **软件工程实践** | 15% | Git(5%) + CI/CD(5%) + 文档(5%) |
| **协作能力** | 10% | 代码审查(5%) + PR质量(5%) |
| **学习成长** | 5% | 期中/期末报告 |

---

## 9. AI工具速查手册

### 9.1 OpenClaw 多Agent框架

```bash
# 启动多Agent协作任务
openclaw --task "实现SQL Parser" --agents architect,developer,tester

# 指定Agent数量
openclaw --parallel --agents 4 "并行实现存储模块"

# 查看任务状态
openclaw status
```

### 9.2 OpenSpec 规范控制

```bash
# 验证SDD规范
openspec validate sdd --file docs/sdd/parser.md

# 验证TDD规范
openspec validate tdd --coverage 80

# 检查接口契约
openspec check contracts --strict

# 生成规范报告
openspec report --format markdown
```

### 9.3 Agent Team 并行协调

```bash
# 启动Agent团队
agent-team init --project sqlcc

# 分发任务
agent-team dispatch --tasks "parser,storage,executor"

# 查看进度
agent-team progress

# 解决冲突
agent-team resolve --conflict <id>
```

### 9.4 Claude Code 命令

| 命令 | 功能 | 示例 |
|:-----|:-----|:-----|
| `/task` | 执行明确任务 | `/task 实现 BufferPool` |
| `/test-driven` | TDD 模式 | `/test-driven 测试 Lexer` |
| `/review` | 代码审查 | `/review 检查 Parser` |
| `/refactor` | 重构建议 | `/refactor 简化代码` |
| `/explain` | 解释代码 | `/explain 这段代码逻辑` |
| `/debug` | 调试辅助 | `/debug 定位 bug` |

### 9.2 GitHub Copilot 快捷键

| 快捷键 | 功能 |
|:-------|:-----|
| `Tab` | 接受补全 |
| `Ctrl+Enter` | 显示所有建议 |
| `Esc` | 拒绝建议 |
| `Alt+]` | 下一个建议 |
| `Alt+[` | 上一个建议 |

### 9.3 高效 CLI 命令速查

```bash
# 代码搜索
rg "fn parse" src/           # 搜索函数
rg -t rust "TODO"            # 搜索 TODO

# 文件查找
fd "*.toml"                  # 查找配置
fd -e rs -x wc -l | sort -rn # 统计代码行数

# Git 操作
git log --oneline -10        # 查看提交
gh pr list                   # 查看 PR

# 开发循环
cargo watch -x test          # 监控并测试
cargo watch -x clippy        # 监控并检查

# 性能分析
cargo flamegraph             # 生成火焰图
cargo bench                 # 运行基准测试
```

---

## 9. 质量保证

### 9.1 编译要求

```bash
cargo build --all-targets
cargo clippy --all-features -- -D warnings
```

### 9.2 测试要求

| 类型 | 覆盖率要求 |
|:----:|:----------:|
| 单元测试 | >= 80% |
| 集成测试 | >= 60% |
| 端到端测试 | >= 40% |

---

## 10. 教材章节速查

| 章节 | 标题 | 对应周次 |
|:----:|:-----|:--------:|
| 第1章 | 数据处理的起源与思想演变 | 1-2 |
| 第2章 | 计算机技术与数据库软件的交织发展 | 1-2 |
| 第3章 | 关系数据库理论基础 | 3-4 |
| 第4章 | DBMS总体架构设计 | 5-6 |
| 第5章 | SQL解析器实现 | 3-4 |
| 第6章 | 存储引擎与执行器 | 5-6 |
| 第7章 | 索引系统与查询优化 | 7-8 |
| 第8章 | OLTP事务处理 | 11-12 |
| 第9章 | 数据库网络通信与安全 | 13-14 |
| 第10章 | 云端零信任数据库安全 | 15-16 |
| 附录A | 实验与实践指南 | 全程 |

---

> **设计评审检查清单**
> - [ ] 教材内容与实践任务对应关系清晰
> - [ ] 软件工程技能点明确
> - [ ] 评分标准公平合理
> - [ ] 时间线可执行

### 5.1 专题划分与时间线

```
┌─────────────────────────────────────────────────────────────────────────────────────┐
│                              16 周 专 题 时 间 线                                    │
├─────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                     │
│  开学前2周 [项目启动]                                                                │
│  ◀─────────────────────────────  提前完成  ───────────────────────────────────────▶│
│                                                                                     │
│  ╔═══════════════════════════════════════════════════════════════════════════════╗ │
│  ║  专题一：工程化基础 + 开发环境 (第1-2周)                                        ║ │
│  ║  ─────────────────────────────────────────────────────────────────────────── ║ │
│  ║  · CI/CD 流水线 · 代码规范 · Git Flow · 第一轮迭代                              ║ │
│  ╚═══════════════════════════════════════════════════════════════════════════════╝ │
│                                    │                                                  │
│                                    ▼                                                  │
│  ╔═══════════════════════════════════════════════════════════════════════════════╗ │
│  ║  专题二：SQL 解析器实现 (第3-4周)                                                ║ │
│  ║  ─────────────────────────────────────────────────────────────────────────── ║ │
│  ║  · 词法分析 · 语法分析 · AST 构建 · 错误恢复                                    ║ │
│  ╚═══════════════════════════════════════════════════════════════════════════════╝ │
│                                    │                                                  │
│                                    ▼                                                  │
│  ╔═══════════════════════════════════════════════════════════════════════════════╗ │
│  ║  专题三：存储引擎核心 (第5-6周)                                                  ║ │
│  ║  ─────────────────────────────────────────────────────────────────────────── ║ │
│  ║  · Page 管理 · Buffer Pool · 磁盘 I/O · 数据编码                               ║ │
│  ╚═══════════════════════════════════════════════════════════════════════════════╝ │
│                                    │                                                  │
│                                    ▼                                                  │
│  ╔═══════════════════════════════════════════════════════════════════════════════╗ │
│  ║  专题四：性能优化专题 (第7-8周) ████████ 期中前 ████████                        ║ │
│  ║  ─────────────────────────────────────────────────────────────────────────── ║ │
│  ║  · Profiling · 瓶颈定位 · 优化策略 · 基准测试                                   ║ │
│  ╚═══════════════════════════════════════════════════════════════════════════════╝ │
│                                    │                                                  │
│                                    ▼                                                  │
│  ╔═══════════════════════════════════════════════════════════════════════════════╗ │
│  ║                          期中汇报演示 (第8周末)                                  ║ │
│  ╚═══════════════════════════════════════════════════════════════════════════════╝ │
│                                    │                                                  │
│                                    ▼                                                  │
│  ╔═══════════════════════════════════════════════════════════════════════════════╗ │
│  ║  专题五：重构与架构优化 (第9-10周)                                                ║ │
│  ║  ─────────────────────────────────────────────────────────────────────────── ║ │
│  ║  · 技术债务 · 模块解耦 · 接口抽象 · 设计模式                                     ║ │
│  ╚═══════════════════════════════════════════════════════════════════════════════╝ │
│                                    │                                                  │
│                                    ▼                                                  │
│  ╔═══════════════════════════════════════════════════════════════════════════════╗ │
│  ║  专题六：高级功能增强 (第11-12周)                                                ║ │
│  ║  ─────────────────────────────────────────────────────────────────────────── ║ │
│  ║  · 事务管理 · MVCC · 并发控制 · WAL                                             ║ │
│  ╚═══════════════════════════════════════════════════════════════════════════════╝ │
│                                    │                                                  │
│                                    ▼                                                  │
│  ╔═══════════════════════════════════════════════════════════════════════════════╗ │
│  ║  专题七：可观测性与运维 (第13-14周)                                               ║ │
│  ║  ─────────────────────────────────────────────────────────────────────────── ║ │
│  ║  · 指标监控 · 日志系统 · 链路追踪 · 告警机制                                     ║ │
│  ╚═══════════════════════════════════════════════════════════════════════════════╝ │
│                                    │                                                  │
│                                    ▼                                                  │
│  ╔═══════════════════════════════════════════════════════════════════════════════╗ │
│  ║  专题八：系统集成与交付 (第15-16周)                                               ║ │
│  ║  ─────────────────────────────────────────────────────────────────────────── ║ │
│  ║  · 完整功能 · 端到端测试 · 文档完善 · 期末演示                                   ║ │
│  ╚═══════════════════════════════════════════════════════════════════════════════╝ │
│                                                                                     │
└─────────────────────────────────────────────────────────────────────────────────────┘
```

### 5.2 专题一：工程化基础 + 开发环境

**真实公司实践重点**:
- 标准化开发流程
- 自动化一切
- 代码质量门禁
- 文档即代码

| 周次 | 日期 | 公司实践 | 实验室任务 | 项目进度 |
|------|------|---------|-----------|---------|
| 开学前2周 | ~02-28 | 环境标准化、工具链统一 | 开发环境配置、CI 搭建 | 项目骨架完成 |
| 第1周 | 03-02~03-08 | 代码规范、静态检查 | Cargo 项目结构、Clippy 修复 | 第1轮迭代完成 |
| 第2周 | 03-09~03-15 | 小步提交、持续集成 | Git Flow 流程、PR 模板 | 第2轮迭代完成 |

**验收标准**:
- [ ] `cargo build` 零警告
- [ ] `cargo test` 通过率 100%
- [ ] GitHub Actions CI 流水线绿色
- [ ] 每个模块有 README + API 文档

**演示内容**:
- CI/CD 自动化流程
- 代码质量门禁
- 文档自动生成

### 5.3 专题二：SQL 解析器实现

**真实公司实践重点**:
- 编译器设计原则
- 错误处理与恢复
- 测试覆盖
- AST 可扩展设计

| 周次 | 日期 | 公司实践 | 实验室任务 | 项目进度 |
|------|------|---------|-----------|---------|
| 第3周 | 03-16~03-22 | 词法分析、Token 化 | Lexer 实现、关键字识别 | DDL 解析 |
| 第4周 | 03-23~03-29 | 语法分析、递归下降 | Parser 实现、AST 生成 | DML 解析 |

**技术难点**:
1. **歧义语法处理** - 如何处理 `SELECT * FROM t WHERE a = 1 AND b = 2`
2. **错误恢复** - 遇到语法错误时如何给出有意义的提示
3. **左递归消除** - 表达式解析的经典问题

**验收标准**:
- [ ] 通过 SQL-92 测试用例集
- [ ] 解析器测试覆盖率 >= 80%
- [ ] 错误信息友好、定位准确

**演示内容**:
- SQL 语句解析过程
- AST 可视化
- 错误提示演示

### 5.4 专题三：存储引擎核心

**真实公司实践重点**:
- 内存管理
- I/O 优化
- 数据布局
- 缓存策略

| 周次 | 日期 | 公司实践 | 实验室任务 | 项目进度 |
|------|------|---------|-----------|---------|
| 第5周 | 03-30~04-05 | Page 布局、磁盘 I/O | Page 结构、读写实现 | 基础存储 |
| 第6周 | 04-06~04-12 | 缓存策略、LRU 实现 | Buffer Pool、替换策略 | 完整存储 |

**技术难点**:
1. **Buffer Pool 锁定** - 并发访问的线程安全
2. **脏页淘汰** - 何时、如何写回磁盘
3. **页分裂** - 插入数据时的空间管理

**验收标准**:
- [ ] 存储引擎可独立运行
- [ ] 读写测试通过
- [ ] Buffer Pool 命中率可观测

**演示内容**:
- Page 读写流程
- Buffer Pool 命中/未命中
- 脏页写回时机

### 5.5 专题四：性能优化专题

**真实公司实践重点**:
- Profiling 工具链
- 瓶颈分析方法
- 优化决策树
- 基准测试

> **期中前重点**: 性能优化是真实公司的核心竞争力

| 周次 | 日期 | 公司实践 | 实验室任务 | 项目进度 |
|------|------|---------|-----------|---------|
| 第7周 | 04-13~04-19 | CPU Profiling、火焰图 | perf/flamegraph 集成 | 瓶颈定位 |
| 第8周 | 04-20~04-26 | 优化策略、缓存优化 | 代码优化、算法改进 | 性能提升 |

**性能优化方法论**:
```
┌─────────────────────────────────────────────────────────────────┐
│                    性能优化决策树                                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   [发现性能问题]                                                 │
│         │                                                        │
│         ▼                                                        │
│   [Profiling] ──► [CPU 瓶颈] ──► 算法优化/向量化                 │
│         │                                                        │
│         ├──► [I/O 瓶颈] ──► 缓存优化/批量 I/O                    │
│         │                                                        │
│         ├──► [内存瓶颈] ──► 减少分配/内存池                       │
│         │                                                        │
│         └──► [锁竞争] ──► 并发策略调整                           │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**验收标准**:
- [ ] 建立性能基准线
- [ ] 完成至少 3 项优化
- [ ] 性能提升 >= 20%
- [ ] 有完整的优化报告

**期中汇报内容**:
- 性能分析报告
- 优化效果展示
- 端到端 SQL 执行演示

### 5.6 专题五：重构与架构优化

**真实公司实践重点**:
- 技术债务识别
- 模块解耦
- 接口抽象
- 设计模式应用

> **期中后重点**: 代码重构是保持系统健康的关键

| 周次 | 日期 | 公司实践 | 实验室任务 | 项目进度 |
|------|------|---------|-----------|---------|
| 第9周 | 04-27~05-03 | 代码度量、技术债务 | 代码分析、重构规划 | 重构设计 |
| 第10周 | 05-04~05-10 | 重构安全、接口隔离 | 实施重构、模式应用 | 架构改善 |

**重构策略**:
```
┌─────────────────────────────────────────────────────────────────┐
│                    重构优先级矩阵                                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   高影响 + 低风险 ──► 立即执行 (提取函数、统一命名)              │
│   高影响 + 高风险 ──► 计划执行 (需要测试覆盖)                    │
│   低影响 + 低风险 ──► 空闲时执行 (格式化、注释)                   │
│   低影响 + 高风险 ──► 暂时搁置 (需要架构变更)                     │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**验收标准**:
- [ ] 代码复杂度下降
- [ ] 模块依赖关系清晰
- [ ] 接口抽象合理
- [ ] 重构后功能测试通过

### 5.7 专题六：高级功能增强

**真实公司实践重点**:
- ACID 事务
- 并发控制
- 故障恢复
- 一致性保证

| 周次 | 日期 | 公司实践 | 实验室任务 | 项目进度 |
|------|------|---------|-----------|---------|
| 第11周 | 05-11~05-17 | 事务隔离、锁管理 | 2PL 实现、锁粒度 | 基础事务 |
| 第12周 | 05-18~05-24 | MVCC、版本管理 | 读已提交/可重复读 | 完整事务 |

**验收标准**:
- [ ] 支持 BEGIN/COMMIT/ROLLBACK
- [ ] 隔离级别可配置
- [ ] 崩溃恢复正常
- [ ] 事务测试用例通过

### 5.8 专题七：可观测性与运维

**真实公司实践重点**:
- 指标监控
- 日志规范
- 链路追踪
- 告警设计

| 周次 | 日期 | 公司实践 | 实验室任务 | 项目进度 |
|------|------|---------|-----------|---------|
| 第13周 | 05-25~05-31 | 指标采集、Prometheus | 指标系统、仪表盘 | 基础监控 |
| 第14周 | 06-01~06-07 | 日志规范、Trace | 日志系统、Trace | 完整可观测 |

**可观测性三支柱**:
```
┌─────────────────────────────────────────────────────────────────┐
│                    可观测性三支柱                                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   [Metrics]          [Logs]           [Traces]                   │
│   ─────────         ───────         ─────────                   │
│   · QPS             · 操作日志       · 请求链路                  │
│   · 延迟分布         · 错误日志       · 瓶颈定位                  │
│   · 错误率          · 审计日志       · 依赖分析                  │
│                                                                  │
│   [Alerting]
│   ─────────
│   · 阈值告警
│   · 异常检测
│   · 告警降噪
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**验收标准**:
- [ ] Prometheus 指标可采集
- [ ] 结构化日志输出
- [ ] 关键操作可追踪
- [ ] Grafana 仪表盘

### 5.9 专题八：系统集成与交付

**真实公司实践重点**:
- 端到端测试
- 部署流水线
- 版本发布
- 用户文档

| 周次 | 日期 | 公司实践 | 实验室任务 | 项目进度 |
|------|------|---------|-----------|---------|
| 第15周 | 06-08~06-14 | E2E 测试、回归测试 | 完整测试套件 | 全面测试 |
| 第16周 | 06-15~06-21 | 发布流程、文档 | Docker 镜像、用户手册 | 期末交付 |

**验收标准**:
- [ ] 所有功能测试通过
- [ ] Docker 镜像可运行
- [ ] 完整用户手册
- [ ] 期末演示成功

---

### 5.10 项目与作业时间差

```
┌─────────────────────────────────────────────────────────────────────────────────────┐
│                         项目演进 vs 作业时间线                                         │
├─────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                     │
│  周次    项目进度              上机任务                                               │
│  ────────────────────────────────────────────────────────────────────────────────  │
│                                                                                     │
│   开学前  项目骨架完成 ──────►►►  学生配置环境                                        │
│                                                                                     │
│   第1周   第1轮迭代完成  ──────►►►  专题一：工程化基础 (上机1)                       │
│   第2周   第2轮迭代完成  ──────►►►                                                    │
│                                                                                     │
│   第3周   DDL解析完成   ──────►►►  专题二：SQL解析 (上机2)                          │
│   第4周   DML解析完成   ──────►►►                                                    │
│                                                                                     │
│   第5周   基础存储完成  ──────►►►  专题三：存储引擎 (上机3)                          │
│   第6周   完整存储完成  ──────►►►                                                    │
│                                                                                     │
│   第7周   瓶颈定位完成  ──────►►►  专题四：性能优化 (上机4 - 期中前)                │
│   第8周   性能优化完成  ──────►►►  期中汇报演示                                      │
│                                                                                     │
│   第9周   重构设计完成  ──────►►►  专题五：重构优化 (上机5)                          │
│   第10周  架构改善完成  ──────►►►                                                    │
│                                                                                     │
│   第11周  基础事务完成  ──────►►►  专题六：高级功能 (上机6)                          │
│   第12周  完整事务完成  ──────►►►                                                    │
│                                                                                     │
│   第13周  基础监控完成  ──────►►►  专题七：可观测性 (上机7)                          │
│   第14周  完整可观测    ──────►►►                                                    │
│                                                                                     │
│   第15周  全面测试完成  ──────►►►  专题八：集成交付 (上机8 - 期末)                   │
│   第16周  期末交付完成  ──────►►►  期末演示                                          │
│                                                                                     │
│  ◄───────────────────────────────────────────────────────  领先 2 周  ─────────────►│
│                                                                                     │
└─────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 6. 计划

### 6.1 第一阶段：基础功能 (2026-02-15 ~ 2026-03-01)

| 周次 | 日期 | 任务 | 交付物 |
|------|------|------|--------|
| 第1周 | 02-15 ~ 02-21 | 项目脚手架、CI/CD 配置 | 可构建的项目骨架 |
| 第2周 | 02-22 ~ 02-28 | SQL Parser (DDL+DML) | 语法解析器 |
| 第3周 | 03-01 ~ 03-07 | MySQL 协议兼容 | 客户端连接演示 |

### 5.2 第二阶段：核心功能 (2026-03-08 ~ 2026-04-15)

| 周次 | 日期 | 任务 | 交付物 |
|------|------|------|--------|
| 第4周 | 03-08 ~ 03-14 | 存储引擎 (Buffer Pool + Page) | 基础存储 |
| 第5周 | 03-15 ~ 03-21 | 执行引擎 (Volcano) | 基础执行器 |
| 第6周 | 03-22 ~ 03-28 | 事务 (MVCC + WAL) | ACID 事务 |
| 第7周 | 03-29 ~ 04-04 | 优化器 (谓词下推) | 查询优化 |
| 第8周 | 04-05 ~ 04-11 | 测试与文档完善 | 期中交付 |

### 5.3 第三阶段：扩展功能 (2026-04-12 ~ 2026-06-30)

| 周次 | 日期 | 任务 | 交付物 |
|------|------|------|--------|
| 第9-10周 | 04-12 ~ 04-25 | SQL-99 (窗口函数、CTE) | SQL-99 支持 |
| 第11-12周 | 04-26 ~ 05-09 | 索引优化 | B+ 树增强 |
| 第13-14周 | 05-10 ~ 05-23 | 监控与运维 | 可观测性 |
| 第15-16周 | 05-24 ~ 06-06 | 性能优化 | 性能提升 |
| 第17-18周 | 06-07 ~ 06-30 | 期末交付 | 完整系统 |

---

## 6. 质量保证

### 6.1 编译要求

```bash
# 必须 100% 编译通过
cargo build --all-features --all-targets

# 禁止警告
cargo clippy --all-features -- -D warnings
cargo fmt -- --check
```

### 6.2 测试要求

| 类型 | 覆盖率要求 | 说明 |
|------|------------|------|
| 单元测试 | >= 80% | 每个模块独立测试 |
| 集成测试 | >= 60% | 模块间协作测试 |
| 端到端测试 | >= 40% | 完整流程测试 |

### 6.3 性能要求

| 指标 | 目标 | 测量方法 |
|------|------|----------|
| 编译时间 | < 5分钟 | clean build |
| 单查询延迟 | < 10ms | 简单 SELECT |
| 并发查询 | > 3000 ops/sec | 8线程并发 |
| 启动时间 | < 2秒 | 服务启动 |

---

## 7. 文档要求

### 7.1 必需文档列表

| 文档 | 位置 | 更新频率 |
|------|------|----------|
| README.md | 根目录 | 每个里程碑 |
| API 文档 | docs/api/ | 每个 PR |
| SDD | docs/sdd/ | 每个模块 |
| 用户手册 | docs/user-guide/ | 每个里程碑 |
| 开发指南 | docs/development/ | 每周 |
| 变更日志 | CHANGELOG.md | 每次发布 |

### 7.2 代码注释规范

```rust
/// 模块简短说明
///
/// # Examples
///
/// ```rust
/// use sqlcc_parser::Parser;
///
/// let sql = "SELECT * FROM t";
/// let ast = Parser::new(sql).parse().unwrap();
/// ```
pub struct Parser { /* ... */ }

/// 函数详细说明
///
/// ## Parameters
///
/// * `input` - 输入 SQL 字符串
/// * `config` - 解析配置选项
///
/// ## Returns
///
/// 解析后的 AST 节点
///
/// ## Errors
///
/// 当 SQL 语法错误时返回 [`ParseError`]
///
/// ## Panics
///
/// 当内存分配失败时可能 panic
pub fn parse(input: &str, config: &Config) -> Result<AST, ParseError> {
    // 实现
}
```

---

## 8. 教学配合计划

### 8.1 每周任务模板

```markdown
## 第 X 周 (日期: YYYY-MM-DD)

### 学习目标
- 理解 XXX 原理
- 掌握 XXX 实现

### 本周任务
- [ ] ISSUE #NNN: 任务描述
- [ ] 分组任务分配

### 参考资料
- 文档链接
- 代码链接

### 验收标准
- [ ] 代码编译通过
- [ ] 测试覆盖达标
- [ ] PR 已合并
```

### 8.2 评分标准

| 类别 | 权重 | 说明 |
|------|------|------|
| 代码质量 | 30% | 编译、测试、注释 |
| 功能完成 | 30% | 功能实现完整性 |
| 文档 | 20% | SDD、用户手册 |
| 协作 | 20% | 代码审查、PR 质量 |

---

## 9. 风险与应对

| 风险 | 可能性 | 影响 | 应对措施 |
|------|--------|------|----------|
| Rust 学习曲线 | 高 | 高 | 提供详细教程 + 结对编程 |
| 性能不达标 | 中 | 中 | 渐进式优化，保留优化空间 |
| 范围蔓延 | 中 | 中 | 严格控制每个里程碑范围 |
| 代码审查积压 | 低 | 中 | 每日代码审查会议 |

---

## 10. 附录

### 10.1 参考文献

1. "Transaction Processing: Concepts and Techniques" - Jim Gray
2. "Database System Concepts" - Silberschatz, Korth, Sudarshan
3. "Designing Data-Intensive Applications" - Martin Kleppmann
4. "The Volcano Iterator Model" - Goetz Graefe
5. Rust 官方文档: https://doc.rust-lang.org/

### 10.2 相关资源

- SQL-92 标准: ISO/IEC 9075:1992
- Apache Arrow: https://arrow.apache.org/
- DataFusion: https://datafusion.apache.org/
- Rust 异步编程: https://tokio.rs/

---

## 11. 设计评审检查清单

### 架构与设计
- [ ] 架构设计是否清晰合理
- [ ] 模块划分是否合理
- [ ] 接口设计是否符合 Rust 最佳实践
- [ ] 错误处理是否完善
- [ ] 可扩展点是否预留

### AI工具整合
- [ ] Claude Code 使用流程是否清晰
- [ ] GitHub Copilot 配置是否完整
- [ ] CLI 工具链是否覆盖全面
- [ ] AI 审查流程是否建立

### 软件工程实践
- [ ] CI/CD 流程是否完整
- [ ] 测试策略是否明确
- [ ] 文档规范是否建立
- [ ] Git 工作流是否清晰

### 教学配合
- [ ] 教材章节对应关系清晰
- [ ] 每周任务可执行
- [ ] 评分标准公平合理
- [ ] 验收标准可量化

---

**文档版本历史**

| 版本 | 日期 | 作者 | 变更说明 |
|------|------|------|----------|
| v0.1 | 2026-02-13 | - | 初始草稿 |
| v1.0 | 2026-02-13 | - | 添加AI工具链 |
| v1.1 | YYYY-MM-DD | - | 设计评审通过 |

---

> **下一步**: 请评审此设计文档，确认后我将创建详细的实施计划。
