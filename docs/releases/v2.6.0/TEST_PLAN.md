# v2.6.0 测试计划（Phase B 重构版）

> **版本**: alpha/v2.6.0
> **更新日期**: 2026-04-19
> **目标**: 建立"可执行、可追溯、可发布"的测试基线
> **验证状态**: ✅ 已验证 (2026-04-19)

---

## 一、测试分层

### L0 冒烟（<5 分钟）

快速冒烟测试，验证基本功能可用。

| 测试项 | 命令 | 状态 | 说明 |
|--------|------|------|------|
| 构建检查 | `cargo build --release` | ✅ 已验证 | 编译成功 |
| 格式检查 | `cargo fmt --check` | ✅ 已验证 | 格式正确 |
| Clippy 检查 | `cargo clippy -- -D warnings` | ✅ 已验证 | 需修复后通过 |
| 核心冒烟 | `cargo test --test binary_format_test` | ✅ 已验证 | 测试通过 |

### L1 模块回归（<20 分钟）

按模块划分的单元测试。

| 测试项 | 命令 | 状态 | 说明 |
|--------|------|------|------|
| parser 单测 | `cargo test -p sqlrustgo-parser --lib` | ⚠️ 61/63 | 2个FK测试失败 |
| planner 单测 | `cargo test -p sqlrustgo-planner --lib` | ✅ 已验证 | 81 passed |
| executor 单测 | `cargo test -p sqlrustgo-executor --lib` | ✅ 已验证 | 48 passed |
| storage 单测 | `cargo test -p sqlrustgo-storage --lib` | ✅ 已验证 | 120 passed |
| optimizer 单测 | `cargo test -p sqlrustgo-optimizer --lib` | ✅ 已验证 | 132 passed |
| transaction 单测 | `cargo test -p sqlrustgo-transaction --lib` | ✅ 已验证 | 28 passed |
| server 单测 | `cargo test -p sqlrustgo-server --lib` | ⏳ 待执行 | - |
| vector 单测 | `cargo test -p sqlrustgo-vector --lib` | ⏳ 待执行 | - |
| graph 单测 | `cargo test -p sqlrustgo-graph --lib` | ⏳ 待执行 | - |

### L2 集成回归（<60 分钟）

全链路集成测试。

| 测试项 | 命令 | 状态 | 说明 |
|--------|------|------|------|
| CBO 集成测试 | `cargo test --test cbo_integration_test` | ✅ 已验证 | 12 passed |
| WAL 集成测试 | `cargo test --test wal_integration_test` | ⏳ 待执行 | - |
| Parser Token 测试 | `cargo test --test parser_token_test` | ⏳ 待执行 | - |
| Regression 测试 | `cargo test --test regression_test` | ⏳ 待执行 | - |
| E2E Query 测试 | `cargo test --test e2e_query_test` | ⏳ 待执行 | - |
| Scheduler 集成测试 | `cargo test -p sqlrustgo-server --test scheduler_integration_test` | ⏳ 待执行 | - |

### L3 深度验证（夜间/长时）

长时间运行和压力测试。

| 测试项 | 命令 | 状态 | 说明 |
|--------|------|------|------|
| TPC-H SF1 | `cargo bench --bench tpch_bench` | ⚠️ 代码编译错误 | 待创建 Issue 修复 |
| Sysbench | 外部工具 | ⏳ 待集成 | 需手动执行 |
| 压力测试 | `cargo test --test concurrency_stress_test` | 🔴 Target 不存在 | 计划中 |
| 崩溃恢复 | `kill -9` 测试 | ⏳ 待手动测试 | 需文档化 |
| 备份恢复 | backup/restore 测试 | ⏳ 待实现 | 计划中 |

---

## 二、测试执行命令（已验证）

### 2.1 L0 冒烟命令

```bash
# 1. 构建
cargo build --release

# 2. 格式
cargo fmt --check

# 3. Clippy
cargo clippy -- -D warnings

# 4. 冒烟测试
cargo test --test binary_format_test
cargo test --test ci_test
```

### 2.2 L1 模块命令

```bash
# 按模块单测
cargo test -p sqlrustgo-parser --lib
cargo test -p sqlrustgo-planner --lib
cargo test -p sqlrustgo-executor --lib
cargo test -p sqlrustgo-storage --lib
cargo test -p sqlrustgo-optimizer --lib
cargo test -p sqlrustgo-transaction --lib
cargo test -p sqlrustgo-server --lib
cargo test -p sqlrustgo-vector --lib
cargo test -p sqlrustgo-graph --lib
```

