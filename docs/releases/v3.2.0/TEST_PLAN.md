# SQLRustGo v3.2.0 测试计划

> **版本**: v1.0
> **日期**: 2026-05-15
> **状态**: 规划中
> **目标**: Trusted GMP Data Platform 质量保障

---

## 一、测试策略

### 1.1 测试金字塔

```
v3.2.0 测试金字塔
────────────────────────────────────────────
                    ▲
                   /E2E\         集成测试 (150+)
                  /───────\
                 / smoke  \       冒烟测试 (80+)
                /─────────\
               /   unit   \     单元测试 (800+)
              /────────────\
             /  coverage   \   覆盖率驱动测试
            └───────────────┘
```

### 1.2 测试类别

| 类别 | v3.1.0 | v3.2.0 目标 | 增长 |
|----------|---------|---------------|------|
| 单元测试 | ~500 | ≥800 | +60% |
| 集成测试 | ~100 | ≥150 | +50% |
| 端到端测试 | ~20 | ≥40 | +100% |
| GMP 合规测试 | 0 | ≥50 | 新增 |
| 混沌/崩溃测试 | 10 | 15 | +50% |
| 并发测试 | 50 | 80 | +60% |
| **合计** | **~700** | **≥1135** | +62% |

### 1.3 测试策略特点

v3.2.0 测试策略重点：

1. **GMP 合规优先**: 电子签名、审计链、数字签名测试
2. **高级 QA 能力**: Fuzzing、Mutation Testing、SQLancer
3. **性能回归**: QPS 基准、内存管理、TPC-H SF=10
4. **覆盖率提升**: 从 85% 提升至 90%

---

## 二、Alpha 阶段测试 (2026-10-01)

### 2.1 GMP 基础测试

#### 数字签名审计链测试

| 测试 | 文件 | 目标 | 优先级 |
|------|------|------|--------|
| `signature_chain_test` | `tests/gmp/signature_chain_test.rs` | 签名/验签流程 | P0 |
| `signature_types_test` | `tests/gmp/signature_types_test.rs` | User/Device/System/Batch | P0 |
| `signature_consistency_test` | `tests/gmp/signature_consistency_test.rs` | 链一致性 | P0 |
| `hash_verification_test` | `tests/gmp/hash_verification_test.rs` | SHA-256 链验证 | P0 |

#### Immutable Record 测试

| 测试 | 文件 | 目标 | 优先级 |
|------|------|------|--------|
| `immutable_insert_test` | `tests/gmp/immutable_insert_test.rs` | INSERT 允许 | P0 |
| `immutable_update_test` | `tests/gmp/immutable_update_test.rs` | UPDATE 禁止 | P0 |
| `immutable_delete_test` | `tests/gmp/immutable_delete_test.rs` | DELETE 禁止 | P0 |
| `immutable_correction_test` | `tests/gmp/immutable_correction_test.rs` | Correction 允许 | P0 |

#### Correction Chain 测试

| 测试 | 文件 | 目标 | 优先级 |
|------|------|------|--------|
| `correction_chain_test` | `tests/gmp/correction_chain_test.rs` | 完整修正链 | P0 |
| `correction_approval_test` | `tests/gmp/correction_approval_test.rs` | 审批流程 | P0 |
| `correction_audit_test` | `tests/gmp/correction_audit_test.rs` | 修正审计 | P0 |

#### Provenance Tracking 测试

| 测试 | 文件 | 目标 | 优先级 |
|------|------|------|--------|
| `provenance_basic_test` | `tests/gmp/provenance_basic_test.rs` | 基础溯源 | P0 |
| `provenance_device_test` | `tests/gmp/provenance_device_test.rs` | 设备指纹 | P0 |
| `provenance_chain_test` | `tests/gmp/provenance_chain_test.rs` | 溯源链 | P0 |

### 2.2 覆盖率目标

| 阶段 | 日期 | 总体 | parser | executor | planner | optimizer |
|-------|------|---------|--------|----------|---------|-----------|
| Alpha | 2026-10-01 | ≥50% | 70% | 60% | 55% | 45% |

---

## 三、Beta 阶段测试 (2026-12-01)

### 3.1 电子签名测试

#### 电子签名功能测试

