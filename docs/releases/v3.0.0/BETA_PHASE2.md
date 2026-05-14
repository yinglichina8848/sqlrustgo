# v3.0.0 Beta Phase 2 — 可信度闭环

> **版本**: 1.0
> **日期**: 2026-05-08
> **阶段**: Beta Phase 2（Beta Phase 1 = 功能框架已完成）
> **战略转向**: 功能开发 → 可信度闭环
> **核心理念**: Beta Phase 1 证明"能实现"，Beta Phase 2 证明"敢用于生产"

---

## 一、战略转向说明

### Beta Phase 1 (已完成)

```
完成内容:
  ✅ Audit Trail (hook + log table + SHA256 + query API)
  ✅ WAL Crash Validation
  ✅ Differential Testing (SQLite diff)
  ✅ INFORMATION_SCHEMA

这些属于"业务层功能"或"infra/tooling"，
特点是：实现快，可行性验证。
```

### Beta Phase 2 (当前)

```
核心问题: "能不能证明正确"
不是: 继续堆功能
而是: 建立生产可信度闭环
```

### 剩余未完成的大特性（月级工程，Beta 外）

```
聚簇索引        → storage 模型重构，3~6月
FULLTEXT        → tokenizer + inverted index，2~4月
真正 HA 复制    → consistency correctness，月级
```

---

## 二、Beta Phase 2 Issue 定义

### BP2-1: Differential Testing 扩展 — SQL Corpus 100万级

**KPI 权重**: 20%
**目标**: SQL 兼容性从 94.1% → ≥98%

**问题**: 当前 SQL Corpus 94.1%，距离 GA 目标 98% 还差约 20 个 case。

**根因分析**:
- LIMIT/OFFSET: 语法支持缺失
- ALTER TABLE INPLACE: DDL 不完整
- EXPLAIN ANALYZE: 输出格式不完整
- REPLACE INTO: UPSERT 语义缺失
- SHOW 命令: 信息模式不完整

**具体任务**:

| Issue | 任务 | 验收条件 | 优先级 |
|-------|------|---------|--------|
| BP2-1a | LIMIT/OFFSET 支持 | `SELECT * FROM t LIMIT 10 OFFSET 5` PASS | P1 |
| BP2-1b | REPLACE INTO 支持 | `REPLACE INTO t VALUES(...)` PASS | P1 |
| BP2-1c | EXPLAIN ANALYZE loops 字段 | QEP 含 loops，数值正确 | P1 |
| BP2-1d | SHOW TABLES/COLUMNS 完整 | 兼容 MySQL 5.7 输出格式 | P2 |
| BP2-1e | TRUNCATE TABLE 支持 | `TRUNCATE t` PASS | P2 |
| BP2-1f | BATCH INSERT 优化 | `INSERT INTO t VALUES(...),(...)` >10 rows PASS | P2 |

**技术约束**:
- 不修改 storage 模型
- 不修改 WAL 格式
- 仅在 executor/parser 层补充

---

### BP2-2: Crash Recovery Validation Matrix

**KPI 权重**: 25%
**目标**: Crash-safe 零数据丢失可形式化证明

**问题**: 当前 crash_recovery_test 存在，但未覆盖全部崩溃点。

**崩溃点矩阵**:

| Crash Point | 是否验证 | 测试命令 | 验收条件 |
|-------------|---------|---------|---------|
| before WAL append | ❌ 未验证 | kill -9 during WAL write | 零数据丢失 |
| after WAL append (uncommitted) | ✅ crash_recovery_test | kill -9 after WAL | 无脏数据 |
| before commit mark | ❌ 未验证 | kill -9 before commit flag | 事务回滚 |
| during checkpoint | ❌ 未验证 | kill -9 during checkpoint | 恢复后一致 |
| page flush partial | ❌ 未验证 | simulate partial page write | 无 torn page |

**具体任务**:

