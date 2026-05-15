# SQLRustGo 双测试系统架构

> 设计日期: 2026-05-13
> 版本: 1.0

## 1. 设计目标

| 目标 | 描述 |
|------|------|
| **快速反馈** | 本地开发周期 <10min |
| **严格门禁** | CI/CD Gate 覆盖全面 |
| **分层测试** | 单元 → 集成 → E2E → 性能 → 稳定性 |
| **双轨并行** | 本地开发轨 + Nomad CI 轨 |

## 2. 双系统架构

```
┌─────────────────────────────────────────────────────────────────┐
│                    SQLRustGo 测试基础设施                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────────────┐    ┌──────────────────────────────┐  │
│  │   SYSTEM 1: LOCAL    │    │     SYSTEM 2: NOMAD CI       │  │
│  │   本地开发测试系统      │    │      Nomad CI 门禁系统       │  │
│  ├──────────────────────┤    ├──────────────────────────────┤  │
│  │ 触发: 开发者手动       │    │ 触发: PR/合并/定时           │  │
│  │ 目标: <10min 反馈     │    │ 目标: 100% 覆盖率门禁         │  │
│  │ 环境: Mac Mini/Z6G4   │    │ 环境: Nomad Cluster          │  │
│  └──────────────────────┘    └──────────────────────────────┘  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## 3. System 1: 本地开发测试系统 (Local)

### 3.1 分层结构

| 层级 | 名称 | 执行时间 | 触发方式 |
|------|------|----------|----------|
| **L0** | 冒烟测试 | <5min | 每次 push 前强制 |
| **L1** | 单元测试 | <10min | 每次 push 前强制 |
| **L2** | 集成测试 | <15min | 每次 PR |
| **L3** | 快速回归 | <20min | 每日 / 关键 PR |

### 3.2 L0 冒烟测试 (每次 push 前)

```
scripts/test/local/l0_smoke.sh
```

| 检查项 | 命令 | 失败处理 |
|--------|------|----------|
| 构建 | `cargo build --release --workspace` | 阻塞 |
| 格式 | `cargo fmt --check --all` | 阻塞 |
| Clippy | `cargo clippy --all-features -- -D warnings` | 阻塞 |
| 类型测试 | `cargo test -p sqlrustgo-types --lib` | 阻塞 |
| Parser 测试 | `cargo test -p sqlrustgo-parser --lib` | 阻塞 |

### 3.3 L1 单元测试 (crate 级别)

```bash
# 核心 crates 全量测试
cargo test -p sqlrustgo-executor --lib
cargo test -p sqlrustgo-planner --lib
cargo test -p sqlrustgo-optimizer --lib
cargo test -p sqlrustgo-storage --lib
cargo test -p sqlrustgo-transaction --lib
cargo test -p sqlrustgo-catalog --lib
cargo test -p sqlrustgo-network --lib
cargo test -p sqlrustgo-gmp --lib
```

### 3.4 L2 集成测试

```bash
# 使用 test manifest
scripts/test/run.sh --dimension correctness --group full
```

| 测试类型 | manifest 标签 | 数量估计 |
|----------|---------------|----------|
| SQL 正确性 | `sql_corpus` | ~2000 |
| 聚合函数 | `aggregate` | ~50 |
| 索引扫描 | `index_scan` | ~30 |
| MVCC 事务 | `mvcc` | ~40 |
| WAL 集成 | `wal` | ~20 |
| 网络协议 | `network` | ~15 |

### 3.5 L3 快速回归 (关键路径)

```bash
scripts/test/local/l3_regression.sh
```

| 测试集 | 覆盖场景 |
|--------|----------|
| DDL 语句 | CREATE/ALTER/DROP |
| DML 语句 | INSERT/UPDATE/DELETE |
| 查询语句 | SELECT 完整覆盖 |
| 事务边界 | COMMIT/ROLLBACK |

---

## 4. System 2: Nomad CI 门禁系统

### 4.1 分层结构

| Gate | 名称 | 触发条件 | 执行位置 |
|------|------|----------|----------|
| **PR-Gate** | PR 合并门禁 | 每个 PR | Nomad runner |
| **B-Gate** | Beta 门禁 | Alpha→Beta | Nomad runner |
| **R-Gate** | RC 门禁 | Beta→RC | Nomad runner |
| **G-Gate** | GA 门禁 | RC→GA | Nomad runner |

### 4.2 PR-Gate (每个 PR 必须通过)

| 检查项 | 阈值 | 执行器 |
|--------|------|--------|
| 构建 | --release | `nomad/cargo-build` |
| 单元测试 | 100% | `nomad/cargo-test` |
| Clippy | 零警告 | `nomad/cargo-clippy` |
| 格式 | fmt --check | `nomad/cargo-fmt` |
| 覆盖率 | ≥60% | `nomad/cargo-llvm-cov` |
| SQL 测试 | ≥80% | `nomad/sql-corpus` |

### 4.3 Beta Gate (Beta 发布门禁)

| 检查项 | 阈值 | 脚本 |
|--------|------|------|
| 构建 | --release | `check_beta.sh` B1 |
| 单元测试 | ≥90% | `check_beta.sh` B2 |
| Clippy | 零警告 | `check_beta.sh` B3 |
| 格式 | fmt --check | `check_beta.sh` B4 |
| 覆盖率 | ≥75% | `check_beta.sh` B5 |
| 安全 | cargo audit | `check_beta.sh` B6 |
| SQL 兼容性 | ≥80% | `check_beta.sh` B7 |
| TPC-H | SF=1 通过 | `check_beta.sh` B8 |
| 形式化证明 | ≥20 | `check_beta.sh` B9 |
| 并发压力 | stress tests | `check_beta.sh` B-S1 |
| 崩溃恢复 | 100 iterations | `check_beta.sh` B-S2 |
| 长期稳定性 | 72h 运行 | `check_beta.sh` B-S3 |

### 4.4 RC Gate (RC 发布门禁)

| 检查项 | 阈值 | 脚本 |
|--------|------|------|
| 构建 | --release | `check_rc.sh` R1 |
| 测试套件 | ≥90% | `check_rc.sh` R2 |
| Clippy | 零警告 | `check_rc.sh` R3 |
| 格式 | fmt --check | `check_rc.sh` R4 |
| 覆盖率 | ≥85% | `check_rc.sh` R5 |
| 安全 | cargo audit | `check_rc.sh` R6 |
| SQL 兼容性 | ≥95% | `check_rc.sh` R7 |
| TPC-H SF=1 | p99<5s | `check_rc.sh` R8 |
| 回归检查 | baseline 对比 | `check_rc.sh` R9 |
| 形式化证明 | ≥30 | `check_rc.sh` R10 |
| 文档链接 | 100% | `check_rc.sh` R11 |

### 4.5 GA Gate (GA 发布门禁)

| 检查项 | 阈值 |
|--------|------|
| 覆盖率 | ≥90% |
| SQL 兼容性 | ≥98% |
| TPC-H 全部 | 22/22 p99<5s |
| QPS 基准 | ≥10000 |
| 安全审计 | 零漏洞 |
| 形式化证明 | ≥50 |

---

## 5. 测试分类矩阵

| 测试类型 | L0 | L1 | L2 | L3 | PR-Gate | B-Gate | R-Gate | G-Gate |
|----------|----|----|----|----|---------|--------|--------|--------|
| 冒烟测试 | ✅ | | | | ✅ | ✅ | ✅ | ✅ |
| 单元测试 | | ✅ | | | ✅ | ✅ | ✅ | ✅ |
| 集成测试 | | | ✅ | | ✅ | ✅ | ✅ | ✅ |
| SQL 正确性 | | | ✅ | | ✅ | ✅ | ✅ | ✅ |
| 性能测试 | | | | ✅ | | ✅ | ✅ | ✅ |
| TPC-H | | | | ✅ | | ✅ | ✅ | ✅ |
| 崩溃恢复 | | | | ✅ | | ✅ | ✅ | ✅ |
| 稳定性测试 | | | | ✅ | | ✅ | ✅ | ✅ |
| 安全审计 | | | | | | ✅ | ✅ | ✅ |
| 形式化证明 | | | | | | ✅ | ✅ | ✅ |
| E2E 测试 | | | | ✅ | | | ✅ | ✅ |

---

## 6. 执行器定义

### 6.1 Nomad 执行器 (System 2)

| 执行器 | 用途 | 容器镜像 |
|--------|------|----------|
| `nomad/cargo-build` | 构建 | `rust:1.75-slim` |
| `nomad/cargo-test` | 测试 | `rust:1.75-slim` |
| `nomad/cargo-clippy` | Lint | `rust:1.75-slim` |
| `nomad/cargo-fmt` | 格式检查 | `rust:1.75-slim` |
| `nomad/cargo-llvm-cov` | 覆盖率 | `x Mess/rust-llvm-cov` |
| `nomad/sql-corpus` | SQL 兼容性 | `sqlrustgo/test-sql` |
| `nomad/tla-dafny` | 形式化证明 | `tlacorporation/tla2tools` |
| `nomad/sysbench` | 性能基准 | `percona/sysbench` |
| `nomad/crash-loop` | 崩溃测试 | `sqlrustgo/test-crash` |

### 6.2 本地执行器 (System 1)

| 执行器 | 用途 |
|--------|------|
| `local/cargo-test` | 单元测试 |
| `local/integration` | 集成测试 |
| `local/smoke` | 冒烟测试 |
| `local/fast-regression` | 快速回归 |

---

## 7. 文件结构

```
scripts/
├── test/
│   ├── local/
│   │   ├── l0_smoke.sh          # L0 冒烟测试
│   │   ├── l1_unit.sh           # L1 单元测试
│   │   ├── l2_integration.sh     # L2 集成测试
│   │   ├── l3_regression.sh      # L3 快速回归
│   │   └── run_local.sh          # 本地测试入口
│   ├── nomad/
│   │   ├── pr_gate.sh            # PR 门禁
│   │   ├── b_gate.sh             # Beta 门禁
│   │   ├── r_gate.sh             # RC 门禁
│   │   ├── g_gate.sh             # GA 门禁
│   │   └── run_nomad.sh          # Nomad 测试入口
│   ├── run.sh                    # 统一测试 runner (manifest)
│   ├── run_integration.sh        # 集成测试
│   ├── run_regression.sh         # 回归测试
│   └── manifest/
│       ├── test_manifest.yaml     # 测试清单
│       └── known_failures.yaml   # 已知失败
├── gate/
│   ├── check_l0_smoke.sh         # Gate L0 检查
│   ├── check_beta.sh             # Beta Gate
│   ├── check_rc.sh               # RC Gate
│   ├── check_ga.sh               # GA Gate
│   ├── check_coverage.sh         # 覆盖率检查
│   ├── check_tpch.sh             # TPC-H 检查
│   ├── check_performance.sh      # 性能检查
│   ├── check_security.sh          # 安全检查
│   ├── check_regression.sh       # 回归检查
│   ├── crash_recovery_loop.sh    # 崩溃恢复
│   └── stability_test.sh         # 稳定性测试
└── ci/
    ├── gitea-actions/
    │   ├── pr_gate.yml           # PR Gate CI
    │   ├── beta_gate.yml         # Beta Gate CI
    │   ├── rc_gate.yml           # RC Gate CI
    │   └── ga_gate.yml           # GA Gate CI
    └── nomad/
        ├── pr_gate.nomad         # PR Gate Nomad Job
        ├── b_gate.nomad          # Beta Gate Nomad Job
        ├── r_gate.nomad          # RC Gate Nomad Job
        └── g_gate.nomad          # GA Gate Nomad Job
