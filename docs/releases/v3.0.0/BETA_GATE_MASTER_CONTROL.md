# v3.0.0 Beta Gate — 阶段门禁任务总控

> **版本**: 1.0
> **日期**: 2026-05-07
> **分支**: `develop/v3.0.0`
> **门禁规范**: `docs/governance/gate_spec_v300.md` (SSOT)
> **状态**: 🔴 未启动

---

## 一、Beta Gate 入口条件

Beta Gate 检查前必须满足：

1. **A-Gate 已通过** — Phase 0-4 开发任务完成
2. **无 P0/P1 Bug** — 所有已知阻塞性问题已关闭
3. **覆盖率 ≥50%** — A-Gate 覆盖率门槛

---

## 二、Beta Gate 检查清单（14 项）

### 基础质量门禁 (B1-B7)

| ID | 检查项 | 命令 | 通过标准 | 状态 |
|----|--------|------|---------|------|
| B1 | Release Build | `cargo build --release --workspace` | 无错误 | ⏳ |
| B2 | 全量测试 | `cargo test --all-features` | ≥90% 通过 | ⏳ |
| B3 | Clippy | `cargo clippy --all-features -- -D warnings` | 零警告 | ⏳ |
| B4 | Format | `cargo fmt --all -- --check` | 无 diff | ⏳ |
| B5 | 覆盖率 | `cargo llvm-cov --all-features` | ≥75% | ⏳ |
| B6 | 安全扫描 | `cargo audit` | 无高危漏洞 | ⏳ |
| B7 | 文档链接 | `bash scripts/gate/check_docs_links.sh` | 零死链 | ⏳ |

### SQL/TPC-H 门禁 (B8-B9)

| ID | 检查项 | 命令 | 通过标准 | 状态 |
|----|--------|------|---------|------|
| B8 | TPC-H SF=0.1 | `scripts/gate/check_tpch.sh sf=0.1` | 22/22 无 OOM | ⏳ |
| B9 | SQL Corpus | `cargo test -p sqlrustgo-sql-corpus` | ≥85% | ⏳ |

### 稳定性测试门禁 (B-S1~B-S5) — v3.0.0 新增

| ID | 检查项 | 命令 | 通过标准 | 状态 | 历史基线 |
|----|--------|------|---------|------|---------|
| **B-S1** | 压力测试 | `cargo test --test concurrency_stress_test` | 全部通过 | ⏳ | v2.4.0: 41 tests ✅ |
| **B-S2** | 崩溃恢复 | `cargo test --test crash_recovery_test` | 全部通过 | ⏳ | v2.4.0: 9 tests ✅ |
| **B-S3** | 长稳测试 | `cargo test --test long_run_stability_test` | 全部通过 | ⏳ | v2.4.0: 10 tests ✅ |
| **B-S4** | WAL 集成 | `cargo test --test wal_integration_test` | 全部通过 | ⏳ | v2.4.0: ✅ |
| **B-S5** | KILL 压力 | `cargo test --test network_tcp_smoke_test` | 全部通过 | ⏳ | v2.9.0: 8 tests ✅ |

**Beta Gate 通过标准**: B1-B9 + B-S1~B-S5 全部 PASS，BLOCKERS = 0

---

## 三、开发任务与测试任务映射

### 3.1 开发任务（对应 Issue）

| Issue | 标题 | 负责人 | 验收条件 | 阻塞门禁 |
|-------|------|--------|---------|---------|
| #376 | Sysbench OLTP 适配 | opencode | 3 场景全部通过 + QPS 可测量 | B-S1 |
| #377 | COM_MULTI 多语句执行 | opencode | sysbench prepare 通过 | B-S5 |
| #378 | Prepared Statement 参数绑定修复 | opencode | 参数化查询正确 | B-S5 |
| #379 | 事务状态机压力测试 | claude | 100 并发无泄漏 | B-S2 |
| #380 | Optimizer 测试扩展 | claude | 覆盖率 ≥70% | B5 |
| #381 | Planner 测试扩展 | claude | 覆盖率 ≥80% | B5 |
| #382 | TPC-H SF=1 CI Gate | TBD | check_tpch.sh --sf1 可运行 | B8 |

### 3.2 稳定性测试任务（对应 Issue #394-398）

| Issue | 测试文件 | 对应门禁 | 验收条件 |
|-------|---------|---------|---------|
| #394 | `concurrency_stress_test.rs` | B-S1 | 并发读/写/死锁检测全部 PASS |
| #395 | `crash_recovery_test.rs` | B-S2 | 9/9 PASS，WAL 正确恢复 |
| #396 | `long_run_stability_test.rs` | B-S3 | 10/10 PASS，内存增长 <10MB |
| #397 | `wal_integration_test.rs` | B-S4 | 崩溃后零数据丢失 |
| #398 | `network_tcp_smoke_test.rs` | B-S5 | KILL 后无连接泄漏 |

---

## 四、Agent 分工

### opencode（开发任务 #376-#378）

```
分支: feat/opencode-sysbench / feat/opencode-com-multi
工作目录: ~/workspace/dev/openheart/sqlrustgo
```

| 任务 | Issue | 工时 | 验收标准 |
|------|-------|------|---------|
| Sysbench OLTP 适配 | #376 | 3d | oltp_read_only/write_only/read_write 全部可运行 |
| COM_MULTI 多语句 | #377 | 2d | sysbench prepare 阶段通过 |
| Prepared Statement 绑定 | #378 | 1d | 参数化查询占位符正确替换 |

**验收命令**:
```bash
# Sysbench
bash scripts/gate/check_sysbench.sh

# COM_MULTI + Prepared Statement
mysql -h 127.0.0.1 -P 6033 -u root -e "SELECT 1; SELECT 2; SELECT 3"
mysql -h 127.0.0.1 -P 6033 -u root -e "PREPARE p1 FROM 'SELECT ?+?'; SET @a=1; SET @b=2; EXECUTE p1 USING @a,@b"
```

