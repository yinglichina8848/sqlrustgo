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

---

## 九、架构决策记录 (ADR)

### 9.1 核心架构决策

| ID | 决策 | 状态 | 日期 |
|----|------|------|------|
| ADR-001 | GMP 模块独立为 `sqlrustgo-gmp` crate | ✅ 已采纳 | 2026-04 |
| ADR-002 | 数字签名使用 k256 ECDSA (P-256) | ✅ 已采纳 | 2026-04 |
| ADR-003 | 审计链采用 SHA-256 哈希链 | ✅ 已采纳 | 2026-04 |
| ADR-004 | 冷存储四层架构 (Hot/Warm/Cold/Archive) | ✅ 已采纳 | 2026-05 |
| ADR-005 | SSI + MVCC 混合并发控制 | ✅ 已采纳 | 2026-03 |
| ADR-006 | TPC-H Spill Framework 磁盘排序 | ✅ 已采纳 | 2026-05 |
| ADR-007 | Workflow Engine 状态机 DSL | ✅ 已采纳 | 2026-04 |
| ADR-008 | HSM 三层抽象 (TPM/HSM/KMS) | ✅ 已采纳 | 2026-04 |

### 9.2 已否决决策

| ID | 决策 | 否决原因 | 日期 |
|----|------|----------|------|
| N/A | 使用 ed25519 签名 | 与现有 PKI 不兼容 | 2026-04 |
| N/A | 纯 MVCC 无 SSI | 写偏序死锁风险 | 2026-03 |

### 9.3 待定决策

| ID | 决策 | 选项 | 状态 |
|----|------|------|------|
| ADR-009 | RECURSIVE CTE 执行策略 | 迭代 vs 图遍历 | 进行中 |
| ADR-010 | 冷存储 S3 签名算法 | v4 vs v4a | 已定 v4 |

---

## 十、API 约定

### 10.1 公开 API 层级

| 层级 | 模块 | 稳定性 |
|------|------|--------|
| **Stable** | `executor`, `storage`, `transaction` | 语义版本锁定 |
| **Beta** | `gmp`, `optimizer`, `planner` | 可能有破坏性变更 |
| **Alpha** | `vector`, `graph`, `network` | 随时变更 |

### 10.2 GMP API 约定

```rust
// 数字签名
pub trait AuditChain {
    fn sign(&self, record: &AuditRecord) -> Result<Signature, GmpError>;
    fn verify(&self, record: &AuditRecord) -> Result<bool, GmpError>;
}

// 电子签名
pub trait ElectronicSignatureProvider {
    fn sign(&self, user: &str, data: &[u8], reason: &str) -> Result<SignatureRecord, GmpError>;
    fn verify(&self, record: &SignatureRecord) -> Result<bool, GmpError>;
}

// HSM 抽象
pub trait HSMProvider: Send + Sync {
    fn sign(&self, key_id: &str, data: &[u8]) -> Result<Vec<u8>, HsmError>;
    fn verify(&self, key_id: &str, data: &[u8], sig: &[u8]) -> Result<bool, HsmError>;
}
```

### 10.3 变更管理

- **Stable API**: 不得破坏，deprecation 需 2 个 minor 版本预告
- **Beta API**: 可破坏，需在 CHANGELOG 标注
- **Alpha API**: 随时破坏，不另行通知

---

## 十一、迁移指南 (从 v3.1.0)

### 11.1 破坏性变更

| 变更类型 | 影响 | 迁移操作 |
|----------|------|----------|
| GMP 模块重命名 | `sqlrustgo-gmp` crate | `Cargo.toml` 中替换依赖 |
| `AuditChain` API | `verify()` 返回 `Result<bool>` | 添加错误处理 |
| `ImmutableRecord` 行为 | DML 默认拒绝 | 检查现有表是否有 IMMUTABLE 标记 |

### 11.2 新增配置项

```toml
[gmp]
enable = true                    # 新增，默认 false
signature_algorithm = "ecdsa"    # 新增
hsm_provider = "software"       # 新增，支持: software/tpm/pkcs11

[storage]
tiering_enabled = true          # 新增，冷存储分层
tier_manager = "storage_tier"   # 新增
```

### 11.3 数据迁移

```sql
-- v3.2.0 新增 GMP 系统表（自动创建）
-- 无需手动迁移
-- 现有数据完整性不受影响
```

### 11.4 兼容性矩阵

| 版本组合 | 兼容？ | 说明 |
|----------|--------|------|
| v3.1.0 client → v3.2.0 server | ✅ | MySQL 协议兼容 |
| v3.2.0 client → v3.1.0 server | ⚠️ | GMP 功能不可用 |
| v3.1.0 数据文件 → v3.2.0 | ✅ | WAL 前向兼容 |

---

## 十二、取消/延期项

### 12.1 已取消 (Cancelled)

| 任务 | 原计划 | 取消原因 | 替代方案 |
|------|--------|----------|----------|
| SQL-4 组复制 | v3.2.0 | 复杂度超标，延期至 v3.3.0 | 手动故障转移脚本 |
| SQL-5 自动故障转移 | v3.2.0 | 依赖组复制，延期至 v3.3.0 | 运维文档 |
| SQL-6 地理分布 | v3.2.0 | 延期至 v3.4.0 | 规划设计档 |