| 测试 | 文件 | 目标 | 优先级 |
|------|------|------|--------|
| `electronic_signature_test` | `tests/gmp/electronic_signature_test.rs` | 基础电子签名 | P0 |
| `signature_reason_test` | `tests/gmp/signature_reason_test.rs` | 签署理由 | P0 |
| `signature_timestamp_test` | `tests/gmp/signature_timestamp_test.rs` | 时间戳 | P0 |
| `four_eyes_test` | `tests/gmp/four_eyes_test.rs` | 双人复核 | P0 |
| `signature_verification_test` | `tests/gmp/signature_verification_test.rs` | 签名验证 | P0 |

#### 21 CFR Part 11 合规测试

| 测试 | 文件 | 目标 | 优先级 |
|------|------|------|--------|
| `audit_trail_test` | `tests/gmp/audit_trail_test.rs` | 审计追踪 | P0 |
| `non_repudiation_test` | `tests/gmp/non_repudiation_test.rs` | 不可抵赖 | P0 |
| `signature_irrevocable_test` | `tests/gmp/signature_irrevocable_test.rs` | 签名不可撤销 | P0 |

### 3.2 HSM/KMS 集成测试

| 测试 | 文件 | 目标 | 优先级 |
|------|------|------|--------|
| `hsm_tpm_test` | `tests/gmp/hsm_tpm_test.rs` | TPM 集成 | P0 |
| `hsm_software_test` | `tests/gmp/hsm_software_test.rs` | 软件模拟 | P0 |
| `kms_integration_test` | `tests/gmp/kms_integration_test.rs` | KMS 集成 | P0 |
| `key_rotation_test` | `tests/gmp/key_rotation_test.rs` | 密钥轮换 | P0 |

### 3.3 Performance Schema 测试

| 测试 | 文件 | 目标 | 覆盖率 |
|------|------|------|--------|
| `ps_setup_actors_test` | `tests/ps/ps_setup_actors_test.rs` | setup_actors 表 | 100% |
| `ps_setup_instruments_test` | `tests/ps/ps_setup_instruments_test.rs` | setup_instruments 表 | 100% |
| `ps_events_statements_test` | `tests/ps/ps_events_statements_test.rs` | events_statements 表 | 100% |
| `ps_events_waits_test` | `tests/ps/ps_events_waits_test.rs` | events_waits 表 | 100% |
| `ps_global_events_test` | `tests/ps/ps_global_events_test.rs` | global_events 表 | 100% |

### 3.4 覆盖率目标

| 阶段 | 日期 | 总体 | parser | executor | planner | optimizer |
|-------|------|---------|--------|----------|---------|-----------|
| Beta | 2026-12-01 | ≥65% | 75% | 70% | 65% | 55% |

---

## 四、RC 阶段测试 (2027-01-15)

### 4.1 GMP 合规完整测试

#### 审计链完整性测试

| 测试 | 场景 | 目标 | 优先级 |
|------|------|------|--------|
| `audit_chain_complete_test` | 完整审计链验证 | 0 中断 | P0 |
| `audit_chain_concurrent_test` | 100 并发审计写入 | 0 丢失 | P0 |
| `audit_chain_recovery_test` | 崩溃后审计恢复 | 完整性 | P0 |
| `audit_chain_verify_tool_test` | 验证工具测试 | 工具正确 | P0 |

#### ALCOA+ 合规测试

| 测试 | ALCOA+ | 目标 | 优先级 |
|------|--------|------|--------|
| `alcoa_attributable_test` | A - Attributable | 数字签名归因 | P0 |
| `alcoa_contemporaneous_test` | C - Contemporaneous | RFC3161 时间戳 | P0 |
| `alcoa_original_test` | O - Original | 原始记录哈希 | P0 |
| `alcoa_complete_test` | +C - Complete | 完整 Provenance | P0 |
| `alcoa_enduring_test` | +E - Enduring | 冷存储持久 | P0 |

### 4.2 性能测试

#### QPS 基准测试

| 测试 | 目标 | 优先级 |
|------|------|--------|
| `qps_point_select_test` | ≥1,000,000 ops/s | P0 |
| `qps_update_test` | ≥800,000 ops/s | P0 |
| `qps_delete_test` | ≥800,000 ops/s | P0 |
| `qps_complex_where_test` | ≥500,000 ops/s | P0 |

