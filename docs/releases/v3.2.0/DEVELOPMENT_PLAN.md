# SQLRustGo v3.2.0 开发计划

> **版本**: v1.0
> **日期**: 2026-05-15
> **状态**: 规划中
> **基于**: v3.1.0 GA
> **策略定位**: Trusted GMP Data Platform

---

## 一、版本概述

### 1.1 战略定位

| 版本 | 定位 | 核心能力 |
|------|------|----------|
| **v3.0.0** | 功能可用 | SQL 基础、事务、基础存储 |
| **v3.1.0** | 工业级可信 OLTP 内核 | SSI + MVCC、WAL 审计链、AES-256、TLA+ 验证 |
| **v3.2.0** | GMP Native 可信数据平台 | 数字签名审计链、电子签名、EBR、工作流、HSM |

### 1.2 核心转变

```
v3.2.0 = 从 "MySQL 替代品" → "GMP Native 可信数据平台"
```

### 1.3 v3.2.0 成功定义

```
一个数据库:
1. 数据不可伪造 (数字签名)
2. 操作可追溯 (Provenance)
3. 签名不可抵赖 (电子签名)
4. 流程受控 (Workflow)
5. 时间可信 (Timestamp)
6. 密钥安全 (HSM)

= SQLRustGo v3.2.0 "Trusted GMP Data Platform"
```

---

## 二、问题→开发任务映射

### 2.1 GMP 合规任务 (GMP-1~12)

| 任务 ID | Issue | 功能 | 优先级 | 工作量 |
|---------|-------|------|--------|--------|
| GMP-1 | #900 | 数字签名审计链 | P0 | 3 周 |
| GMP-2 | #901 | 电子签名 (21 CFR Part 11) | P0 | 3 周 |
| GMP-3 | #902 | Immutable Record / EBR | P0 | 2 周 |
| GMP-4 | #903 | Correction Chain | P0 | 2 周 |
| GMP-5 | #904 | Provenance Tracking | P0 | 2 周 |
| GMP-6 | #905 | Trusted Timestamp (RFC3161) | P0 | 1 周 |
| GMP-7 | #906 | 审计链验证工具 | P0 | 1 周 |
| GMP-8 | #907 | HSM/KMS 集成 (TPM/HSM/KMS) | P0 | 3 周 |
| GMP-9 | #908 | GMP Workflow Engine | P1 | 3 周 |
| GMP-10 | #909 | 移动端可信采集 | P1 | 2 周 |
| GMP-11 | #910 | SOP/培训绑定 | P1 | 2 周 |
| GMP-12 | #911 | Device Calibration | P1 | 1 周 |

### 2.2 性能任务 (PERF-1~5)

| 任务 ID | Issue | 功能 | 目标 | 工作量 |
|---------|-------|------|------|--------|
| PERF-1 | #920 | Point SELECT QPS | ≥1M ops/s | 2 周 |
| PERF-2 | #921 | TPC-H SF=10 | 22/22 无 OOM | 3 周 |
| PERF-3 | #922 | 并发增强 | 200+ 连接 | 1 周 |
| PERF-4 | #923 | 死锁检测优化 | <50ms | 1 周 |
| PERF-5 | #924 | 内存优化 | -15% 占用 | 2 周 |

### 2.3 SQL 任务 (SQL-1~8)

| 任务 ID | Issue | 功能 | 目标 | 工作量 |
|---------|-------|------|------|--------|
| SQL-1 | #930 | RECURSIVE CTE | 完整支持 | 2 周 |
| SQL-2 | #931 | Performance Schema | ≥60% | 2 周 |
| SQL-3 | #932 | 冷存储集成 | S3/OSS | 3 周 |
| SQL-4 | #933 | 组复制 | v3.3.0 | - |
| SQL-5 | #934 | 自动故障转移 | v3.3.0 | - |
| SQL-6 | #935 | 地理分布 | v3.3.0 | - |
| SQL-7 | #936 | DCL 权限链完善 | 100% | 1 周 |
| SQL-8 | #937 | FULLTEXT 完善 | 中英文 | 1 周 |

### 2.4 OO 文档任务 (OO-1~8)

