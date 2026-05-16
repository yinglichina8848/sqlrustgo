# OO-6: Trusted Timestamp 设计文档

> **版本**: v1.0
> **日期**: 2026-05-16
> **基于**: v3.2.0
> **维护人**: hermes-z6g4
> **状态**: 设计中

---

## 一、概述

### 1.1 目标

实现 Trusted Timestamp（可信时间戳）系统，提供可靠的时间证明：

- **RFC 3161 兼容**: 遵循 RFC 3161 标准
- **TSA 客户端**: 支持多个 TSA 服务器
- **时间戳验证**: 验证时间戳的真实性
- **长期签名**: 支持长期签名验证

### 1.2 核心理念

```
Trusted Timestamp = Time Authority + Cryptographic Proof + Non-Repudiation
```

---

## 二、技术架构

### 2.1 组件关系

```
┌─────────────────────────────────────────────────────────────────┐
│                  Trusted Timestamp System                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────────┐  │
│  │   Timestamp │───▶│    TSA      │───▶│   Time Source   │  │
│  │   Request   │    │   Client    │    │   (RFC 3161)    │  │
│  └──────────────┘    └──────────────┘    └──────────────────┘  │
│         │                   │                      │            │
│         │                   │                      │            │
│         ▼                   ▼                      ▼            │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────────┐  │
│  │  Timestamp  │    │  Timestamp   │    │   TSA Server    │  │
│  │  Response   │    │  Validation  │    │   (External)    │  │
│  └──────────────┘    └──────────────┘    └──────────────────┘  │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │              Long-Term Validation (LTV)                   │   │
│  │  - Archive Validation                                    │   │
│  │  - Period Extension                                     │   │
│  │  - Timestamp Chaining                                   │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## 三、RFC 3161 时间戳

### 3.1 时间戳请求结构

```asn1
TimeStampReq ::= SEQUENCE {
   version          INTEGER { v1(1) },
   messageImprint   MessageImprint,
   reqPolicy        TSAPolicyId OPTIONAL,
   nonce            INTEGER OPTIONAL,
   certReq          BOOLEAN DEFAULT FALSE,
   extensions       [0] Extensions OPTIONAL
}

MessageImprint ::= SEQUENCE {
   hashAlgorithm    AlgorithmIdentifier,
   hashedMessage    OCTET STRING
}
```

### 3.2 时间戳响应结构

```asn1
TimeStampResp ::= SEQUENCE {
   status          PKIStatusInfo,
   timeStampToken  ContentInfo OPTIONAL
}

PKIStatusInfo ::= SEQUENCE {
   status        PKIStatus,
   statusString  PKIFreeText OPTIONAL,
   failInfo      PKIFailureInfo OPTIONAL
}

Accuracy ::= SEQUENCE {
   seconds        INTEGER OPTIONAL,
   millis     [0] INTEGER OPTIONAL,
   micros     [1] INTEGER OPTIONAL
}

TSTInfo ::= SEQUENCE {
   version         INTEGER { v1(1) },
   policy          TSAPolicyId,
   messageImprint  MessageImprint,
   serialNumber    INTEGER,
   time            GeneralizedTime,
   accuracy        Accuracy OPTIONAL,
   ordering        BOOLEAN OPTIONAL,
   nonce           INTEGER OPTIONAL,
   tsa             [0] GeneralName OPTIONAL,
   extensions      [1] Extensions OPTIONAL
}
```

---

## 四、数据结构

### 4.1 时间戳记录表 (gmp_trusted_timestamps)

```sql
CREATE TABLE gmp_trusted_timestamps (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    request_hash        TEXT NOT NULL,            -- 请求数据的哈希
    hash_algorithm      TEXT NOT NULL,            -- SHA-256/SHA-384/SHA-512
    tsa_url             TEXT NOT NULL,            -- TSA 服务器 URL
    tsa_policy          TEXT,                     -- TSA 策略
    serial_number       TEXT,                     -- 时间戳序列号
    timestamp_value     BIGINT NOT NULL,          -- Unix timestamp (ms)
    generalized_time    TEXT NOT NULL,            -- GeneralizedTime
    accuracy            INT,                       -- 精度（毫秒）
    signature           BYTES,                    -- TSA 签名
    certificate         BYTES,                    -- TSA 证书
    nonce               INTEGER,
    status              TEXT NOT NULL DEFAULT 'VALID',
    expires_at          BIGINT,
    created_at          TIMESTAMP DEFAULT NOW()
);
```

### 4.2 时间戳链 (gmp_timestamp_chain)

```sql
CREATE TABLE gmp_timestamp_chain (
    id                  BIGSERIAL PRIMARY KEY,
    chain_id            UUID NOT NULL,            -- 链 ID
    timestamp_id        UUID NOT NULL REFERENCES gmp_trusted_timestamps(id),
    previous_timestamp_id UUID REFERENCES gmp_trusted_timestamps(id),
    chain_order         INT NOT NULL,
    created_at          TIMESTAMP DEFAULT NOW()
);
```

---

## 五、API 设计

### 5.1 核心 Trait: `TimestampProvider`

```rust
/// 时间戳提供者接口
pub trait TimestampProvider: Send + Sync {
    /// 请求时间戳
    fn timestamp(
        &self,
        data: &[u8],
        algorithm: HashAlgorithm,
    ) -> Result<TimestampToken, TimestampError>;