#### TPC-H 测试

| 测试 | 目标 | 优先级 |
|------|------|--------|
| `tpch_sf10_test` | 22/22 通过 | P0 |
| `tpch_sf1_test` | 22/22 基准 | P0 |

#### 内存管理测试

| 测试 | 目标 | 优先级 |
|------|------|--------|
| `memory_footprint_test` | ≤85% v3.1.0 | P0 |
| `memory_leak_test` | 无泄漏 | P0 |

### 4.3 RECURSIVE CTE 测试

| 测试 | 场景 | 目标 | 优先级 |
|------|------|------|--------|
| `recursive_cte_basic_test` | 基础递归 | 正确结果 | P1 |
| `recursive_cte_depth_test` | 深度递归 | 正确截断 | P1 |
| `recursive_cte_terminate_test` | 递归终止 | 正确终止 | P1 |

### 4.4 覆盖率目标

| 阶段 | 日期 | 总体 | parser | executor | planner | optimizer |
|-------|------|---------|--------|----------|---------|-----------|
| RC | 2027-01-15 | ≥75% | 80% | 75% | 70% | 65% |

---

## 五、GA 阶段测试 (2027-02-15)

### 5.1 全面回归测试

| 测试套件 | 测试数 | 目标 | 通过率 |
|----------|--------|------|--------|
| 单元测试 | ≥800 | 全部 | 100% |
| 集成测试 | ≥150 | 全部 | 100% |
| E2E 测试 | ≥40 | 全部 | 100% |
| GMP 合规测试 | ≥50 | 全部 | 100% |

### 5.2 GMP 合规验证

| 验证项 | 目标 | 优先级 |
|--------|------|--------|
| 电子签名合规 | 21 CFR Part 11 | P0 |
| 审计链完整性 | 100% | P0 |
| ALCOA+ 支撑 | 5/5 | P0 |
| HSM/KMS 安全 | 密钥安全 | P0 |

### 5.3 稳定性测试

| 测试 | 目标 | 优先级 |
|------|------|--------|
| `long_run_stability_test` | 48h 无错误 | P0 |
| `crash_recovery_test` | 全部 PASS | P0 |
| `concurrency_stress_test` | 200 并发无错误 | P0 |

### 5.4 覆盖率目标

| 阶段 | 日期 | 总体 | parser | executor | planner | optimizer |
|-------|------|---------|--------|----------|---------|-----------|
| GA | 2027-02-15 | **≥90%** | **85%** | **85%** | **80%** | **75%** |

---

## 六、GMP 场景测试

### 6.1 GMP 合规测试矩阵

| 测试类别 | 测试数 | 覆盖的 GMP 要求 |
|----------|--------|-----------------|
| 数字签名 | 10 | Data Integrity, Non-Repudiation |
| 电子签名 | 8 | Electronic Signature, Accountability |
| Immutable Record | 5 | Data Integrity, Auditability |
| Correction Chain | 5 | Traceability, Accountability |
| Provenance | 6 | Traceability, +C Complete |
| Timestamp | 4 | Contemporaneous |
| HSM/KMS | 6 | Security, Key Management |
| Workflow | 6 | Accountability, Four Eyes |
| **总计** | **50** | |

### 6.2 GMP 审计链测试

```
审计链测试场景:
├── 正常写入 → 签名链完整
├── 崩溃恢复 → 签名链无损坏
├── 并发写入 → 签名链无丢失
├── 恶意篡改 → 签名链验证失败
└── 过期签名 → 签名链拒绝
```

### 6.3 21 CFR Part 11 测试场景

| 场景 | 测试 | 验证点 |
|------|------|--------|
| 电子签名创建 | `signature_creation_test` | 签名要素完整 |
| 签署理由 | `signature_reason_test` | 理由必填 |
| 时间戳 | `signature_timestamp_test` | RFC3161 |
| 双人复核 | `four_eyes_test` | 审批完整 |
| 签名验证 | `signature_verification_test` | 验证通过 |
| 不可抵赖 | `non_repudiation_test` | 签名有效 |

---

## 七、测试数据准备

### 7.1 测试数据库

