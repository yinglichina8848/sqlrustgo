# SQLRustGo v2.8.0 测试手册

**版本**: v2.8.0 (GA)
**发布日期**: 2026-05-02

---

## 一、测试环境准备

### 1.1 环境要求

| 组件 | 要求 |
|------|------|
| Rust | 1.70+ (验证: v1.94.1) |
| 内存 | 8GB+ (部分测试需 16GB+) |
| 磁盘 | 10GB+ (含 target 编译产物) |
| 操作系统 | macOS 14.x / Linux |
| Cargo | 1.70+ |

### 1.2 环境搭建

```bash
# 1. 克隆代码
git clone git@gitea-devstack:openclaw/sqlrustgo.git
cd sqlrustgo

# 2. 切换到 v2.8.0
git checkout develop/v2.8.0

# 3. 编译项目（仅检查编译）
cargo check --all-features

# 4. 全量编译（含测试）
cargo build --all-features

# 5. 运行完整测试
cargo test --all-features 2>&1 | tee test_result.log
```

### 1.3 测试数据准备

```bash
# 创建测试数据库（通过 REPL）
cargo run --bin sqlrustgo
# 在 REPL 中: CREATE DATABASE test_db;

# 加载测试数据（如可用）
# cargo run --bin tpch_data -- --sf 0.1 --output ./test_data/tpch_sf0.1
```

---

## 二、测试分类详解

### 2.1 单元测试 (Unit Tests)

分布在各个 crate 的 `src/` 目录下，使用 `#[test]` 标注。

```bash
# 运行全部单元测试
cargo test --lib --all-features

# 运行特定模块测试
cargo test -p sqlrustgo-parser --lib
cargo test -p sqlrustgo-storage --lib
cargo test -p sqlrustgo-executor --lib
cargo test -p sqlrustgo-security --lib
```

**v2.8.0 单元测试规模**: 258 通过, 0 失败, 33 忽略

### 2.2 集成测试 (Integration Tests)

位于项目根目录 `tests/` 下，每个 `.rs` 文件是一个独立的测试可执行文件。

```bash
# 运行所有集成测试
cargo test --tests --all-features

# 运行特定集成测试文件
cargo test --test crash_recovery_test
cargo test --test wal_integration_test
cargo test --test mvcc_transaction_test
cargo test --test regression_test
cargo test --test expression_operators_test
cargo test --test distinct_test
cargo test --test qps_benchmark_test
cargo test --test concurrency_stress_test
cargo test --test stored_proc_catalog_test
cargo test --test stored_procedure_parser_test

# 查看测试列表（不运行）
cargo test --tests --all-features -- --list
```

**集成测试文件一览**:

| 文件 | 测试数 | 说明 |
|------|--------|------|
| `tests/crash_recovery_test.rs` | 8 | 崩溃恢复场景 |
| `tests/wal_integration_test.rs` | 16 | WAL 日志恢复 |
| `tests/mvcc_transaction_test.rs` | 4 | MVCC 事务隔离 |
| `tests/regression_test.rs` | 16 | 回归测试套件 |
| `tests/aggregate_functions_test.rs` | 9 | 聚合函数 |
| `tests/distinct_test.rs` | 6 | DISTINCT 查询 |
| `tests/expression_operators_test.rs` | 3 | 表达式运算符 |
| `tests/concurrency_stress_test.rs` | 9 | 并发压力 |
| `tests/qps_benchmark_test.rs` | 22 | QPS 基准 |
| `tests/stored_proc_catalog_test.rs` | 16 | 存储过程编目 |

### 2.3 分布式测试 (Distributed Tests)

位于 `crates/distributed/` 下，需 `--features distributed` 或 `--all-features`。

```bash
# 全量分布式测试 (685 tests)
cargo test -p sqlrustgo-distributed --all-features

# 按模块筛选
cargo test -p sqlrustgo-distributed -- partition
cargo test -p sqlrustgo-distributed -- replication
cargo test -p sqlrustgo-distributed -- failover
cargo test -p sqlrustgo-distributed -- read_write_split
cargo test -p sqlrustgo-distributed -- load_balancing
```

