# v3.2.0 Beta Gate 开发计划

> **版本**: v3.2.0
> **创建日期**: 2026-05-15
> **维护人**: hermes-z6g4
> **Milestone**: Beta Gate
> **前置条件**: Alpha Gate 必须通过

---

## 一、概述与目标

### 1.1 版本目标

完成 Beta Gate 所有 P1 任务，达到 Beta Gate 准入条件。

### 1.2 关键交付物

| 交付物 | 类型 | 关联 Issue |
|--------|------|-------------|
| GMP-9 GMP Workflow Engine | Feature | #908 |
| GMP-10 移动端可信采集 | Feature | #909 |
| GMP-11 SOP/培训绑定 | Feature | #910 |
| GMP-12 Device Calibration | Feature | #911 |
| PERF-4 死锁检测 <50ms | Performance | #923 |
| SQL-1 RECURSIVE CTE | Feature | #930 |
| SQL-2 Performance Schema | Feature | #931 |

### 1.3 与前一版本的差异

| 方面 | v3.1.0 | v3.2.0 | 变更说明 |
|------|---------|---------|----------|
| 核心功能 | Alpha Gate | Beta Gate | P1 功能完整实现 |
| Alpha Gate | 阻塞中 | 进行中 | PERF-1 修复中 |
| Beta Gate | 未开始 | 进行中 | 8 个 P1 任务 |

### 1.4 成功的定义

- [ ] Alpha Gate 全部通过 (13/13)
- [ ] 所有 P1 任务完成 (8/8)
- [ ] Beta Gate 门禁通过 (25/25)

---

## 二、功能列表

### 2.1 P0 任务 (Alpha Gate 必须项)

| 任务 | Issue | 状态 | PR |
|------|-------|------|-----|
| GMP-1 数字签名审计链 | #900 | ✅ 完成 | #1012 |
| GMP-2 电子签名 | #901 | ✅ 完成 | #1004 |
| GMP-3 Immutable Record | #902 | ✅ 完成 | #1029 |
| GMP-4 Correction Chain | #903 | ✅ 完成 | #1023 |
| GMP-5 Provenance Tracking | #904 | ✅ 完成 | #1024 |
| GMP-6 Trusted Timestamp | #905 | ✅ 完成 | #1017 |
| GMP-7 审计链验证工具 | #906 | ✅ 完成 | #1020 |
| GMP-8 HSM/KMS 集成 | #907 | ✅ 完成 | #1025 |
| PERF-1 MySQL flush | #920 | 🔄 修复中 | - |

### 2.2 P1 任务 (Beta Gate 必须项)

| 任务 | Issue | 计划工时 | 状态 | 依赖 |
|------|-------|----------|------|------|
| GMP-9 Workflow Engine | #908 | 3周 | ✅ 完成 | GMP-2, GMP-7 |
| GMP-10 移动端采集 | #909 | 2周 | ✅ 完成 | GMP-6 |
| GMP-11 SOP绑定 | #910 | 2周 | ✅ 完成 | GMP-2 |
| GMP-12 Device Calibration | #911 | 1周 | ✅ 完成 | GMP-6 |
| PERF-3 并发 200+ | #922 | 1周 | ✅ 完成 | - |
| PERF-4 死锁检测 <50ms | #923 | 1周 | ❌ 未开始 | - |
| SQL-1 RECURSIVE CTE | #930 | 2周 | ✅ 完成 | - |
| SQL-2 Performance Schema | #931 | 2周 | ✅ 完成 | - |

### 2.3 P2 任务 (v3.3.0)

| 任务 | Issue | 状态 |
|------|-------|------|
| SQL-4 组复制 | #933 | 规划中 |
| SQL-5 故障转移 | #934 | 规划中 |
| SQL-6 地理分布 | #935 | 规划中 |

---

## 三、技术任务

### 3.1 架构任务

无重大架构变更。

### 3.2 依赖升级

| 依赖 | 当前版本 | 目标版本 | 理由 |
|------|----------|----------|------|
| tokio | 1.x | 保持 | 稳定 |
| serde | 1.x | 保持 | 稳定 |

### 3.3 工具链