### claude（开发任务 #379-#381）

```
分支: feat/claude-v3-tx-stress
工作目录: ~/workspace/dev/yinglichina163/sqlrustgo (需同步到 Gitea)
```

| 任务 | Issue | 工时 | 验收标准 |
|------|-------|------|---------|
| 事务状态机压力测试 | #379 | 2d | 100 并发 BEGIN/COMMIT/ROLLBACK 无状态泄漏 |
| Optimizer 测试扩展 | #380 | 2d | 覆盖率 ≥70%，Predicate Pushdown + Projection Pruning |
| Planner 测试扩展 | #381 | 2d | 覆盖率 ≥80%，SELECT/INSERT/UPDATE/DELETE/JOIN 计划正确 |

**验收命令**:
```bash
# 事务压力
cargo test -p sqlrustgo-transaction stress
cargo test --test mvcc_transaction_test

# 覆盖率
cargo llvm-cov --all-features --lcov --output-path /tmp/lcov.info
# optimizer ≥70%, planner ≥80%

# B-S2 崩溃恢复（所有 agent 共享）
cargo test --test crash_recovery_test
```

---

## 五、执行流程

```
[Phase 1] 开发完成 → Phase 2 开发完成
              ↓
[Phase 3] 门禁自检（本地）
bash scripts/gate/check_beta_v300.sh
              ↓
         BLOCKERS > 0?
            /    \
          YES     NO
           ↓       ↓
    修复问题    提交 PR
           ↓       ↓
    重新自检    review
                  ↓
              PR 合并
                  ↓
         [Phase 4] Beta Gate 正式验证
         bash scripts/gate/check_beta_v300.sh
                  ↓
             BLOCKERS = 0?
                  ↓
            ✅ Beta Gate PASSED
```

---

## 六、门禁脚本

### 本地自检脚本

```bash
# 在 develop/v3.0.0 分支上运行
cd ~/workspace/dev/openheart/sqlrustgo
git checkout develop/v3.0.0
git pull origin develop/v3.0.0

# 运行 Beta Gate 检查（完整 14 项）
bash scripts/gate/check_beta_v300.sh
```

### 预期输出

```
=== v3.0.0 Beta Gate ===
[beta-v3.0.0] B1: cargo build --release --workspace ... PASS
[beta-v3.0.0] B2: cargo test --all-features (≥90%) ... PASS (92% = 138/150)
[beta-v3.0.0] B3: cargo clippy --all-features ... PASS
[beta-v3.0.0] B4: cargo fmt --check ... PASS
[beta-v3.0.0] B5: Coverage ≥75% ... PASS (84.18%)
[beta-v3.0.0] B6: cargo audit ... PASS
[beta-v3.0.0] B7: check_docs_links.sh ... PASS
[beta-v3.0.0] B8: TPC-H SF=0.1 (22/22) ... PASS (22/22)
[beta-v3.0.0] B9: SQL Corpus ≥85% ... PASS (100%)
[beta-v3.0.0] B-S1: concurrency_stress_test ... PASS
[beta-v3.0.0] B-S2: crash_recovery_test ... PASS
[beta-v3.0.0] B-S3: long_run_stability_test ... PASS
[beta-v3.0.0] B-S4: wal_integration_test ... PASS
[beta-v3.0.0] B-S5: network_tcp_smoke_test ... PASS

=== Beta Gate Results: PASS=14 / 14, BLOCKERS=0 ===
✅ Beta Gate PASSED
```

---

## 七、风险与阻塞项

### 当前阻塞项

| 阻塞项 | 影响 | 缓解措施 |
|--------|------|---------|
| #376 Sysbench OLTP 未完成 | B-S1 压力测试无法通过 | opencode 优先处理 |
| #377 COM_MULTI 未完成 | B-S5 KILL 压力测试无法通过 | opencode 优先处理 |
| #378 Prepared Statement 未完成 | B-S5 KILL 压力测试无法通过 | opencode 优先处理 |
| #379 事务状态机未完成 | B-S2 崩溃恢复无法验证 | claude 优先处理 |

### 豁免申请

如遇以下情况可申请 Beta Gate 豁免（需 Architect 审批）：

1. **B-S 稳定性测试基础设施不可用**（CI runner 问题）→ DevOps Lead 审批
2. **特定测试文件不存在**（历史版本曾有但当前缺失）→ Tech Lead 审批
3. **性能环境不稳定**（测试结果波动）→ QA Lead 审批

---

## 八、版本对齐

Beta Gate 通过后：

1. 创建 `beta/v3.0.0` 分支
2. 更新 `docs/releases/v3.0.0/CHANGELOG.md`
3. 更新 `docs/releases/v3.0.0/RELEASE_NOTES.md` Beta 阶段状态
4. 关闭 Beta Gate 后进入 **R-Gate (RC Gate)** 准备阶段

---

## 九、相关文档

| 文档 | 说明 |
|------|------|
| `docs/governance/gate_spec_v300.md` | Beta Gate 完整定义（SSOT） |
| `docs/releases/v3.0.0/AGENT_TASKS.md` | Agent 分工与提示词 |
| `docs/releases/v3.0.0/B_GROUP_IMPLEMENTATION_PLAN.md` | TPC-H 扩展计划 |
| `scripts/gate/check_beta_v300.sh` | Beta Gate 检查脚本 |

---

*本文档由 hermes-z6g4 创建，是 v3.0.0 Beta Gate 的唯一权威任务总控文档。*
*门禁规范变更必须同步更新本文档和 gate_spec_v300.md。*