| 任务 ID | Issue | 文档 | 优先级 | 工作量 |
|---------|-------|------|--------|--------|
| OO-1 | #940 | 数字签名审计链设计 | P0 | 1 周 |
| OO-2 | #941 | 电子签名设计 | P0 | 1 周 |
| OO-3 | #942 | Immutable Record 设计 | P0 | 1 周 |
| OO-4 | #943 | Correction Chain 设计 | P0 | 1 周 |
| OO-5 | #944 | Provenance Tracking 设计 | P0 | 1 周 |
| OO-6 | #945 | HSM/KMS 集成设计 | P0 | 1 周 |
| OO-7 | #946 | GMP Workflow Engine 设计 | P1 | 1 周 |
| OO-8 | #947 | Trusted Timestamp 设计 | P1 | 1 周 |

### 2.5 任务汇总

| 类别 | P0 | P1 | P2 | 总计 |
|------|----|----|-----|------|
| GMP | 8 | 4 | 0 | 12 |
| 性能 | 3 | 2 | 0 | 5 |
| SQL | 2 | 4 | 2 | 8 |
| OO 文档 | 6 | 2 | 0 | 8 |
| **总计** | **19** | **12** | **2** | **33** |

---

## 三、开发里程碑 (M1~M8, RC1, GA)

### 3.1 20 周里程碑计划

```
v3.2.0-alpha  ────────────────────────────────────────────────────────── 2026-10-01
  ├── M1: GMP 基础框架 (GMP-1~3 核心签名)
  ├── M2: Immutable Record + Correction Chain
  ├── M3: Provenance Tracking + Timestamp
  └── Alpha Gate: 基础编译 + 单元测试 ≥60%

v3.2.0-beta   ──────────────────────────────────────────────────────────── 2026-12-01
  ├── M4: HSM/KMS 集成
  ├── M5: 电子签名 + 审计链验证工具
  ├── M6: Performance Schema 完善
  └── Beta Gate: 集成测试 + 稳定性测试

v3.2.0-rc      ──────────────────────────────────────────────────────────── 2027-01-15
  ├── M7: QPS 优化 + 内存管理
  ├── M8: RECURSIVE CTE + 冷存储
  ├── RC1: TPC-H SF=10 通过
  └── RC Gate: 全面测试 + 性能基准

v3.2.0-ga      ──────────────────────────────────────────────────────────── 2027-02-15
  ├── GA Gate: 23/23 PASS
  ├── Formal proofs ≥30
  ├── GMP 合规验证
  └── 综合评分 ≥80/100
```

### 3.2 详细里程碑

#### M1: GMP 基础框架 (第 1-3 周)

| 任务 | 交付物 | 验收条件 |
|------|--------|----------|
| GMP-1 | 数字签名审计链 | sign/verify API 完成 |
| GMP-6 | Trusted Timestamp | RFC3161 集成 |
| OO-1 | 签名链设计文档 | 文档完成 |

#### M2: Immutable Record + Correction Chain (第 4-5 周)

| 任务 | 交付物 | 验收条件 |
|------|--------|----------|
| GMP-3 | Immutable Record | CREATE TABLE IMMUTABLE |
| GMP-4 | Correction Chain | CORRECT RECORD 语句 |
| OO-3 | EBR 设计文档 | 文档完成 |
| OO-4 | Correction 设计文档 | 文档完成 |

#### M3: Provenance Tracking (第 6-7 周)

| 任务 | 交付物 | 验收条件 |
|------|--------|----------|
| GMP-5 | Provenance Tracking | 完整溯源链 |
| GMP-7 | 审计链验证工具 | 工具完成 |
| OO-5 | Provenance 设计文档 | 文档完成 |

#### M4: HSM/KMS 集成 (第 8-10 周)

| 任务 | 交付物 | 验收条件 |
|------|--------|----------|
| GMP-8 | HSM/KMS 集成 | TPM/HSM/KMS 支持 |
| OO-6 | HSM 集成设计文档 | 文档完成 |

#### M5: 电子签名 + 审计链验证 (第 11-12 周)

| 任务 | 交付物 | 验收条件 |
|------|--------|----------|
| GMP-2 | 电子签名 | 21 CFR Part 11 |
| OO-2 | 电子签名设计文档 | 文档完成 |

#### M6: Performance Schema 完善 (第 13-14 周)