| Issue | 任务 | 验收条件 | 优先级 |
|-------|------|---------|--------|
| BP2-2a | WAL write crash injection | before WAL append 验证 | P1 |
| BP2-2b | pre-commit crash | before commit mark 验证 | P1 |
| BP2-2c | checkpoint partial write | during checkpoint 验证 | P2 |
| BP2-2d | torn page prevention | page flush partial 验证 | P2 |
| BP2-2e | crash_recovery_test 扩展 | 覆盖全部 5 个 crash point | P1 |

**技术方案**:
```bash
# 使用 cargo test --test crash_recovery_test -- --crash-injection
# 每个 crash point 注入 kill -9 信号，验证 recovery 后状态
```

---

### BP2-3: Soak Test 自动化 — 72h Nightly

**KPI 权重**: 20%
**目标**: 长期稳定性可验证

**问题**: 当前 long_run_stability_test 存在，但未自动化/夜间化。

**具体任务**:

| Issue | 任务 | 验收条件 | 优先级 |
|-------|------|---------|--------|
| BP2-3a | 72h nightly soak test | cronjob 每日执行，结果归档 | P1 |
| BP2-3b | memory leak detection | 72h 后内存增长 <10MB | P1 |
| BP2-3c | connection pool leak | 72h 后无 fd 泄漏 | P1 |
| BP2-3d | WAL growth bounded | 72h 后 WAL size <1GB 或自动 checkpoint | P2 |

**技术方案**:
```bash
# scripts/test/soak_test.sh
# cron: 0 2 * * *  (每日 02:00 UTC 执行)
# 输出: /tmp/soak_YYYYMMDD.log
# 监控: memory, fd count, wal_size
```

---

### BP2-4: Deadlock Fuzzer

**KPI 权重**: 15%
**目标**: 死锁检测 100% 正确，无泄漏

**问题**: 当前 concurrency_stress_test 覆盖基本并发，但缺少随机死锁注入。

**具体任务**:

| Issue | 任务 | 验收条件 | 优先级 |
|-------|------|---------|--------|
| BP2-4a | 死锁图生成器 | 随机构造 2~8 表死锁环 | P1 |
| BP2-4b | 死锁检测验证 | 检测到的死锁 100% 为真实死锁 | P1 |
| BP2-4c | 无漏报验证 | 已知死锁场景 100% 被检测 | P2 |
| BP2-4d | 死锁恢复验证 | 检测后 100% 成功回滚 | P1 |

**技术方案**:
```bash
# cargo test --test deadlock_fuzzer -- --iterations=1000
# 每次迭代: 构造死锁图 → 并发执行 → 验证检测 → 验证回滚
```

---

### BP2-5: Lock Contention Profiling

**KPI 权重**: 15%
**目标**: GMP workload 下锁竞争可视化

**问题**: GMP 场景下事务延迟的主要瓶颈是锁竞争，但当前无 profiling 工具。

**具体任务**:

| Issue | 任务 | 验收条件 | 优先级 |
|-------|------|---------|--------|
| BP2-5a | 锁等待图可视化 | 输出 JSON 格式锁等待链 | P1 |
| BP2-5b | 热点锁识别 | top-5 最高竞争锁可输出 | P2 |
| BP2-5c | lock_timeout 配置 | `SET lock_wait_timeout=N` 支持 | P2 |

**技术方案**:
```bash
# cargo test --test lock_contention_profile -- --workload=gmp
# 输出: /tmp/contention_YYYYMMDD.json
# 格式: { "lock_id": "txn_a", "waiters": ["txn_b", "txn_c"], "wait_time_ms": 123 }
```

---

### BP2-6: Audit Trail 生产验证

**KPI 权重**: 15%
**目标**: Audit log 可作为 GMP 合规证据

**问题**: Audit Trail 已实现（hook + log table + SHA256），但未验证：
- crash-safe（崩溃后审计日志不丢失）
- append-only（不可篡改）
- WAL replay 一致性

**具体任务**:

