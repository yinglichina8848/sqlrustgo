# SQLRustGo v3.2.0 问题汇总

> **版本**: v1.0
> **日期**: 2026-05-15
> **维护人**: hermes-agent
> **目标**: v3.1.0 遗留问题 + GMP 需求 + MySQL 差距汇总

---

## 一、v3.1.0 OO 文档缺失问题

### 1.1 遗留 OO 文档未完成

| Issue | 文档 | 优先级 | 状态 |
|-------|------|--------|------|
| #625 | MVCC 形式化验证 (TLA+) | P1 | ✅ 已完成 |
| #626 | WAL + 审计链集成文档 | P1 | ✅ 已完成 |
| #630 | SSI 死锁检测文档 | P1 | ✅ 已完成 |
| #530 | Event Scheduler 设计文档 | P2 | ✅ 已完成 |
| #877 | DCL 权限链测试 | P1 | ❌ 未完成 |

### 1.2 v3.2.0 OO 文档缺口

| Issue | 文档 | 优先级 | 说明 |
|-------|------|--------|------|
| OO-1 | 数字签名审计链设计 | P0 | GMP 合规核心 |
| OO-2 | 电子签名 (21 CFR Part 11) | P0 | 签署理由 + 时间戳 |
| OO-3 | Immutable Record / EBR | P0 | 禁止 UPDATE/DELETE |
| OO-4 | Correction Chain 设计 | P0 | 数据修正链 |
| OO-5 | Provenance Tracking | P0 | 数据溯源 |
| OO-6 | HSM/KMS 集成设计 | P0 | 密钥管理 |
| OO-7 | GMP Workflow Engine | P1 | 流程受控 |
| OO-8 | Trusted Timestamp 设计 | P1 | RFC3161 |

---

## 二、系统瓶颈遗留 (A-F)

### 2.1 性能瓶颈

| 瓶颈 ID | 描述 | 根因 | v3.1.0 状态 | v3.2.0 目标 |
|---------|------|------|-------------|-------------|
| PERF-A | TPC-H SF=1 OOM | Hash Join 内存分配 | 22/22 通过 | 优化内存管理 |
| PERF-B | Point SELECT QPS | 查询缓存 | 743K | ≥1000K |
| PERF-C | Complex WHERE QPS | 代价模型 | 228K | ≥500K |
| PERF-D | INSERT QPS | WAL 写入 | 434K | ≥800K |
| PERF-E | UPDATE QPS | 锁竞争 | 564K | ≥800K |
| PERF-F | DELETE QPS | 索引维护 | 612K | ≥800K |

### 2.2 并发瓶颈

| 瓶颈 ID | 描述 | v3.1.0 状态 | v3.2.0 目标 |
|---------|------|-------------|-------------|
| PERF-G | 100 并发连接池 | 11/11 通过 | 200 并发 |
| PERF-H | 死锁检测延迟 | <100ms | <50ms |
| PERF-I | SSI 死锁预防 | 7 tests | 全部通过 |

---

## 三、代码级 P0 问题

### 3.1 核心 P0 问题

| Issue | 问题 | 优先级 | 影响 |
|-------|------|--------|------|
| P0-1 | 数字签名审计链缺失 | P0 | GMP/FDA 合规 |
| P0-2 | 电子签名未实现 | P0 | 21 CFR Part 11 |
| P0-3 | Immutable Record 未实现 | P0 | EBR 核心 |
| P0-4 | Correction Chain 未实现 | P0 | 数据完整性 |
| P0-5 | Provenance Tracking 不完整 | P0 | ALCOA+ |
| P0-6 | Trusted Timestamp 未实现 | P0 | 合规要求 |
| P0-7 | 审计链验证工具缺失 | P0 | 审计检查 |
| P0-8 | HSM/KMS 集成未完成 | P0 | 密钥安全 |

### 3.2 P0 问题详解

#### P0-1: 数字签名审计链

**当前状态**: hash(prev_hash || content) - 无签名

**目标状态**: sign(hash(prev_hash || content), user_private_key)

**签名类型**:
- User Signature - 用户操作签名 (ECDSA-P256)
- Device Signature - 设备采集签名
- System Signature - 系统操作签名 (HMAC-SHA256)
- Batch Signature - 批记录签名 (RSA-2048)

#### P0-2: 电子签名

满足 FDA 21 CFR Part 11:
```
电子签名 = 私钥签名 + 签署理由 + 时间戳
```

双人复核 (Four Eyes):
```sql
CREATE APPROVAL POLICY batch_release (
    required_signatures = 2,
    required_roles = ('QA_MANAGER', 'PRODUCTION_MANAGER'),
    sequential = TRUE
);
```

#### P0-3: Immutable Record

```sql
CREATE TABLE batch_record (...) IMMUTABLE;
-- INSERT 允许
-- UPDATE 禁止
-- DELETE 禁止
-- 修正通过 Correction Chain
```

---

## 四、窗口函数缺失

### 4.1 v3.1.0 窗口函数状态

