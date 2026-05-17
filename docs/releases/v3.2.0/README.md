# v3.2.0 README

> **Version**: 3.2.0
> **Date**: 2026-05-16
> **Status**: Beta → RC Transition

---

## SQLRustGo v3.2.0

**Trusted GMP Data Platform**

SQLRustGo v3.2.0 是 GMP (Good Manufacturing Practice) Native 可信数据平台。

### 核心特性

| 特性 | 说明 |
|------|------|
| **GMP Framework** | 完整 GMP 合规支持 (9 模块) |
| **电子签名** | 21 CFR Part 11 合规 |
| **审计链** | 数字签名 + 哈希链 |
| **Immutable Record** | 不可篡改记录 (EBR) |
| **Correction Chain** | 纠错追溯链 |
| **Provenance Tracking** | 数据溯源 |
| **Trusted Timestamp** | RFC 3161 可信时间戳 |
| **HSM/KMS 集成** | 硬件安全模块 |
| **Workflow Engine** | GMP 工作流引擎 |

---

## 快速开始

```bash
# 构建
cargo build --all-features

# 运行
cargo run --bin sqlrustgo

# 测试
cargo test --all-features
```

详见 [QUICK_START.md](./QUICK_START.md)

---

## 文档

| 文档 | 说明 |
|------|------|
| [DEVELOPMENT_PLAN.md](./DEVELOPMENT_PLAN.md) | 开发计划 |
| [TEST_PLAN.md](./TEST_PLAN.md) | 测试计划 |
| [RELEASE_NOTES.md](./RELEASE_NOTES.md) | 发布说明 |
| [CHANGELOG.md](./CHANGELOG.md) | 变更日志 |
| [INSTALL.md](./INSTALL.md) | 安装指南 |
| [DEPLOYMENT_GUIDE.md](./DEPLOYMENT_GUIDE.md) | 部署指南 |

---

## 门禁状态

| Gate | 状态 | 日期 |
|------|------|------|
| Alpha Gate | ✅ 通过 | 2026-05-15 |
| Beta Gate | ✅ 通过 (18/18) | 2026-05-16 |
| RC Gate | ⏸️ 进行中 | - |
| GA Gate | ⏸️ 等待 | - |

---

## 已完成 PR (2026-05-16)

| PR | 功能 |
|----|------|
| #1094 | sync: rc/v3.2.0 <- develop/v3.2.0 |
| #1093 | fix(storage): AWS S3 SigV4 signing |
| #1092 | chore: refresh reports, gate scripts |
| #1091 | feat(storage): 冷存储完善 (S3签名 + StorageTierManager) |
| #1090 | feat(catalog): DCL 权限链 (RowLevelSecurity + 角色嵌套) |

---

## 里程碑

```
v3.2.0 ─── Alpha ✅ ─── Beta ✅ ─── RC 🔄 ─── GA
             │          │          │
          M1-M4 ✅    M5-M8 ✅   R1-R16 🔄
```

详见 [DEVELOPMENT_PLAN.md](./DEVELOPMENT_PLAN.md)

---

## 资源

- **代码**: https://github.com/openclaw/sqlrustgo
- **内部**: http://192.168.0.252:3000/openclaw/sqlrustgo
- **Wiki**: http://192.168.0.252:3000/openclaw/sqlrustgo-wiki

---

**维护人**: hermes-z6g4
**生成日期**: 2026-05-16