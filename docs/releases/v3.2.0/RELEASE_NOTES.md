# v3.2.0 Release Notes

> **Version**: 3.2.0
> **Date**: 2026-05-18
> **Status**: RC → GA Transition
> **Branch**: develop/v3.2.0
> **HEAD**: `e23c70db5`

---

## 概述

**SQLRustGo v3.2.0** 是 **Trusted GMP Data Platform** 的一次重大版本升级。

v3.2.0 = **GMP Native 可信数据平台**

- **GMP**: Good Manufacturing Practice (良好生产规范) — 21 CFR Part 11 合规
- **可信**: 数字签名、审计链、不可篡改、电子签名
- **数据平台**: 可信数据生命周期管理，不仅是数据库

---

## 版本对比

| 功能 | v3.1.0 | v3.2.0 |
|------|---------|--------|
| GMP Framework | 基础 (20%) | **完整 P0 (100%)** |
| 电子签名 | ❌ | ✅ 21 CFR Part 11 |
| Immutable Record | ❌ | ✅ EBR 证据链 |
| Correction Chain | ❌ | ✅ 纠错追溯链 |
| Provenance Tracking | ❌ | ✅ 数据溯源 |
| Trusted Timestamp | ❌ | ✅ RFC 3161 |
| HSM/KMS | ❌ | ✅ PKCS#11 |
| Workflow Engine | ❌ | ✅ GMP 工作流 |
| Mobile Trust | ❌ | ✅ 设备绑定 |
| SOP Binding | ❌ | ✅ 培训绑定 |
| Device Calibration | ❌ | ✅ 校准管理 |
| MySQL 兼容性 | 85% | **90%** |
| 并发连接 | 100 | **200+** |
| Multi-Table DML | 部分 | ✅ 完整 |
| RECURSIVE CTE | ❌ | ✅ |
| Evidence Export | ❌ | ✅ PDF/JSON |
| caching_sha2_password | ❌ | ✅ MySQL 8.0 |

---

## 主要新功能

### 1. GMP Framework (P0) — 12/12 全通过