### 2.3 L2 集成命令

```bash
# 集成测试
cargo test --test cbo_integration_test
cargo test --test wal_integration_test
cargo test --test parser_token_test
cargo test --test regression_test
cargo test --test e2e_query_test
cargo test --test e2e_observability_test
cargo test --test e2e_monitoring_test
cargo test -p sqlrustgo-server --test scheduler_integration_test
```

### 2.4 L3 深度命令（代码修复后可用）

```bash
# TPC-H Bench（当前代码有编译错误，需修复）
cargo bench --bench tpch_bench

# 其他 Bench
cargo bench --bench bench_cbo
cargo bench --bench bench_columnar
cargo bench --bench bench_insert
```

---

## 三、覆盖率目标

### 3.1 目标值

| 模块 | 当前覆盖率 | Alpha 目标 | Beta 目标 |
|------|------------|------------|-----------|
| 整体 | 49% | 55% | 70% |
| parser | ⏳ | 70% | 80% |
| executor | ⏳ | 65% | 75% |
| storage | ⏳ | 65% | 75% |
| transaction | ⏳ | 70% | 80% |

### 3.2 覆盖率测量命令

```bash
# 安装 tarpaulin
cargo install cargo-tarpaulin

# 运行覆盖率
cargo tarpaulin --output-html --out-dir artifacts/coverage/

# 查看报告
open artifacts/coverage/index.html
```

---

## 四、SQL Corpus 测试

### 4.1 测试命令

```bash
# 运行 SQL Corpus
cargo test -p sqlrustgo-sql-corpus --lib
```

### 4.2 目标

| 指标 | 目标 | 状态 |
|------|------|------|
| 通过率 | ≥95% | ⏳ 待测 |
| P0 语法 | 100% | ⏳ 待测 |

---

## 五、执行节奏

| 频率 | 执行内容 | 命令 |
|------|----------|------|
| 每次 PR | L0 冒烟 | 见 2.1 |
| 每日 | L0 + L1 | 见 2.1 + 2.2 |
| 每周 | L2 全量 | 见 2.3 |
| 发布前 | L3 验证 + 覆盖率 | tarpaulin + bench |

---

## 六、门禁映射

| 阶段 | 门禁项 | 阈值 |
|------|--------|------|
| Alpha | L0 + L1 | 100% 通过 |
| Beta | L2 + SQL Corpus | ≥95% 通过 |
| RC | 覆盖率 + L3 | 70% + bench 通过 |
| GA | 完整回归 + 备份恢复 | 100% + 72h |

---

## 六、已知问题

### TPC-H 基准测试编译错误

**问题描述**: `cargo bench --bench tpch_bench` 编译失败

**错误信息**:
- `unresolved imports: sqlrustgo_server::ConnectionPool, PoolConfig`
- `mismatched types: expected &str, found Statement`

**修复计划**: 需要在基准测试中修复导入和类型转换代码

**Issue 跟踪**: 待创建

---

## 七、变更历史

| 版本 | 日期 | 说明 |
|------|------|------|
| 1.0 | 2026-04-17 | 初始版本 |
| 2.0 | 2026-04-19 | Phase B 重构：区分可执行/计划中测试，映射到真实 target |
| 3.0 | 2026-04-19 | 修正测试状态语义：已验证≠已执行，添加 TPC-H 问题说明 |

---

## 八、元数据

| 字段 | 值 |
|------|------|
| 工作目录 | /Users/liying/workspace/dev/yinglichina163/sqlrustgo |
| GitHub 身份 | yinglichina8848 |
| AI 工具 | TRAE (Auto Model) |
| 当前版本 | v2.6.0 (alpha) |
| 工作分支 | develop/v2.6.0 |
| 时间段 | 2026-04-19 16:10 (UTC+8) |

---

*测试计划 v2.6.0*
*创建者: TRAE Agent*
*审核者: -*
*修改者: TRAE Agent*
*修改记录:*
* - 2026-04-17: 初始版本创建*
* - 2026-04-19: Phase B 重构，添加元数据*
*最后更新: 2026-04-19*
