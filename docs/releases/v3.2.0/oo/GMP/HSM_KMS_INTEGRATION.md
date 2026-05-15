# OO-6: HSM/KMS 集成设计文档

> **版本**: v1.0
> **日期**: 2026-05-16
> **基于**: v3.2.0
> **维护人**: hermes-z6g4
> **状态**: 设计中

---

## 一、概述

### 1.1 目标

实现 HSM/KMS (Hardware Security Module / Key Management Service) 集成：

- **密钥管理**: 安全存储和管理加密密钥
- **TPM 支持**: 支持 TPM 2.0 硬件安全模块
- **云 KMS**: 支持 AWS KMS、Azure Key Vault
- **软件模拟**: 开发/测试环境的软件实现

### 1.2 核心理念

```
HSM/KMS = Secure Key Storage + Cryptographic Operations + Key Rotation
```

---

## 二、技术架构

### 2.1 组件关系

```
┌─────────────────────────────────────────────────────────────────┐
│                    HSM/KMS Integration                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────────┐  │
│  │   Crypto    │───▶│    HSM      │───▶│     KMS         │  │
│  │   Module    │    │   Provider   │    │    Provider     │  │
│  └──────────────┘    └──────────────┘    └──────────────────┘  │
│         │                   │                      │            │
│         │                   │                      │            │
│         ▼                   ▼                      ▼            │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────────┐  │
│  │  Software   │    │    TPM       │    │   Cloud KMS      │  │
│  │  Provider   │    │   Provider   │    │   Provider       │  │
│  │  (SoftKMS)  │    │   (TPM2)     │    │   (AWS/Azure)    │  │
│  └──────────────┘    └──────────────┘    └──────────────────┘  │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## 三、提供者类型

### 3.1 提供者矩阵

| 提供者 | 类型 | 用途 | 状态 |
|--------|------|------|------|
| SoftwareProvider | 软件 | 开发/测试 | ✅ 已实现 |
| TpmProvider | TPM 2.0 | 企业部署 | 🔜 规划中 |
| AwsKmsProvider | AWS KMS | AWS 云部署 | 🔜 规划中 |
| AzureKeyVaultProvider | Azure KV | Azure 云部署 | 🔜 规划中 |

---

## 四、数据结构

### 4.1 密钥元数据表 (gmp_key_metadata)

```sql
CREATE TABLE gmp_key_metadata (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    key_id              TEXT UNIQUE NOT NULL,    -- 外部密钥 ID
    key_name            TEXT NOT NULL,
    provider_type       TEXT NOT NULL,           -- SOFTWARE/TPM/AWS_KMS/Azure_KV
    algorithm           TEXT NOT NULL,           -- ED25519/ECDSA/AES
    key_size            INT,
    key_usage           TEXT[] NOT NULL,         -- SIGN/VERIFY/ENCRYPT/DECRYPT
    status              TEXT NOT NULL DEFAULT 'ACTIVE',
    creation_date       BIGINT NOT NULL,
    rotation_date       BIGINT,
    expiration_date     BIGINT,
    parent_key_id       UUID REFERENCES gmp_key_metadata(id),
    metadata_           JSONB,
    created_at          TIMESTAMP DEFAULT NOW(),
    updated_at          TIMESTAMP DEFAULT NOW()
);
```

### 4.2 密钥使用审计表 (gmp_key_usage_log)

```sql
CREATE TABLE gmp_key_usage_log (
    id                  BIGSERIAL PRIMARY KEY,
    key_id              UUID NOT NULL REFERENCES gmp_key_metadata(id),
    operation           TEXT NOT NULL,           -- SIGN/VERIFY/ENCRYPT/DECRYPT
    success             BOOLEAN NOT NULL,
    error_message       TEXT,
    caller_id           TEXT,
    caller_ip           TEXT,
    timestamp           BIGINT NOT NULL,
    created_at          TIMESTAMP DEFAULT NOW()
);
```

---

## 五、API 设计

### 5.1 核心 Trait: `KmsProvider`

```rust
/// KMS 提供者接口
pub trait KmsProvider: Send + Sync {
    /// 创建密钥
    fn create_key(
        &self,
        spec: &KeySpec,
    ) -> Result<KeyMetadata, KmsError>;

    /// 获取密钥公钥
    fn get_public_key(
        &self,
        key_id: &str,
    ) -> Result<Vec<u8>, KmsError>;

    /// 签名操作
    fn sign(
        &self,
        key_id: &str,
        data: &[u8],
    ) -> Result<Vec<u8>, KmsError>;

    /// 验证签名
    fn verify(
        &self,
        key_id: &str,
        data: &[u8],
        signature: &[u8],
    ) -> Result<bool, KmsError>;

    /// 加密操作
    fn encrypt(
        &self,
        key_id: &str,
        plaintext: &[u8],
    ) -> Result<Vec<u8>, KmsError>;