**分布式测试规模**: 685 通过, 0 失败

### 2.4 E2E 测试

位于 `tests/e2e/` 下：

```bash
# E2E 查询测试
cargo test --test e2e_query_test

# E2E 监控测试
cargo test --test e2e_monitoring_test

# E2E 可观测性测试
cargo test --test e2e_observability_test
```

### 2.5 稳定性测试

位于 `tests/` 下，默认标记 `#[ignore]`，需显式运行：

```bash
# 长稳测试（加速模拟）
cargo test --test long_run_stability_test -- --ignored

# 72h 长稳测试（加速模拟）
cargo test --test long_run_stability_72h_test -- --ignored
```

### 2.6 基准测试

```bash
# BufferPool 基准
cargo test --test buffer_pool_benchmark_test

# 页 I/O 基准（标记 #[ignore]）
cargo test --test page_io_benchmark_test -- --ignored

# QPS 基准
cargo test --test qps_benchmark_test
```

### 2.7 SQL Corpus 回归测试

```bash
# 运行 SQL 语料库回归测试
cargo test -p sql-corpus --all-features
```

**v2.8.0 结果**: 174/426 通过 (40.8%)

---

## 三、测试结果解读

### 3.1 测试输出格式

```
test result: ok. N passed; 0 failed; M ignored; 0 measured; 0 filtered out; finished in X.XXs
```

| 字段 | 含义 |
|------|------|
| passed | 通过的测试数 |
| failed | 失败的测试数 (v2.8.0: 0) |
| ignored | 标记 `#[ignore]` 的测试数 |
| measured | 基准测试数 (bench) |
| filtered out | 被过滤掉的测试数 |

### 3.2 常见失败原因

| 符号 | 含义 | 处理 |
|------|------|------|
| `test X ... FAILED` | 测试失败 | 查看具体错误信息 |
| `test X ... ok` | 测试通过 | 正常 |
| `test X ... ignored` | 测试被跳过 | `-- --ignored` 运行 |

---

## 四、故障排查

### 4.1 编译错误

```bash
# 检查 Rust 版本
rustc --version  # 需要 1.70+

# 清理编译缓存（极端情况）
cargo clean
cargo build --all-features
```

### 4.2 内存不足

Cargo.toml 配置了 8GB 测试内存限制：
```toml
[profile.test]
opt-level = 0
```

如果 OOM，可减少并发测试数：
```bash
cargo test --all-features -- --test-threads=4
```

### 4.3 特定测试调试

```bash
# 单测试 + 详细输出
cargo test test_name -- --nocapture

# 查看完整错误栈
cargo test test_name -- --show-output

# 跳过编译直接运行
cargo test --no-run && ./target/debug/tests/your_test_binary
```

### 4.4 分布式测试环境

分布式测试需要本地回环网络支持。如果测试因网络原因失败：

```bash
# 检查本地回环接口
ifconfig lo0
ping 127.0.0.1

# macOS 防火墙
sudo pfctl -d  # 临时禁用，仅调试用
```

---

## 五、测试维护指南

### 5.1 添加新测试

```rust
#[test]
fn test_new_feature() {
    // Arrange: 设置测试环境
    let engine = create_test_engine();

    // Act: 执行测试操作
    let result = engine.execute("YOUR SQL HERE");

    // Assert: 验证结果
    assert!(result.is_ok());
    let rows = result.unwrap();
    assert_eq!(rows.len(), expected_count);
}
```

### 5.2 标记忽略测试

```rust
// 已知问题或需要特定环境的测试
#[test]
#[ignore = "需要 MySQL 服务器"]
fn test_mysql_compat() {
    // ...
}
```

### 5.3 预提交检查

```bash
# 完整检查流程
cargo fmt --check --all          # 格式检查
cargo clippy --all-features -- -D warnings  # Lint 检查
cargo test --all-features        # 测试
```
