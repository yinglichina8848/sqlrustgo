# RC 阶段测试计划

> v3.1.0 RC 门禁测试任务定义

## 测试执行方式

### 方式一：本地开发测试 (System 1 - Local)

**用途**: 开发者本地快速验证，<20min 完成

| 层级 | 命令 | 验证内容 | 执行时间 |
|------|------|----------|----------|
| L0 | `./scripts/test/local/l0_smoke.sh` | 构建/格式/Clippy/基础测试 | <5min |
| L1 | `./scripts/test/local/l1_unit.sh` | 单元测试 (crate 级别) | <10min |
| L2 | `./scripts/test/local/l2_integration.sh` | 集成测试 | <15min |
| L3 | `./scripts/test/local/run_local.sh all` | 完整本地测试 | <20min |

**触发**: 每次 push 前、PR 创建前

### 方式二：Nomad CI 门禁测试 (System 2 - CI)

**用途**: RC/GA 发布前的完整验证，6h-12h 完成

| Gate | 命令 | 验证内容 | 执行时间 |
|------|------|----------|----------|
| PR-Gate | `./scripts/test/nomad/pr_gate.sh` | PR 合并前验证 | ~20min |
| B-Gate | `./scripts/test/nomad/b_gate.sh` | Beta 发布验证 | ~2h |
| **R-Gate** | `./scripts/test/nomad/rc_gate.sh` | **RC 发布验证** | **~6h** |
| G-Gate | `./scripts/test/nomad/g_gate.sh` | GA 发布验证 | ~12h |

**触发**: PR 合并、版本标签、定时任务

## RC 门禁测试任务矩阵

| # | 测试任务 | 方式 | 执行命令 | 阈值 | RC 前完成 |
|---|----------|------|----------|------|-----------|
| 1 | 基础构建 | Local/CI | `cargo build --release` | 成功 | ✅ |
| 2 | 单元测试 | Local/CI | `cargo test --lib` | 100% pass | ✅ |
| 3 | Clippy 检查 | Local/CI | `cargo clippy --all-features` | 零警告 | ✅ |
| 4 | 格式检查 | Local/CI | `cargo fmt --check` | 通过 | ✅ |
| 5 | 覆盖率 | CI | `cargo llvm-cov` | ≥85% | ⏳ |
| 6 | SQL 兼容性 | CI | `check_sql_compat.sh` | ≥95% | ⏳ |
| 7 | TPC-H SF=0.1 | CI | `check_tpch.sh sf=0.1` | Q1≤10s, Q6≤6s | ✅ |
| 8 | TPC-H SF=1 | CI | `tpch_sf1.sh` | 22/22 完成 | ⏳ |
| 9 | Sysbench QPS | CI | `check_sysbench.sh` | ≥10000 QPS | ⏳ |
| 10 | 崩溃恢复 100次 | CI | `crash_recovery_100.sh` | 100% pass | ⏳ |
| 11 | 稳定性 2h | CI | `stability_short.sh` | 无崩溃 | ⏳ |
| 12 | 稳定性 72h | CI | `stability_72h.sh` | 无崩溃 | ⏳ |
| 13 | 并发压力 | CI | `check_concurrency_stress.sh` | 通过 | ⏳ |
| 14 | 安全审计 | CI | `cargo audit` | 零漏洞 | ⏳ |
| 15 | 回归检查 | CI | `check_regression.sh` | 通过 | ⏳ |
| 16 | 形式化证明 | CI | `check_proof.sh` | ≥30 | ⏳ |
| 17 | E2E 测试 | CI | `run_e2e.sh` | 全部通过 | ⏳ |
| 18 | TPC-H SF=10 | CI | `tpch_sf10.sh` | 22/22 完成 | ⏳ (GA) |

## RC 门禁执行流程

