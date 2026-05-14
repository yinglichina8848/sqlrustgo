# 密钥生命周期管理 SOP

**版本**: v3.1.0  
**日期**: 2026-05-14  
**状态**: Beta 完成

---

## 一、密钥类型

| 密钥类型 | 用途 | 算法 | 轮换周期 | 状态 |
|----------|------|------|----------|------|
| 审计签名密钥 | 审计事件签名 | RSA-2048 | 90 天 | ⚠️ |
| TLS 证书 | 传输加密 | RSA-2048 | 1 年 | ⚠️ |
| 存储加密密钥 | 数据加密 (v3.2.0) | AES-256 | 1 年 | 规划中 |

---

## 二、密钥生成

### 2.1 审计签名密钥生成

```bash
#!/bin/bash
# 生成审计签名密钥

# 1. 生成私钥
openssl genrsa -out audit_private.pem 2048

# 2. 提取公钥
openssl rsa -in audit_private.pem -pubout -out audit_public.pem

# 3. 设置权限
chmod 600 audit_private.pem
chmod 644 audit_public.pem

# 4. 验证密钥
openssl rsa -in audit_private.pem -check
```

### 2.2 密钥生成标准

| 参数 | 值 |
|------|-----|
| 算法 | RSA |
| 密钥长度 | 2048 bit (最低) |
| 推荐长度 | 3072 bit |
| 指数 | 65537 |
| 格式 | PKCS#8 |

---

## 三、密钥存储

### 3.1 安全存储要求

| 要求 | 说明 |
|------|------|
| 加密存储 | 密钥必须加密存储 |
| 访问控制 | 最小权限原则 |
| 审计日志 | 所有访问必须记录 |
| 备份 | 加密备份 |

### 3.2 推荐存储方案

```yaml
# 选项 1: 文件系统 (测试/开发)
key_storage:
  type: file
  path: /etc/sqlrustgo/keys
  permissions: "0600"
  backup: true

# 选项 2: HSM (生产环境)
key_storage:
  type: hsm
  provider: aws-kms  # 或: azure-keyvault, gcp-kms
  key_id: arn:aws:kms:us-east-1:123456789012:key/1234abcd

# 选项 3: Kubernetes Secret
key_storage:
  type: k8s_secret
  secret_name: sqlrustgo-audit-keys
  namespace: sqlrustgo
```

---

## 四、密钥轮换

### 4.1 轮换流程

```
Day 0          Day 90         Day 180
   |              |               |
   v              v               v
[生成密钥] -> [开始轮换] -> [完成轮换]
                   |               |
                   v               v
              [新旧密钥共存]  [新密钥单独运行]
```

### 4.2 审计签名密钥轮换 SOP

```bash
#!/bin/bash
# 审计签名密钥轮换脚本

set -euo pipefail

# 配置
KEY_DIR="/etc/sqlrustgo/keys"
BACKUP_DIR="/var/backup/keys"
RETENTION_DAYS=730

# 1. 备份当前密钥
cp "$KEY_DIR/audit_private.pem" "$BACKUP_DIR/audit_private.pem.$(date +%Y%m%d)"
cp "$KEY_DIR/audit_public.pem" "$BACKUP_DIR/audit_public.pem.$(date +%Y%m%d)"

# 2. 生成新密钥
openssl genrsa -out "$KEY_DIR/audit_private.pem.new" 2048
openssl rsa -in "$KEY_DIR/audit_private.pem.new" -pubout -out "$KEY_DIR/audit_public.pem.new"

# 3. 部署新密钥 (原子操作)
mv "$KEY_DIR/audit_private.pem" "$KEY_DIR/audit_private.pem.old"
mv "$KEY_DIR/audit_public.pem" "$KEY_DIR/audit_public.pem.old"
mv "$KEY_DIR/audit_private.pem.new" "$KEY_DIR/audit_private.pem"
mv "$KEY_DIR/audit_public.pem.new" "$KEY_DIR/audit_public.pem"

# 4. 重新加载服务
./sqlrustgo admin reload-keys --type audit

# 5. 验证
./sqlrustgo audit verify --chain

# 6. 清理旧密钥 (保留 2 年)
find "$BACKUP_DIR" -name "audit_private.pem.*" -mtime +$RETENTION_DAYS -delete
```

### 4.3 TLS 证书轮换 SOP

```bash
#!/bin/bash
# TLS 证书轮换

# 1. 生成 CSR
openssl req -new -key server.key -out server.csr

# 2. 提交给 CA
# (通过外部流程)

# 3. 安装新证书
cp new_cert.pem /etc/sqlrustgo/tls/server.crt

# 4. 重新加载
./sqlrustgo admin reload-config

# 5. 验证
openssl s_client -connect localhost:3306 -showcerts
```

---

## 五、密钥吊销

### 5.1 吊销流程

```bash
#!/bin/bash
# 密钥吊销 SOP

# 1. 标识密钥
KEY_ID="audit-key-$(date +%Y%m%d)"

# 2. 生成吊销列表
echo "revoked: $KEY_ID at $(date)" >> /etc/sqlrustgo/keys/revocation.log

# 3. 吊销检查
./sqlrustgo admin verify-key --id $KEY_ID --revocation-list /etc/sqlrustgo/keys/revocation.log

# 4. 通知相关方
./sqlrustgo admin notify-key-revocation --key-id $KEY_ID
```

### 5.2 吊销检查

```rust
impl KeyVerifier {
    fn is_revoked(&self, key_id: &str) -> bool {
        let revocation_list = self.load_revocation_list();
        revocation_list.contains(&key_id.to_string())
    }
}
```

---

## 六、密钥审计

### 6.1 审计事件

| 事件 | 级别 | 说明 |
|------|------|------|
| KEY_GENERATED | INFO | 新密钥生成 |
| KEY_ROTATED | INFO | 密钥轮换 |
| KEY_REVOKED | WARN | 密钥吊销 |
| KEY_ACCESSED | DEBUG | 密钥访问 |
| KEY_VERIFY_FAILED | WARN | 验签失败 |

### 6.2 审计日志格式

```json
{
  "event_type": "KEY_ROTATED",
  "key_id": "audit-key-20260514",
  "key_type": "RSA-2048",
  "operator": "ops@company.com",
  "timestamp": "2026-05-14T10:00:00Z",
  "result": "success"
}
```

---

## 七、应急响应

### 7.1 密钥泄露响应

```bash
#!/bin/bash
# 密钥泄露响应 SOP

# 1. 立即吊销
./sqlrustgo admin revoke-key --key-id COMPROMISED_KEY_ID

# 2. 生成新密钥
openssl genrsa -out audit_private.pem.new 2048

# 3. 部署新密钥
mv audit_private.pem.new audit_private.pem

# 4. 重新加载
./sqlrustgo admin reload-keys --type audit

# 5. 通知
# - 安全团队
# - 合规团队
# - 受影响用户

# 6. 评估影响
# - 哪些审计事件使用泄露密钥签名？
# - 是否需要重新签名？
```

### 7.2 联系信息

| 角色 | 职责 | 联系 |
|------|------|------|
| 安全团队 | 密钥泄露响应 | security@company.com |
| 合规团队 | GMP 合规 | compliance@company.com |
| 密钥管理员 | 日常管理 | keys-admin@company.com |