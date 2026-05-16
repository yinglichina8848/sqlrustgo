# OO-2: 电子签名设计文档

> **版本**: v1.0
> **日期**: 2026-05-15
> **基于**: v3.1.0 GA
> **维护人**: hermes-z6g4
> **状态**: 设计中

---

## 一、概述

### 1.1 目标

实现符合 FDA 21 CFR Part 11 的电子签名系统，确保：
- 数据完整性 (Integrity)
- 签名不可抵赖 (Non-repudiation)
- 操作可追溯 (Accountability)

### 1.2 电子签名公式

```
电子签名 = 私钥签名 + 签署理由 + 时间戳
```

---

## 二、21 CFR Part 11 合规要求

### 2.1 必需元素

| 元素 | 说明 | 实现方式 |
|------|------|----------|
| 签名 | 用户私钥签名 | ED25519 (已有 `signature.rs`) |
| 签署理由 | 签名的原因/意图 | `signing_reason` 字段 |
| 时间戳 | 签名时间 | Trusted Timestamp (GMP-6) |
| 用户标识 | 谁签的名 | `user_id` + `session_id` |
| 链接 | 签名与数据绑定 | SHA-256 链接到审计链 |

### 2.2 双人复核 (Four Eyes Principle)

某些操作需要多人签署：

```sql
CREATE APPROVAL POLICY batch_release (
    required_signatures = 2,
    required_roles = ('QA_MANAGER', 'PRODUCTION_MANAGER'),
    sequential = TRUE
);
```

---

## 三、系统架构

### 3.1 组件关系

```
┌─────────────────────────────────────────────────────────────────┐
│                      Electronic Signature System                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────────┐  │
│  │    User     │───▶│  Signature   │───▶│   Audit Chain    │  │
│  │   (ECDSA)   │    │   Manager    │    │   (GMP-1)        │  │
│  └──────────────┘    └──────────────┘    └──────────────────┘  │
│         │                   │                      │            │
│         │                   │                      │            │
│         ▼                   ▼                      ▼            │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────────┐  │
│  │ Signing      │    │ Signature    │    │   Timestamp      │  │
│  │ Reason       │    │ Verification │    │   (GMP-6)        │  │
│  └──────────────┘    └──────────────┘    └──────────────────┘  │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │              Approval Policy Engine                      │   │
│  │  - Policy Evaluation                                     │   │
│  │  - Sequential/Parallel Signatures                        │   │
│  │  - Role-based Requirements                                │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 3.2 数据流

```
1. User initiates signature request
         │
         ▼
2. System prompts for signing reason
         │
         ▼
3. User enters reason (e.g., "Approved for batch release")
         │
         ▼
4. System collects:
   - user_id + session_id
   - timestamp (from GMP-6)
   - data_hash (hash of record being signed)
   - reason
         │
         ▼
5. User's private key signs: sign(data_hash || reason || timestamp)
         │
         ▼
6. Signature stored in gmp_electronic_signatures table
         │
         ▼
7. Signature linked to audit chain entry
```

---

## 四、数据结构

### 4.1 电子签名表 (gmp_electronic_signatures)

```sql
CREATE TABLE gmp_electronic_signatures (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    audit_chain_id  BIGINT NOT NULL REFERENCES gmp_audit_log(id),
    user_id         TEXT NOT NULL,
    session_id      TEXT,
    role            TEXT,
    reason          TEXT NOT NULL,           -- 签署理由
    data_hash       TEXT NOT NULL,           -- 被签名数据的哈希
    signature       BYTES NOT NULL,           -- ED25519 签名
    verifying_key   BYTES NOT NULL,          -- 公钥
    timestamp       BIGINT NOT NULL,         -- Unix ms
    policy_id       UUID,                    -- 如果需要审批策略
    policy_name     TEXT,                    -- 策略名称
    seq_in_policy   INT,                     -- 在策略中的顺序
    created_at      TIMESTAMP DEFAULT NOW()
);
```

### 4.2 审批策略表 (gmp_approval_policies)

```sql
CREATE TABLE gmp_approval_policies (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name                TEXT UNIQUE NOT NULL,
    required_signatures INT NOT NULL DEFAULT 1,
    required_roles     TEXT[] NOT NULL,      -- ARRAY['QA_MANAGER', 'PRODUCTION_MANAGER']
    sequential         BOOLEAN NOT NULL DEFAULT TRUE,  -- TRUE=顺序签署
    timeout_hours      INT DEFAULT 72,
    description        TEXT,
    created_at         TIMESTAMP DEFAULT NOW(),
    updated_at         TIMESTAMP DEFAULT NOW(),
    active             BOOLEAN DEFAULT TRUE
);
```

### 4.3 签署请求表 (gmp_signature_requests)

```sql
CREATE TABLE gmp_signature_requests (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    policy_id       UUID NOT NULL REFERENCES gmp_approval_policies(id),
    record_table    TEXT NOT NULL,
    record_id       TEXT NOT NULL,
    status          TEXT NOT NULL DEFAULT 'PENDING',  -- PENDING/APPROVED/REJECTED/EXPIRED
    current_step    INT NOT NULL DEFAULT 1,
    created_at      TIMESTAMP DEFAULT NOW(),
    updated_at      TIMESTAMP DEFAULT NOW(),
    expires_at      TIMESTAMP
);
```

---

## 五、API 设计

### 5.1 核心 Trait: `ElectronicSignatureProvider`

```rust
/// 电子签名提供者接口
pub trait ElectronicSignatureProvider {
    /// 创建电子签名
    fn sign(
        &self,
        user_id: &str,
        session_id: Option<&str>,
        data_hash: &[u8],
        reason: &str,
        signing_key: &[u8; 32],
    ) -> Result<ElectronicSignature, SignatureError>;

