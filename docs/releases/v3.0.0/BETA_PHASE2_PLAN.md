# v3.0.0 Beta Phase 2 开发计划

> **版本**: v3.0.0-beta.2
> **日期**: 2026-05-08
> **分支**: develop/v3.0.0
> **目标**: GMP 核心可信度基础设施

---

## 一、背景

v3.0.0 GA 已于 2026-05-07 发布。虽然功能开发完成，但作为 GMP (Good Manufacturing Practice) 数据库，我们需要在以下方面进行验证和增强：

1. **Audit Trail（审计日志）** - GMP 合规核心
2. **WAL Crash Recovery 验证** - 证明崩溃不会丢失数据
3. **Differential Testing** - 与 MySQL 5.7 结果比对
4. **INFORMATION_SCHEMA 扩展** - JDBC/ORM 兼容性
5. **Window Functions 补全** - SQL 标准支持

---

## 二、任务列表

### P0 必须完成

| Issue | 任务 | 验收条件 | 工期 | 阻塞门禁 |
|-------|------|---------|------|---------|
| #P2-1 | Audit Trail 系统 | `system.audit_log` 可查询，DML 前后值捕获 | 2 周 | BP2-1 |
| #P2-2 | WAL Crash Validation | crash_injector 可触发，100 次循环无数据丢失 | 2 周 | BP2-2 |
| #P2-3 | Differential Testing 框架 | SQL ↔ MySQL 5.7 比对 ≥10 万用例 | 1 周 | BP2-3 |
| #P2-4 | INFORMATION_SCHEMA 扩展 | TRIGGERS/ROUTINES/PRIVILEGES 可查询 | 1 周 | BP2-4 |
| #P2-5 | EXPLAIN ANALYZE 增强 | actual_rows/loops/timing 输出 | 1 周 | BP2-5 |
| #P2-6 | Window Functions 补全 | LEAD/LAG/NTILE/FIRST_VALUE/LAST_VALUE 正确 | 1 周 | BP2-6 |

### P1 可选完成

| Issue | 任务 | 验收条件 | 工期 |
|-------|------|---------|------|
| #P2-7 | RANGE Partition (planner pruning) | WHERE 条件分区裁剪正确 | 2-3 周 |
| #P2-8 | Cursor 基础版 | DECLARE/FETCH/CLOSE 正确 | 2 周 |
| #P2-9 | Trigger Chain | BEFORE/AFTER 有序执行 | 1-2 周 |

### QA 验证

| Issue | 任务 | 验收条件 | 工期 |
|-------|------|---------|------|
| #P2-QA1 | Soak Testing 72h | WAL 压力/并发/崩溃恢复 72h 无 leak | 持续 |

---

## 三、详细任务说明

### Issue #P2-1: Audit Trail 系统

**GMP 价值**: 批记录变更必须可追溯

**实现内容**:

```rust
// executor hook
pub trait AuditHook {
    fn on_insert(&self, ctx: &ExecCtx, row: &Row);
    fn on_update(&self, ctx: &ExecCtx, old: &Row, new: &Row);
    fn on_delete(&self, ctx: &ExecCtx, row: &Row);
    fn on_ddl(&self, ctx: &ExecCtx, sql: &str);
}
```

**新增系统表**:
```sql
CREATE TABLE system.audit_log (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    timestamp TIMESTAMP(6) NOT NULL,
    tx_id BIGINT NOT NULL,
    user VARCHAR(64) NOT NULL,
    operation ENUM('INSERT','UPDATE','DELETE','DDL') NOT NULL,
    table_name VARCHAR(64),
    row_id VARCHAR(255),
    old_value JSON,
    new_value JSON,
    sql_text TEXT,
    INDEX idx_table_row (table_name, row_id),
    INDEX idx_tx (tx_id),
    INDEX idx_user_time (user, timestamp)
);
```

**验收测试**:
```bash
# 1. 插入数据后 audit_log 有记录
INSERT INTO audit_log_test VALUES (1, 'test');
SELECT * FROM system.audit_log WHERE table_name = 'audit_log_test';
# 应有 1 条 INSERT 记录

# 2. 更新数据后 audit_log 有前后值
UPDATE audit_log_test SET name = 'updated' WHERE id = 1;
SELECT old_value, new_value FROM system.audit_log WHERE operation = 'UPDATE';
# old_value = {"id":1,"name":"test"}, new_value = {"id":1,"name":"updated"}
```

