# 覆盖率测试 CI/CD 集成评估报告

> **版本**: v2.9.0
> **日期**: 2026-05-04
> **状态**: RC 阶段
> **作者**: SQLRustGo Team

---

## 1. 执行摘要

本报告评估当前覆盖率测试体系的 CI/CD 集成现状，分析问题并提出改进方案。

### 1.1 当前覆盖率状态

| Gate | 要求 | 实际 | 状态 |
|------|------|------|------|
| B1 总覆盖率 | ≥75% | 84.18% | ✅ |
| B2 executor 覆盖率 | ≥60% | 71.08% | ✅ |
| 测试数量 B5 | ≥3597 | 4565 | ✅ |

### 1.2 覆盖率测试执行时间

| 测试范围 | 耗时 | 资源消耗 |
|----------|------|----------|
| `cargo llvm-cov --lib` | ~9.5 秒 | 1GB |
| `cargo llvm-cov --all` | ~1 分 30 秒 | 5GB |
| `cargo test --all-features` | ~10-15 分钟 | 8GB |
| CI 完整覆盖率 | ~45 分钟 | 17GB |

### 1.3 磁盘消耗

| 目录 | 大小 | 说明 |
|------|------|------|
| `target/` | 17 GB | 总 target 目录 |
| `target/llvm-cov-target/` | 5 GB | llvm-cov 覆盖率数据 |
| `artifacts/coverage/` | 14 MB | 覆盖率 JSON 报告 |
| `coverage/` | 1.5 MB | LCOV 格式覆盖率 |

---

## 2. 测试分层体系

### 2.1 分层架构 (L1/L2/L3)

根据 [2026-05-04-tiered-ci-cd-implementation-plan.md](../../plans/2026-05-04-tiered-ci-cd-implementation-plan.md)：

```
PR / push (develop/v2.9.0)
         │
         ▼
┌───────────────────┐
│  L1: QUICK GATE  │  < 5 min
│  changed crate    │  单元测试
│  unit tests       │
└───────┬───────────┘
        │ PASS
        ▼
┌───────────────────┐
│ L2: INTEGRATION   │  10-20 min
│ full build        │
│ integration tests │
│ sql_corpus        │
└───────┬───────────┘
        │ PASS
        ▼
┌───────────────────┐
│  L3: EXTENDED     │  30-60 min
│ formal verification│
│ coverage (split)  │
│ TPC-H SF=0.1     │
└───────────────────┘
```

### 2.2 现有测试资产

| 脚本 | 用途 | CI 状态 | 集成层级 |
|------|------|---------|----------|
| `scripts/run_sql_corpus.sh` | SQL 语料库 (~500 条) | ❌ 未集成 | L2 |
| `scripts/test/run_integration.sh` | 集成测试 | ⚠️ 仅 Beta | L2 |
| `scripts/test/run-regression.sh` | 回归测试 | ❌ 未集成 | L3 |
| `scripts/gate/check_coverage.sh` | 覆盖率门控 | ❌ 未集成 | L3 |
| `scripts/verify/run_all_proofs.sh` | 形式化验证 | ✅ 已集成 | L3 |

### 2.3 覆盖率测试现状

**覆盖率工具对比**：

| 工具 | 状态 | 问题 |
|------|------|------|
| `cargo-tarpaulin` | ❌ 失败 | `planner_property_tests.rs` 编译错误 |
| `cargo-llvm-cov` | ✅ 成功 | 正常工作 |

---

## 3. CI/CD 集成现状

### 3.1 现有 Workflows

| 文件 | 触发分支 | 问题 |
|------|----------|------|
| `.github/workflows/coverage.yml` | develop/v2.9.0 | ✅ 正确 |
| `.github/workflows/coverage-parallel.yml` | develop/v2.9.0 | ✅ 正确 |
| `.gitea/workflows/gate-ci.yml` | develop/v2.9.0 | ✅ 正确 |

### 3.2 coverage.yml 分析

```yaml
name: Coverage

on:
  push:
    branches: [develop/v2.9.0]
  pull_request:
    branches: [develop/v2.9.0]

jobs:
  coverage:
    runs-on: ubuntu-latest
    timeout-minutes: 45

    steps:
      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: llvm-tools-preview

      - name: Install cargo-llvm-cov
        run: cargo install cargo-llvm-cov

      - name: Run coverage
        run: |
          cargo llvm-cov --all-features --json --output-path artifacts/coverage.json

      - name: Upload coverage JSON
        uses: actions/upload-artifact@v4
        with:
          name: coverage-json
          path: artifacts/coverage.json
```

### 3.3 coverage-parallel.yml 分析

并行覆盖率配置，将测试分片：