- Rust 1.85+
- cargo clippy
- cargo llvm-cov (覆盖率)
- cargo test

---

## 四、测试策略

### 4.1 单元测试

每个 P1 功能需要单元测试覆盖 >= 80%。

### 4.2 集成测试

| 测试 | 目标 | Issue |
|------|------|-------|
| gmp_workflow_test | GMP-9 | #908 |
| gmp_mobile_test | GMP-10 | #909 |
| gmp_sop_test | GMP-11 | #910 |
| gmp_calibration_test | GMP-12 | #911 |
| deadlock_detection_test | PERF-4 | #923 |
| recursive_cte_test | SQL-1 | #930 |
| performance_schema_test | SQL-2 | #931 |

### 4.3 Beta Gate 门禁脚本

```bash
bash scripts/gate/check_beta_v320.sh
```

---

## 五、门禁计划

### 5.1 Alpha Gate (当前状态)

| 检查项 | 状态 | 说明 |
|--------|------|------|
| A1 Build | ✅ | |
| A2 L1 test >= 90% | ✅ | |
| A3 Clippy | ✅ | |
| A4 Format | ❌ | trailing whitespace |
| A5 Coverage >= 75% | ✅ | 80.94% |
| A6 HSM/KMS | ❌ | electronic_signature.rs bug |
| A7 MySQL Protocol | ✅ | |
| A8 OO Docs | ✅ | |
| A-S1 Stability | ❌ | 测试不存在 |
| A-S2 Crash Recovery | ❌ | 测试不存在 |
| A-S3 Long Run | ❌ | 测试不存在 |

### 5.2 Beta Gate 准入条件

- [ ] Alpha Gate 全部通过
- [ ] 所有 P0 任务完成
- [ ] P1 任务全部实现

### 5.3 Beta Gate 检查项

| 检查项 | 描述 | 目标 | 状态 |
|--------|------|------|------|
| B1 | Build | PASS | - |
| B2 | L1 test >= 90% | PASS | - |
| B3 | Clippy | PASS | - |
| B4 | Format | PASS | - |
| B5 | Coverage >= 75% | PASS | - |
| B6 | Security | PASS | - |
| B7 | SQL Compat >= 80% | PASS | - |
| B8 | TPC-H SF=1 | PASS | - |
| B9 | OO Docs | PASS | - |
| B10 | GMP-9 Workflow | PASS | ❌ |
| B11 | GMP-10 Mobile | PASS | ❌ |
| B12 | GMP-11 SOP | PASS | ❌ |
| B13 | GMP-12 Calibration | PASS | ❌ |
| B14 | PERF-4 Deadlock | PASS | ❌ |
| B15 | SQL-1 CTE | PASS | ❌ |
| B16 | SQL-2 Perf Schema | PASS | ❌ |

---

## 六、版本延续任务

### 6.1 v3.1.0 未完成任务

| 任务 | v3.1.0 状态 | v3.2.0 计划 |
|------|--------------|--------------|
| PERF-1 MySQL flush | 阻塞 Alpha | 修复中 |
| PERF-3 并发 200+ | 已完成 | 保持 |

---

## 七、风险评估

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| GMP-9 Workflow Engine 复杂度高 | 高 | 3周时间缓冲 |
| SQL-1 RECURSIVE CTE 实现难度 | 中 | 2周时间缓冲 |
| Alpha Gate 阻塞项修复时间 | 中 | 优先修复 |

---

## 八、里程碑

| 里程碑 | 目标日期 | 说明 |
|--------|----------|------|
| Alpha Gate 通过 | 2026-05-15 | PERF-1 修复 |
| Beta Gate 开发完成 | 2026-05-22 | 所有 P1 完成 |
| Beta Gate 通过 | 2026-05-25 | 门禁验证 |

---

## 九、附录

### 9.1 相关文档

- [Alpha Gate 状态](../issues/973)
- [Beta Gate 状态](../issues/974)
- [总控Issue](../issues/972)

### 9.2 OpenSpec 设计文档

- `openspec/changes/gmp-4-correction-chain/` - GMP-4
- `openspec/changes/gmp-5-provenance-tracking/` - GMP-5
- `openspec/changes/gmp-8-hsm-kms-integration/` - GMP-8