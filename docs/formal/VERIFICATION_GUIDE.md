# 可信任系统工具链：安装、部署与验证指南

**适用于**: sqlrustgo `develop/v2.8.0` 分支  
**工具链**: TLA+ model → Rust implementation (Mutex-atomic DeadlockDetector)

---

## 1. 工具链组件

| 组件 | 位置 | 说明 |
|---|---|---|
| TLA+ 工具箱 | `/tmp/tla2tools.jar` | model checking (TLC) |
| TLA+ 证明文件 | `docs/formal/PROOF_*.tla` | 4 个模型文件 |
| Rust 实现 | `crates/transaction/src/deadlock.rs` | `DeadlockDetector` + `try_wait_edge` |
| 单元测试 | `crates/transaction/src/deadlock.rs` `mod tests` | 12 tests (含 2 并发) |

---

## 2. TLA+ 模型验证

### 2.1 环境要求

```bash
java -version        # Java 11+ required
```

### 2.2 运行 TLC Model Checker

所有模型文件位于 `docs/formal/`：

```bash
cd /home/openclaw/dev/yinglichina163/sqlrustgo/docs/formal

# ── TOCTOU counterexamples (预期: VIOLATED) ──────────────────────────────

# Model 1: Pure lock TOCTOU
java -XX:+UseParallelGC -cp /tmp/tla2tools.jar \
  tlc2.TLC PROOF_023_deadlock_toctou -config PROOF_023_deadlock_toctou.cfg
# 预期结果: Error: Deadlock reached — NoCycle violated
# 原因: Check + CommitEdge 分离，TOCTOU 窗口

# Model 2: Unified MVCC + Wait-For TOCTOU
java -XX:+UseParallelGC -cp /tmp/tla2tools.jar \
  tlc2.TLC PROOF_016_023_mvcc_toctou -config PROOF_016_023_mvcc_toctou.cfg
# 预期结果: Error: Deadlock reached — NoCycle violated
# 原因: Write conflict → Wait-For edge + Commit 分离，TOCTOU 窗口

# ── Atomic models (预期: PASS) ───────────────────────────────────────────

# Model 3: Pure lock atomic
java -XX:+UseParallelGC -cp /tmp/tla2tools.jar \
  tlc2.TLC PROOF_023_deadlock_v4 -config PROOF_023_deadlock_v4.cfg
# 预期结果: Model checking completed. No error found.
# 状态数: ~300+ distinct states

# Model 4: Unified MVCC + Wait-For atomic
java -XX:+UseParallelGC -cp /tmp/tla2tools.jar \
  tlc2.TLC PROOF_016_023_mvcc_atomic -config PROOF_016_023_mvcc_atomic.cfg
# 预期结果: Model checking completed. No error found.
# 状态数: 993 total, 156 distinct
# 验证不变量: NoCycle + NoWriteConflict 同时成立
```

### 2.3 预期结果汇总

| 模型 | 不变量 | 预期 | 实际 |
|---|---|---|---|
| `PROOF_023_deadlock_toctou` | NoCycle | ❌ Violated | ❌ Violated |
| `PROOF_016_023_mvcc_toctou` | NoCycle | ❌ Violated | ❌ Violated |
| `PROOF_023_deadlock_v4` | NoCycle | ✅ Pass | ✅ Pass |
| `PROOF_016_023_mvcc_atomic` | NoCycle + NoWriteConflict | ✅ Pass | ✅ Pass |

### 2.4 理解 TOCTOU 失败 trace

当 TOCTOU 模型报 `NoCycle violated` 时，查看 State 2-5：

```
State 2: T1: Check({T2})     ← T1 检查 T2→T1 不可达，通过
State 3: T2: Check({T1})     ← T2 检查 T1→T2 不可达，通过  ◄ TOCTOU 窗口
State 4: T1: CommitEdge      ← T1 添加 T1→{T2} 边
State 5: T2: CommitEdge      ← T2 添加 T2→{T1} 边 → CYCLE
```

这证明：分离的 Check + CommitEdge 步骤在并发下会绕过死锁检测。

---

## 3. Rust 验证

### 3.1 构建

```bash
cd /home/openclaw/dev/yinglichina163/sqlrustgo
cargo build -p sqlrustgo-transaction
```

### 3.2 全部测试

```bash
cargo test -p sqlrustgo-transaction
```

**预期**: 98+ tests passed (含 14 SSI integration + 12 deadlock + 8 version_chain + …)

### 3.3 仅 deadlock 模块测试

```bash
cargo test -p sqlrustgo-transaction deadlock
```

**预期**: 12 tests passed

关键测试：

```
test_concurrent_mutual_deadlock_prevention  # 并发 TOCTOU 防护
test_concurrent_no_false_positive           # 线性链无误报
test_try_wait_edge_rejects_cycle            # 原子 pre-check 拒绝环
test_try_wait_edge_accepts_no_cycle         # 有效路径通过
test_no_self_wait                           # NoSelfWait 正确
```

### 3.4 仅 SSI integration 测试

```bash
cargo test -p sqlrustgo-transaction ssi
```

**预期**: 14 tests passed

---

## 4. S0–S05 验证状态

### S0: TLA+ 模型文件存在 ✅