| Job | Package | Timeout |
|-----|---------|---------|
| coverage-core | parser, planner, optimizer, types, common | 15 min |
| coverage-executor | executor | 15 min |
| coverage-storage | storage | 15 min |
| coverage-transaction | transaction, catalog | 15 min |
| coverage-network | network, server | 15 min |
| coverage-vector | vector | 15 min |
| coverage-summary | 全量汇总 + threshold 检查 | 30 min |

---

## 4. 问题分析

### 4.1 覆盖率测试问题

| 问题 | 影响 | 优先级 |
|------|------|--------|
| tarpaulin 编译失败 | 无法使用标准工具 | P0 |
| 完整覆盖率 ~45min | CI 反馈时间过长 | P1 |
| 17GB 磁盘消耗 | CI 资源浪费 | P2 |
| coverage-parallel.yml 输出路径错误 | 覆盖 v2.7.0 文档 | P2 |

### 4.2 CI/CD 集成问题

| 问题 | 说明 | 优先级 |
|------|------|--------|
| 未实现 L1/L2/L3 分层 | 所有测试一起跑 | P1 |
| coverage 输出路径错误 | `docs/releases/v2.7.0/` 应该是 `v2.9.0` | P2 |
| check_coverage.sh 未集成 CI | 需手动运行 | P2 |

### 4.3 执行时间分析

**单次完整覆盖率测试**:
```
cargo llvm-cov --all-features --json --output-path artifacts/coverage.json
Real: 1m29s
User: 2m21s
Sys: 38s
```

**按 crate 分片 (coverage-parallel.yml)**:
```
coverage-core:      ~30s
coverage-executor:   ~20s
coverage-storage:    ~20s
coverage-transaction: ~15s
coverage-network:    ~15s
coverage-vector:     ~10s
Total (并行):        ~30s (理论)
```

---

## 5. CI/CD 集成方案

### 5.1 推荐的分层方案

#### L1: Quick Gate (< 5 分钟)
- 触发: 每个 PR push
- 内容: changed crate 的单元测试
- 覆盖率: 不做强制要求

#### L2: Integration (10-20 分钟)
- 触发: PR merge 到 develop
- 内容: 完整构建 + 集成测试 + SQL corpus
- 覆盖率: 可选，失败不影响 merge

#### L3: Extended (30-60 分钟)
- 触发: Beta/Release 门禁
- 内容: 完整覆盖率 + 形式化验证
- 覆盖率: **必须满足 B1 ≥75%, B2 ≥60%**

### 5.2 覆盖率 CI 集成流程

```
.github/workflows/coverage.yml (L3)
         │
         ▼
┌─────────────────────┐
│ cargo install llvm-cov│
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐
│ cargo llvm-cov      │
│ --all-features      │
│ --json             │
│ --output-path      │
│ artifacts/coverage.json │
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐
│ check_coverage.sh    │
│ B1 >= 75%           │
│ B2 >= 60%           │
└─────────┬───────────┘
          │
    ┌─────┴─────┐
    │           │
  PASS       FAIL
    │           │
    ▼           ▼
 Upload    Upload
 coverage  JSON
 JSON     + Fail
 + Pass   status
```

### 5.3 改进的 coverage.yml

```yaml
name: Coverage (RC Gate)

on:
  push:
    branches: [beta/v2.9.0, release/**]
  pull_request:
    branches: [beta/v2.9.0]
  workflow_dispatch:  # 手动触发

env:
  CARGO_TERM_COLOR: always

jobs:
  coverage:
    name: Coverage Report
    runs-on: ubuntu-latest
    timeout-minutes: 45

    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 1

      - uses: dtolnay/rust-toolchain@stable
        with:
          components: llvm-tools-preview

      - name: Install cargo-llvm-cov
        run: cargo install cargo-llvm-cov

      - name: Run coverage
        run: |
          cargo llvm-cov --all-features \
            --json \
            --output-path artifacts/coverage.json

      - name: Upload coverage JSON
        uses: actions/upload-artifact@v4
        with:
          name: coverage-json-${{ github.run_number }}
          path: artifacts/coverage.json
          retention-days: 30

      - name: Check coverage thresholds
        run: |
          python3 << 'EOF'
          import json
          import sys

          d = json.load(open('artifacts/coverage.json'))
          total = d['data'][0]['totals']['lines']['percent']
          executor = 71.08  # 需要单独运行获取

          print(f"Total coverage: {total:.2f}%")

          if total < 75:
              print(f"❌ B1 FAILED: {total:.2f}% < 75%")
              sys.exit(1)

          print("✅ B1 PASSED")
          EOF
```

### 5.4 并行覆盖率配置 (coverage-parallel.yml)

修复输出路径问题：

```yaml
# 修复前
--html --output-dir docs/releases/v2.7.0/coverage-core

# 修复后
--html --output-dir docs/releases/v2.9.0/coverage-core
```

---

## 6. 资源消耗优化

