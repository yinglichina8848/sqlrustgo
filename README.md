# SQLRustGo

<p align="center">
  <img src="https://img.shields.io/badge/Rust-1.85+-dea584?style=flat-square&logo=rust" alt="Rust">
  <img src="https://img.shields.io/badge/version-v2.5.0-blue" alt="Version">
  <img src="https://img.shields.io/badge/branch-develop%2Fv2.6.0-green" alt="Branch">
  <img src="https://img.shields.io/badge/license-MIT-green" alt="License">
</p>

SQLRustGo 是一个使用 Rust 实现的关系型数据库教学与工程化项目，支持 SQL-92 子集，包含解析、规划、执行、存储、事务与网络模块。

## v2.5.0 发布状态

| 项目 | 状态 |
|------|------|
| 当前版本 | **v2.5.0** |
| 当前开发分支 | **develop/v2.6.0** |
| 当前阶段 | **GA (正式发布)** |
| 上一稳定版本 | v2.4.0 |

---

## 核心能力

### SQL 支持

| 类型 | 支持的语句 |
|------|----------|
| **DQL** | `SELECT`, `SELECT DISTINCT`, `WHERE`, `ORDER BY`, `LIMIT/OFFSET` |
| **DML** | `INSERT`, `INSERT SELECT`, `UPDATE`, `DELETE` |
| **DDL** | `CREATE TABLE`, `DROP TABLE`, `ALTER TABLE (ADD COLUMN, RENAME TO)` |
| **高级** | `WITH (CTE)`, `子查询 (EXISTS, IN, ALL, ANY, SOME)` |

### 约束支持

| 约束类型 | 状态 | 说明 |
|----------|------|------|
| PRIMARY KEY | ✅ 已实现 | INSERT 时唯一性验证 |
| UNIQUE | ✅ 已实现 | INSERT 时唯一性验证 |
| FOREIGN KEY | ✅ 已实现 | CASCADE 语义支持 |
| NOT NULL | ✅ 已实现 | 解析支持 |
| CHECK | 🔜 v2.6.0 | 规划中 |

### 存储引擎

| 组件 | 说明 |
|------|------|
| **MemoryStorage** | 内存存储引擎 |
| **FileStorage** | 基于 B+Tree 的持久化存储 |
| **BufferPool** | 缓存池管理 |
| **WAL** | Write-Ahead Logging 崩溃恢复 |

### 索引

| 类型 | 状态 |
|------|------|
| B+ Tree 索引 | ✅ 已实现 |
| 索引扫描 | 🔜 v2.6.0 (集成中) |

### 事务

| 隔离级别 | 状态 |
|----------|------|
| READ UNCOMMITTED | 🔜 v2.6.0 |
| READ COMMITTED | ✅ 已实现 |
| REPEATABLE READ (快照隔离) | ✅ 已实现 |
| SERIALIZABLE (SSI) | 🔜 v2.6.0 |

### 网络与协议

| 组件 | 说明 |
|------|------|
| MySQL 风格协议 | TCP 网络交互 |
| REPL | 命令行交互模式 |

---

## 测试结果

### 单元测试

| 测试套件 | 结果 |
|----------|------|
| Storage lib tests | ✅ 55/55 通过 |
| Parser tests | ✅ 37/37 通过 |
| Executor lib tests | ✅ 9/9 通过 |
| 整体覆盖率 | **49%** |

### SQL Regression Corpus

| 指标 | 值 |
|------|-----|
| 总测试用例 | 59 |
| 通过 | 12 (20.3%) |
| 失败 | 47 |

> **注意**: 20.3% pass rate 是因为 SQL-92 子集限制。v2.6.0 将扩展语法支持。

---

## 快速开始

```bash
# 构建
cargo build --all-features

# 运行测试
cargo test --all-features

# 运行特定测试
cargo test -p sqlrustgo-storage --lib
cargo test -p sqlrustgo-parser --lib

# 启动 REPL
cargo run --bin sqlrustgo

# 代码检查
cargo clippy --all-targets -- -D warnings

# 覆盖率报告
cargo tarpaulin --workspace --exclude-dir crates/tpch
```

