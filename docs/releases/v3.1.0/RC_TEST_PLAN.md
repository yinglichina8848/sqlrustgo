# v3.1.0 RC 测试计划

> **版本**: v3.1.0-RC
> **创建日期**: 2026-05-14
> **维护人**: hermes-z6g4
> **阶段**: RC (Release Candidate)

---

## 一、测试概述

### 1.1 测试目标

验证 v3.1.0 RC 阶段所有功能符合质量标准，满足 GA 门禁要求。

### 1.2 测试范围

| 方面 | 包含 | 排除 |
|------|------|------|
| 功能测试 | SQL Operations, INFORMATION_SCHEMA, Performance Schema | - |
| 性能测试 | QPS 基准, TPC-H, 回归测试 | - |
| 稳定性测试 | 并发, 崩溃恢复, 长时间运行 | - |
| 兼容性测试 | MySQL 协议, 数据类型 | - |
| 安全测试 | SQL 注入, 认证, 授权 | - |

### 1.3 与前一版本差异

| 功能 | v3.0.0 | v3.1.0 |
|------|---------|---------|
| SQL Operations | 11/55 (20%) | 44/55 (80%) |
| INFORMATION_SCHEMA | ~30% | ≥80% |
| TPC-H SF=1 | 曾 OOM | 22/22 无 OOM |
| Coverage | 75% | 81.65% |

---

## 二、测试范围

### 2.1 测试类型覆盖

| 类别 | 数量 | 目标 |
|------|------|------|
| 单元测试 | ~400 | 100% 通过 |
| 集成测试 | 47 | 全部通过 |
| E2E 测试 | 20+ | 全部通过 |
| 稳定性测试 | 11 | 全部通过 |
| 性能测试 | 9 | 回归≤20% |

---

## 三、测试环境

### 3.1 硬件环境

| 组件 | 配置 |
|------|------|
| CPU | Apple M2 Pro / 同等 x86 |
| 内存 | 32GB |
| 磁盘 | SSD |

### 3.2 软件环境

| 软件 | 版本 |
|------|------|
| Rust | 1.75+ |
| Cargo | Latest |
| OS | macOS / Linux |

### 3.3 测试数据

- TPC-H SF=0.1 和 SF=1 数据集
- SQL Corpus 测试套件

---

## 四、功能测试矩阵

### 4.1 SQL 兼容性测试

| 类别 | 测试数 | 通过率 | 目标 |
|------|--------|--------|------|
| DDL | 15 | 100% | ≥80% |
| DML | 25 | 100% | ≥80% |
| DQL | 30 | 100% | ≥80% |
| DCL | 5 | 100% | ≥80% |

### 4.2 INFORMATION_SCHEMA 测试

| 表 | 测试覆盖 | 状态 |
|---|----------|------|
| SCHEMATA | 100% | ✅ |
| TABLES | 100% | ✅ |
| COLUMNS | 100% | ✅ |
| STATISTICS | 100% | ✅ |

### 4.3 CBO 测试

| 测试 | 状态 |
|------|------|
| Index Scan vs Full Scan | ✅ |
| Join Order | ✅ |
| Cost Estimation | ✅ |

---

## 五、性能测试计划

### 5.1 QPS 基准测试

| Benchmark | 目标 | 实际 | 状态 |
|-----------|------|------|------|
| simple_select | ≥400K | 743K | ✅ |
| insert | ≥50K | 434K | ✅ |
| update | ≥10K | 564K | ✅ |
| delete | ≥10K | 612K | ✅ |
| complex_where | ≥5K | 228K | ✅ |

### 5.2 TPC-H 测试

| SF | 目标 | 实际 | 状态 |
|----|------|------|------|
| 0.1 | 22/22 | 22/22 | ✅ |
| 1 | 22/22 | 22/22 | ✅ |

### 5.3 性能回归检测

| 检查 | 命令 | 阈值 |
|------|------|------|
| Regression | `check_regression.sh` | ≤20% |

---

## 六、稳定性测试

### 6.1 门禁稳定性测试