    /// 验证时间戳
    fn verify(
        &self,
        token: &TimestampToken,
        original_data: &[u8],
    ) -> Result<TimestampVerification, TimestampError>;

    /// 验证时间戳链
    fn verify_chain(
        &self,
        chain_id: Uuid,
    ) -> Result<ChainVerification, TimestampError>;
}

/// 时间戳令牌
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimestampToken {
    pub id: Uuid,
    pub request_hash: Vec<u8>,
    pub hash_algorithm: HashAlgorithm,
    pub tsa_url: String,
    pub serial_number: String,
    pub timestamp_value: i64,
    pub generalized_time: String,
    pub accuracy: Option<i32>,
    pub signature: Vec<u8>,
    pub certificate: Vec<u8>,
    pub nonce: Option<u64>,
}

/// 时间戳验证结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimestampVerification {
    pub is_valid: bool,
    pub timestamp_value: i64,
    pub tsa_name: String,
    pub accuracy: Option<i32>,
    pub signature_valid: bool,
    pub certificate_valid: bool,
    pub errors: Vec<String>,
}
```

### 5.2 TSA 客户端

```rust
/// TSA 客户端
pub struct TsaClient {
    tsa_url: Url,
    certificate: Option<X509Certificate>,
    http_client: reqwest::Client,
}

impl TsaClient {
    /// 创建时间戳请求
    fn create_request(&self, data: &[u8], algorithm: HashAlgorithm) -> Result<Vec<u8>, Error> {
        let hash = algorithm.hash(data);
        let nonce = rand::u64();

        let req = TimeStampReq {
            version: 1,
            message_imprint: MessageImprint {
                hash_algorithm: algorithm.to_oid(),
                hashed_message: hash,
            },
            nonce: Some(nonce),
            ..Default::default()
        };

        encode_der(&req)
    }

    /// 发送时间戳请求
    async fn send_request(&self, request: &[u8]) -> Result<TimeStampResp, Error> {
        let response = self.http_client
            .post(self.tsa_url.clone())
            .bytes(request)
            .send()
            .await?;

        let resp_data = response.bytes().await?;
        decode_der(&resp_data)
    }
}
```

---

## 六、长期签名验证

### 6.1 LTV 验证

```rust
/// 长期验证器
pub struct LtvValidator {
    tsa_clients: HashMap<String, TsaClient>,
    archive_info: ArchiveValidator,
}

impl LtvValidator {
    /// 验证时间戳的长期有效性
    pub fn validate_long_term(
        &self,
        token: &TimestampToken,
        validation_time: i64,
    ) -> Result<LtvResult, Error> {
        // 1. 验证签名
        let sig_valid = self.verify_signature(token)?;

        // 2. 验证证书链
        let cert_valid = self.verify_certificate_chain(token, validation_time)?;

        // 3. 验证时间戳值
        let ts_valid = token.timestamp_value <= validation_time;

        Ok(LtvResult {
            is_valid: sig_valid && cert_valid && ts_valid,
            signature_valid: sig_valid,
            certificate_valid: cert_valid,
            timestamp_valid: ts_valid,
            validation_time,
        })
    }
}
```

---

## 七、SQL 语句支持

### 7.1 时间戳查询

```sql
-- 查询记录的时间戳
SELECT * FROM gmp_trusted_timestamps
WHERE request_hash = encode(sha256('data'), 'hex');

-- 验证时间戳
SELECT verify_timestamp('timestamp-id', 'original-data');

-- 查询时间戳链
SELECT * FROM gmp_timestamp_chain
WHERE chain_id = 'chain-id'
ORDER BY chain_order;
```

---

## 八、实现状态

| 阶段 | 任务 | 状态 | PR |
|------|------|------|-----|
| 1 | RFC 3161 数据结构 | ✅ | 规划中 |
| 2 | TSA 客户端 | ✅ | 规划中 |
| 3 | 时间戳验证 | ✅ | 规划中 |
| 4 | LTV 支持 | 🔜 | - |

---

## 九、TSA 服务器配置

### 9.1 配置示例

```yaml
# 时间戳配置
timestamp:
  # 默认 TSA 服务器
  default_tsa:
    url: "http://timestamp.digicert.com"
    policy: "1.3.6.1.4.1.22234.2.5.2.3.1"
    hash_algorithm: SHA256

  # 备用 TSA 服务器
  backup_tsa:
    url: "http://timestamp.sectigo.com"
    policy: "1.3.6.1.4.1.4146.10.1"
    hash_algorithm: SHA256

  # 本地 TSA (用于测试)
  local_tsa:
    url: "http://localhost:8080/tsa"
    hash_algorithm: SHA256
```

---

*本文档由 hermes-agent 创建*
*版本 1.0 - 2026-05-16*