# v3.2.0 README

> **Version**: 3.2.0
> **Date**: 2026-05-15
> **Status**: Beta Phase

---

## SQLRustGo v3.2.0

**Trusted GMP Data Platform**

SQLRustGo v3.2.0 是 GMP (Good Manufacturing Practice) Native 可信数据平台。

### 核心特性

| 特性 | 说明 |
|------|------|
| **GMP Framework** | 完整 GMP 合规支持 |
| **电子签名** | 21 CFR Part 11 合规 |
| **审计链** | 数字签名 + 哈希链 |
| **Immutable Record** | 不可篡改记录 |
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

| Gate | 状态 |
|------|------|
| Alpha Gate | 🟡 条件性通过 |
| Beta Gate | ⏸️ 进行中 |
| RC Gate | ⏸️ 等待 |
| GA Gate | ⏸️ 等待 |

---

## 里程碑

```
v3.2.0 ─── Alpha ✅ ─── Beta 🔄 ─── RC 🔄 ─── GA
             │          │
          M1-M4 ✅    M5-M8 🔄
```

详见 [DEVELOPMENT_PLAN.md](./DEVELOPMENT_PLAN.md)

---

## 资源

- **代码**: https://github.com/openclaw/sqlrustgo
- **内部**: http://192.168.0.252:3000/openclaw/sqlrustgo
- **Wiki**: http://192.168.0.252:3000/openclaw/sqlrustgo-wiki

---

**维护人**: hermes-z6g4
**生成日期**: 2026-05-15