| Issue | 任务 | 验收条件 | 优先级 |
|-------|------|---------|--------|
| BP2-6a | audit log crash-safe | kill -9 后 audit log 零丢失 | P1 |
| BP2-6b | append-only 验证 | 已写入 audit log 不可修改（SHA256 链验证） | P1 |
| BP2-6c | WAL replay audit 一致性 | recovery 后 audit_count 正确 | P1 |
| BP2-6d | tamper-evidence chain | 任意篡改可被检测（SHA256 断裂报警） | P2 |

**GMP 合规检查表**:

| GMP 要求 | 当前状态 | BP2 验证方法 |
|---------|---------|------------|
| append-only 不可篡改 | 已实现 SHA256 chain | BP2-6b: 篡改检测测试 |
| 审计日志 crash-safe | 假设实现 | BP2-6a: crash injection 验证 |
| audit WAL replay correctness | 未验证 | BP2-6c: recovery 后 audit count |
| tx rollback audit consistency | 未验证 | BP2-6c: ROLLBACK 后 audit 记录 |
| tamper-evidence chain | 已实现 | BP2-6d: 篡改触发报警 |

---

### BP2-7: B-S 稳定性测试完整验证

**KPI 权重**: 25%（复用已有成果）

**当前状态**（根据 BETA_GATE_MASTER_CONTROL.md）:

| 测试 | 状态 | 需修复 |
|------|------|--------|
| B-S1: concurrency_stress_test | ❌ 需验证 | 执行 + 确认通过 |
| B-S2: crash_recovery_test | ❌ 需验证 | 执行 + 确认通过 |
| B-S3: long_run_stability_test | ❌ 需验证 | 执行 + 确认通过 |
| B-S4: wal_integration_test | ❌ 需验证 | 执行 + 确认通过 |
| B-S5: network_tcp_smoke_test | ❌ 需验证 | 执行 + 确认通过 |

**具体任务**:

| Issue | 任务 | 验收条件 | 优先级 |
|-------|------|---------|--------|
| BP2-7a | B-S1 验证 | `cargo test --test concurrency_stress_test` 全部 PASS | P1 |
| BP2-7b | B-S2 验证 | `cargo test --test crash_recovery_test` 全部 PASS | P1 |
| BP2-7c | B-S3 验证 | `cargo test --test long_run_stability_test` 全部 PASS | P1 |
| BP2-7d | B-S4 验证 | `cargo test --test wal_integration_test` 全部 PASS | P1 |
| BP2-7e | B-S5 验证 | `cargo test --test network_tcp_smoke_test` 全部 PASS | P1 |

---

## 三、KPI 权重与验收

### KPI 权重（Beta Phase 2）

| KPI | 权重 | Beta Phase 2 目标 |
|-----|------|-----------------|
| Crash Recovery Correctness | 25% | 5/5 crash point matrix 验证 |
| Long-run Stability | 20% | 72h nightly soak test 自动化 |
| Differential Compatibility | 20% | SQL Corpus ≥98% (从 94.1% 提升) |
| Auditability | 15% | Audit Trail crash-safe + tamper-evident |
| Transaction Correctness | 15% | Deadlock Fuzzer 100% 检测 + 无泄漏 |
| Lock Contention | 5% | 热点锁可识别 |

### Beta Phase 2 通过条件

```
BP2-1 ~ BP2-7 全部完成
BLOCKERS = 0
```

---

## 四、技术约束（铁律）

Beta Phase 2 **禁止**:

```
❌ 修改 storage page layout
❌ 修改 WAL 格式
❌ 修改 MVCC 可见性算法
❌ 添加新的存储引擎
❌ 重构 optimizer/planner 架构
```

这些属于"架构重特性"，是 GA 后或 v3.1.0 的工作。

Beta Phase 2 **允许**:

```
✅ 补充 SQL 语法支持（executor/parser 层）
✅ 扩展测试覆盖率（不改变核心逻辑）
✅ 自动化测试（cronjob, fuzzer）
✅ 修复测试本身 bug
✅ 补充 audit/crash 功能（业务层）
✅ 性能 profiling 工具
```