| 函数 | MySQL | v3.1.0 | 状态 |
|------|-------|--------|------|
| ROW_NUMBER | ✅ | ✅ | 已实现 |
| RANK | ✅ | ✅ | 已实现 |
| DENSE_RANK | ✅ | ✅ | 已实现 |
| LEAD | ✅ | ✅ | 已实现 |
| LAG | ✅ | ✅ | 已实现 |
| NTILE | ✅ | ✅ | 已实现 |

**结论**: 窗口函数 6/6 已全部实现，v3.2.0 无需继续投入。

---

## 五、多表 DML 覆盖缺口

### 5.1 MERGE 语句

| 功能 | 状态 | 测试 |
|------|------|------|
| MERGE matched UPDATE | ✅ | 17 tests |
| MERGE NOT MATCHED INSERT | ✅ | 通过 |
| MERGE 多表 | ✅ | 通过 |

### 5.2 多表 UPDATE/DELETE

| 功能 | v3.1.0 | v3.2.0 目标 |
|------|--------|-------------|
| UPDATE t1, t2 SET ... | ✅ | 完善 |
| DELETE t1, t2 FROM ... | ✅ | 完善 |
| JOIN in UPDATE | ⚠️ | 100% 覆盖 |
| JOIN in DELETE | ⚠️ | 100% 覆盖 |

### 5.3 遗留缺口

| Issue | 描述 | 优先级 |
|-------|------|--------|
| #877 | DCL 权限链测试 | P1 |

---

## 六、GMP 合规差距分析

### 6.1 GMP 核心需求体系

| 核心 | 本质 | v3.1.0 | v3.2.0 目标 |
|------|------|--------|-------------|
| Data Integrity | 数据不可伪造 | WAL + SHA-256 | 数字签名 |
| Traceability | 可追溯 | 审计日志 | 完整 Provenance |
| Accountability | 谁做的 | 用户关联 | 电子签名 |
| Non-Repudiation | 不可抵赖 | 基础 | 私钥签名 + 验签 |
| Electronic Signature | 电子签名 | ❌ | ✅ P0 |
| Auditability | 审计能力 | WAL 审计链 | 可验证审计链 |

### 6.2 GMP 合规差距

| 功能 | FDA 21 CFR Part 11 | v3.1.0 | v3.2.0 差距 |
|------|-------------------|--------|-------------|
| 电子签名 | 必须 | ❌ | 需实现 |
| 签署理由 | 必须 | ❌ | 需实现 |
| 时间戳 | 必须 | ⚠️ 基础 | 需 RFC3161 |
| 双人复核 | 必须 | ❌ | 需实现 |
| 审计追踪 | 必须 | ⚠️ WAL | 需数字签名 |
| 记录不可变 | 必须 | ❌ | 需 EBR |

---

## 七、ALCOA+ 支撑矩阵

| ALCOA+ | 说明 | v3.1.0 | v3.2.0 |
|--------|------|--------|--------|
| A - Attributable | 可归因 | 用户 ID | 数字签名 + 设备指纹 |
| C - Contemporaneous | 实时 | 时间戳 | 可信时间戳 (RFC3161) |
| O - Original | 原始 | WAL | 原始记录 + 哈希锚定 |
| +C - Complete | 完整 | 审计日志 | 完整 Provenance |
| +E - Enduring | 持久 | WAL 持久 | 冷存储集成 |

### 7.1 ALCOA+ 实现状态

| 属性 | v3.2.0 实现 | 任务 ID |
|------|-------------|---------|
| Attributable | 用户私钥签名 | GMP-1, GMP-2 |
| Contemporaneous | RFC3161 时间戳 | GMP-6 |
| Original | 原始记录哈希锚定 | GMP-4 |
| Complete | 完整 Provenance | GMP-5 |
| Enduring | 冷存储集成 | GMP-7 |

---

## 八、MySQL 兼容性差距

### 8.1 MySQL 兼容性评分

| 维度 | v3.1.0 | v3.2.0 目标 | 差距 |
|------|--------|-------------|------|
| SQL 语言 | 85/100 | 90/100 | 窗口函数/递归 CTE |
| 存储引擎 | 75/100 | 80/100 | 聚簇索引完善 |
| 可观测性 | 65/100 | 75/100 | Performance Schema |
| 安全 | 80/100 | 90/100 | 电子签名 |
| 高可用 | 60/100 | 75/100 | 组复制/故障转移 |
| **总体** | **75/100** | **≥80/100** | |

### 8.2 功能缺口

| 功能 | MySQL 8.0 | v3.1.0 | v3.2.0 |
|------|-----------|--------|--------|
| 窗口函数 | ✅ 6/6 | ✅ 6/6 | ✅ |
| RECURSIVE CTE | ✅ | ⚠️ | 完善 |
| CREATE EVENT | ✅ | ✅ | 完善 |
| FULLTEXT 索引 | ✅ | ⚠️ 基础 | 完善 |
| 聚簇索引 | ✅ | ✅ | 完善 |
| 组复制 | ✅ | ❌ | v3.3.0 |

### 8.3 SQL 覆盖率

