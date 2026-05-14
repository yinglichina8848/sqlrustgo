# SQLRustGo v3.2.0 战略开发指导

> **版本**: v1.0
> **日期**: 2026-05-15
> **维护人**: hermes-z6g4
> **策略来源**: 基于 GMP 行业分析与 ChatGPT 战略建议

---

## 一、战略转型声明

### 1.1 版本定位

| 版本 | 定位 | 核心能力 |
|------|------|----------|
| **v3.0.0** | 功能可用 | SQL 基础、事务、基础存储 |
| **v3.1.0** | 工业级可信 OLTP 内核 | SSI + MVCC、WAL 审计链、AES-256、TLA+ 验证 |
| **v3.2.0** | GMP Native 可信数据平台 | 数字签名审计链、电子签名、EBR、工作流、HSM |

### 1.2 战略转向说明

**旧定位**: MySQL 兼容替代品

**新定位**: 世界上第一个 GMP Native Trusted Database

**核心差异**:

```
旧路线: 功能堆叠 → 更多 SQL → 更像 PostgreSQL
新路线: 可信闭环 → GMP Native → 审计底座
```

---

## 二、GMP 核心需求体系

### 2.1 GMP 真正需要什么

| 核心 | 本质 | v3.1.0 | v3.2.0 目标 |
|------|------|---------|-------------|
| Data Integrity | 数据不可伪造 | WAL + SHA-256 | 数字签名 |
| Traceability | 可追溯 | 审计日志 | 完整 Provenance |
| Accountability | 谁做的 | 用户关联 | 电子签名 |
| Non-Repudiation | 不可抵赖 | 基础 | 私钥签名 + 验签 |
| Electronic Signature | 电子签名 | ❌ | ✅ P0 |
| Auditability | 审计能力 | WAL 审计链 | 可验证审计链 |

### 2.2 ALCOA+ 支撑矩阵

| ALCOA+ | 说明 | v3.1.0 | v3.2.0 |
|--------|------|---------|---------|
| A - Attributable | 可归因 | 用户 ID | 数字签名 + 设备指纹 |
| C - Contemporaneous | 实时 | 时间戳 | 可信时间戳 (RFC3161) |
| O - Original | 原始 | WAL | 原始记录 + 哈希锚定 |
| +C - Complete | 完整 | 审计日志 | 完整 Provenance |
| +E - Enduring | 持久 | WAL 持久 | 冷存储集成 |

---

## 三、v3.2.0 技术优先级

### 3.1 P0 必须 (GMP 合规内核)

| 优先级 | 功能 | 重要性 |
|--------|------|--------|
| P0-1 | 数字签名审计链 | GMP/FDA 核心 |
| P0-2 | 电子签名 | 21 CFR Part 11 |
| P0-3 | Immutable Record | EBR 核心 |
| P0-4 | Correction Chain | 数据完整性 |
| P0-5 | Provenance Tracking | ALCOA+ |
| P0-6 | Trusted Timestamp | 合规要求 |
| P0-7 | 审计链验证工具 | 审计检查 |
| P0-8 | HSM/KMS 集成 | 密钥安全 |

### 3.2 P1 重要 (GMP 平台能力)

| 优先级 | 功能 | 说明 |
|--------|------|------|
| P1-1 | GMP Workflow Engine | 流程受控 |
| P1-2 | 移动端可信采集 | 现场生产 |
| P1-3 | SOP/培训绑定 | 人员合规 |
| P1-4 | Device Calibration | 仪器管理 |
| P1-5 | 列式分析引擎 | QMS 分析 |
| P1-6 | Syslog/SIEM | 企业接入 |
| P1-7 | Agent Audit | AI 合规 |

### 3.3 P2 后续

| 优先级 | 功能 | 说明 |
|--------|------|------|
| P2-1 | 分布式事务 | 当前不是核心 |
| P2-2 | MPP | GMP 优先级低 |