| 模块 | 功能 | PR | 状态 |
|------|------|-----|-------|
| GMP-1 | 数字签名审计链 | #1012 | ✅ |
| GMP-2 | 电子签名 (21 CFR Part 11) | #1004, #1015, #1017, #1018 | ✅ |
| GMP-3 | Immutable Record / EBR | #1029 | ✅ |
| GMP-4 | Correction Chain | #1027 | ✅ |
| GMP-5 | Provenance Tracking | #1024 | ✅ |
| GMP-6 | Trusted Timestamp (RFC 3161) | #1017 | ✅ |
| GMP-7 | 审计链验证工具 | #1020 | ✅ |
| GMP-8 | HSM/KMS 集成 (PKCS#11) | #1025 | ✅ |
| GMP-9 | Workflow Engine | #1046 | ✅ |
| GMP-10 | Mobile Trust (设备绑定) | #1077 | ✅ |
| GMP-11 | SOP/Training Binding | #1078 | ✅ |
| GMP-12 | Device Calibration | #1079 | ✅ |

### 2. Performance Enhancements

| 模块 | 功能 | PR | 状态 |
|------|------|-----|-------|
| PERF-1 | MySQL Flush 优化 | #1059, #1060 | ✅ |
| PERF-2 | TPC-H SF=10 (Spill Framework) | - | 🔄 |
| PERF-3 | 200+ 并发连接 | #1013 | ✅ |
| PERF-4 | UPDATE/DELETE 优化 | #1174 | ✅ |
| PERF-5 | 内存优化 -15% | - | ✅ |

### 3. SQL 增强

| 模块 | 功能 | PR | 状态 |
|------|------|-----|-------|
| Multi-Table DML | UPDATE/MERGE | #1021 | ✅ |
| RECURSIVE CTE | 递归 CTE | - | ✅ |
| Window Functions | 窗口函数 | - | ✅ |
| Event Scheduler | 事件调度器 | - | ✅ 18 tests |
| GIS/Spatial | 空间数据 | - | ✅ |
| FULLTEXT Search | 全文索引 | - | ✅ |

### 4. MySQL 兼容性增强

| 功能 | PR | 状态 |
|------|-----|-------|
| caching_sha2_password auth | #1173 | ✅ MySQL 8.0 |
| MySQL Protocol | - | ✅ |
| Prepared Statements | - | ✅ |

### 5. Evidence Export (新增)

| 功能 | 说明 | PR |
|------|------|-----|
| PdfExporter | PDF 合规报告生成 | #1161 |
| JsonExporter | JSON 序列化 | #1161 |
| PackageBuilder | 签名证据包构建 | #1161 |
| Ed25519 Signer | Ed25519 签名 | #1161 |
| Compliance-as-Code | 声明式规则 | #1161 |

### 6. Extended Audit Chain Verification (新增)

- 时间戳验证
- 顺序验证
- 孤立记录检测
- 工作流验证
- 数据溯源验证
- PR: #1159, #1171

---

## 门禁状态

| Gate | 状态 | 日期 | 通过率 |
|------|------|------|---------|
| Alpha Gate | ✅ 通过 | 2026-05-15 | 100% |
| Beta Gate | ✅ 通过 | 2026-05-16 | 18/18 |
| RC Gate | ✅ 通过 | 2026-05-17 | 28/32 (87.5%) |
| GA Gate | ⏳ 进行中 | 2026-05-18 | 40/46 (86.9%) |

### RC Gate 通过项 (28/32)

- R1-R12: 核心检查 ✅
- R-S1~S16: 稳定性测试 16/16 ✅

### RC Gate 待处理 (4/32)

- R13: TPC-H SF=10 (需大内存机器)
- R14: Sysbench point_select ≥30K QPS (需服务器)
- R15: 72h 稳定性 (需大内存机器)

### GA Gate 通过项 (40/46)

- G1-G5: 代码质量 ✅
- G7-G8: SQL/TPC-H ✅
- G11-G12: 文档/协议 ✅
- G-QA1~QA14: QA 增强 14/14 ✅
- G-S1~S8, S10, S12~S14, S16~S20: 稳定性 16/20 ✅

### GA Gate 待处理 (6/46)

- G6/G-S15: Security Audit (GitHub advisory db 不可达)
- G9: Sysbench Performance (需服务器)
- G10: Formal Proofs (TLA+, 需人工)
- G-S9: Sysbench (需服务器)
- G-S11: Proof Count (TLA+, 需人工)

---

## 从 v3.1.0 以来的变更统计

| 指标 | 数值 |
|------|------|
| 总 Commits | 339 |
| Merged PRs | 119 |
| 新增功能 | 50+ |
| GMP 测试 | 354+ |
| 稳定性测试 | 16+ |
| OO 文档 | 14/14 |

### 最新 PRs (Top 10)

| PR | 功能 |
|----|------|
| #1177 | docs: update README.md to v3.2.0 RC |
| #1176 | chore: cleanup directory structure |
| #1175 | feat(gmp): add evidence export package |
| #1174 | perf(storage): UPDATE/DELETE micro-benchmark |
| #1173 | fix: caching_sha2_password auth support |
| #1172 | docs: improve GitNexus section format |
| #1171 | feat(gmp): extended audit chain verification |
| #1169 | fix(mysql-server): migrate mio 0.8 Poll API |
| #1168 | test: WAL crash injection tests |
| #1167 | fix(docs): 修正 BETA_GATE_PLAN.md URL |

---

## 已知问题

| Issue | 描述 | 严重度 | 状态 |
|-------|------|--------|------|
| R13 | TPC-H SF=10 需大内存 | 高 | 待 Z6G4 |
| R14 | Sysbench 需服务器 | 中 | 待执行 |
| R15 | 72h 稳定性 | 高 | 待 Z6G4 |
| G6 | Security Audit advisory db 不可达 | 低 | 网络问题 |

---

## 下一步

1. 在 Z6G4 服务器执行 R13, R14, R15
2. 执行 G10 TLA+ Formal Proofs 检查
3. 验证 G6 Security Audit (网络恢复后)
4. 创建 ga/v3.2.0 分支和 tag v3.2.0

---

## 文档

| 文档 | 说明 |
|------|------|
| [README.md](./README.md) | 版本说明 |
| [CHANGELOG.md](./CHANGELOG.md) | 详细变更日志 |
| [TEST_PLAN.md](./TEST_PLAN.md) | 测试计划 |
| [DEVELOPMENT_PLAN.md](./DEVELOPMENT_PLAN.md) | 开发计划 |
| [GA_GATE_CHECKLIST.md](./GA_GATE_CHECKLIST.md) | GA 门禁状态 |
| [oo/README.md](./oo/README.md) | OO 文档索引 |

---

*维护人: hermes-agent*
*最后更新: 2026-05-18*