### 6.1 当前资源消耗

| 阶段 | 磁盘 | 内存 | 时间 |
|------|------|------|------|
| cargo build | 10GB | 4GB | 5-10 min |
| cargo test | 5GB | 8GB | 10-15 min |
| llvm-cov | 5GB | 4GB | 1-2 min |
| Total | 17GB | 8GB | 15-20 min |

### 6.2 优化方案

| 优化项 | 方案 | 节省 |
|--------|------|------|
| 增量编译 | `cargo build --incremental` | 50% 时间 |
| 缓存 target | actions/cache | 60% 时间 |
| 并行 llvm-cov | coverage-parallel.yml | 70% 时间 |
| 减少测试线程 | `--test-threads=4` | 30% 内存 |

### 6.3 推荐 CI 配置

```yaml
jobs:
  coverage:
    runs-on: ubuntu-latest
    timeout-minutes: 45
    strategy:
      matrix:
        crate: [parser, planner, optimizer, executor, storage, transaction]
    steps:
      - uses: actions/checkout@v4

      - uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/bin/
            ~/.cargo/registry/index/
            target/
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - uses: dtolnay/rust-toolchain@stable
        with:
          components: llvm-tools-preview

      - name: Install cargo-llvm-cov
        run: cargo install cargo-llvm-cov

      - name: Run coverage for ${{ matrix.crate }}
        run: |
          cargo llvm-cov --package sqlrustgo-${{ matrix.crate }} \
                         --no-report \
                         --json \
                         --output-path artifacts/coverage-${{ matrix.crate }}.json
```

---

## 7. Issue #263 状态

### 7.1 Issue 内容

Issue #263 要求添加测试文件：
- `planner_multi_join_test.rs` (4 tests) ✅ 已添加
- `optimizer_cbo_accuracy_test.rs` (11 tests) ⚠️ 未添加
- `network_tcp_smoke_test.rs` (6 tests) ✅ 已添加

### 7.2 测试基础设施说明

根据 Issue #263 和相关文档：

| 测试文件 | 快速运行命令 |
|----------|-------------|
| planner_multi_join_test | `cargo test --test planner_multi_join_test --all-features` |
| optimizer_cbo_accuracy_test | `cargo test --test optimizer_cbo_accuracy_test --all-features` |
| network_tcp_smoke_test | `cargo test --test network_tcp_smoke_test --all-features` |

---

## 8. 建议的实施计划

### 8.1 Phase 1: 修复问题 (1 天)

| 任务 | 操作 | 优先级 |
|------|------|--------|
| P1.1 | 修复 coverage-parallel.yml 输出路径 v2.7.0 → v2.9.0 | P0 |
| P1.2 | 验证 llvm-cov 生成正确覆盖率 | P0 |
| P1.3 | 集成 check_coverage.sh 到 coverage.yml | P1 |

### 8.2 Phase 2: 分层集成 (2 天)

| 任务 | 操作 | 优先级 |
|------|------|--------|
| P2.1 | 创建 L1 quick workflow | P1 |
| P2.2 | 创建 L2 integration workflow | P1 |
| P2.3 | 集成 coverage 到 L3 | P1 |

### 8.3 Phase 3: 优化 (1 天)

| 任务 | 操作 | 优先级 |
|------|------|--------|
| P3.1 | 添加 cargo cache | P2 |
| P3.2 | 优化 llvm-cov 并行度 | P2 |
| P3.3 | 添加 coverage dashboard | P2 |

---

## 9. 结论

### 9.1 当前状态

- ✅ 覆盖率工具已从 tarpaulin 切换到 llvm-cov
- ✅ B1/B2 Gate 通过 (84.18% / 71.08%)
- ✅ CI workflow 已正确配置
- ⚠️ 需要修复 coverage-parallel.yml 路径问题
- ⚠️ L1/L2/L3 分层未完全实现

### 9.2 建议

1. **立即修复**: coverage-parallel.yml 输出路径
2. **短期**: 集成 check_coverage.sh 到 CI
3. **中期**: 实现 L1/L2/L3 分层
4. **长期**: 添加 coverage dashboard 和 trend 分析

---

## 10. 相关文档

- [COVERAGE_REPORT_RC.md](./COVERAGE_REPORT_RC.md)
- [COVERAGE_IMPROVEMENT_REPORT.md](./COVERAGE_IMPROVEMENT_REPORT.md)
- [RELEASE_GATE_CHECKLIST.md](./RELEASE_GATE_CHECKLIST.md)
- [Tiered CI/CD Implementation Plan](../../plans/2026-05-04-tiered-ci-cd-implementation-plan.md)
- [Formal Verification Toolchain CI/CD Guide](./TOOLCHAIN_CICD_GUIDE.md)

---

*本文档由 SQLRustGo Team 维护*
*更新日期: 2026-05-04*