---

## 版本历史

### v2.5.0 (当前)

**主要功能**:
- ✅ ALTER TABLE 执行支持 (ADD COLUMN, RENAME TO)
- ✅ CTE (WITH clause) 支持
- ✅ 子查询 (EXISTS, IN, ALL, ANY, SOME)
- ✅ INSERT SELECT
- ✅ PRIMARY KEY / UNIQUE 约束验证
- ✅ 外键约束 (CASCADE)
- ✅ SQL Regression Corpus 测试框架

**分支**: `release/v2.5.0` (已锁定)

### v2.4.0

上一稳定版本

---

## 文档导航

### 版本文档

| 文档 | 说明 |
|------|------|
| [CHANGELOG](CHANGELOG.md) | 变更日志 |
| [VERSION](VERSION) | 当前版本号 |
| [RELEASE_NOTES](RELEASE_NOTES.md) | 发布说明 |
| [CHANGELOG (详细)](releases/v2.5.0/CHANGELOG.md) | v2.5.0 详细变更 |
| [发布说明](releases/v2.5.0/RELEASE_NOTES.md) | v2.5.0 发布说明 |
| [版本路线图](docs/releases/VERSION_ROADMAP.md) | 所有版本历史 |

### 文档总览

| 文档 | 说明 |
|------|------|
| [文档总索引](docs/README.md) | 所有文档索引 |
| [架构设计](docs/architecture.md) | 系统架构 |
| [SQL92 合规性](docs/SQL92_COMPLIANCE.md) | SQL-92 支持情况 |

### 当前版本

- [v2.5.0 发布说明](releases/v2.5.0/)
- [v2.6.0 开发计划](issues/1501)

### 历史版本

| 版本 | 文档 |
|------|------|
| v2.4.0 | [文档](docs/releases/v2.4.0/) |
| v2.3.0 | [文档](docs/releases/v2.3/) |
| v2.2.0 | [文档](docs/releases/v2.2.0/) |
| v2.1.0 | [文档](docs/releases/v2.1.0/) |
| v2.0.0 | [文档](docs/releases/v2.0.0/) |

### 长期规划

| 文档 | 说明 |
|------|------|
| [长期路线图](docs/releases/LONG_TERM_ROADMAP.md) | 版本演进规划 |
| [v2.0 路线图](docs/v2.0/SQLRUSTGO_2_0_ROADMAP.md) | 2.0 详细规划 |

---

## 教学资源

### 教学计划 (教师材料)

| 文档 | 说明 |
|------|------|
| [教学进度计划](docs/教学计划/AI增强软件工程-教学进度计划.md) | 教学进度安排 |
| [上机实验指导书](docs/教学计划/上机实验指导书.md) | 实验指导 |
| [实验报告模版](docs/教学计划/实验报告模版.md) | 报告模板 |

### PPT 讲义

| 讲次 | 主题 | 链接 |
|------|------|------|
| 第1讲 | 软件工程概述与项目导论 | [PPT](docs/教学计划/PPT/第1讲-软件工程概述与项目导论.md) |
| 第2讲 | 结构化设计与UML基础 | [PPT](docs/教学计划/PPT/第2讲-结构化设计与UML基础.md) |
| 第3讲 | 面向对象设计与类图 | [PPT](docs/教学计划/PPT/第3讲-面向对象设计与类图.md) |
| 第4讲 | 顺序图状态图与架构设计 | [PPT](docs/教学计划/PPT/第4讲-顺序图状态图与架构设计.md) |
| 第5讲 | 架构设计原理与SQLRustGo架构 | [PPT](docs/教学计划/PPT/第5讲-架构设计原理与SQLRustGo架构.md) |
| 第6讲 | 功能模块划分与接口设计 | [PPT](docs/教学计划/PPT/第6讲-功能模块划分与接口设计.md) |
| 第7讲 | AI辅助核心模块实现 | [PPT](docs/教学计划/PPT/第7讲-AI辅助核心模块实现.md) |
| 第8讲 | 测试驱动开发与Alpha版本 | [PPT](docs/教学计划/PPT/第8讲-测试驱动开发与Alpha版本.md) |
| 第9讲 | 软件治理与分支策略 | [PPT](docs/教学计划/PPT/第9讲-软件治理与分支策略.md) |
| 第10讲 | PR工作流与项目成熟度评估 | [PPT](docs/教学计划/PPT/第10讲-PR工作流与项目成熟度评估.md) |

