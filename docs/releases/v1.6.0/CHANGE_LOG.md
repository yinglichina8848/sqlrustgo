# SQLRustGo v1.6.0 变更日志

> **版本**: v1.6.0
> **发布日期**: 2026-03-19

---

## 一、变更概述

v1.6.0 是继 v1.5.0 之后的重大更新，专注于事务隔离和并发控制。

---

## 二、功能变更

### 2.1 新增功能

#### 事务支持

| 功能 | PR | 说明 |
|------|-----|------|
| MVCC 骨架 | #607 | 快照隔离、版本链管理 |
| 事务管理器 | #616 | BEGIN/COMMIT/ROLLBACK |
| READ COMMITTED | #620 | 隔离级别实现 |
| 行级锁 | #633 | 读写锁机制 |
| 死锁检测 | #642 | Wait-For Graph |
| SAVEPOINT | #642 | 保存点支持 |

#### WAL 改进

| 功能 | PR | 说明 |
|------|-----|------|
| 并发写入 | #608 | 多线程 WAL |
| 检查点优化 | #613 | 定期检查点 |
| WAL 归档 | #619 | 日志归档清理 |

#### 索引增强

| 功能 | PR | 说明 |
|------|-----|------|
| 唯一索引 | #612 | UNIQUE 约束 |
| 复合索引 | #615 | 多列索引 |
| 索引统计 | #618 | 直方图/基数估计 |
| 全文索引 | #626 | FTS 基础 |

#### 性能优化

| 功能 | PR | 说明 |
|------|-----|------|
| 查询缓存 | #627 | LRU + TTL |
| 连接池 | #655 | 共享存储 |
| TPC-H | #657 | Q1/Q6 Benchmark |

#### 数据类型

| 功能 | PR | 说明 |
|------|-----|------|
| DATE | #624 | 日期类型 |
| TIMESTAMP | #634 | 时间戳类型 |

---

## 三、文档更新

| 文档 | 变更 |
|------|------|
| ARCHITECTURE_DESIGN.md | 新增架构设计 |
| VERSION_PLAN.md | 更新版本计划 |
| DEFECT_REPORT.md | 新增缺陷报告 |
| MATURITY_ASSESSMENT.md | 新增成熟度评估 |
| SECURITY_ANALYSIS.md | 新增安全分析 |
| PERFORMANCE_ANALYSIS_REPORT.md | 新增性能分析 |
| TEST_MANUAL.md | 新增测试手册 |
| USER_MANUAL.md | 新增用户手册 |

---

## 四、代码变更

### 4.1 新增文件

```
crates/transaction/src/deadlock.rs     # 死锁检测
crates/transaction/src/savepoint.rs  # 保存点
benches/tpch_comprehensive.rs        # TPC-H 基准
```

### 4.2 修改文件

```
crates/transaction/src/mvcc.rs       # MVCC 增强
crates/transaction/src/lock.rs       # 锁管理
crates/transaction/src/manager.rs    # 事务管理
crates/storage/src/wal.rs           # WAL 改进
crates/executor/src/query_cache.rs   # 查询缓存
crates/server/src/connection_pool.rs # 连接池修复
```

---

## 五、测试变更

### 5.1 新增测试

| 测试 | 说明 |
|------|------|
| MVCC 并发测试 | 事务并发场景 |
| 死锁检测测试 | 死锁环检测 |
| TPC-H 测试 | Q1/Q6 性能 |
| 查询缓存测试 | 缓存命中率 |

### 5.2 测试统计

| 类型 | 数量 |
|------|------|
| 单元测试 | 900+ |
| 集成测试 | 50+ |
| 基准测试 | 10+ |

---

## 六、依赖变更

### 6.1 新增依赖

| 依赖 | 版本 | 用途 |
|------|------|------|
| tokio | 1.x | 异步运行时 |
| crossbeam-channel | 0.5 | 通道 |

### 6.2 版本更新

| 依赖 | 旧版本 | 新版本 |
|------|--------|--------|
| serde | 1.x | 1.x |
| actix-web | 4.x | 4.x |

---

## 七、已知问题

| 问题 | 严重性 | 说明 |
|------|--------|------|
| 覆盖率略低 | 低 | 70.72% vs 75% 目标 |
| Planner 测试 | 低 | 部分边界条件未覆盖 |

---

## 八、升级说明

### 8.1 从 v1.5.0 升级

v1.6.0 完全向后兼容 v1.5.0。

### 8.2 新增 API

```rust
// 事务 API
let mut tx = transaction_manager.begin()?;
let commit_ts = transaction_manager.commit()?;

// 查询缓存 API
let cache = QueryCache::new(config);
cache.get(&key)?;
cache.put(key, entry, tables)?;
```

---

*本文档由 AI 辅助分析生成*
*生成日期: 2026-03-19*
