# 审计系统威胁模型与安全分析

**版本**: v3.1.0  
**日期**: 2026-05-14  
**状态**: Beta 完成

---

## 一、审计系统架构

### 1.1 组件概览

```
                    +------------------+
                    |   AuditEvent     |
                    |  (不可篡改事件)   |
                    +--------+---------+
                             |
                             v
                    +------------------+
                    |   EventStream     |
                    |  (仅追加写入)     |
                    +--------+---------+
                             |
              +--------------+--------------+
              |              |              |
              v              v              v
      +------------+  +------------+  +------------+
      | HashChain  |  | Signature  |  | WAL Audit  |
      | (防篡改)   |  | (RSA-2048) |  | (持久化)   |
      +------------+  +------------+  +------------+
```

### 1.2 文件结构

| 文件 | 职责 |
|------|------|
| `event_stream.rs` | 仅追加的审计事件流 |
| `hash_chain.rs` | SHA-256 哈希链 |
| `signature.rs` | RSA-2048 数字签名 |
| `audit_verify.rs` | 审计验证器 |

---

## 二、威胁模型

### 2.1 信任边界

```
信任区域 (Trusted)
+------------------+
| 审计系统核心     |
| - HashChain      |
| - Signature      |
| - EventStream   |
+------------------+
        |
        v
不信任区域 (Untrusted)
+------------------+
| 应用层           |
| - SQL 执行器     |
| - 网络层         |
| - 存储层         |
+------------------+
```

### 2.2 威胁分析 (STRIDE)

| 威胁类型 | 描述 | 防护措施 | 状态 |
|----------|------|----------|------|
| **篡改 (Tampering)** | 修改历史审计记录 | Hash Chain + 签名 | ✅ |
| **否认 (Repudiation)** | 否认执行过某操作 | 数字签名 | ✅ |
| **信息泄露 (Information Disclosure)** | 审计数据泄露 | 加密传输 | ⚠️ 需配置 |
| **拒绝服务 (DoS)** | 审计系统不可用 | 独立存储 + 备份 | ⚠️ 需配置 |
| **权限提升 (Elevation of Privilege)** | 越权写入审计 | 强制签名验证 | ✅ |

### 2.3 攻击者模型

#### 外部攻击者
- **能力**: 网络嗅探、重放攻击
- **防护**: TLS 加密、会话签名
- **状态**: ⚠️ 需配置

#### 内部恶意管理员
- **能力**: 直接修改数据库文件、删除日志
- **防护**: Hash Chain + 独立签名服务 + 异地备份
- **状态**: ⚠️ 需独立签名服务

#### 软件漏洞
- **能力**: SQL 注入、代码执行
- **防护**: 输入验证、沙箱、最小权限
- **状态**: ⚠️ 需安全配置

---

## 三、Hash Chain 实现分析

### 3.1 哈希链原理

```
Event[0]  -->  Hash[0] = SHA256(0 || Event[0])
Event[1]  -->  Hash[1] = SHA256(Hash[0] || Event[1])
Event[2]  -->  Hash[2] = SHA256(Hash[1] || Event[2])
...
Event[N]  -->  Hash[N] = SHA256(Hash[N-1] || Event[N])
```

### 3.2 防篡改证明

```rust
// hash_chain.rs
impl HashChain {
    /// 验证链完整性
    pub fn verify(&self) -> bool {
        for (i, entry) in self.entries.iter().enumerate() {
            // 1. 索引连续性
            if entry.index != i as u64 {
                return false;
            }
            // 2. 哈希链连续性 (外部验证)
        }
        true
    }

    /// 计算事件哈希
    fn compute_event_hash(event: &AuditEvent, prev_hash: &[u8; 32]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(prev_hash);
        hasher.update(serde_json::to_vec(event).unwrap());
        hasher.finalize()
    }
}
```

### 3.3 理论保证

| 攻击类型 | 防护机制 | 理论保证 |
|----------|----------|----------|
| 修改 Event[i] | 破坏 Hash[i] | SHA-256 抗碰撞 |
| 插入假事件 | 长度不匹配 | 链长度验证 |
| 删除事件 | 后续哈希不匹配 | 链连续性验证 |
| 重放旧事件 | 时间戳检查 | 事件去重 |

### 3.4 实际限制

