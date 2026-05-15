# v3.2.0 Alpha Gate Checklist

> **版本**: v3.2.0-alpha-gate
> **创建日期**: 2026-05-15
> **维护人**: hermes-agent
> **阶段**: Alpha
> **关联门禁规范**: `docs/governance/GATE_SPEC_MASTER.md`

---

## 一、门禁信息

### 1.1 门禁定义

| 属性 | 值 |
|------|-----|
| 门禁类型 | Alpha Gate |
| 执行日期 | TBD |
| 执行人 | TBD |
| 脚本 | `scripts/gate/check_alpha_v320.sh` |
| 规范版本 | gate_spec_v320.md |

### 1.2 入口条件

- [ ] M1: GMP 基础框架完成 (GMP-1 数字签名审计链, GMP-6 Trusted Timestamp, OO-1)
- [ ] M2: Immutable Record + Correction Chain 完成 (GMP-3, GMP-4, OO-3, OO-4)
- [ ] 所有 P0 功能代码已提交
- [ ] 单元测试覆盖率 ≥60%
- [ ] 基础编译成功

---

## 二、Pre-Gate 自检清单

### 2.1 代码质量检查

| 检查项 | 命令 | 期望结果 | 状态 |
|--------|------|----------|------|
| cargo build | `cargo build --all-features` | 编译成功 | ⬜ |
| cargo clippy | `cargo clippy --all-features -- -D warnings` | 零警告 | ⬜ |
| cargo fmt | `cargo fmt --all -- --check` | 通过 | ⬜ |
| cargo test | `cargo test --lib` | 全部通过 | ⬜ |

### 2.2 覆盖率检查

| 检查项 | 命令 | 期望结果 | 实际结果 | 状态 |
|--------|------|----------|----------|------|
| L1 覆盖率 | `cargo llvm-cov` | ≥75% | TBD | ⬜ |

---

## 三、正式门禁检查 (Alpha)

### 3.1 核心检查 (A1-A8)

| # | 检查项 | 命令 | 期望结果 | 实际结果 | 状态 |
|---|--------|------|----------|----------|------|
| A1 | Build | `cargo build --all-features` | 成功 | TBD | ⬜ |
| A2 | L1 Test | `cargo test --lib` | ≥90% | TBD | ⬜ |
| A3 | Clippy | `cargo clippy --all-features -- -D warnings` | 零警告 | TBD | ⬜ |
| A4 | Format | `cargo fmt --check` | 通过 | TBD | ⬜ |
| A5 | Coverage | `cargo llvm-cov --lib` | ≥75% | TBD | ⬜ |
| A6 | HSM/KMS | `cargo test -p hsm_kms` | 单元测试通过 | TBD | ⬜ |
| A7 | MySQL Protocol | flush optimization | 已应用 | TBD | ⬜ |
| A8 | OO Documentation | docs/releases/v3.2.0/oo/ | 文档存在 | TBD | ⬜ |

### 3.2 稳定性测试 (A-S1~S3)

| # | 检查项 | 命令 | 期望结果 | 实际结果 | 状态 |
|---|--------|------|----------|----------|------|
| A-S1 | hsm_unit_test | `cargo test --test hsm_unit_test` | PASS | TBD | ⬜ |
| A-S2 | protocol_batch_write_test | `cargo test --test protocol_batch_write_test` | PASS | TBD | ⬜ |
| A-S3 | basic_transaction_test | `cargo test --test basic_transaction_test` | PASS | TBD | ⬜ |

---

## 四、检查结果汇总

### 4.1 通过情况

| 类别 | 通过数 | 总数 | 通过率 |
|------|--------|------|--------|
| 核心检查 A1-A8 | 0 | 8 | TBD% |
| 稳定性测试 A-S1~S3 | 0 | 3 | TBD% |
| **总计** | **0** | **11** | **TBD%** |

### 4.2 Alpha Gate 最终结果

```
=== v3.2.0 Alpha Gate ===
A1: Build ................... ⬜ PENDING
A2: L1 Test (≥90%) ......... ⬜ PENDING
A3: Clippy .................. ⬜ PENDING
A4: Format .................. ⬜ PENDING
A5: Coverage (≥75%) ........ ⬜ PENDING
A6: HSM/KMS ................ ⬜ PENDING
A7: MySQL Protocol ......... ⬜ PENDING
A8: OO Documentation ........ ⬜ PENDING
A-S1: hsm_unit_test ........ ⬜ PENDING
A-S2: protocol_batch_write . ⬜ PENDING
A-S3: basic_transaction .... ⬜ PENDING

Alpha Gate: 0/11 PASS ⬜
RESULT: PENDING
```

---

## 五、失败项处理

### 5.1 失败项记录

| Issue | 描述 | 解决方案 | 状态 |
|-------|------|----------|------|
| 无 | - | - | - |

### 5.2 豁免申请

无

---

## 六、Post-Gate 收尾

### 6.1 文档更新

- [ ] 更新 DEVELOPMENT_PLAN.md (标记 M1, M2 完成)
- [ ] 更新 TEST_PLAN.md
- [ ] 更新 CHANGELOG.md

### 6.2 分支操作

- [ ] 创建 `alpha/v3.2.0` 分支
- [ ] 归档开发分支

---

## 七、审查与签名

| 角色 | 姓名 | 日期 | 签名 |
|------|------|------|------|
| 执行人 | TBD | TBD | ⬜ |
| 审查人 | TBD | TBD | ⬜ |

---

## 八、附录

### A.1 相关文档

- 门禁规范: `docs/governance/GATE_SPEC_MASTER.md`
- 测试计划: `docs/releases/v3.2.0/TEST_PLAN.md`
- 开发计划: `docs/releases/v3.2.0/DEVELOPMENT_PLAN.md`
- OO 文档索引: `docs/releases/v3.2.0/oo/README.md`

### A.2 门禁脚本

- Alpha Gate 脚本: `scripts/gate/check_alpha_v320.sh`

### A.3 M1/M2 里程碑交付物

#### M1: GMP 基础框架

| 任务 | 交付物 | 验收条件 |
|------|--------|----------|
| GMP-1 | 数字签名审计链 | sign/verify API 完成 |
| GMP-6 | Trusted Timestamp | RFC3161 集成 |
| OO-1 | 签名链设计文档 | 文档完成 |

#### M2: Immutable Record + Correction Chain

| 任务 | 交付物 | 验收条件 |
|------|--------|----------|
| GMP-3 | Immutable Record | CREATE TABLE IMMUTABLE |
| GMP-4 | Correction Chain | CORRECT RECORD 语句 |
| OO-3 | EBR 设计文档 | 文档完成 |
| OO-4 | Correction 设计文档 | 文档完成 |

---

*最后更新: 2026-05-15*
