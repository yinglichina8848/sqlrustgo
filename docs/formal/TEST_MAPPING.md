# Formal Verification Test Mapping

**目的**: 建立 formal proof → implementation → test 的三层绑定关系。

---

## PROOF-023: Deadlock Freedom

### Spec → Invariant
- **TLA+ Invariants**: `NoCycle`, `NoSelfWait`
- **文件**: `docs/formal/PROOF_023_deadlock_v4.tla`

### Invariant → Code
```rust
// crates/transaction/src/deadlock.rs
pub fn try_wait_edge(&self, tx_id: TxnId, holders: HashSet<TxnId>)
    -> Result<(), LockError>
{
    let mut inner = self.inner.lock().unwrap();
    if holders.contains(&tx_id) { return Err(LockError::SelfWait); }
    if inner.would_create_cycle(tx_id, &holders) { return Err(LockError::Deadlock); }
    inner.add_edge(tx_id, holders);
    Ok(())
}
```

### Code → Test
- `test_concurrent_mutual_deadlock_prevention`
- `test_no_cycle_on_wait_graph`
- `test_try_wait_edge_rejects_cycle`

### CI 绑定
| 层级 | 触发条件 | 执行内容 |
|------|----------|----------|
| PR Gate | deadlock.rs | `cargo test -p sqlrustgo-transaction deadlock` |
| Smoke | deadlock_v4 | `formal_smoke.sh` |
| Chaos Weekly | 自动 | chaos_test.sh 注入 bug → 验证 fail |

---

## PROOF-026: Write Skew (SSI)

### Spec → Invariant
- **TLA+**: `NoWriteSkew`, `NoSerializationCycle`
- **文件**: `PROOF_026_write_skew.tla` (VIOLATED ✅)

### Invariant → Code
```rust
// TODO(PROOF-026):
// Current MVCC is NOT serializable.
// Write skew possible. See PROOF_026_write_skew.tla.
```

### CI 绑定
| 层级 | 触发条件 | 执行内容 |
|------|----------|----------|
| PR Gate | mvcc.rs | `cargo test -p sqlrustgo-transaction ssi` |
| Smoke | write_skew | `formal_smoke.sh` (VIOLATED 是预期) |

---

## Proof Cost Classification

| Proof | Area | Cost | CI Layer |
|-------|------|------|----------|
| PROOF_023_deadlock_v4 | transaction | S | PR Gate |
| PROOF_023_deadlock_toctou | transaction | S | PR Gate |
| PROOF_026_write_skew | transaction | S | Smoke |
| PROOF_016_023_mvcc_atomic | transaction | M | Nightly |
| PROOF_016_023_mvcc_toctou | transaction | M | Nightly |

---

## PR Diff → Proof Selector

```bash
if changed_files > 5: run FULL proof suite
```

| 文件 | Proof |
|------|-------|
| deadlock.rs | PROOF_023 |
| mvcc.rs | PROOF_016, PROOF_026 |
| wal.rs | WAL |
| docs/formal/* | FULL |

---

*最后更新: 2026-05-03*