| 任务 | 交付物 | 验收条件 |
|------|--------|----------|
| SQL-2 | Performance Schema | ≥60% 覆盖率 |

#### M7: QPS 优化 + 内存管理 (第 15-17 周)

| 任务 | 交付物 | 验收条件 |
|------|--------|----------|
| PERF-1 | Point SELECT QPS | ≥1M ops/s |
| PERF-2 | TPC-H SF=10 | 22/22 通过 |
| PERF-5 | 内存优化 | -15% 占用 |

#### M8: RECURSIVE CTE + 冷存储 (第 18-19 周)

| 任务 | 交付物 | 验收条件 |
|------|--------|----------|
| SQL-1 | RECURSIVE CTE | 完整支持 |
| SQL-3 | 冷存储集成 | S3/OSS |

---

## 四、测试计划

### 4.1 Alpha 阶段测试 (2026-10-01)

| 测试 | 目标 | 覆盖率 |
|------|------|--------|
| 签名审计链单元 | GMP-1 | ≥70% |
| Immutable Record | GMP-3 | ≥80% |
| Correction Chain | GMP-4 | ≥80% |
| Provenance Tracking | GMP-5 | ≥70% |

### 4.2 Beta 阶段测试 (2026-12-01)

| 测试 | 目标 | 覆盖率 |
|------|------|--------|
| 电子签名集成 | GMP-2 | ≥85% |
| HSM/KMS 集成 | GMP-8 | ≥80% |
| Performance Schema | SQL-2 | ≥60% |
| TPC-H SF=1 | PERF-2 | 22/22 |

### 4.3 RC 阶段测试 (2027-01-15)

| 测试 | 目标 | 覆盖率 |
|------|------|--------|
| GMP 合规测试 | 全模块 | ≥85% |
| TPC-H SF=10 | PERF-2 | 22/22 |
| QPS 基准 | PERF-1 | 全部通过 |
| 压力测试 | PERF-3 | 200 并发 |

### 4.4 GA 门禁

| Gate | 检查项 | 通过标准 |
|------|--------|----------|
| GA-1 | Release Build | cargo build --release --workspace |
| GA-2 | 测试 100% | cargo test --all-features 0 failures |
| GA-3 | Clippy | cargo clippy --all-features -- -D warnings |
| GA-4 | Format | cargo fmt --all -- --check |
| GA-5 | 覆盖率 ≥80% | cargo llvm-cov --all-features --lib ≥80% |
| GA-6 | 安全扫描 | cargo audit |
| GA-7 | GMP 合规 | 电子签名 + 审计链验证 |
| GA-8 | TPC-H SF=10 | 22/22 |
| GA-9 | QPS 基准 | 全部 ≥目标值 |
| GA-10 | Formal proofs | ≥30 个 |

---

## 五、OO 文档计划

### 5.1 OO 文档交付物

| 任务 ID | 文档 | 优先级 | 交付周 |
|---------|------|--------|--------|
| OO-1 | 数字签名审计链设计 | P0 | M1 |
| OO-2 | 电子签名设计 | P0 | M5 |
| OO-3 | Immutable Record 设计 | P0 | M2 |
| OO-4 | Correction Chain 设计 | P0 | M2 |
| OO-5 | Provenance Tracking 设计 | P0 | M3 |
| OO-6 | HSM/KMS 集成设计 | P0 | M4 |
| OO-7 | GMP Workflow Engine 设计 | P1 | M5 |
| OO-8 | Trusted Timestamp 设计 | P1 | M1 |

### 5.2 OO 文档结构

```
docs/releases/v3.2.0/oo/
├── GMP/
│   ├── DIGITAL_SIGNATURE_CHAIN.md      # OO-1
│   ├── ELECTRONIC_SIGNATURE.md         # OO-2
│   ├── IMMUTABLE_RECORD.md             # OO-3
│   ├── CORRECTION_CHAIN.md              # OO-4
│   ├── PROVENANCE_TRACKING.md           # OO-5
│   ├── HSM_KMS_INTEGRATION.md          # OO-6
│   ├── GMP_WORKFLOW_ENGINE.md          # OO-7
│   └── TRUSTED_TIMESTAMP.md            # OO-8
└── README.md                           # 索引
```

---

## 六、资源估算