    /// 解密操作
    fn decrypt(
        &self,
        key_id: &str,
        ciphertext: &[u8],
    ) -> Result<Vec<u8>, KmsError>;

    /// 密钥轮换
    fn rotate_key(
        &self,
        key_id: &str,
    ) -> Result<KeyMetadata, KmsError>;

    /// 删除密钥
    fn delete_key(
        &self,
        key_id: &str,
    ) -> Result<(), KmsError>;
}

/// 密钥规格
#[derive(Debug, Clone)]
pub struct KeySpec {
    pub key_name: String,
    pub algorithm: Algorithm,
    pub key_usage: Vec<KeyUsage>,
    pub rotation_period_days: Option<i32>,
}

/// 密钥元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMetadata {
    pub id: Uuid,
    pub key_id: String,
    pub provider_type: ProviderType,
    pub algorithm: Algorithm,
    pub status: KeyStatus,
    pub creation_date: i64,
    pub expiration_date: Option<i64>,
}
```

### 5.2 软件实现: `SoftwareKmsProvider`

```rust
/// 软件 KMS 提供者（用于开发和测试）
pub struct SoftwareKmsProvider {
    keys: RwLock<HashMap<String, KeyData>>,
    key_store_path: PathBuf,
}

impl SoftwareKmsProvider {
    pub fn new(key_store_path: PathBuf) -> Self {
        Self {
            keys: RwLock::new(HashMap::new()),
            key_store_path,
        }
    }
}

#[derive(Debug, Clone)]
struct KeyData {
    metadata: KeyMetadata,
    private_key: Vec<u8>,
    public_key: Vec<u8>,
}
```

---

## 六、TPM 2.0 集成

### 6.1 TPM 操作

```rust
/// TPM 提供者
pub struct TpmProvider {
    context: tpm2::Context,
    primary_handle: tpm2::Handle,
}

impl KmsProvider for TpmProvider {
    fn sign(&self, key_id: &str, data: &[u8]) -> Result<Vec<u8>, KmsError> {
        // 使用 TPM 进行签名
        let key_handle = self.load_key_from_tpm(key_id)?;
        self.tpm.sign(key_handle, data, HashAlgorithm::Sha256)
    }

    fn verify(&self, key_id: &str, data: &[u8], signature: &[u8]) -> Result<bool, KmsError> {
        let key_handle = self.load_key_from_tpm(key_id)?;
        self.tpm.verify(key_handle, data, signature)
    }
}
```

---

## 七、云 KMS 集成

### 7.1 AWS KMS

```rust
/// AWS KMS 提供者
pub struct AwsKmsProvider {
    client: aws_kms::Client,
    region: Region,
}

impl KmsProvider for AwsKmsProvider {
    fn sign(&self, key_id: &str, data: &[u8]) -> Result<Vec<u8>, KmsError> {
        let output = self.client.sign()
            .key_id(key_id)
            .message(Cow::Borrowed(data))
            .signing_algorithm(SigningAlgorithm::EcdsaSha256)
            .send()?;

        output.signature()
            .map(|s| s.as_ref().to_vec())
            .ok_or(KmsError::SignatureNotFound)
    }
}
```

---

## 八、密钥轮换

### 8.1 轮换策略

```rust
/// 密钥轮换管理器
pub struct KeyRotationManager {
    provider: Arc<dyn KmsProvider>,
    rotation_scheduler: RotationScheduler,
}

impl KeyRotationManager {
    /// 执行密钥轮换
    pub async fn rotate(&self, key_id: &str) -> Result<KeyMetadata, Error> {
        // 1. 创建新密钥
        let new_key = self.provider.create_key(&self.get_rotation_spec(key_id)?)?;

        // 2. 重新加密使用旧密钥加密的数据
        self.re_encrypt_data(key_id, &new_key.key_id).await?;

        // 3. 标记旧密钥为轮换状态
        self.provider.update_key_status(key_id, KeyStatus::Rotated)?;

        Ok(new_key)
    }
}
```

---

## 九、实现状态

| 阶段 | 任务 | 状态 | PR |
|------|------|------|-----|
| 1 | 软件 Provider | ✅ | #1025 |
| 2 | 密钥管理 | ✅ | #1025 |
| 3 | 签名操作 | ✅ | #1025 |
| 4 | TPM Provider | 🔜 | - |
| 5 | 云 KMS Provider | 🔜 | - |

---

## 十、配置

### 10.1 配置文件

```yaml
# HSM/KMS 配置
kms:
  provider: SOFTWARE  # SOFTWARE/TPM/AWS_KMS/Azure_KV

  software:
    key_store_path: /var/lib/sqlrustgo/keys
    key_rotation_days: 90

  tpm:
    device: /dev/tpm0
    primary_handle: 0x81000000

  aws_kms:
    region: us-east-1
    key_id: alias/sqlrustgo-master-key
```

---

*本文档由 hermes-agent 创建*
*版本 1.0 - 2026-05-16*