---

### Issue #P2-2: WAL Crash Validation

**GMP 价值**: 证明崩溃恢复不会丢失数据

**实现内容**:

```rust
// crash_injector module
pub enum CrashPoint {
    BeforeWalWrite,
    AfterWalWrite,
    BeforeCheckpoint,
    AfterPageFlush,
    BeforeCommit,
}

pub struct CrashInjector {
    point: CrashPoint,
    probability: f64,
    enabled: bool,
}
```

**验证脚本**:
```bash
# crash_recovery_loop.sh - 100 次循环
for i in {1..100}; do
    inject_crash at BeforeCommit
    restart_and_recover
    verify_data_integrity
done
# 100 次全部通过才算 BP2-2 通过
```

---

### Issue #P2-3: Differential Testing 框架

**GMP 价值**: 真正知道与 MySQL 5.7 兼容多少

**实现内容**:

```
sqlrustgo-tester/
├── diff_framework/
│   ├── compare.py          # 结果比对
│   ├── query_generator.py  # SQL 生成
│   ├── mysql_runner.py     # MySQL 执行
│   └── sqlrustgo_runner.py # SQLRustGo 执行
└── corpus/
    ├── basic/             # 基础 SQL (5,000)
    ├── edge/              # 边界条件 (5,000)
    ├── null/              # NULL 处理 (1,000)
    └── transaction/        # 事务 (2,000)
```

**验收**:
- [ ] 10,000+ SQL cases 可运行
- [ ] 与 MySQL 5.7 结果比对通过率 ≥85%

---

### Issue #P2-4: INFORMATION_SCHEMA 扩展

**已有**: TABLES, COLUMNS, STATISTICS
**新增**:

```sql
information_schema.TRIGGERS
information_schema.ROUTINES
information_schema.PARAMETERS
information_schema.USER_PRIVILEGES
information_schema.SCHEMA_PRIVILEGES
information_schema.TABLE_PRIVILEGES
information_schema.COLUMN_PRIVILEGES
```

**验收**:
```sql
-- TRIGGERS 可查询
SELECT TRIGGER_NAME, EVENT_MANIPULATION, ACTION_TIMING
FROM information_schema.TRIGGERS
WHERE TRIGGER_SCHEMA = 'test';

-- ROUTINES 可查询
SELECT ROUTINE_NAME, ROUTINE_TYPE, DTD_IDENTIFIER
FROM information_schema.ROUTINES
WHERE ROUTINE_SCHEMA = 'test';
```

---

### Issue #P2-5: EXPLAIN ANALYZE 增强

**目标输出**:
```json
{
  "plan": [
    {
      "id": 1,
      "operation": "HashJoin",
      "actual_rows": 1000,
      "loops": 1,
      "timing_ms": 15.5,
      "children": [...]
    }
  ]
}
```

---

### Issue #P2-6: Window Functions 补全

| 函数 | 状态 | 难度 |
|------|------|------|
| ROW_NUMBER | ✅ | - |
| RANK | ✅ | - |
| DENSE_RANK | ✅ | - |
| NTILE | ❌ | 中 |
| LEAD | ❌ | 低 |
| LAG | ❌ | 低 |
| FIRST_VALUE | ❌ | 低 |
| LAST_VALUE | ❌ | 低 |
| NTH_VALUE | ❌ | 中 |

**验收**:
```sql
SELECT
    id,
    name,
    LAG(name) OVER (ORDER BY id) as prev_name,
    LEAD(name) OVER (ORDER BY id) as next_name,
    FIRST_VALUE(name) OVER (ORDER BY id) as first_name,
    LAST_VALUE(name) OVER (ORDER BY id) as last_name,
    NTILE(4) OVER (ORDER BY id) as quartile
FROM users;
-- 结果正确
```

---

## 四、门禁检查

### Beta Phase 2 Gate (BP2-Gate)