### 12.2 延期至下一版本

| 任务 | 原计划 | 延期至 | 延期原因 |
|------|--------|--------|----------|
| PERF-1 QPS ≥1M | v3.2.0 RC | v3.3.0 | SIMD 优化周期超过预期 |
| PERF-2 TPC-H SF=10 | v3.2.0 RC | v3.3.0 | Spill Framework 需完善 |
| SQL-8 FULLTEXT | v3.2.0 | v3.3.0 | 资源有限，优先级调整 |
| PERF-4 死锁检测 <50ms | v3.2.0 | v3.2.1 | 已达 <100ms，需进一步优化 |

### 12.3 范围缩减 (Scope Reduction)

| 原范围 | 缩减后 | 原因 |
|--------|--------|------|
| RECURSIVE CTE 完整支持 | 已完成 | 无缩减 |
| Performance Schema ≥60% | 已完成 | 无缩减 |
| DCL 权限链 100% | 已完成 (PR #1090) | 无缩减 |

---

## 十三、延续任务 (从 v3.1.0 继承)

### 13.1 来自 v3.1.0 的未完成任务

| Issue | 任务 | 优先级 | 状态 |
|-------|------|--------|------|
| #801 | MVCC GC 优化 | P1 | 进行中 |
| #802 | WAL 压缩 | P2 | 未开始 |
| #803 | Buffer Pool LRU 优化 | P1 | 进行中 |

### 13.2 跨版本技术债务

| 债务项 | 影响 | 计划修复版本 |
|--------|------|--------------|
| 旧 Query Planner 替换 | 性能 | v3.3.0 |
| Graph Store v2 API | 功能 | v3.3.0 |
| Vector Index 内存管理 | 稳定性 | v3.2.1 |

### 13.3 版本间依赖关系

```
v3.1.0 ─── MVCC GC ──────────────────────┐
    └── WAL 压缩 ──────────────────────────→ v3.3.0 性能基线
                                            ↑
v3.2.0 ─── PERF-1/2 延期 ────────────────┘
    └── HSM 集成 ✅ ──── GMP 合规 ✅
```

---

## 十四、验收标准

### 14.1 功能验收（已交付状态）

| 任务 | 计划目标 | CHANGELOG 记录 | 实际状态 |
|------|----------|----------------|----------|
| GMP-1 数字签名审计链 | P0 | ✅ PR #1012 | ✅ 已交付 |
| GMP-2 电子签名 | P0 | ✅ PR #1004/1015/1017/1018 | ✅ 已交付 |
| GMP-3 Immutable Record | P0 | ✅ PR #1029 | ✅ 已交付 |
| GMP-4 Correction Chain | P0 | ✅ PR #1027 | ✅ 已交付 |
| GMP-5 Provenance Tracking | P0 | ✅ PR #1024 | ✅ 已交付 |
| GMP-6 Trusted Timestamp | P0 | ✅ PR #1017 | ✅ 已交付 |
| GMP-7 审计链验证工具 | P0 | ✅ PR #1020 | ✅ 已交付 |
| GMP-8 HSM/KMS 集成 | P0 | ✅ PR #1025 | ✅ 已交付 |
| GMP-9 Workflow Engine | P1 | ✅ PR #1046 | ✅ 已交付 |
| GMP-10 移动端可信采集 | P1 | ⚠️ 代码存在 | ⚠️ CHANGELOG 遗漏 |
| GMP-11 SOP/培训绑定 | P1 | ⚠️ 代码存在 | ⚠️ CHANGELOG 遗漏 |
| GMP-12 Device Calibration | P1 | ⚠️ 代码存在 | ⚠️ CHANGELOG 遗漏 |
| PERF-1 QPS ≥1M | P0 | ⚠️ MySQL Flush 优化 | 🔄 延期至 v3.3.0 |
| PERF-2 TPC-H SF=10 | P0 | ⚠️ Spill Framework | 🔄 延期至 v3.3.0 |
| PERF-3 200+ 并发 | P1 | ✅ PR #1013 | ✅ 已交付 |
| PERF-4 死锁检测 <50ms | P1 | ✅ PR #1043 | ⚠️ 达 <100ms |
| PERF-5 内存优化 -15% | P1 | ✅ PR #1045 | ✅ 已交付 |
| SQL-1 RECURSIVE CTE | P1 | ✅ PR #1065 | ✅ 已交付 |
| SQL-2 Performance Schema | P1 | ✅ PR #1071 | ✅ 已交付 |
| SQL-3 冷存储集成 | P1 | ✅ PR #1091/1093 | ✅ 已交付 |
| SQL-7 DCL 权限链 | P1 | ✅ PR #1090 | ✅ 已交付 |
| Multi-Table DML | M6 | ✅ PR #1021 | ✅ 已交付（超范围） |

### 14.2 质量验收

- [ ] GA Gate 23/23 PASS
- [ ] 测试覆盖率 ≥80%
- [ ] Clippy 零警告
- [ ] Formal proofs ≥30 个
- [ ] GMP 合规验证通过

### 14.3 性能验收

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

### 14.4 MySQL 兼容性目标

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