```

---

## 8. 触发机制

### 8.1 本地触发 (System 1)

```bash
# 快速冒烟 (<5min)
./scripts/test/local/l0_smoke.sh

# 完整本地测试 (<20min)
./scripts/test/local/run_local.sh --level 3

# 指定层级
./scripts/test/local/run_local.sh --level 1 --crate executor
```

### 8.2 CI 触发 (System 2)

| 触发条件 | 执行 Gate | 资源 |
|----------|-----------|------|
| PR 创建/更新 | PR-Gate | 1x runner |
| develop 合并 | PR-Gate | 1x runner |
| Tag v*alpha* | B-Gate | 4x runners |
| Tag v*beta* | R-Gate | 4x runners |
| Tag v*rc* | G-Gate | 8x runners |
| 每日定时 (02:00) | R-Gate | 4x runners |

---

## 9. 报告机制

### 9.1 本地报告

```bash
# JSON 格式输出
./scripts/test/local/l0_smoke.sh --json

# 输出示例
{
  "system": "local",
  "level": "L0",
  "timestamp": "2026-05-13T12:00:00Z",
  "duration_seconds": 180,
  "results": {
    "build": "PASS",
    "format": "PASS",
    "clippy": "PASS",
    "types_test": "PASS",
    "parser_test": "PASS"
  },
  "total": 5,
  "passed": 5,
  "failed": 0
}
```

### 9.2 Nomad CI 报告

```bash
# 写入 artifacts
/artifacts/
├── gate_report.json          # 门禁报告
├── coverage_report.json      # 覆盖率报告
├── tpch_report.json          # TPC-H 报告
├── security_report.json      # 安全报告
└── proof_report.json         # 形式化证明报告
```

---

## 10. 故障处理

### 10.1 本地失败

| 层级 | 失败处理 |
|------|----------|
| L0 | 阻塞 push，提示修复 |
| L1 | 阻塞 push，输出失败测试 |
| L2 | 允许 push，标记 WARN |
| L3 | 仅通知，不阻塞 |

### 10.2 CI 失败

| Gate | 失败处理 |
|------|----------|
| PR-Gate | 阻塞合并，通知作者 |
| B-Gate | 阻塞 Beta 发布，通知团队 |
| R-Gate | 阻塞 RC 发布，阻止进入 GA |
| G-Gate | 阻塞 GA 发布，完整审查 |

---

## 11. 已知问题

- [ ] L3 回归测试需要完善 manifest
- [ ] E2E 测试框架需要实现
- [ ] Nomad job 定义需要部署
- [ ] 稳定性测试需要 72h 环境
