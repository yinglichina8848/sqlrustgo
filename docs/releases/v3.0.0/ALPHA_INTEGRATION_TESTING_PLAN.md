# v3.0.0 Alpha 集成测试计划

> **版本**: v3.0.0-alpha
> **日期**: 2026-05-06

---

## 测试策略

Alpha 阶段采用三层测试金字塔：

1. **单元测试**: `cargo test --all-features --workspace`
2. **集成测试**: `scripts/test/run_integration.sh --quick`
3. **端到端**: TPC-H SF=0.1 + SQL Corpus + QPS Benchmarks

---

## 测试分层

### Layer 1: 快速门禁 (<5min)

| 测试 | 命令 | 超时 |
|------|------|------|
| 编译 | `cargo build --all-features` | 3min |
| Clippy | `cargo clippy --all-features -- -D warnings` | 2min |
| 格式 | `cargo fmt --all -- --check` | 30s |
| 单元测试 | `cargo test --all-features --workspace --lib` | 5min |

### Layer 2: 功能验证 (5-15min)

| 测试 | 命令 | 超时 |
|------|------|------|
| SQL Corpus | `cargo test -p sqlrustgo-sql-corpus` | 5min |
| 集成测试 | `bash scripts/test/run_integration.sh --quick` | 10min |
| TPC-H Q1/Q6 | `bash scripts/gate/check_tpch.sh sf=0.1` | 5min |

### Layer 3: 性能回归 (15-30min)

| 测试 | 命令 | 超时 |
|------|------|------|
| QPS 基准 | `bash scripts/gate/check_regression.sh` | 20min |
| 覆盖率 | `cargo llvm-cov --all --all-features` | 15min |

---

## 缺陷严重等级

| 等级 | 定义 | 阻塞发布 |
|------|------|---------|
| P0 | 编译失败 / 核心功能不工作 | ✅ |
| P1 | 重要功能有已知 bug | ⚠️ |
| P2 | 边缘用例失败 | ❌ |

---

## 测试自动化

所有 Layer 1 测试在每次 `git push` 时通过 Gitea Actions gate-ci.yml 自动运行。

Layer 2/3 通过 `r-gate.yml` 在 push 到 develop/v3.0.0 时触发。

---

## 回归管理

- `check_regression.sh`: QPS 回归超过 20% → PR 阻塞
- `check_coverage.sh`: 覆盖率低于阶段目标 → PR 阻塞
- TPC-H: SF=0.1 任何查询 OOM → P0 缺陷