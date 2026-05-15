# OO-1: 数字签名审计链设计文档

> **版本**: v1.0
> **日期**: 2026-05-16
> **基于**: v3.2.0
> **维护人**: hermes-z6g4
> **状态**: 已完成

---

## 一、概述

### 1.1 目标

实现 GMP 数字签名审计链，提供数据完整性和不可伪造性保证：

- **数据完整性**: 通过哈希链确保数据未被篡改
- **签名不可抵赖**: 通过数字签名确保签名人无法否认
- **操作可追溯**: 完整的审计日志记录

### 1.2 核心理念

```
数字签名审计链 = 哈希链 + 数字签名 + 时间戳
```

---

## 二、技术架构

### 2.1 组件关系

```
┌─────────────────────────────────────────────────────────────────┐
│                    Digital Signature Audit Chain                   │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────────┐  │
│  │    Hash     │───▶│  Signature   │───▶│   Audit Log      │  │
│  │    Chain    │    │   Manager    │    │   (GMP-1)       │  │
│  └──────────────┘    └──────────────┘    └──────────────────┘  │
│         │                   │                      │            │
│         │                   │                      │            │
│         ▼                   ▼                      ▼            │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────────┐  │
│  │ Integrity    │    │ Signature    │    │   Timestamp      │  │
│  │ Verification │    │ Verification │    │   (GMP-6)       │  │
│  └──────────────┘    └──────────────┘    └──────────────────┘  │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │              Cryptographic Primitives                    │   │
│  │  - ED25519 (default)                                   │   │
│  │  - ECDSA P-256 (legacy support)                        │   │
│  │  - RSA-SHA256 (legacy support)                          │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## 三、数据结构

### 3.1 审计链条目表 (gmp_audit_chain)

```sql
CREATE TABLE gmp_audit_chain (
    id              BIGSERIAL PRIMARY KEY,
    table_name      TEXT NOT NULL,
    record_id       TEXT NOT NULL,
    operation       TEXT NOT NULL,           -- INSERT/UPDATE/DELETE
    prev_hash       TEXT,                    -- 前一个条目的哈希
    current_hash    TEXT NOT NULL,          -- 当前条目的哈希
    signature       BYTES,                  -- 数字签名
    signer_id       TEXT,                   -- 签名人ID
    verifying_key   BYTES,                  -- 公钥
    timestamp       BIGINT NOT NULL,         -- Unix ms
    created_at      TIMESTAMP DEFAULT NOW()
);
```

### 3.2 签名元数据表 (gmp_signature_metadata)

```sql
CREATE TABLE gmp_signature_metadata (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    audit_chain_id      BIGINT NOT NULL REFERENCES gmp_audit_chain(id),
    algorithm           TEXT NOT NULL,           -- ED25519/ECDSA/RSA
    signature_data      BYTES NOT NULL,
    public_key          BYTES NOT NULL,
    signer_id           TEXT NOT NULL,
    signer_role         TEXT,
    timestamp           BIGINT NOT NULL,
    reason              TEXT,
    created_at          TIMESTAMP DEFAULT NOW()
);
```

---

## 四、API 设计

### 4.1 核心 Trait: `SignatureProvider`

```rust
/// 数字签名提供者接口
pub trait SignatureProvider {
    /// 创建签名
    fn sign(
        &self,
        data: &[u8],
        signing_key: &[u8; 32],
    ) -> Result<Signature, SignatureError>;

    /// 验证签名
    fn verify(
        &self,
        signature: &Signature,
        data: &[u8],
        public_key: &[u8],
    ) -> Result<bool, SignatureError>;

    /// 生成密钥对
    fn generate_keypair(&self) -> Result<(KeyPair, PublicKey), SignatureError>;
}

/// 签名结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    pub algorithm: SignatureAlgorithm,
    pub signature_data: Vec<u8>,
    pub public_key: Vec<u8>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SignatureAlgorithm {
    Ed25519,
    EcdsaP256,
    RsaSha256,
}
```

### 4.2 哈希链 Trait: `HashChainProvider`

```rust
/// 哈希链提供者接口
pub trait HashChainProvider {
    /// 添加条目到链
    fn append(
        &self,
        table_name: &str,
        record_id: &str,
        operation: &str,
        data: &[u8],
    ) -> Result<HashChainEntry, HashChainError>;

    /// 验证链的完整性
    fn verify(&self, start_id: i64, end_id: i64) -> Result<VerificationResult, HashChainError>;

    /// 获取链的最新条目
    fn latest(&self, table_name: &str, record_id: &str) -> Result<Option<HashChainEntry>, HashChainError>;
}

/// 哈希链条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashChainEntry {
    pub id: i64,
    pub table_name: String,
    pub record_id: String,
    pub operation: String,
    pub prev_hash: Option<String>,
    pub current_hash: String,
    pub signature: Option<Signature>,
    pub timestamp: i64,
}
```

---

## 五、签名算法

### 5.1 ED25519 (默认)

```rust
// 签名流程
let message = compute_message_hash(data, timestamp);
let signature = ed25519::sign(&message, signing_key);

// 验证流程
let verified = ed25519::verify(&signature, &message, public_key);
```

### 5.2 ECDSA P-256

```rust
// 签名流程
let signature = ecdsa::sign_prehashed(&message, signing_key);

// 验证流程
let verified = ecdsa::verify_prehashed(&signature, &message, public_key);
```

### 5.3 签名范围

| 操作 | 是否签名 | 签名内容 |
|------|----------|----------|
| INSERT | ✅ | 插入的数据 |
| UPDATE | ✅ | 更新前 + 更新后 |
| DELETE | ✅ | 删除前的数据 |
| SELECT | ❌ | - |

---

## 六、安全考虑

### 6.1 密钥管理

- 私钥存储在 HSM/KMS 中 (GMP-8)
- 软件实现用于开发和测试
- 密钥轮换策略

### 6.2 完整性验证

```rust
// 链完整性检查
fn verify_chain(chain: &[HashChainEntry]) -> bool {
    for i in 1..chain.len() {
        let expected_hash = compute_hash(&chain[i-1]);
        if chain[i].prev_hash != expected_hash {
            return false;
        }
    }
    true
}
```

---

## 七、实现状态

| 阶段 | 任务 | 状态 | PR |
|------|------|------|-----|
| 1 | 数据结构定义 | ✅ | - |
| 2 | ED25519 签名 | ✅ | #1073 |
| 3 | 哈希链实现 | ✅ | #1076 |
| 4 | 验证工具 | ✅ | #1076 |

---

## 八、测试策略

### 8.1 单元测试

- 签名创建和验证
- 哈希链追加和验证
- 多算法支持

### 8.2 集成测试

- 完整签名链流程
- 跨版本数据验证
- 性能基准

---

*本文档由 hermes-agent 创建*
*版本 1.0 - 2026-05-16*