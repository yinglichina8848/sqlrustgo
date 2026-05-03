# Formal Verification System — Phase B Status

**状态**: 🔒 **FROZEN (Phase B)**
**生效日期**: 2026-05-03
**退出条件**: CI gate 稳定运行 ≥ 7 天，Nightly 无 flaky

---

## 冻结范围

| 允许 | 禁止 |
|------|------|
| 运行现有 proofs | 添加新 proof |
| 修复 bug | 扩展 formal domains (如 SSI) |
| CI 稳定性调优 | 研究新序列化算法 |
| TEST_MAPPING 补齐 | 新增 TLA+ 模型 |

---

## 当前系统状态

| 能力 | 状态 | CI Layer |
|------|------|----------|
| Deadlock (PROOF-023) | ✅ 完整闭环 | PR Gate |
| TOCTOU | ✅ 已形式化 + 已工程修复 | PR Gate |
| MVCC (PROOF-016) | ✅ Spec→Code→Test | PR Gate |
| Write Skew (PROOF-026) | ⚠️ 只证明问题存在，未闭环 | Smoke only |
| B+Tree (PROOF-004) | ✅ Dafny 验证 | Nightly |

---

## CI 分层

| 层级 | 触发 | 内容 | 成本 |
|------|------|------|------|
| PR Gate | 每个PR | formal_smoke.sh (5 models) + deadlock/mvcc tests | < 10 min |
| Nightly | 每日 | 全量 Rust + TLA+ 中等模型 | 10-60 min |
| Chaos Weekly | 每周一 | chaos_test.sh 自动注入 bug 验证 CI | ~15 min |
| Proof Gate | 手动 | 全量 formal suite | 60+ min |

---

## chaos_test 验证

```bash
./scripts/formal/chaos_test.sh --inject-deadlock  # 注入 bug
./scripts/formal/chaos_test.sh --verify            # CI 应该 FAIL
./scripts/formal/chaos_test.sh --restore           # 恢复
```

### CI 自动 chaos test

`.github/workflows/chaos-test-weekly.yml` — 每周一 03:00 UTC 自动跑。

---

## Proof Selector Fallback

改动 > 5 个文件 → 跑全量 proof suite（防止跨模块 bug 漏检）

```bash
if changed_files > 5:
    run FULL proof suite
```

---

## Invariant Anti-Trivial 保护

| 保护层 | 机制 |
|--------|------|
| 1 | cfg 必须声明 INVARIANT（formal_smoke.sh 检查） |
| 2 | TLA+ 模型语义由 human review 保证 |
| 3 | chaos_test weekly 验证 CI 有效性 |
| 4 | Code review reviewer 检查 invariant 有意义 |

---

## PROOF-026 已知风险区域

```
Status: KNOWN UNSAFE AREA
Guarantee: System prevents deadlock
Risk: Write skew possible (PROOF_026_write_skew VIOLATED)
Mitigation: 不用于需要 serializable 隔离的生产负载
```

---

## Phase B 退出 Checklist

- [ ] CI gate 稳定运行 ≥ 7 天
- [ ] chaos_test weekly 无 fail
- [ ] Branch protection 开启 (Gitea)
- [ ] Formal smoke < 2 min
- [ ] TEST_MAPPING 覆盖 ≥ 95%

---

*最后更新: 2026-05-03*
