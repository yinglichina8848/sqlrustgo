# SQLRustGo v2.9.0 测试手册

> **版本**: v2.9.0
> **发布日期**: 2026-05-05

---

## 一、测试环境准备

### 1.1 环境要求

| 组件 | 要求 |
|------|------|
| Rust | 1.85+ (验证: 1.94.1) |
| 内存 | 8GB+ |
| 磁盘 | 10GB+ |
| OS | macOS / Linux |

### 1.2 依赖安装

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update

# 安装 LLVM（覆盖率工具）
apt install llvm-15 llvm-15-dev clang-15
```

---

## 二、测试类型

### 2.1 单元测试

```bash
# 运行所有单元测试
cargo test --all-features

# 运行特定 crate
cargo test -p sqlrustgo-executor --all-features

# 运行特定测试
cargo test test_hash_join --all-features
```

### 2.2 集成测试

```bash
# 运行集成测试
cargo test --test '*_integration'

# 运行 E2E 测试
cargo test --test '*_e2e'
```

### 2.3 覆盖率测试

```bash
# 行覆盖率
cargo install cargo-tarpaulin
cargo tarpaulin --all-features --out html

# 分支覆盖率
cargo install cargo-llvm-cov
cargo llvm-cov --all-features --lcov --output-path lcov.info
```

---

## 三、测试数据准备

### 3.1 TPC-H 数据

```bash
# 生成 SF=0.1 测试数据
cargo run --bin bench-cli -- tpch gen --sf 0.1 --path data/tpch-sf01/

# 生成 SF=1 测试数据
cargo run --bin bench-cli -- tpch gen --sf 1 --path data/tpch-sf1/
```

### 3.2 SQL Corpus

```bash
# SQL Corpus 测试数据位于 tests/corpus/
ls tests/corpus/
```

---

## 四、基准测试

### 4.1 TPC-H 基准

```bash
# SF=0.1
cargo run --bin bench-cli -- tpch bench --queries all --sf 0.1 --iterations 3

# SF=1
cargo run --bin bench-cli -- tpch bench --queries Q1,Q3,Q6 --sf 1 --iterations 3
```

### 4.2 sysbench

```bash
# Point Select
cargo run --bin bench-cli -- sysbench point_select --threads 4 --time 30

# Range Scan
cargo run --bin bench-cli -- sysbench range_scan --threads 4 --time 30
```

---

## 五、混沌测试

### 5.1 混沌注入

```bash
cargo test -p sqlrustgo-executor --test chaos_tests
```

### 5.2 故障注入

| 故障类型 | 注入方式 | 验证 |
|----------|----------|------|
| 网络延迟 | tc qdisc | 恢复正确 |
| 进程崩溃 | SIGKILL | 数据完整 |
| 磁盘满 | truncate | 优雅降级 |

---

## 六、测试报告

### 6.1 生成报告

```bash
cargo test --all-features -- --report-time
cargo tarpaulin --all-features --out html --output-dir coverage-report/
```

### 6.2 CI 报告

CI 自动生成 `gate_report.json`，包含测试结果和覆盖率数据。

---

*本文档由 Hermes Agent 维护*
*更新日期: 2026-05-05*
