# SQLRustGo v3.2.0 测试报告

> **测试版本**: v3.2.0 (RC)
> **测试日期**: 2026-05-17
> **测试执行者**: Hermes Agent
> **测试环境**: ai@250 / Linux / Rust 1.96.0

---

## 一、测试概要

### 1.1 测试版本信息

| 项目 | 值 |
|------|-----|
| 版本 | v3.2.0 (RC) |
| 分支 | develop/v3.2.0 |
| Git HEAD | 最新 origin/develop/v3.2.0 |
| Rust 版本 | 1.96.0 |
| Cargo 版本 | 1.96.0 |
| 测试工具 | cargo test --all-features |

### 1.2 RC Gate 测试结果

| Gate | 检查项 | 状态 | 说明 |
|------|--------|------|------|
| R1 | cargo build --release --workspace | ✅ PASS | 编译通过 |
| R2 | Full test suite | ✅ PASS | 100% 通过率 |
| R3 | cargo clippy --all-features | ✅ PASS | 零警告 |
| R4 | cargo fmt --check | ✅ PASS | 格式通过 |
| R5 | Coverage ≥85% | ❌ FAIL | 实际 83.60% (executor 70.70% 拉低平均) |
| R6 | cargo audit | ✅ PASS | 无已知漏洞 |
| R7 | SQL Operations ≥95% | ✅ PASS | 100% (3/3) |
| R8 | TPC-H SF=1 p99 < 5s | ✅ PASS | 通过 |
| R9 | check_regression.sh | ✅ PASS | 回归测试通过 |
| R10 | formal proof count ≥30 | ✅ PASS | 通过 |
| R11 | check_docs_links.sh | ❌ FAIL | 2个失效链接 |
| R12 | HSM/KMS integration | ❌ FAIL | sqlrustgo-hsm 包不存在 |
| R13 | MySQL protocol | ❌ FAIL | 测试路径问题 |
| R14 | window_function_test | ✅ PASS | 1 test |
| R15 | dml_multi_table_test | ✅ PASS | 1 test |
| R16 | hash_join_test | ✅ PASS | 1 test |

**RC Gate 通过率**: 12/16 (75%)

### 1.3 覆盖率详情

| Crate | 覆盖率 |
|-------|--------|
| types | 87.65% |
| planner | 89.99% |
| optimizer | 82.41% |
| executor | 70.70% |
| storage | 78.08% |
| transaction | 87.88% |
| catalog | 88.52% |
| **平均** | **83.60%** |

---

## 二、已知问题

### 2.1 R5 - 覆盖率不足

- **问题**: executor crate 覆盖率仅 70.70%，拉低整体平均值至 83.60%
- **差距**: 需再提升 1.4% 达到 85% 门槛
- **建议**: 增加 executor 单元测试覆盖边界情况

### 2.2 R11 - 文档链接失效

- `docs/releases/v1.6.1/RELEASE_GATE_CHECKLIST.md` → `benchmark_results/BENCHMARK_COMPARISON_REPORT.md` (不存在)
- `README.md` → `docs/releases/v3.2.0/TEST_REPORT.md` (本文件)

### 2.3 R12 - HSM/KMS 包不存在

- `sqlrustgo-hsm` 包在 workspace 中不存在
- 建议：确认包名或标记为 SKIP

### 2.4 R13 - MySQL 协议测试路径问题

- 测试使用相对路径 `target/release/sqlrustgo-mysql-server`
- 建议：使用 `CARGO_MANIFEST_DIR` 或绝对路径

---

## 三、已验证功能

### 3.1 核心功能 ✅

- SELECT/INSERT/UPDATE/DELETE 操作正常
- JOIN (INNER/LEFT/RIGHT) 支持
- 聚合函数 (SUM/AVG/COUNT/MIN/MAX)
- 窗口函数
- 存储过程
- MVCC 事务
- WAL 崩溃恢复

### 3.2 性能基准 ✅

| 操作 | 性能 |
|------|------|
| UPDATE | ~40K QPS (24-25µs/op) |
| DELETE | ~3.3M QPS (0.30µs/op) |
| TPC-H SF=1 | p99 < 5s |

---

*本报告为 v3.2.0 RC 阶段测试报告，完整 GA 报告将在所有 Gate 通过后生成。*
