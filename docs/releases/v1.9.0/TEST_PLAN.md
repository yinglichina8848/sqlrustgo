# SQLRustGo v1.9.0 测试计划（建议版）

> **版本**: v1.9.0
> **更新日期**: 2026-03-26
> **状态**: 开发中

---

## 1. 测试概述

本文档按照 **7 Gate 结构**组织 v1.9.0 的测试计划，确保每个核心子系统都有对应的测试验证。

---

## ① Storage Engine Gate 测试

### 1.1 B+Tree 索引测试

```bash
cargo test bplus_tree
```

| 测试项 | 测试用例 | 状态 |
|--------|----------|------|
| 插入/搜索 | B+Tree 插入键值，搜索返回正确 | ✅ |
| 范围查询 | `range_query(1, 100)` 返回正确 | ✅ |
| 多列索引 | CompositeBTreeIndex | ✅ |
| 唯一约束 | 唯一键重复检测 | ✅ |

### 1.2 WAL 崩溃恢复测试

```bash
cargo test --test crash_recovery_test
```

| 测试项 | 测试用例 | 状态 |
|--------|----------|------|
| 完整恢复 | 提交事务全部恢复 | ✅ |
| 部分回滚 | 未提交事务正确回滚 | ✅ |
| Checkpoint | Checkpoint 后恢复 | ✅ |
| 大数据量 | 1000+ 条目恢复性能 | ✅ |

---

## ② Transaction Gate 测试

### 2.1 事务隔离级别测试

```bash
cargo test transaction
```

| 测试项 | 测试用例 | 状态 |
|--------|----------|------|
| READ COMMITTED | 提交读 | ⬜ |
| REPEATABLE READ | 可重复读 | ⬜ |
| MVCC 可见性 | 多版本可见性判断 | ⬜ |

### 2.2 死锁检测测试

```bash
cargo test deadlock
```

| 测试项 | 测试用例 | 状态 |
|--------|----------|------|
| 死锁检测 | 循环等待检测 | ⬜ |
| 超时回滚 | 事务超时处理 | ⬜ |

---

## ③ Query Engine Gate 测试

### 3.1 JOIN 测试

```bash
cargo test join
```

| 测试项 | 测试用例 | 状态 |
|--------|----------|------|
| INNER JOIN | 多表连接正确性 | ⬜ |
| LEFT JOIN | 左表保留 + NULL 填充 | ⬜ |
| Hash Join | 哈希连接实现 | ⬜ |
| Index NL Join | 索引嵌套循环连接 | ⬜ |

### 3.2 子查询测试

```bash
cargo test subquery
```

| 测试项 | 测试用例 | 状态 |
|--------|----------|------|
| Scalar Subquery | 单值返回 | ✅ |
| IN Subquery | 多值匹配 | ✅ |
| EXISTS | 存在性检查 | ✅ |

---

## ④ Optimizer Gate 测试

### 4.1 统计信息测试

```bash
cargo test optimizer_stats
```

| 测试项 | 测试用例 | 状态 |
|--------|----------|------|
| ANALYZE TABLE | 统计信息收集 | ⬜ |
| Cardinality 估算 | 行数估算准确性 | ⬜ |
| 直方图 | 列数据分布统计 | ⬜ |

### 4.2 成本模型测试

```bash
cargo test optimizer_cost
```

| 测试项 | 测试用例 | 状态 |
|--------|----------|------|
| Join 重排 | 多表连接顺序优化 | ⬜ |
| 成本计算 | 基于统计信息的成本估算 | ⬜ |

---

## ⑤ SQL Compatibility Gate 测试

### 5.1 外键约束测试

```bash
cargo test foreign_key
```

| 测试项 | 测试用例 | 状态 |
|--------|----------|------|
| 单表外键 | FOREIGN KEY 约束验证 | ⬜ |
| 级联删除 | ON DELETE CASCADE | ⬜ |
| 外键违反 | 约束报错 | ⬜ |

### 5.2 AUTO_INCREMENT 测试

```bash
cargo test auto_increment
```

| 测试项 | 测试用例 | 状态 |
|--------|----------|------|
| 自动序列 | 自增 ID 生成 | ⬜ |
| LAST_INSERT_ID | 获取最后插入 ID | ⬜ |

---

## ⑥ Observability Gate 测试

### 6.1 EXPLAIN 测试

```bash
cargo test explain
```

| 测试项 | 测试用例 | 状态 |
|--------|----------|------|
| EXPLAIN | 执行计划展示 | ✅ |
| EXPLAIN ANALYZE | 带时间分析 | ⬜ |

### 6.2 日志系统测试

```bash
cargo test sql_log
```

| 测试项 | 测试用例 | 状态 |
|--------|----------|------|
| 日志持久化 | persist_to_file | ✅ |
| 日志恢复 | recover_from_file | ✅ |
| 备份/恢复 | backup/restore | ✅ |
| 损坏恢复 | corruption recovery | ✅ |
| 断电模拟 | power failure | ✅ |

---

## ⑦ Stability Gate 测试（最关键）

### 7.1 压力测试

```bash
cargo test --test stress_test
```

| 测试项 | 测试时长 | 状态 |
|--------|----------|------|
| 24h 压力测试 | 24 小时连续运行 | ⬜ |
| 并发连接 | 50+ 并发连接 | ⬜ |
| 高 QPS | 1000+ QPS | ⬜ |

### 7.2 崩溃恢复测试

```bash
cargo test --test crash_recovery_test
```

| 测试项 | 测试用例 | 状态 |
|--------|----------|------|
| 完整恢复 | 事务提交后崩溃恢复 | ✅ |
| 部分回滚 | 未提交事务回滚 | ✅ |
| WAL 损坏 | 部分写入恢复 | ✅ |
| 索引损坏 | 索引损坏修复 | ⬜ |

### 7.3 并发事务测试

```bash
cargo test --test concurrency_stress_test
```

| 测试项 | 测试用例 | 状态 |
|--------|----------|------|
| 并发读 | 多线程同时读 | ✅ |
| 并发写 | 多线程同时写 | ✅ |
| 混合读写 | 读写混合场景 | ✅ |

---

## 测试覆盖度统计

| Gate | 目标测试数 | 已完成 | 待开发 |
|------|------------|--------|--------|
| Storage Engine | 20+ | 15+ | 5 |
| Transaction | 15+ | 0 | 15+ |
| Query Engine | 30+ | 10+ | 20+ |
| Optimizer | 20+ | 0 | 20+ |
| SQL Compatibility | 25+ | 0 | 25+ |
| Observability | 20+ | 15+ | 5 |
| Stability | 30+ | 20+ | 10 |
| **总计** | **160+** | **60+** | **100+** |

---

## 发布前必须通过的测试

### 必须通过 (Must Pass)

- [ ] cargo build --release (编译通过)
- [ ] cargo test --workspace (所有测试通过)
- [ ] cargo clippy (无 error)
- [ ] cargo fmt (格式正确)
- [ ] Storage Engine 全部测试通过
- [ ] WAL 崩溃恢复测试通过
- [ ] 日志系统测试通过
- [ ] 生产场景测试通过

### 建议通过 (Should Pass)

- [ ] 覆盖率 ≥75%
- [ ] 24h 压力测试通过
- [ ] 并发事务测试通过

---

*本文档由 OpenCode AI 生成*
*生成日期: 2026-03-26*
*版本: v1.9.0*
