# v2.6.0 测试计划

> **版本**: beta/v2.6.0
> **更新日期**: 2026-04-20
> **目标**: 规定测试类别、命令、阈值目标，不含执行结果

---

## 一、测试分层规范

### 1.1 测试层级定义

| 层级 | 名称 | 说明 | 执行频率 |
|------|------|------|----------|
| L0 | 冒烟测试 | 基本功能可用性验证 | 每次 PR |
| L1 | 模块回归 | 按模块划分的单元测试 | 每日 |
| L2 | 集成回归 | 全链路集成测试 | 每周 |
| L3 | 深度验证 | 性能、压力测试 | 发布前 |
| L4 | SQL Corpus | SQL 语法合规性 | 每周 |

### 1.2 测试通过阈值

| 层级 | 通过率要求 | 说明 |
|------|------------|------|
| L0 | 100% | 必须全部通过 |
| L1 | 100% | 必须全部通过 |
| L2 | 100% | 必须全部通过 |
| L3 | 基准测试通过 | 允许已知问题 |
| L4 | ≥95% | 允许部分不支持语法 |

---

## 二、L0 冒烟测试规范

### 2.1 测试命令

```bash
# 1. 构建检查
cargo build --release

# 2. 格式检查
cargo fmt --check

# 3. Clippy 检查
cargo clippy -- -D warnings

# 4. 冒烟测试
cargo test --test binary_format_test
cargo test --test ci_test
```

### 2.2 阈值要求

| 测试项 | 阈值 |
|--------|------|
| 构建 | 成功，退出码 0 |
| 格式 | 无问题 |
| Clippy | 0 警告 |
| 冒烟测试 | 100% 通过 |

---

## 三、L1 模块回归规范

### 3.1 测试命令

```bash
# parser 模块
cargo test -p sqlrustgo-parser --lib

# planner 模块
cargo test -p sqlrustgo-planner --lib

# executor 模块
cargo test -p sqlrustgo-executor --lib

# storage 模块
cargo test -p sqlrustgo-storage --lib

# optimizer 模块
cargo test -p sqlrustgo-optimizer --lib

# transaction 模块
cargo test -p sqlrustgo-transaction --lib

# server 模块
cargo test -p sqlrustgo-server --lib

# vector 模块
cargo test -p sqlrustgo-vector --lib

# graph 模块
cargo test -p sqlrustgo-graph --lib
```

### 3.2 阈值要求

| 模块 | 通过率要求 |
|------|------------|
| parser | 100% |
| planner | 100% |
| executor | 100% |
| storage | 100% |
| optimizer | 100% |
| transaction | 100% |
| server | 100% |
| vector | 100% |
| graph | 100% |

---

## 四、L2 集成回归规范

### 4.1 测试命令

```bash
# CBO 集成测试
cargo test --test cbo_integration_test

# WAL 集成测试
cargo test --test wal_integration_test

# Parser Token 测试
cargo test --test parser_token_test

# Regression 测试
cargo test --test regression_test

# E2E Query 测试
cargo test --test e2e_query_test

# E2E Observability 测试
cargo test --test e2e_observability_test

# E2E Monitoring 测试
cargo test --test e2e_monitoring_test

# Scheduler 集成测试
cargo test -p sqlrustgo-server --test scheduler_integration_test
```

### 4.2 阈值要求

| 测试项 | 通过率要求 |
|--------|------------|
| cbo_integration_test | 100% |
| wal_integration_test | 100% |
| parser_token_test | 100% |
| regression_test | 100% |
| e2e_query_test | 100% |
| e2e_observability_test | 100% |
| e2e_monitoring_test | 100% |
| scheduler_integration_test | 100% |

---

## 五、L3 深度验证规范

### 5.1 基准测试命令

```bash
# TPC-H Bench
cargo bench --bench tpch_bench

# CBO Bench
cargo bench --bench bench_cbo

# Columnar Bench
cargo bench --bench bench_columnar

# Insert Bench
cargo bench --bench bench_insert
```

### 5.2 压力测试命令

```bash
# 并发压力测试
cargo test --test concurrency_stress_test

# 崩溃恢复测试（手动）
kill -9 <pid>

# 备份恢复测试（手动）
# 见 DEPLOYMENT_GUIDE.md
```

### 5.3 阈值要求

| 测试项 | 阈值 | 说明 |
|--------|------|------|
| TPC-H SF=1 | < 5s | 允许已知编译问题 |
| 压力测试 | 通过 | |
| 崩溃恢复 | 恢复成功 | 手动验证 |

---

## 六、L4 SQL Corpus 规范

### 6.1 测试命令

```bash
# 运行 SQL Corpus
cargo test -p sqlrustgo-sql-corpus
```

### 6.2 阈值要求

| 指标 | 目标 |
|------|------|
| 通过率 | ≥95% |
| P0 语法 | 100% |

### 6.3 不支持的语法

以下语法当前版本不支持，不计入失败：

| 语法 | 原因 |
|------|------|
| FULL OUTER JOIN | Beta 阶段支持 |
| 窗口函数 | 部分支持 |
| 复杂 CTE | 计划中 |

---

## 七、覆盖率规范

### 7.1 测量命令

```bash
# 安装 tarpaulin
cargo install cargo-tarpaulin

# 运行覆盖率
cargo tarpaulin --output-html --out-dir artifacts/coverage/

# 查看报告
open artifacts/coverage/index.html
```

### 7.2 覆盖率目标

| 阶段 | 整体覆盖率目标 | 说明 |
|------|---------------|------|
| Alpha | ≥55% | 基准线 |
| Beta | ≥65% | 集成后 |
| RC | ≥70% | 完整测试 |
| GA | ≥70% | 保持 |

---

## 八、阶段映射

### 8.1 门禁阶段要求

| 阶段 | 必须通过的测试 |
|------|----------------|
| Alpha | L0 + L1 |
| Beta | L2 + L4 (SQL Corpus ≥95%) |
| RC | L0~L2 + 覆盖率 ≥70% + L3 (TPC-H) |
| GA | L0~L4 100% + 长稳测试 |

### 8.2 测试报告要求

| 阶段 | 必须生成的报告 |
|------|----------------|
| Alpha | `report/ALPHA_TEST_REPORT.md` |
| Beta | `report/BETA_TEST_REPORT.md` |
| RC | `report/RC_TEST_REPORT.md` |
| GA | `report/GA_TEST_REPORT.md` |

---

## 九、执行节奏

| 频率 | 执行内容 |
|------|----------|
| 每次 PR | L0 冒烟 |
| 每日 | L0 + L1 |
| 每周 | L2 全量 + L4 |
| 发布前 | L3 验证 + 覆盖率 |

---

## 十、变更历史

| 版本 | 日期 | 说明 |
|------|------|------|
| 1.0 | 2026-04-17 | 初始版本 |
| 2.0 | 2026-04-20 | 重构：移除结果数据，只保留测试规范 |

---

## 十一、元数据

| 字段 | 值 |
|------|------|
| 工作目录 | /Users/liying/workspace/dev/yinglichina163/sqlrustgo |
| 当前版本 | v2.6.0 |
| 工作分支 | develop/v2.6.0 |

---

*测试计划 v2.6.0*
*本文件规定测试类别、命令、阈值，不含执行结果*
*执行结果见对应阶段的测试报告*