| 数据库 | 用途 | 数据量 |
|--------|------|--------|
| `sqlrustgo_gmp_test` | GMP 合规测试 | 1M 行 |
| `sqlrustgo_tpch_sf1` | TPC-H SF=1 | 1GB |
| `sqlrustgo_tpch_sf10` | TPC-H SF=10 | 10GB |
| `sqlrustgo_perf_test` | QPS 基准 | 10M 行 |
| `sqlrustgo_stress_test` | 并发压力 | 100 并发 |

### 7.2 测试 fixtures

```
tests/fixtures/
├── schema/
│   ├── gmp_schema.sql         # GMP 测试模式
│   ├── tpch_schema.sql        # TPC-H 模式
│   └── performance_schema.sql # PS 模式
├── data/
│   ├── gmp_test_data.csv      # GMP 测试数据
│   ├── tpch_sf1/             # TPC-H SF=1 数据
│   └── tpch_sf10/            # TPC-H SF=10 数据
├── signatures/
│   ├── test_certs/           # 测试证书
│   └── test_keys/            # 测试密钥
└── expected/
    ├── signature_expected.json
    └── audit_chain_expected.json
```

### 7.3 测试密钥管理

```
测试密钥层次:
├── 软件模拟密钥 (开发测试)
├── TPM 模拟密钥 (CI/CD)
└── HSM 模拟密钥 (集成测试)
```

---

## 八、测试工具

### 8.1 标准测试工具

| 工具 | 用途 | 版本 |
|------|------|------|
| `cargo test` | 单元/集成测试 | stable |
| `cargo llvm-cov` | 覆盖率分析 | latest |
| `cargo clippy` | 代码质量 | stable |
| `cargo fmt` | 代码格式 | stable |
| `cargo audit` | 安全扫描 | latest |

### 8.2 高级 QA 工具

#### Fuzzing

| 工具 | 目标 | 集成阶段 |
|------|------|----------|
| `cargo-fuzz` | SQL Parser | Alpha |
| `cargo-fuzz` | SQL Executor | Beta |
| `cargo-fuzz` | Storage Engine | RC |

#### Mutation Testing

| 工具 | 目标 | 阈值 |
|------|------|------|
| `cargo-mutants` | 核心算法 | >70% |

#### SQLancer

| 工具 | 策略 | 目标 |
|------|------|------|
| `sqlancer` | PQS/NoREC/TPE | P0 bugs |

### 8.3 性能测试工具

| 工具 | 用途 | 目标 |
|------|------|------|
| `sysbench` | QPS 基准 | Point SELECT ≥1M |
| `tpch-kit` | TPC-H 基准 | SF=10 22/22 |
| `perf` | 性能分析 | CPU/Memory |
| `valgrind` | 内存分析 | 无泄漏 |

### 8.4 混沌测试工具

| 工具 | 目标 | 集成阶段 |
|------|------|----------|
| `pumba` | 网络混沌 | RC |
| `toxiproxy` | 延迟注入 | RC |
| chaos testing | 崩溃恢复 | RC |

---

## 九、测试覆盖率目标

### 9.1 覆盖率里程碑

| 阶段 | 日期 | 总体 | parser | executor | planner | optimizer | storage | transaction |
|-------|------|---------|--------|----------|---------|-----------|---------|-------------|
| Alpha | 2026-10-01 | ≥50% | 70% | 60% | 55% | 45% | 60% | 65% |
| Beta | 2026-12-01 | ≥65% | 75% | 70% | 65% | 55% | 70% | 70% |
| RC | 2027-01-15 | ≥75% | 80% | 75% | 70% | 65% | 75% | 75% |
| GA | 2027-02-15 | **≥85%** | **85%** | **85%** | **80%** | **75%** | **80%** | **80%** |

### 9.2 覆盖率驱动策略

1. **L0 (单元测试)**: 每个 PR 必须提升覆盖率
2. **L1 (集成测试)**: 每个功能必须配套集成测试
3. **L2 (E2E 测试)**: 关键路径 E2E 覆盖
4. **L3 (混沌测试)**: 生产环境模拟

### 9.3 GMP 模块覆盖率目标

| 模块 | GA 目标 | 说明 |
|------|---------|------|
| `sqlrustgo-gmp` | ≥95% | GMP 合规核心 |
| `sqlrustgo-signature` | ≥95% | 签名模块 |
| `sqlrustgo-audit` | ≥90% | 审计模块 |
| `sqlrustgo-hsm` | ≥85% | HSM 集成 |