    /// 验证电子签名
    fn verify(
        &self,
        signature: &ElectronicSignature,
        data_hash: &[u8],
    ) -> Result<bool, SignatureError>;

    /// 获取用户签名信息
    fn get_user_signatures(
        &self,
        user_id: &str,
        limit: usize,
    ) -> Result<Vec<ElectronicSignature>, SignatureError>;
}
```

### 5.2 审批策略 Trait: `ApprovalPolicyProvider`

```rust
/// 审批策略提供者接口
pub trait ApprovalPolicyProvider {
    /// 创建审批策略
    fn create_policy(
        &self,
        name: &str,
        required_signatures: usize,
        required_roles: &[&str],
        sequential: bool,
    ) -> Result<ApprovalPolicy, PolicyError>;

    /// 评估是否满足策略
    fn evaluate_policy(
        &self,
        policy_id: Uuid,
        record_table: &str,
        record_id: &str,
    ) -> Result<PolicyEvaluation, PolicyError>;

    /// 添加签名到策略请求
    fn add_signature_to_request(
        &self,
        request_id: Uuid,
        signature: &ElectronicSignature,
    ) -> Result<PolicyEvaluation, PolicyError>;
}
```

### 5.3 签名请求结构

```rust
/// 电子签名请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElectronicSignatureRequest {
    pub user_id: String,
    pub session_id: Option<String>,
    pub role: Option<String>,
    pub data_hash: Vec<u8>,
    pub reason: String,
    pub policy_id: Option<Uuid>,
}

/// 电子签名结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElectronicSignature {
    pub id: Uuid,
    pub user_id: String,
    pub session_id: Option<String>,
    pub reason: String,
    pub data_hash: Vec<u8>,
    pub signature: Vec<u8>,
    pub verifying_key: Vec<u8>,
    pub timestamp: i64,
    pub policy_id: Option<Uuid>,
    pub policy_name: Option<String>,
    pub seq_in_policy: Option<i32>,
}

/// 审批策略评估结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyEvaluation {
    pub policy_id: Uuid,
    pub request_id: Uuid,
    pub status: PolicyStatus,
    pub current_signatures: usize,
    pub required_signatures: usize,
    pub is_complete: bool,
    pub missing_roles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PolicyStatus {
    Pending,
    Approved,
    Rejected,
    Expired,
}
```

---

## 六、SQL 语句支持

### 6.1 创建审批策略

```sql
CREATE APPROVAL POLICY policy_name (
    required_signatures = n,
    required_roles = ('ROLE1', 'ROLE2', ...),
    sequential = TRUE|FALSE,
    timeout_hours = h
);
```

### 6.2 请求签名

```sql
-- 单人签名
SIGN RECORD table_name
FOR record_id
WITH REASON 'reason text'
[BY POLICY policy_name];

-- 例子
SIGN RECORD batch_records
FOR 'batch-2024-001'
WITH REASON 'Approved for release'
BY POLICY batch_release;
```

### 6.3 查询签名

```sql
-- 查询记录的所有签名
SELECT * FROM gmp_electronic_signatures
WHERE audit_chain_id = (
    SELECT id FROM gmp_audit_log
    WHERE table_name = 'batch_records'
    AND record_id = 'batch-2024-001'
);

-- 查询待审批请求
SELECT * FROM gmp_signature_requests
WHERE status = 'PENDING';
```

---

## 七、错误处理

### 7.1 错误类型

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignatureError {
    /// 签名验证失败
    VerificationFailed { reason: String },
    /// 无效的密钥
    InvalidKey { reason: String },
    /// 用户权限不足
    InsufficientPermissions { required_role: String },
    /// 策略未满足
    PolicyNotSatisfied { missing: Vec<String> },
    /// 签名已存在
    SignatureAlreadyExists { signature_id: Uuid },
    /// 请求已过期
    RequestExpired { request_id: Uuid },
    /// 审批顺序错误
    SequentialOrderViolation { expected_step: i32, actual_step: i32 },
}
```

---

## 八、安全考虑

### 8.1 私钥保护

- 私钥存储在 HSM/KMS 中 (GMP-8)
- 软件模拟模式用于开发和测试
- 私钥永不离开 HSM

### 8.2 签名验证

- 每次操作前验证签名
- 验证链接到审计链的完整性
- 时间戳使用可信时间源

### 8.3 审计

- 所有签名操作记录到审计链
- 签名验证失败也记录
- 保留所有签名历史

---

## 九、实现计划

### 9.1 阶段划分

| 阶段 | 任务 | 交付物 |
|------|------|--------|
| 1 | 数据结构定义 | `gmp_electronic_signatures`, `gmp_approval_policies` 表 |
| 2 | 核心签名逻辑 | `ElectronicSignatureProvider` trait + 实现 |
| 3 | 审批策略引擎 | `ApprovalPolicyProvider` trait + 实现 |
| 4 | SQL 语句解析 | `SIGN RECORD` 和 `CREATE APPROVAL POLICY` 语法 |
| 5 | 集成测试 | 完整流程测试 |
| 6 | 文档 | API 文档和使用示例 |

### 9.2 依赖

- GMP-1: 审计链 (已完成基础)
- GMP-6: Trusted Timestamp (待实现)
- GMP-8: HSM/KMS (待实现，可先用软件模拟)

---

## 十、测试策略

### 10.1 单元测试

- 签名创建和验证
- 策略评估逻辑
- 顺序/并行签名验证

### 10.2 集成测试

- 完整签名流程
- 审计链集成
- SQL 语句执行

### 10.3 合规测试

- 21 CFR Part 11 要求验证
- 四眼原则验证
- 超时和过期处理

---

*本文档由 hermes-agent 创建*
*版本 1.0 - 2026-05-15*