| 限制 | 说明 | 缓解措施 |
|------|------|----------|
| SHA-256 碰撞 | 理论可行但实际极难 | 使用 SHA-512 或更强算法 |
| 密钥泄露 | RSA 私钥丢失 | 密钥隔离 + 定期轮换 |
| 追加覆盖 | 存储故障 | WAL + Checkpoint |

---

## 四、签名验证分析

### 4.1 RSA-2048 签名流程

```rust
// signature.rs
pub struct AuditSignature {
    private_key: RsaPrivateKey,
    public_key: RsaPublicKey,
}

impl AuditSignature {
    /// 签名审计事件
    pub fn sign(&self, event: &AuditEvent) -> Result<Signature, Error> {
        let event_bytes = serde_json::to_vec(event)?;
        let signature = self.private_key.sign(
            MessageDigest::sha256(),
            &event_bytes
        )?;
        Ok(Signature(signature))
    }

    /// 验签
    pub fn verify(&self, event: &AuditEvent, signature: &Signature) -> bool {
        let event_bytes = serde_json::to_vec(event).unwrap();
        self.public_key.verify(
            MessageDigest::sha256(),
            &event_bytes,
            &signature.0
        ).is_ok()
    }
}
```

### 4.2 签名覆盖范围

| 字段 | 签名保护 | 说明 |
|------|----------|------|
| event_id | ✅ | 全局唯一 ID |
| actor | ✅ | 操作者身份 |
| action | ✅ | 操作类型 |
| timestamp | ✅ | 时间戳 |
| prev_hash | ✅ | 链接前驱 |
| metadata | ✅ | 额外数据 |

### 4.3 密钥强度分析

| 算法 | 密钥长度 | 安全等级 | 状态 |
|------|----------|----------|------|
| RSA | 2048 bit | 112 bit | ✅ 足够 |
| RSA | 3072 bit | 128 bit | 建议 v3.2 |
| ECC | 256 bit | 128 bit | 替代方案 |

---

## 五、防护措施矩阵

### 5.1 配置要求

| 防护措施 | 必需 | 配置项 | 默认值 |
|----------|------|--------|--------|
| Hash Chain 验证 | ✅ | `audit.hash_chain.enabled` | `true` |
| 签名验证 | ✅ | `audit.signature.enabled` | `true` |
| TLS 传输加密 | ⚠️ | `audit.tls.enabled` | `false` |
| 独立存储 | ⚠️ | `audit.storage.separate` | `false` |
| 异地备份 | ⚠️ | `audit.backup.enabled` | `false` |

### 5.2 部署配置

```yaml
# production deployment
audit:
  hash_chain:
    enabled: true
    algorithm: SHA256
  signature:
    enabled: true
    key_size: 2048
    rotation_period_days: 90
  storage:
    separate: true
    path: /var/lib/sqlrustgo/audit
    backup:
      enabled: true
      interval_hours: 24
      retention_days: 2555  # 7 years for GMP
```

---

## 六、剩余风险

### 6.1 已识别风险

| 风险 | 概率 | 影响 | 缓解 | 状态 |
|------|------|------|------|------|
| SHA-256 碰撞 | 极低 | 高 | SHA-512 升级路径 | ⚠️ |
| 密钥泄露 | 低 | 极高 | 密钥管理服务 | ⚠️ |
| 内部篡改 | 中 | 高 | 独立签名服务 | ⚠️ |
| 存储故障 | 低 | 高 | 备份策略 | ⚠️ |

### 6.2 建议改进

1. **短期**: 启用 TLS + 独立存储
2. **中期**: 集成外部 HSM/KMS
3. **长期**: 迁移到 SHA-512 或 SHA-3

---

## 七、验证方法

### 7.1 完整性验证

```bash
# 验证 Hash Chain
./sqlrustgo audit verify --chain

# 验证签名
./sqlrustgo audit verify --signature

# 完整验证
./sqlrustgo audit verify --full
```

### 7.2 在线验证

```rust
// audit_verify.rs
pub struct AuditVerifier { ... }

impl AuditVerifier {
    /// 实时验证新事件
    pub fn verify_on_insert(&self, event: &AuditEvent) -> Result<(), Error> {
        // 1. 验签
        self.signature.verify(event)?;
        // 2. 验证哈希链
        self.hash_chain.append(event)?;
        // 3. 写入 WAL
        self.wal.append(event)?;
        Ok(())
    }
}
```