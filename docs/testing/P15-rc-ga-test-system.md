# P15 - 双测试系统架构 (v2.0)

## Pattern: RC/GA 双测试系统架构

### 背景

v3.1.0 RC/GA 阶段需要完整的长期测试验证，包括 TPC-H SF=1/SF=10、72h 稳定性、崩溃恢复等。

### 问题

- 测试体系分散，缺乏统一入口
- 长期测试（6h+）没有设计
- 本地开发测试和 CI 测试没有分层

### 解决方案

**双测试系统架构 v2.0**

```
┌─────────────────────────────────────────────────────────────────────┐
│                        SQLRustGo 测试基础设施 v2.0                      │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌───────────────────────────┐      ┌────────────────────────────┐   │
│  │   SYSTEM 1: LOCAL DEV     │      │   SYSTEM 2: NOMAD CI       │   │
│  │   本地开发测试系统          │      │   Nomad CI 门禁系统         │   │
│  ├───────────────────────────┤      ├────────────────────────────┤   │
│  │ L0: 冒烟 (<5min)         │      │ PR-Gate: PR 合并门禁      │   │
│  │ L1: 单元 (10min)          │      │ B-Gate: Beta 门禁         │   │
│  │ L2: 集成 (15min)          │      │ R-Gate: RC 门禁           │   │
│  │ L3: 回归 (20min)          │      │ G-Gate: GA 门禁           │   │
│  └───────────────────────────┘      └────────────────────────────┘   │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### 实现

#### System 1: 本地开发测试

| 层级 | 脚本 | 执行时间 | 触发 |
|------|------|----------|------|
| L0 | `l0_smoke.sh` | <5min | 每次 push 前 |
| L1 | `l1_unit.sh` | <10min | 每次 push |
| L2 | `l2_integration.sh` | <15min | PR |
| L3 | `l3_regression.sh` | <20min | 每日 |

#### System 2: Nomad CI 门禁

| Gate | 脚本 | 执行时间 | 触发 |
|------|------|----------|------|
| PR-Gate | `pr_gate.sh` | ~20min | 每个 PR |
| B-Gate | `b_gate.sh` | ~2h | Alpha→Beta |
| R-Gate | `r_gate.sh` | ~6h | Beta→RC |
| G-Gate | `g_gate.sh` | ~12h | RC→GA |

### RC 阶段测试计划 (~6h)

```
Phase 1: 基础门禁 (30min)
  - B1: cargo build --release
  - B2: cargo test --lib
  - B3: cargo clippy
  - B4: cargo fmt --check
  - B5: coverage >=85%

Phase 2: SQL兼容性 (1h)
  - SQL Corpus >=95%
  - TPC-H SF=0.1

Phase 3: 性能测试 (2h)
  - TPC-H SF=1 (22 queries, 并行)
  - Sysbench QPS

Phase 4: 稳定性测试 (2.5h)
  - 崩溃恢复 100次
  - 并发压力测试
  - 快速稳定性 2h

Phase 5: 安全回归 (30min)
  - cargo audit
  - regression check
  - formal proofs >=30
```

### GA 阶段测试计划 (~12h)

```
额外增加:
  - TPC-H SF=10 (6h)
  - 完整 72h 稳定性测试
```

### 关键脚本

| 脚本 | 用途 |
|------|------|
| `scripts/test/nomad/rc_gate.sh` | RC Gate 主入口 |
| `scripts/test/nomad/tpch_sf1.sh` | TPC-H SF=1 并行执行 |
| `scripts/test/nomad/stability_72h.sh` | 72h 稳定性测试 |
| `scripts/test/nomad/crash_recovery_100.sh` | 100次崩溃恢复 |
| `scripts/test/local/l0_smoke.sh` | L0 冒烟测试 |

### 触发方式

```bash
# 本地测试
./scripts/test/local/l0_smoke.sh      # L0 冒烟
./scripts/test/local/run_local.sh all  # 全部本地测试

# RC Gate
./scripts/test/nomad/rc_gate.sh        # 完整 6h
./scripts/test/nomad/tpch_sf1.sh       # 仅 TPC-H SF=1
./scripts/test/nomad/stability_72h.sh  # 仅 72h 稳定性
```

### 验证状态

- [x] 双测试系统架构设计完成
- [x] L0-L3 本地测试脚本
- [x] PR/B/R/G Gate 脚本
- [x] RC Gate 入口脚本
- [x] TPC-H SF=1 并行脚本
- [x] 72h 稳定性测试脚本
- [x] 100次崩溃恢复脚本
- [x] 同步到 develop/v3.1.0

### 相关 Issue

- #606: v3.1.0 RC 门禁检查
- #713: 覆盖率 85% (已关闭)
- #714: SQL 兼容性 95%
- #715: 形式化证明 30+
- #716: 安全+回归
- #717: Sysbench+FTS

### 创建时间

2026-05-13