```
┌─────────────────────────────────────────────────────────────┐
│                    RC Gate 执行流程                           │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Hour 0:00                                                  │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ Phase 1: 基础门禁 (30min)                           │   │
│  │   - cargo build --release                           │   │
│  │   - cargo test --lib                                │   │
│  │   - cargo clippy                                    │   │
│  │   - cargo fmt --check                               │   │
│  │   - cargo llvm-cov (coverage >=85%)                 │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  Hour 0:30                                                  │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ Phase 2: SQL 兼容性 (1h)                            │   │
│  │   - SQL Corpus (>=95%)                              │   │
│  │   - TPC-H SF=0.1                                    │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  Hour 1:30                                                  │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ Phase 3: 性能测试 (2h)                              │   │
│  │   - TPC-H SF=1 (并行 6x)                            │   │
│  │   - Sysbench QPS                                    │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  Hour 3:30                                                  │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ Phase 4: 稳定性测试 (2.5h)                          │   │
│  │   - 崩溃恢复 100次                                   │   │
│  │   - 并发压力测试                                     │   │
│  │   - 快速稳定性 2h                                   │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  Hour 6:00                                                  │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ Phase 5: 安全与回归 (30min)                         │   │
│  │   - cargo audit                                     │   │
│  │   - regression check                                │   │
│  │   - formal proofs                                   │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  Hour 6:30 → 报告生成 → 发布决定                             │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## 测试脚本清单

### 本地测试 (scripts/test/local/)

```
scripts/test/local/
├── l0_smoke.sh          # L0 冒烟测试 (<5min)
├── l1_unit.sh           # L1 单元测试 (<10min)
├── l2_integration.sh    # L2 集成测试 (<15min)
├── l3_regression.sh     # L3 回归测试 (<20min)
└── run_local.sh         # 本地测试统一入口
```

### Nomad CI 测试 (scripts/test/nomad/)

```
scripts/test/nomad/
├── pr_gate.sh           # PR 门禁 (~20min)
├── b_gate.sh            # Beta 门禁 (~2h)
├── r_gate.sh            # RC 门禁 (~6h)
├── g_gate.sh            # GA 门禁 (~12h)
├── rc_gate.sh           # RC Gate 主入口 (~6h)
├── tpch_sf1.sh          # TPC-H SF=1 并行执行 (~1h)
├── stability_72h.sh     # 72h 稳定性测试 (72h)
├── stability_short.sh    # 短期稳定性测试 (2-24h)
└── crash_recovery_100.sh # 100次崩溃恢复 (~1h)
```

## 测试报告

### 报告输出位置

```
/tmp/
├── rc_gate_artifacts/           # RC Gate 报告
│   └── rc_gate_report.json
├── tpch_artifacts/              # TPC-H 报告
│   └── tpch_sf1_report.json
├── stability_artifacts/         # 稳定性报告
│   └── stability_report.json
├── crash_artifacts/             # 崩溃恢复报告
│   └── crash_recovery_report.json
└── *_artifacts/                # 其他报告
```

### 报告格式

```json
{
  "gate": "RC-GATE",
  "version": "v3.1.0",
  "timestamp_start": "2026-05-13T00:00:00Z",
  "timestamp_end": "2026-05-13T06:00:00Z",
  "duration_seconds": 21600,
  "status": "PASS | FAIL",
  "results": {
    "passed": 18,
    "failed": 0,
    "skipped": 0
  },
  "tests": [
    {"name": "build", "status": "PASS", "duration_ms": 120000},
    {"name": "tpch_sf1", "status": "PASS", "duration_ms": 3600000},
    ...
  ]
}
```

## 触发方式

### 本地触发

```bash
# 快速冒烟测试
./scripts/test/local/l0_smoke.sh

# 完整本地测试
./scripts/test/local/run_local.sh all

# 指定层级
./scripts/test/local/run_local.sh L1 executor
```

### Nomad CI 触发

```bash
# 通过 cron 定时触发
0 2 * * 0  bash /repo/scripts/test/nomad/rc_gate.sh

# 手动触发
nomad job run /repo/scripts/test/nomad/rc_gate.nomad
```

## 验收标准

RC 发布必须满足：

| 测试类型 | 最低要求 | 理想目标 |
|----------|----------|----------|
| 覆盖率 | ≥85% | ≥90% |
| SQL 兼容性 | ≥95% | ≥98% |
| TPC-H SF=1 | 22/22 完成 | 22/22 p99<5s |
| 崩溃恢复 | 100/100 | 100/100 |
| 稳定性 | 2h 无崩溃 | 72h 无崩溃 |
| 形式化证明 | ≥30 | ≥50 |

## 相关文档

- `TEST_SYSTEM_ARCHITECTURE.md` - 双测试系统架构文档
- `scripts/test/README.md` - 测试系统使用说明
- `P15-rc-ga-test-system.md` - GBrain Pattern

## 创建时间

2026-05-13