| ID | 检查项 | 命令 | 通过标准 | 对应 Issue |
|----|--------|------|---------|-----------|
| BP2-1 | Audit Trail | `cargo test --test audit_trail_test` | 全部通过 | #P2-1 |
| BP2-2 | WAL Crash Validation | `cargo test --test crash_inject_test` | 100 次循环全部通过 | #P2-2 |
| BP2-3 | Differential Testing | `cargo test -p sqlrustgo-sql-corpus` | ≥85% | #P2-3 |
| BP2-4 | INFORMATION_SCHEMA | `cargo test --test information_schema_test` | TRIGGERS/ROUTINES 可查询 | #P2-4 |
| BP2-5 | EXPLAIN ANALYZE | `cargo test --test explain_analyze_test` | actual_rows 输出正确 | #P2-5 |
| BP2-6 | Window Functions | `cargo test --test window_function_test` | LEAD/LAG/NTILE 正确 | #P2-6 |
| BP2-7 | RANGE Partition | `cargo test --test partition_test` | 分区裁剪正确 | #P2-7 |
| BP2-8 | Cursor | `cargo test --test cursor_test` | FETCH 正确 | #P2-8 |
| BP2-9 | Trigger Chain | `cargo test --test trigger_chain_test` | 有序执行正确 | #P2-9 |
| BP2-QA1 | Soak Test 72h | `cargo test --test long_run_stability_72h_test` | 72h 无 leak | #P2-QA1 |

---

## 五、执行计划

### Week 1

| 任务 | 负责人 | 交付物 |
|------|--------|--------|
| #P2-1 Audit Trail (executor hook) | Agent 1 | executor hook + audit_log 表 |
| #P2-3 Differential Testing 框架 | Agent 2 | diff_framework/ 目录 |

### Week 2

| 任务 | 负责人 | 交付物 |
|------|--------|--------|
| #P2-1 Audit Trail (WAL append) | Agent 1 | audit_log WAL 写入 |
| #P2-2 WAL Crash Validation | Agent 2 | crash_injector 模块 |

### Week 3

| 任务 | 负责人 | 交付物 |
|------|--------|--------|
| #P2-4 INFORMATION_SCHEMA | Agent 3 | TRIGGERS/ROUTINES/PRIVILEGES |
| #P2-5 EXPLAIN ANALYZE | Agent 1 | actual_rows/loops/timing |

### Week 4

| 任务 | 负责人 | 交付物 |
|------|--------|--------|
| #P2-6 Window Functions | Agent 2 | LEAD/LAG/NTILE/FIRST/LAST |
| #P2-7 RANGE Partition | Agent 3 | planner pruning |

### Week 5

| 任务 | 负责人 | 交付物 |
|------|--------|--------|
| #P2-8 Cursor | Agent 1 | DECLARE/FETCH/CLOSE |
| #P2-9 Trigger Chain | Agent 2 | BEFORE/AFTER 执行 |

### Week 6

| 任务 | 负责人 | 交付物 |
|------|--------|--------|
| #P2-QA1 Soak Test 72h | Agent 3 | 72h 稳定性报告 |
| BP2-Gate 验证 | All | 门禁检查通过 |

---

## 六、交付物清单

| 交付物 | 类型 | 位置 |
|--------|------|------|
| Audit Trail | 功能 | `system.audit_log` |
| Differential Framework | 工具 | `scripts/diff_test/` |
| Crash Injector | 工具 | `crash_injector.rs` |
| INFORMATION_SCHEMA 扩展 | 功能 | `information_schema/` |
| EXPLAIN ANALYZE 增强 | 功能 | `executor/` |
| Window Functions 补全 | 功能 | `executor/` |
| RANGE Partition | 功能 | `planner/` |
| Cursor 基础版 | 功能 | `executor/` |
| Trigger Chain | 功能 | `executor/` |
| Soak Test 72h | 测试 | `tests/soak/` |

---

## 七、相关文档

| 文档 | 说明 |
|------|------|
| `docs/releases/v3.0.0/GMP_ROADMAP.md` | GMP 生产路线图 |
| `scripts/gate/check_beta_v300_phase2.sh` | Beta Phase 2 门禁检查脚本 |
| `docs/governance/gate_spec.md` | 门禁规范 |

---

*本文档由 Sisyphus 规划*
*规划日期: 2026-05-08*