---

## 五、执行计划

### Week 1: 基础设施 + B-S 验证

```
Day 1-2: BP2-7 (B-S1~B-S5 验证)
Day 3-4: BP2-6 (Audit Trail 生产验证)
Day 5:   BP2-3a (Soak test cronjob 搭建)
```

### Week 2: 正确性闭环

```
Day 6-8:  BP2-2 (Crash Recovery Matrix)
Day 9-10: BP2-4 (Deadlock Fuzzer)
```

### Week 3: 性能与兼容

```
Day 11-12: BP2-5 (Lock Contention)
Day 13-14: BP2-1 (SQL Corpus 扩展至 98%)
Day 15:    整合测试 + Beta Phase 2 关门
```

---

## 六、相关文档

| 文档 | 作用 |
|------|------|
| `BETA_GATE_MASTER_CONTROL.md` | Beta Gate 门禁清单 |
| `TEST_PLAN.md` | 测试分层与 L0~L3 说明 |
| `HARNESS_ENGINEERING_AUDIT.md` | CI/CD + Gate 审核 |
| `GATE_EXEMPTIONS.md` | 豁免记录（EX-004~EX-006） |

---

## 七、Issue 汇总表

| Issue | 名称 | KPI 权重 | 优先级 | 预计工时 |
|-------|------|---------|--------|---------|
| BP2-1a | LIMIT/OFFSET 支持 | 20% | P1 | 2h |
| BP2-1b | REPLACE INTO 支持 | 20% | P1 | 4h |
| BP2-1c | EXPLAIN ANALYZE loops | 20% | P1 | 2h |
| BP2-1d | SHOW 命令完整 | 20% | P2 | 4h |
| BP2-1e | TRUNCATE TABLE | 20% | P2 | 2h |
| BP2-1f | BATCH INSERT | 20% | P2 | 3h |
| BP2-2a | WAL write crash injection | 25% | P1 | 4h |
| BP2-2b | pre-commit crash | 25% | P1 | 4h |
| BP2-2c | checkpoint partial | 25% | P2 | 6h |
| BP2-2d | torn page prevention | 25% | P2 | 6h |
| BP2-2e | crash_recovery 扩展 | 25% | P1 | 8h |
| BP2-3a | 72h soak cronjob | 20% | P1 | 4h |
| BP2-3b | memory leak detection | 20% | P1 | 2h |
| BP2-3c | connection pool leak | 20% | P1 | 2h |
| BP2-3d | WAL growth bounded | 20% | P2 | 2h |
| BP2-4a | 死锁图生成器 | 15% | P1 | 6h |
| BP2-4b | 死锁检测验证 | 15% | P1 | 4h |
| BP2-4c | 无漏报验证 | 15% | P2 | 4h |
| BP2-4d | 死锁恢复验证 | 15% | P1 | 4h |
| BP2-5a | 锁等待图可视化 | 5% | P1 | 4h |
| BP2-5b | 热点锁识别 | 5% | P2 | 4h |
| BP2-5c | lock_timeout | 5% | P2 | 3h |
| BP2-6a | audit crash-safe | 15% | P1 | 4h |
| BP2-6b | append-only 验证 | 15% | P1 | 4h |
| BP2-6c | WAL replay audit | 15% | P1 | 6h |
| BP2-6d | tamper-evidence | 15% | P2 | 8h |
| BP2-7a | B-S1 验证 | 25% | P1 | 2h |
| BP2-7b | B-S2 验证 | 25% | P1 | 2h |
| BP2-7c | B-S3 验证 | 25% | P1 | 2h |
| BP2-7d | B-S4 验证 | 25% | P1 | 2h |
| BP2-7e | B-S5 验证 | 25% | P1 | 2h |

**总计**: 31 个 Issue，预计 15 个工作日（3 周）

---

*本文档由 hermes agent 创建，基于 ChatGPT 战略分析*
*日期: 2026-05-08*