### 教学实践 (学生材料)

| 文档 | 说明 |
|------|------|
| [教学实践索引](docs/教学实践/README.md) | 学生实践材料索引 |
| [学生执行手册](docs/教学实践/v1.1.0-beta/handbook-student.md) | 学生可复现步骤 |
| [助教执行手册](docs/教学实践/v1.1.0-beta/handbook-ta.md) | PR 证据链示例 |

### 教程

| 文档 | 说明 |
|------|------|
| [连接池指南](docs/tutorials/connection-pool-guide.md) | 连接池配置 |
| [物理备份指南](docs/tutorials/physical-backup-guide.md) | 备份操作 |
| [配置热重载指南](docs/tutorials/config-hot-reload-guide.md) | 热重载配置 |

---

## 分支结构

```
main                    # 稳定版本 (生产环境)
├── release/v2.5.0     # v2.5.0 正式版 (已锁定)
└── develop/v2.6.0     # v2.6.0 开发分支 (活跃)
```

### 分支策略

1. **main**: 稳定版本，只接受 release 分支合并
2. **release/v{version}**: 发布分支，锁定后禁止直接推送
3. **develop/v{version}**: 开发分支，新功能开发

### 提交流程

```bash
# 1. 从 develop/v2.6.0 拉出功能分支
git checkout -b feature/my-feature develop/v2.6.0

# 2. 开发并提交
git commit -m "feat: my feature"

# 3. 提交 PR 到 develop/v2.6.0
# 4. CI 通过后合并
```

---

## 项目结构

```
sqlrustgo/
├── crates/
│   ├── sqlrustgo/          # 主 crate
│   ├── parser/             # SQL 解析器
│   ├── executor/           # 执行器
│   ├── planner/            # 查询规划器
│   ├── optimizer/          # 查询优化器
│   ├── storage/            # 存储引擎
│   ├── catalog/            # 元数据管理
│   ├── transaction/        # 事务管理
│   ├── server/             # 网络服务器
│   ├── types/              # 类型系统
│   ├── common/             # 公共模块
│   ├── optimizer/          # 优化器
│   ├── sql-corpus/         # SQL 回归测试
│   └── ...
├── sql_corpus/             # SQL 测试用例库
├── docs/                  # 文档
└── tests/                 # 集成测试
```

---

## v2.6.0 开发计划

### P0 - 必须完成

| 功能 | 说明 | Issue |
|------|------|-------|
| 功能集成 | CBO、索引、存储过程、触发器 | #1501 |
| SQL 语法 | 聚合函数、JOIN、GROUP BY | #1501 |
| MVCC SSI | 可串行化快照隔离 | #1389 |

### P1 - 重要功能

| 功能 | 说明 |
|------|------|
| DELETE 语句 | 完整支持 |
| FULL OUTER JOIN | 完整支持 |
| 外键约束完善 | Executor 验证 |

### P2 - 增强功能

| 功能 | 说明 |
|------|------|
| CREATE INDEX | 索引 DDL |
| 覆盖率提升 | 49% → 70% |

**详细计划**: [Issue #1501](issues/1501)

---

## 社区与贡献

### 贡献者

欢迎提交 Issue 和 PR！

### 许可证

MIT License

---

## 联系方式

- GitHub Issues: [issues](https://github.com/minzuuniversity/sqlrustgo/issues)
- 讨论: [discussions](https://github.com/minzuuniversity/sqlrustgo/discussions)