### 6.1 人力估算

| 类别 | 任务数 | 工作量 | 优先级 |
|------|--------|--------|--------|
| GMP 合规 | 12 | 18 周 | P0/P1 |
| 性能优化 | 5 | 9 周 | P0/P1 |
| SQL 功能 | 8 | 9 周 | P1/P2 |
| OO 文档 | 8 | 8 周 | P0/P1 |
| **总计** | **33** | **44 周** | |

### 6.2 人月估算

假设团队 4 人并行开发：

- GMP 模块: 18 周 / 4 人 = 4.5 人月
- 性能模块: 9 周 / 4 人 = 2.25 人月
- SQL 模块: 9 周 / 4 人 = 2.25 人月
- OO 文档: 8 周 / 2 人 = 4 人月

**总计**: ~13 人月

### 6.3 基础设施

| 资源 | 需求 |
|------|------|
| CI runner | 2x 8核 32GB |
| 性能测试机 | 2x 16核 64GB |
| 存储 | 500GB NVMe |
| HSM 模拟器 | 软件 TPM |

---

## 七、风险管理

### 7.1 高风险项

| 风险 | 影响 | 概率 | 缓解措施 |
|------|------|------|----------|
| HSM 集成复杂度超预期 | 高 | 中 | 预留 1 周缓冲，先做软件模拟 |
| TPC-H SF=10 OOM | 高 | 中 | 分阶段优化内存管理 |
| 电子签名合规验证 | 高 | 低 | 提前与合规团队沟通 |
| QPS 优化效果不达预期 | 中 | 中 | 多方案并行验证 |

### 7.2 中风险项

| 风险 | 影响 | 概率 | 缓解措施 |
|------|------|------|----------|
| 冷存储集成延迟 | 中 | 中 | 使用 mock 先完成功能 |
| RECURSIVE CTE 复杂度 | 中 | 低 | 参考 PostgreSQL 实现 |
| 人员不足 | 中 | 中 | 优先级排序 + 迭代交付 |

### 7.3 缓冲策略

- 每个 P0 任务预留 10% 缓冲时间
- Beta 前预留 2 周缓冲
- RC 后预留 1 周缓冲

---

## 八、验收标准

### 8.1 功能验收

- [ ] GMP-1: 数字签名审计链 sign/verify API
- [ ] GMP-2: 电子签名 21 CFR Part 11 合规
- [ ] GMP-3: Immutable Record 禁止 DML
- [ ] GMP-4: Correction Chain 完整修正链
- [ ] GMP-5: Provenance Tracking 完整溯源
- [ ] GMP-6: Trusted Timestamp RFC3161
- [ ] GMP-7: 审计链验证工具
- [ ] GMP-8: HSM/KMS 集成 (TPM/HSM/KMS)
- [ ] PERF-1: Point SELECT QPS ≥1M
- [ ] PERF-2: TPC-H SF=10 22/22
- [ ] SQL-1: RECURSIVE CTE 完整支持
- [ ] SQL-2: Performance Schema ≥60%

### 8.2 质量验收

- [ ] GA Gate 23/23 PASS
- [ ] 测试覆盖率 ≥80%
- [ ] Clippy 零警告
- [ ] Formal proofs ≥30 个
- [ ] GMP 合规验证通过

### 8.3 性能验收

| 指标 | 目标 |
|------|------|
| Point SELECT QPS | ≥1,000,000 ops/s |
| Complex WHERE QPS | ≥500,000 ops/s |
| INSERT QPS | ≥800,000 ops/s |
| UPDATE QPS | ≥800,000 ops/s |
| DELETE QPS | ≥800,000 ops/s |
| TPC-H SF=10 | 22/22 通过 |
| 内存占用 | ≤85% (较 v3.1.0 -15%) |
| 死锁检测延迟 | <50ms |

### 8.4 MySQL 兼容性目标

| 维度 | v3.2.0 目标 |
|------|-------------|
| SQL 语言 | 90/100 |
| 存储引擎 | 80/100 |
| 可观测性 | 75/100 |
| 安全 | 90/100 |
| 高可用 | 75/100 |
| **总体** | **≥80/100** |

---

*本文档由 hermes-agent 创建*
*版本 1.0 - 2026-05-15*
