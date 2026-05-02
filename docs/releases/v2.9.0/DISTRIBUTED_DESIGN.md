# Issue #119: 分布式增强详细设计

> **Issue**: #119
> **版本**: v2.9.0
> **目标**: 分布式复制增强 (MySQL 5.7 水平)
> **基线**: v2.8.0 (GTID 复制、故障转移、读写分离已完成)

---

## 一、v2.8.0 现状

| 功能 | 状态 | 测试数 | MySQL 5.7 差距 |
|------|------|--------|----------------|
| GTID 复制 | ✅ 完成 | 79 tests | 持平 |
| 故障转移 | ✅ 完成 | 55 tests | 持平 |
| 读写分离 | ✅ 完成 | 27 tests | 持平 |
| 负载均衡 | ✅ 完成 | 集成验证 | 持平 |
| **半同步复制** | ❌ 缺失 | 0 | -100% |
| **并行复制 (MTS)** | ❌ 缺失 | 0 | -100% |
| **多源复制** | ❌ 缺失 | 0 | -100% |
| **2PC 强化** | ⚠️ 部分 | 50 | -50% |

---

## 二、MySQL 5.7 分布式功能要求

### 2.1 半同步复制 (Semisynchronous Replication)

```sql
-- MySQL 5.7 半同步复制配置
INSTALL PLUGIN rpl_semi_sync_master SONAME 'semisync_master.so';
INSTALL PLUGIN rpl_semi_sync_slave SONAME 'semisync_slave.so';

SET GLOBAL rpl_semi_sync_master_enabled = 1;
SET GLOBAL rpl_semi_sync_slave_enabled = 1;

-- 关键参数
rpl_semi_sync_master_timeout = 10000 (ms)
rpl_semi_sync_master_wait_for_slave_count = 1
```

**AFTER_SYNC vs AFTER_COMMIT 模式**:
- `AFTER_SYNC` (5.7+): 主库等从库接收 relay log 后返回客户端 (更安全)
- `AFTER_COMMIT`: 主库等从库提交后返回 (旧模式)

### 2.2 并行复制 (Multi-Threaded Slave, MTS)

```sql
-- MySQL 5.7 并行复制配置
SET GLOBAL slave_parallel_workers = 4;
SET GLOBAL slave_parallel_type = 'LOGICAL_CLOCK';
SET GLOBAL slave_preserve_commit_order = 1;

-- 两种模式
-- DATABASE: 按数据库分发的并行复制
-- LOGICAL_CLOCK: 基于组提交的真正并行复制
```

**LOGICAL_CLOCK 工作原理**:
1. 主库二进制日志组提交 (BLGC)
2. 主库为事务分配 logical_timestamp
3. 从库按 logical_timestamp 并行回放
4. 保证提交顺序 (slave_preserve_commit_order = 1)

### 2.3 多源复制 (Multi-Source Replication)

```sql
-- MySQL 5.7 多源复制
CHANGE MASTER TO master_host='host1' FOR CHANNEL 'channel1';
CHANGE MASTER TO master_host='host2' FOR CHANNEL 'channel2';

-- 验证
SELECT * FROM performance_schema.replication_connection_status;
```

### 2.4 2PC 分布式事务 (XA)

```sql
-- MySQL 5.7 XA 事务
XA START 'xid1';
INSERT INTO t VALUES (1);
XA END 'xid1';
XA PREPARE 'xid1';
XA COMMIT 'xid1';

-- XA 恢复
XA RECOVER;
```

---

## 三、任务详细设计

### D-01: 半同步复制完善

**目标**: 支持 MySQL 5.7 半同步复制协议

**交付物**:
1. `crates/distributed/src/semisync.rs`
2. `crates/distributed/src/replication.rs` 增强
3. `tests/distributed/test_semisync.rs`

**实现要点**:
```
1. 插件系统
   - rpl_semi_sync_master.so (主库端)
   - rpl_semi_sync_slave.so (从库端)

2. 主库端 (SemisyncMaster)
   - wait_for_slave_count: 等多少个从库确认
   - timeout: 等待超时，降级为异步
   - ACK 收集器

3. 从库端 (SemisyncSlave)
   - 回复 ACK 给主库
   - relay log 接收确认

4. 降级机制
   - 超时自动降级异步
   - 网络恢复自动恢复半同步
```

**测试用例**:
| 用例 | 预期 | 验证 |
|------|------|------|
| 基础半同步 | INSERT 返回前等 ACK | 主库等待时间 |
| 超时降级 | 10s 超时后异步 | rpl_semi_sync_master_status |
| 恢复半同步 | 网络恢复后自动恢复 | 监控状态切换 |
| AFTER_SYNC vs AFTER_COMMIT | 两者行为差异 | 数据一致性 |

**工时**: 3d (P1)

---

### D-02: 并行复制 (MTS)

**目标**: 支持 LOGICAL_CLOCK 并行复制

**交付物**:
1. `crates/distributed/src/mts.rs`
2. `crates/distributed/src/worker_pool.rs`
3. `tests/distributed/test_mts.rs`

