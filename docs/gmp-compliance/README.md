# GMP 合规性文档总览

**版本**: v3.1.0  
**日期**: 2026-05-14  
**状态**: Beta 完成，RC 阶段

---

## 文档结构

```
docs/gmp-compliance/
├── proof/                    # 形式化证明矩阵
│   ├── PROOF_INDEX.md        # 证明索引与状态
│   ├── PROOF-003-wal-recovery-spec.md
│   ├── PROOF-005-mvcc-snapshot-spec.md
│   └── PROOF-012-wal-acid-spec.md
├── stability/                # 稳定性与压力测试
│   ├── STABILITY_REPORT.md   # 稳定性测试报告
│   ├── STRESS_TEST_REPORT.md # 压力测试报告
│   └── SSI_ISOLATION_REPORT.md
├── audit/                   # 审计系统文档
│   ├── AUDIT_THREAT_MODEL.md # 威胁模型
│   ├── KEY_MANAGEMENT_SOP.md # 密钥生命周期
│   └── AUDIT_VERIFICATION.md # 在线验证机制
├── security/                # 安全审查
│   ├── RC_GATE_R6_R11.md    # R6-R11 详细状态
│   └── R_S1_S5_SECURITY.md  # R-S1~R-S5 安全审查
├── coverage/                # 覆盖率分层
│   └── COVERAGE_BY_MODULE.md # 分模块覆盖率
└── deployment/              # 部署运维
    ├── CONFIG_HARDENING.md  # 配置加固指南
    ├── OPS_MANUAL.md        # 运维手册
    └── SBOM.md              # 依赖清单与漏洞扫描
```

---

## 快速导航

| 类别 | 文档 | 状态 |
|------|------|------|
| 形式化证明 | [PROOF_INDEX.md](proof/PROOF_INDEX.md) | ✅ |
| 稳定性测试 | [STABILITY_REPORT.md](stability/STABILITY_REPORT.md) | ✅ |
| 审计系统 | [AUDIT_THREAT_MODEL.md](audit/AUDIT_THREAT_MODEL.md) | ✅ |
| 安全审查 | [RC_GATE_R6_R11.md](security/RC_GATE_R6_R11.md) | ✅ |
| 覆盖率 | [COVERAGE_BY_MODULE.md](coverage/COVERAGE_BY_MODULE.md) | ✅ |
| 部署运维 | [CONFIG_HARDENING.md](deployment/CONFIG_HARDENING.md) | ✅ |

---

## GMP 合规性总结

### 可审计 ✅
- 31 个形式化证明文件
- Hash Chain + Digital Signature
- 完整审计事件流

### 可信任 ✅
- WAL + Crash Recovery
- Recovery Trust Chain
- 72h 稳定性验证

### 完整性 ✅
- Beta Gate 18/18 PASS
- L1 Coverage 81.67%
- SQL Corpus 80.0%