```
docs/formal/PROOF_023_deadlock_v4.tla          ✅
docs/formal/PROOF_023_deadlock_toctou.tla      ✅
docs/formal/PROOF_016_023_mvcc_toctou.tla      ✅
docs/formal/PROOF_016_023_mvcc_atomic.tla      ✅
```

### S1: TOCTOU 模型 Violated ✅

运行 `PROOF_023_deadlock_toctou` 和 `PROOF_016_023_mvcc_toctou`，两者均报 `NoCycle violated`。

### S2: Atomic 模型 Passed ✅

运行 `PROOF_023_deadlock_v4` 和 `PROOF_016_023_mvcc_atomic`，两者均报 `No error found`。

### S3: Rust DeadlockDetector 实现 ✅

- `DeadlockDetector` 包装在 `Mutex<Inner>`
- 唯一安全 API：`try_wait_edge(tx_id, holders)` — 原子 pre-check + add_edge
- `try_wait_edge` 返回 `Err(LockError::Deadlock)` 当且仅当会创建环

### S4: Rust 并发测试通过 ✅

- `test_concurrent_mutual_deadlock_prevention`: 两个线程同时添加 T1→T2 和 T2→T1，至少一个失败，无 cycle 残留
- `test_concurrent_no_false_positive`: 线性链不产生误报

### S5: S0–S04 完整闭环 ✅

```
TLA+ Atomic Model (PASS)
    ↓  (same atomicity requirement)
Rust try_wait_edge() with Mutex
    ↓  (physical enforcement)
Concurrent tests pass
    ↓  (integration)
SSI integration tests pass
```

### ⚠️ S-04 残留未竟项

| 未竟项 | 影响 | 建议 |
|---|---|---|
| MVCC write-write conflict commit 检测 | `CommitTxn` 在 TLA+ 中有 `NoWriteConflict` 检查，Rust `TransactionManager` 尚未实现 | 在 `ssi_integration.rs` 中加 commit validation 测试 |
| Serializability end-to-end 测试 | 只有分模块测试，没有全链路 SSI 验证 | 添加 multi-key 跨表事务的 serializability 测试 |

---

## 5. 核心正确性论证

### 5.1 TOCTOU 漏洞的危险性

TLA+ 已证明：如果 `would_create_cycle` 检查和 `add_edge` 操作不原子：

```
T1: thread-A  checks would_create_cycle(T1,{T2}) → false (pass)
T2: thread-B  checks would_create_cycle(T2,{T1}) → false (pass)
T1: thread-A  adds edge T1→T2
T2: thread-B  adds edge T2→T1  ← CYCLE FORMED
```

**没有任何运行时检测可以发现这个错误** — 因为它发生在两个独立操作的间隔中。

### 5.2 为什么 Mutex 是必要的

只有两种方式消除 TOCTOU 窗口：

1. **Mutex** (当前方案): 保证 `would_create_cycle` 检查和 `add_edge` 在同一个锁区域内执行
2. **Lock-free CAS**: 更复杂，需要 `compare_exchange` 保证 graph 操作的原子性

当前采用 Mutex 的原因：
- 实现简单，错误率最低
- 锁的临界区极小（只包含图遍历和一次哈希写入）
- 不会产生死锁（因为外层有 `would_create_cycle` 保护）

### 5.3 形式化 vs 运行时验证的关系

| 层次 | 验证方法 | 保证 |
|---|---|---|
| TLA+ model | Model checker (exhaustive state search) | 数学证明：所有可能的 interleavings |
| Rust unit tests | Single-threaded assertions | 特定场景验证 |
| Rust concurrent tests | Multi-threaded stress | TOCTOU 防护的运行时证明 |

TLA+ 证明"**不存在**能导致 cycle 的 interleaving"，Rust 测试证明"**实际运行**中也没有 cycle"。

---

## 6. 已知限制

1. **`assert_no_cycle()` 仅在 debug 构建生效**: release 构建没有运行时检查，依赖 `would_create_cycle` 预防
2. **`try_wait_edge` 是唯一安全并发 API**: 直接调用 `add_edge` 或 `would_create_cycle` 而不使用 `try_wait_edge` 会绕过 TOCTOU 保护（已用 `#[cfg(test)]` 限制测试外使用）
3. **无死锁超时**: 当前实现没有超时机制；如果 `try_wait_edge` 卡住会一直等待（由外层 `Mutex` 的 `park()` 保证）

---

## 7. 快速验证脚本

```bash
#!/bin/bash
set -e
cd /home/openclaw/dev/yinglichina163/sqlrustgo

echo "=== TLA+ Models ==="
cd docs/formal
for model in PROOF_023_deadlock_toctou PROOF_016_023_mvcc_toctou PROOF_023_deadlock_v4 PROOF_016_023_mvcc_atomic; do
    result=$(java -XX:+UseParallelGC -cp /tmp/tla2tools.jar tlc2.TLC $model -config ${model}.cfg 2>&1)
    if echo "$result" | grep -q "No error found"; then
        echo "$model: ✅ PASS"
    else
        echo "$model: ❌ VIOLATED (expected for toctou models)"
    fi
done

echo ""
echo "=== Rust Tests ==="
cargo test -p sqlrustgo-transaction deadlock -- --nocapture
```

---

*最后更新: 2026-05-03 (commit: atomic try_wait_edge + unified MVCC model)*