**实现要点**:
```
1. 事务分发器 (Transaction Dispatcher)
   - 按 logical_timestamp 分组
   - 不同组可并行执行
   - 同组内按顺序提交

2. 工作池 (Worker Pool)
   - 多个 worker 线程
   - 每个 worker 独立执行引擎
   - Worker 数量可配置 (slave_parallel_workers)

3. 提交协调器 (Commit Coordinator)
   - 保证提交顺序 (slave_preserve_commit_order = 1)
   - 等待前置组提交完成

4. 冲突检测
   - 同分片事务无冲突
   - 跨分片事务无冲突
   - 同分片写冲突 → 串行化
```

**测试用例**:
| 用例 | 预期 | 验证 |
|------|------|------|
| 并行回放 | 4 workers, 事务并行执行 | 执行时间 |
| 顺序保证 | commit_order 保持 | 验证数据一致性 |
| 性能提升 | 并行 vs 串行 | TPS 对比 |
| Worker 扩缩容 | 动态调整 worker 数 | 运行时验证 |

**工时**: 5d (P1)

---

### D-03: 多源复制

**目标**: 支持一从库多主源复制

**交付物**:
1. `crates/distributed/src/multi_source.rs`
2. `tests/distributed/test_multi_source.rs`

**实现要点**:
```
1. 通道管理 (Channel Management)
   - 每个主库独立通道
   - 通道状态独立
   - 通道故障隔离

2. 冲突解决策略
   - LATEST_SOURCE_FIRST: 最新源优先
   - FIRST_SOURCE_FIRST: 首选源优先
   - 源特定策略

3. 复制拓扑
   - 一主一从 (已有)
   - 多主一从 (新增)
   - 级联复制 (未来)

4. 监控
   - per-channel 状态
   - per-channel 延迟
   - 冲突告警
```

**测试用例**:
| 用例 | 预期 | 验证 |
|------|------|------|
| 双主复制 | 两个主库数据汇聚 | 数据完整性 |
| 通道故障隔离 | 单通道故障不影响其他 | 服务可用性 |
| 冲突检测 | 同数据并发写入 | 冲突日志 |

**工时**: 8d (P2)

---

### D-04: 2PC 强化

**目标**: 完善 XA 事务支持，达到 MySQL 5.7 水平

**交付物**:
1. `crates/distributed/src/xa_coordinator.rs`
2. `tests/distributed/test_xa.rs`

**实现要点**:
```
1. XA 事务状态机
   - ACTIVE → IDLE → PREPARED → COMMITTED/ROLLEDBACK
   - 状态持久化

2. XA 恢复
   - 启动时扫描未完成 XA 事务
   - RECOVER 返回未决事务列表
   - 自动提交或回滚

3. XA 隔离级别
   - SERIALIZABLE (强制)
   - 防止脏读

4. 死锁检测
   - XA 事务死锁
   - 普通事务与 XA 冲突
```

**测试用例**:
| 用例 | 预期 | 验证 |
|------|------|------|
| 基础 XA | XA START/COMMIT 执行 | 数据正确 |
| XA RECOVER | 崩溃后恢复未决事务 | 恢复后数据 |
| 死锁处理 | XA 死锁检测与回滚 | 错误码 |
| XA 性能 | vs 普通事务 | TPS 对比 |

**工时**: 5d (P1)

---

## 四、技术架构

### 4.1 目录结构

```
crates/distributed/src/
├── lib.rs
├── replication.rs        # GTID 基础复制 (已有)
├── semisync.rs           # D-01: 半同步复制
├── mts.rs                # D-02: 并行复制
├── worker_pool.rs        # D-02: Worker 池
├── multi_source.rs       # D-03: 多源复制
├── xa_coordinator.rs     # D-04: XA 协调器
└── failover.rs          # 故障转移 (已有)
```

### 4.2 依赖关系

```
D-01 (半同步)
  └── D-04 (2PC) 基础: 需要 XA 事务支持

D-02 (MTS)
  └── D-01 完成后: 半同步 + MTS 组合

D-03 (多源)
  └── D-01, D-02 完成后: 多通道 MTS
```

---

## 五、里程碑

| 里程碑 | 日期 | 交付 |
|--------|------|------|
| D-01 完成 | +3d | 半同步复制 |
| D-02 完成 | +8d | MTS |
| D-04 完成 | +13d | XA 强化 |
| D-03 完成 | +21d | 多源复制 |

---

## 六、参考文档

- MySQL 5.7 Reference: Semisynchronous Replication
- MySQL 5.7 Reference: Parallel Replication
- MySQL 5.7 Reference: Multi-Source Replication
- MySQL 5.7 Reference: XA Transaction
- docs/releases/v2.8.0/MYSQL_83_ROADMAP_AND_MATURITY_ASSESSMENT.md
- docs/releases/v2.8.0/DISTRIBUTED_TEST_DESIGN.md

---

*Issue #119 详细设计*
*最后更新: 2026-05-02*