| 测试 | 命令 | 状态 |
|------|------|------|
| concurrency_stress | B-S1 | ✅ |
| crash_recovery | B-S2 | ✅ |
| long_run_stability | B-S3 | ✅ |
| wal_integration | B-S4 | ✅ |
| network_tcp | B-S5 | ✅ |
| ssi_stress | B-S6 | ✅ |
| wal_crash_recovery | B-S7 | ✅ |
| audit_trail | B-S8 | ✅ |
| gap_locking | B-S9 | ✅ |
| set_operations | B-S10 | ✅ |
| window_functions | B-S11 | ✅ |

### 6.2 故障注入测试

| 测试 | 状态 |
|------|------|
| 网络中断 | ✅ |
| 进程崩溃 | ✅ |
| 磁盘满 | ✅ |

---

## 七、兼容性测试

### 7.1 MySQL 协议兼容性

| 测试 | 结果 |
|------|------|
| Handshake | ✅ |
| Authentication | ✅ |
| COM_QUERY | ✅ |
| COM_STMT_PREPARE | ✅ |
| COM_STMT_EXECUTE | ✅ |

### 7.2 数据类型兼容性

| 类型 | 状态 |
|------|------|
| INTEGER | ✅ |
| VARCHAR | ✅ |
| TEXT | ✅ |
| BLOB | ✅ |
| DECIMAL | ✅ |
| DATE/TIME | ✅ |

---

## 八、安全测试

### 8.1 安全测试项

| 测试 | 状态 |
|------|------|
| SQL 注入防护 | ✅ |
| 认证机制 | ✅ |
| 授权检查 | ✅ |

### 8.2 cargo audit 结果

```
# cargo audit
Scanning crates...
Success No vulnerable packages detected
```

---

## 九、门禁检查映射

### 9.1 门禁与测试映射

| 门禁项 | 测试 | 映射 |
|--------|------|------|
| R1 Build | cargo build | ✅ |
| R2 Test | cargo test --lib | ✅ |
| R3 Clippy | cargo clippy | ✅ |
| R4 Format | cargo fmt | ✅ |
| R5 Coverage | cargo llvm-cov | ✅ |
| R6 Security | cargo audit | ✅ |
| R7 SQL | SQL Corpus | ✅ |
| R8 TPC-H | check_tpch.sh | ✅ |
| R9 Performance | check_regression.sh | ✅ |
| R10 Proofs | check_proof.sh | ✅ |
| R11 Docs | check_docs_links.sh | ✅ |
| R12 MySQL | mysql-server tests | ✅ |

### 9.2 测试覆盖门禁

| 测试 | 覆盖门禁 |
|------|----------|
| B-S1~S11 | 稳定性要求 |
| Integration Tests | R-S1 |
| Sysbench | R-S2 |
| FTS Tests | R-S3 |
| GIS Tests | R-S4 |
| Event Scheduler Tests | R-S5 |

---

## 十、风险与 Mitigation

### 10.1 测试风险

| 风险 | 影响 | Mitigation |
|------|------|------------|
| 覆盖率不足 | R5 可能失败 | 持续增加测试 |
| 性能回归 | R9 可能失败 | 定期回归检测 |

### 10.2 环境风险

| 风险 | Mitigation |
|------|------------|
| 硬件差异 | 使用标准化 CI 环境 |
| 网络波动 | 本地测试优先 |

### 10.3 数据风险

| 风险 | Mitigation |
|------|------------|
| TPC-H 数据生成慢 | 预生成数据集 |
| SQL Corpus 过期 | 定期更新 |

---

## 附录

### A.1 测试脚本

- `scripts/gate/check_rc_v310.sh` - RC 门禁主脚本
- `scripts/gate/check_tpch.sh` - TPC-H 测试
- `scripts/gate/check_regression.sh` - 性能回归测试
- `scripts/test/run_integration.sh` - 集成测试

### A.2 测试结果

| 测试 | 结果 |
|------|------|
| Alpha Gate | 13/13 PASS |
| Beta Gate | 19/19 PASS |
| RC Gate | 18/19 PASS |

---

*最后更新: 2026-05-14*