---

## 十、测试执行

### 10.1 本地测试

```bash
# 完整测试套件
cargo test --all-features

# 覆盖率报告
cargo llvm-cov --all-features --html
open target/llvm-cov/html/index.html

# GMP 合规测试
cargo test --test gmp_signature_test
cargo test --test gmp_electronic_signature_test

# TPC-H 测试
cargo test --test tpch_sf10_test

# QPS 基准
cargo test --test qps_benchmark_test -- --ignored
```

### 10.2 CI/CD 门禁

```bash
# Alpha Gate (M1~M3 结束)
bash scripts/gate/check_alpha_v320.sh

# Beta Gate (M4~M6 结束)
bash scripts/gate/check_beta_v320.sh

# RC Gate (M7~M8 结束)
bash scripts/gate/check_rc_v320.sh

# GA Gate
bash scripts/gate/check_ga_v320.sh
```

### 10.3 门禁检查清单

#### Alpha Gate

| ID | 检查项 | 通过标准 |
|----|--------|----------|
| A1 | Release Build | cargo build --release |
| A2 | 单元测试 ≥60% | cargo test 90% 通过 |
| A3 | Clippy | cargo clippy -D warnings |
| A4 | Format | cargo fmt --check |
| A5 | 覆盖率 ≥50% | cargo llvm-cov ≥50% |
| A6 | 安全扫描 | cargo audit |
| A7 | GMP 签名测试 | ≥70% |

#### Beta Gate

| ID | 检查项 | 通过标准 |
|----|--------|----------|
| B1 | Release Build | cargo build --release |
| B2 | 单元测试 ≥70% | cargo test 95% 通过 |
| B3 | Clippy | cargo clippy -D warnings |
| B4 | Format | cargo fmt --check |
| B5 | 覆盖率 ≥65% | cargo llvm-cov ≥65% |
| B6 | 安全扫描 | cargo audit |
| B7 | TPC-H SF=1 | 22/22 |
| B8 | GMP 电子签名 | ≥80% |

#### RC Gate

| ID | 检查项 | 通过标准 |
|----|--------|----------|
| R1 | Release Build | cargo build --release |
| R2 | 测试 100% | cargo test 0 failures |
| R3 | Clippy | cargo clippy -D warnings |
| R4 | Format | cargo fmt --check |
| R5 | 覆盖率 ≥75% | cargo llvm-cov ≥75% |
| R6 | 安全扫描 | cargo audit |
| R7 | TPC-H SF=10 | 22/22 |
| R8 | QPS 基准 | 全部达标 |
| R9 | GMP 合规 | 全部通过 |

#### GA Gate

| ID | 检查项 | 通过标准 |
|----|--------|----------|
| GA-1 | Release Build | cargo build --release --workspace |
| GA-2 | 测试 100% | cargo test --all-features 0 failures |
| GA-3 | Integration tests | bash scripts/test/run_integration.sh |
| GA-4 | Clippy | cargo clippy --all-features -D warnings |
| GA-5 | Format | cargo fmt --all --check |
| GA-6 | 覆盖率 ≥85% | cargo llvm-cov ≥85% |
| GA-7 | 安全扫描 | cargo audit |
| GA-8 | TPC-H SF=10 | 22/22 |
| GA-9 | QPS 基准 | 全部 ≥目标值 |
| GA-10 | Formal proofs | ≥30 |
| GA-11 | GMP 合规验证 | 全部通过 |
| GA-12 | OO 文档 | 8/8 存在 |

---

## 十一、测试基础设施

### 11.1 测试环境

| 环境 | 用途 | 配置 |
|------|------|------|
| 本地开发 | PR 测试 | 8 核 16GB |
| CI Runner | 自动测试 | 16 核 32GB |
| 性能测试机 | QPS/TPC-H | 32 核 64GB |

### 11.2 测试监控

| 指标 | 告警阈值 |
|------|----------|
| 测试失败率 | >1% |
| 覆盖率下降 | >5% |
| QPS 下降 | >10% |
| 内存增长 | >20% |

---

*本文档由 hermes-agent 维护*
*版本 1.0 - 2026-05-15*