---

## 四、P0 功能设计

### 4.1 数字签名审计链 (P0-1)

**升级路径**:
```
当前: hash(prev_hash || content)
目标: sign(hash(prev_hash || content), user_private_key)
```

**签名类型**:
- User Signature - 用户操作签名 (ECDSA-P256)
- Device Signature - 设备采集签名
- System Signature - 系统操作签名 (HMAC-SHA256)
- Batch Signature - 批记录签名 (RSA-2048)

### 4.2 电子签名 (P0-2)

满足 FDA 21 CFR Part 11:
```
电子签名 = 私钥签名 + 签署理由 + 时间戳
```

**双人复核 (Four Eyes)**:
```sql
CREATE APPROVAL POLICY batch_release (
    required_signatures = 2,
    required_roles = ('QA_MANAGER', 'PRODUCTION_MANAGER'),
    sequential = TRUE
);
```

### 4.3 Immutable Record (P0-3)

```sql
CREATE TABLE batch_record (...) IMMUTABLE;
-- INSERT 允许
-- UPDATE 禁止
-- DELETE 禁止
-- 修正通过 Correction Chain
```

### 4.4 Correction Chain (P0-4)

```
原值 → 修改值 → 原因 → 审批人 → 时间 → 电子签名
```

```sql
CORRECT RECORD batch_record
    SET quantity = 1000
    WHERE id = 'uuid-xxx'
    REASON '批次拆分'
    APPROVED BY 'qa-manager-uuid';
```

### 4.5 Provenance Tracking (P0-5)

```sql
CREATE TABLE data_provenance (
    source_device   TEXT,
    source_ip       TEXT,
    source_gps      TEXT,
    source_app_version TEXT,
    operator_id     UUID,
    operation_type  TEXT,
    chain_hash      TEXT
);
```

### 4.6 HSM/KMS 集成 (P0-8)

```sql
SET KEY_MANAGEMENT = (
    provider = 'tpm',  -- tpm | hsm | kms
    endpoint = 'unix:///dev/tpm0',
    algorithm = 'ecc_p256'
);
```

---

## 五、架构转型

### 5.1 从 Database 到 Trusted Execution Platform

```
v3.2.0 架构:
┌─────────────────────────────────────────────────────────┐
│           Trusted GMP Execution Platform                 │
├─────────────────────────────────────────────────────────┤
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐       │
│  │   SQL   │ │ 审计链  │ │ 工作流  │ │ 签名引擎 │       │
│  │ Engine  │ │         │ │ Engine  │ │         │       │
│  └─────────┘ └─────────┘ └─────────┘ └─────────┘       │
├─────────────────────────────────────────────────────────┤
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐       │
│  │ Provenance│ │ 设备管理│ │ SOP绑定 │ │ Agent   │       │
│  └─────────┘ └─────────┘ └─────────┘ └─────────┘       │
├─────────────────────────────────────────────────────────┤
│              HSM / KMS / TPM                           │
└─────────────────────────────────────────────────────────┘
```

---

## 六、战略成功指标

### 6.1 市场指标

| 指标 | v3.2.0 目标 |
|------|-------------|
| GMP 系统合作客户 | 3 家 |
| MES/QMS 集成案例 | 5 个 |
| FDA 审计通过客户 | 1 家 |

### 6.2 技术指标

| 指标 | v3.2.0 目标 |
|------|-------------|
| TLA+ 验证覆盖率 | 100% P0 |
| 测试覆盖率 | ≥ 85% |
| 密钥管理集成 | 3 种 (TPM/HSM/KMS) |

---

## 七、总结

### 7.1 核心转变

```
v3.2.0 = 从 "MySQL 替代品" → "GMP Native 可信数据平台"
```

### 7.2 v3.2.0 成功定义

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

*本文档由 hermes-z6g4 维护*
*版本 1.0 - 2026-05-15*