| 指标 | v3.1.0 | v3.2.0 目标 |
|------|--------|-------------|
| SQL 语料库 | 95.4% | ≥98% |
| DDL 覆盖率 | ~90% | ≥95% |
| DML 覆盖率 | ~95% | ≥98% |
| DCL 覆盖率 | ~80% | ≥90% |

---

## 九、v3.2.0 问题分类 (P0/P1/P2)

### 9.1 P0 必须 (GMP 合规内核)

| 任务 ID | 功能 | 说明 |
|---------|------|------|
| GMP-1 | 数字签名审计链 | 签名类型 (User/Device/System/Batch) |
| GMP-2 | 电子签名 | 21 CFR Part 11 合规 |
| GMP-3 | Immutable Record | EBR 核心，禁止 DML |
| GMP-4 | Correction Chain | 数据修正链 |
| GMP-5 | Provenance Tracking | 完整溯源 |
| GMP-6 | Trusted Timestamp | RFC3161 |
| GMP-7 | 审计链验证工具 | 验证完整性 |
| GMP-8 | HSM/KMS 集成 | TPM/HSM/KMS |

### 9.2 P1 重要 (GMP 平台能力)

| 任务 ID | 功能 | 说明 |
|---------|------|------|
| PERF-1 | QPS 提升 30% | Point SELECT ≥1M |
| PERF-2 | TPC-H SF=10 | 22/22 无 OOM |
| PERF-3 | 并发 200+ | 连接池增强 |
| PERF-4 | 死锁检测 <50ms | SSI 优化 |
| PERF-5 | 内存占用 -15% | 优化内存管理 |
| SQL-1 | RECURSIVE CTE | 递归查询 |
| SQL-2 | Performance Schema | ≥60% |
| SQL-3 | 冷存储集成 | S3/OSS |

### 9.3 P2 后续

| 任务 ID | 功能 | 说明 |
|---------|------|------|
| OO-1~8 | OO 文档完善 | 见第一章 |
| SQL-4 | 组复制 | v3.3.0 |
| SQL-5 | 自动故障转移 | v3.3.0 |
| SQL-6 | 地理分布 | v3.3.0 |

---

## 十、问题→任务映射

### 10.1 GMP 合规任务 (GMP-1~12)

| Issue | 任务 ID | 功能 | 优先级 |
|-------|---------|------|--------|
| #900 | GMP-1 | 数字签名审计链 | P0 |
| #901 | GMP-2 | 电子签名 | P0 |
| #902 | GMP-3 | Immutable Record | P0 |
| #903 | GMP-4 | Correction Chain | P0 |
| #904 | GMP-5 | Provenance Tracking | P0 |
| #905 | GMP-6 | Trusted Timestamp | P0 |
| #906 | GMP-7 | 审计链验证工具 | P0 |
| #907 | GMP-8 | HSM/KMS 集成 | P0 |
| #908 | GMP-9 | GMP Workflow Engine | P1 |
| #909 | GMP-10 | 移动端可信采集 | P1 |
| #910 | GMP-11 | SOP/培训绑定 | P1 |
| #911 | GMP-12 | Device Calibration | P1 |

### 10.2 性能任务 (PERF-1~5)

| Issue | 任务 ID | 功能 | 目标 |
|-------|---------|------|------|
| #920 | PERF-1 | Point SELECT QPS | ≥1M ops/s |
| #921 | PERF-2 | TPC-H SF=10 | 22/22 通过 |
| #922 | PERF-3 | 并发增强 | 200+ 连接 |
| #923 | PERF-4 | 死锁检测优化 | <50ms |
| #924 | PERF-5 | 内存优化 | -15% 占用 |

### 10.3 SQL 任务 (SQL-1~8)

| Issue | 任务 ID | 功能 | 目标 |
|-------|---------|------|------|
| #930 | SQL-1 | RECURSIVE CTE | 完整支持 |
| #931 | SQL-2 | Performance Schema | ≥60% |
| #932 | SQL-3 | 冷存储集成 | S3/OSS |
| #933 | SQL-4 | 组复制 | v3.3.0 |
| #934 | SQL-5 | 自动故障转移 | v3.3.0 |
| #935 | SQL-6 | 地理分布 | v3.3.0 |
| #936 | SQL-7 | DCL 权限链完善 | 100% |
| #937 | SQL-8 | FULLTEXT 完善 | 中英文 |

### 10.4 OO 文档任务 (OO-1~8)

| Issue | 任务 ID | 文档 | 优先级 |
|-------|---------|------|--------|
| #940 | OO-1 | 数字签名审计链设计 | P0 |
| #941 | OO-2 | 电子签名设计 | P0 |
| #942 | OO-3 | Immutable Record 设计 | P0 |
| #943 | OO-4 | Correction Chain 设计 | P0 |
| #944 | OO-5 | Provenance Tracking 设计 | P0 |
| #945 | OO-6 | HSM/KMS 集成设计 | P0 |
| #946 | OO-7 | GMP Workflow Engine 设计 | P1 |
| #947 | OO-8 | Trusted Timestamp 设计 | P1 |

---

*本文档由 hermes-agent 维护*
*版本 1.0 - 2026-05-15*
