# GMP 合规性变更日志

**版本**: v3.1.0  
**日期**: 2026-05-14  
**用途**: 记录所有影响 GMP 合规性的变更

---

## 概述

本文档记录 SQLRustGo 所有影响 GMP 合规性的变更，包括：
- 签名算法变更
- WAL 格式变更
- 审计格式变更
- 安全配置变更
- 密钥管理变更

---

## 变更分类

| 类别 | 说明 | 严重性 |
|------|------|--------|
| **Breaking** | 可能破坏兼容性或审计链 | Critical |
| **Security** | 安全相关变更 | High |
| **Format** | 数据格式变更 | High |
| **Config** | 配置项变更 | Medium |
| **Feature** | 新功能 | Low |

---

## v3.1.0 变更记录

### 2026-05-14: v3.1.0 Beta 发布

| 类别 | 变更 | 影响 | 风险 |
|------|------|------|------|
| **Breaking** | Ed25519 签名替代 RSA-2048 | 签名验证需更新 | 低 |
| **Breaking** | WAL 格式 v2 替代 v1 | 旧版 WAL 不可用 | 中 |
| **Feature** | 新增 HashChain 审计模块 | 增强可审计性 | 无 |
| **Feature** | 新增 SSI 隔离级别 | 改善并发安全 | 无 |
| **Security** | TLS 1.2+ 强制 | 禁用旧 TLS | 低 |

### 变更详情

#### Ed25519 签名替代 RSA-2048

**变更日期**: 2026-05-14  
**变更类型**: Breaking  
**严重性**: Low  

**原因**:
- Ed25519 性能更好 (签名: 0.01ms vs 0.3ms)
- 密钥更小 (32 bytes vs 256 bytes)
- 更现代、更安全

**影响**:
- 旧版签名密钥不可用
- 审计历史中的旧签名无法验证
- 建议: 在升级前完成审计备份

**迁移步骤**:
```bash
# 1. 备份现有审计历史
sqlrustgo-cli audit export > audit_backup_$(date +%Y%m%d).json

# 2. 升级数据库
sqlrustgo --upgrade

# 3. 验证升级
sqlrustgo-cli audit verify
```

#### WAL 格式 v2

**变更日期**: 2026-05-14  
**变更类型**: Breaking  
**严重性**: Medium  

**变更内容**:
- 新增 Checksum 字段 (CRC32)
- 新增 PagePointer 字段
- 压缩元组头部格式

**影响**:
- v3.0 WAL 无法直接在 v3.1.0 恢复
- 需先在 v3.0 关闭数据库，再升级

**迁移步骤**:
```bash
# 1. 在 v3.0 中正常关闭
sqlrustgo-cli shutdown

# 2. 备份 WAL 文件
cp -r /var/lib/sqlrustgo/wal /backup/wal_v3.0

# 3. 升级到 v3.1.0
sqlrustgo --upgrade

# 4. 验证 WAL 格式
sqlrustgo-cli wal status
# 应显示: WAL_VERSION=2
```

---

## v3.0.0 变更记录

### 2025-12-01: v3.0.0 GA 发布

| 类别 | 变更 | 影响 | 风险 |
|------|------|------|------|
| **Feature** | 初始 GMP 功能集 | 基准版本 | 无 |
| **Security** | RBAC 权限系统 | 增强安全 | 无 |
| **Security** | bcrypt 密码哈希 | 增强安全 | 无 |
| **Feature** | 审计日志基础 | 基础可审计性 | 无 |

---

## 未来变更预告

### v3.2.0 (计划中)

| 类别 | 变更 | 严重性 | 状态 |
|------|------|--------|------|
| **Security** | MFA (TOTP) | High | 计划 |
| **Security** | 审计存储加密 (AES-256-GCM) | High | 计划 |
| **Security** | HSM/KMS 集成 | Medium | 计划 |
| **Format** | WAL v3 格式 | Medium | 计划 |
| **Feature** | 列级权限 | Medium | 计划 |

---

## 变更影响评估模板

```markdown
## 变更影响评估

**变更 ID**: CHANGE-YYYY-NNN
**日期**: YYYY-MM-DD
**变更类型**: Breaking / Security / Format / Config / Feature
**严重性**: Critical / High / Medium / Low

### 变更描述
[详细描述变更内容]

### 影响分析
- 数据迁移: [是/否]
- 配置变更: [是/否]
- 密钥轮换: [是/否]
- 审计链影响: [是/否]

### 风险评估
| 风险 | 影响 | 可能性 | 缓解措施 |
|------|------|--------|----------|
| ... | ... | ... | ... |

### 迁移步骤
1. [步骤 1]
2. [步骤 2]
3. [步骤 3]

### 回滚计划
[如何回滚到变更前状态]

### 验证步骤
1. [验证 1]
2. [验证 2]
```

---

## 合规性检查清单

在每个版本发布前，确认：

- [ ] 所有 Breaking 变更已记录
- [ ] 所有 Security 变更已记录
- [ ] 迁移步骤已测试
- [ ] 回滚计划已测试
- [ ] 审计链完整性已验证
- [ ] 文档已更新

---

## 签名

| 角色 | 姓名 | 日期 | 签名 |
|------|------|------|------|
| 变更责任人 | | | |
| 安全审查 | | | |
| QA 审查 | | | |
| 合规审查 | | | |