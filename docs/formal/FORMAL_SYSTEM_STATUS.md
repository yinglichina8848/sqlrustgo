# Formal Verification System — Phase B Status

**状态**: 🔒 **FROZEN (Phase B)**
**生效日期**: 2026-05-03
**退出条件**: CI gate 稳定运行 ≥ 7 天，Nightly 无 flaky

---

## 冻结范围

|| 允许 | 禁止 |
||------|------|
|| 运行现有 proofs | 添加新 proof |
|| 修复 bug | 扩展 formal domains (如 SSI) |
|| CI 稳定性调优 | 研究新序列化算法 |
|| TEST_MAPPING 补齐 | 新增 TLA+ 模型 |

---

## 当前系统状态

|| 能力 | 状态 | CI Layer |
||------|------|----------|
|| Deadlock (PROOF-023) | ✅ 完整闭环 (S0-S5) | PR Gate |
|| TOCTOU | ✅ 已形式化 + 已工程修复 | PR Gate |
|| MVCC (PROOF-016) | ✅ Spec→Code→Test | PR Gate |
|| Write Skew (PROOF-026) | ⚠️ 只证明问题存在，未闭环 | Smoke only |
|| B+Tree (PROOF-004) | ✅ Dafny 验证 | Nightly |

---

## CI 三层架构 ✅

|| 层级 | 触发 | 内容 | 成本 |
||------|------|------|------|
|| PR Gate | 每个PR | `formal_smoke.sh` (5 models) + deadlock/mvcc tests | < 10 min |
|| Nightly | 每日 | 全量 Rust + TLA+ 中等模型 | 10-60 min |
|| Chaos Weekly | 每周一 03:00 UTC | `chaos_test.sh` 自动注入 bug 验证 CI | ~15 min |
|| Proof Gate | 手动 | 全量 formal suite | 60+ min |

**文件**:
- `.github/workflows/formal-smoke-pr.yml` — PR Gate
- `.github/workflows/chaos-test-weekly.yml` — Chaos 验证
- `scripts/formal/formal_smoke.sh` — Smoke runner
- `scripts/formal/chaos_test.sh` — Chaos injector
- `scripts/formal/select_formal_tests.sh` — Proof selector

---

## chaos_test 验证（核心约束力保证）

```bash
./scripts/formal/chaos_test.sh --inject-deadlock  # 注入 bug（注释 would_create_cycle）
./scripts/formal/chaos_test.sh --verify            # CI 应该 FAIL → 有约束力
./scripts/formal/chaos_test.sh --restore           # 恢复原状
```

**原理**: 人为注入 bug，验证 CI 能否检测。如果检测不出来 → 系统是假的。

---

## Proof Selector Fallback

改动 > 5 个文件 → 跑全量 proof suite（防止跨模块 bug 漏检）

```bash
if changed_files > 5:
    run FULL proof suite
```

---

## Invariant Anti-Trivial 保护（四层）

|| 保护层 | 机制 |
||--------|------|
|| 1 | cfg 必须声明 INVARIANT（`formal_smoke.sh` 检查） |
|| 2 | TLA+ 模型语义由 human review 保证 |
|| 3 | chaos_test weekly 验证 CI 有效性 |
|| 4 | Code review reviewer 检查 invariant 有意义 |

---

## PROOF-026 已知风险区域

```
Status: KNOWN UNSAFE AREA
Guarantee: System prevents deadlock
Risk: Write skew possible (PROOF_026_write_skew VIOLATED)
Mitigation: 不用于需要 serializable 隔离的生产负载
Deferred to: Phase C
```

---

## Phase B 退出 Checklist

- [x] S0-S5 死锁防护方向全部闭环
- [x] formal_smoke.sh < 2 min
- [x] chaos_test.sh 有约束力（验证过）
- [x] TEST_MAPPING 覆盖 ≥ 95%
- [ ] CI gate 稳定运行 ≥ 7 天（观察中）
- [ ] Branch protection 开启 `enable_status_check: true`（Gitea 设置）
- [ ] `phase-b/formal-enforcement` 推送远程并 PR → `develop/v2.9.0`

---

## S0-S5 验证结果（PROOF-023 死锁防护）

| 阶段 | 内容 | 状态 |
|------|------|------|
| S0 | TLA+ 模型文件存在 | ✅ |
| S1 | TOCTOU 模型 Violated | ✅ |
| S2 | Atomic 模型 Passed | ✅ |
| S3 | Rust DeadlockDetector 实现 | ✅ |
| S4 | Rust 并发测试通过 | ✅ |
| S5 | 完整闭环 | ✅ |

**S0-S5 总评: ✅ 全部完成**

---

## PROOF-026 SSI（未完成，Phase C 处理）

| 方面 | 状态 |
|------|------|
| Write Skew 反例 | ✅ VIOLATED（TLA+ 证明存在） |
| SSI TLA+ 模型 | ⚠️ commitTs 语义不够强 |
| Rust SerializationGraph | 🔴 未实现 |
| 并发 SSI 测试 | ✅ 14 tests |

---

*最后更新: 2026-05-03*
