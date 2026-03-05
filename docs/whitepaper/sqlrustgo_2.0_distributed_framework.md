# SQLRustGo 2.0 分布式执行框架

> **版本**: 2.0 (规划中)
> **代号**: Distributed Engine
> **日期**: 2026-03-05

---

## 1. 2.0 版本目标

SQLRustGo 2.0 引入**分布式查询执行**能力，从单机执行引擎扩展为分布式 SQL 执行系统。

### 核心目标

- **水平扩展**: 支持多节点并行执行
- **数据分片**: 支持数据分布存储
- **查询并行**: 支持分布式查询优化
- **容错处理**: 支持节点故障恢复

---

## 2. 架构概览

### 2.1 系统架构

```
                 Query Coordinator
                         │
                         ▼
                 Distributed Planner
                         │
                         ▼
                  DAG Execution Plan
                 /        |        \
              Node1     Node2     Node3
                │         │         │
                ▼         ▼         ▼
             Result   Result    Result
                │         │         │
                 \        |        /
                      Aggregation
                         │
                         ▼
                     Final Result
```

### 2.2 组件职责

| 组件 | 职责 |
|------|------|
| Query Coordinator | SQL 解析、查询优化、任务调度 |
| Distributed Planner | 生成分布式执行计划 |
| DAG Executor | 多节点协调执行 |
| Worker Node | 实际数据处理 |

---

## 3. 核心组件

### 3.1 Query Coordinator

负责：

- SQL 解析
- 查询优化
- 生成 DAG
- 任务调度
- 结果聚合

```rust
pub struct QueryCoordinator {
    parser: Parser,
    optimizer: DistributedOptimizer,
    scheduler: TaskScheduler,
}
```

### 3.2 Distributed Planner

把 Local Plan 转换为分布式执行计划：

```rust
pub trait DistributedPlanner {
    fn plan(&self, logical_plan: LogicalPlan) -> DistributedPlan;
}
```

#### 转换规则

| 本地算子 | 分布式算子 |
|----------|------------|
| Scan | RemoteScan |
| Filter | RemoteFilter |
| Join | ShuffleJoin / BroadcastJoin |
| Aggregate | PartialAggregate + FinalAggregate |

### 3.3 Exchange 算子

核心数据交换算子：

```rust
pub enum Exchange {
    /// Hash 分区
    Hash { key: Expr },
    /// 广播
    Broadcast,
    /// 随机分区
    RoundRobin,
    /// 排序分区
    Sort { key: Expr },
}
```

#### Exchange 策略选择

| 数据量 | 推荐策略 |
|--------|----------|
| 小表 | BroadcastJoin |
| 大表等分 | HashPartition |
| 无排序要求 | RoundRobin |

### 3.4 Worker Node

每个 Worker 节点运行：

```rust
pub struct WorkerNode {
    pub node_id: NodeId,
    pub storage: Box<dyn StorageEngine>,
    pub executor: LocalExecutor,
    pub network: NetworkClient,
}
```

职责：

- 本地数据扫描
- Join/Filter 执行
- 结果返回 Coordinator

---

## 4. DAG 执行模型

### 4.1 执行流程

```
     Scan
       │
       │ (Shuffle by join key)
       ▼
   Exchange
       │
       ▼
     Join
       │
       │ (Partition by group key)
       ▼
   Exchange
       │
       ▼
  Aggregate
       │
       ▼
   Result
```

### 4.2 Stage 划分

```
Query: SELECT a.id, COUNT(*) FROM a JOIN b ON a.id = b.id GROUP BY a.id

Stage 1:
  ├── Node1: Scan(a) → Hash(id) → Exchange
  └── Node2: Scan(b) → Hash(id) → Exchange

Stage 2:
  ├── Node1: Receive → Join
  └── Node2: Receive → Join

Stage 3:
  ├── Node1: Aggregate → Result
  └── Node2: Aggregate → Result

Stage 4:
  Coordinator: Merge Results → Final
```

---

## 5. 数据分布策略

### 5.1 Distribution 类型

```rust
pub enum Distribution {
    /// 单机存储
    Single,
    /// 哈希分片
    Hash { key: String, shard_count: usize },
    /// 范围分片
    Range { key: String, ranges: Vec<Range> },
    /// 全量复制
    Replicated { replica_count: usize },
}
```

### 5.2 Sharding 规则

| 分片方式 | 适用场景 | 示例 |
|----------|----------|------|
| Hash | 等值查询 | `user_id % 10` |
| Range | 范围查询 | 按时间区间 |
| Replicated | 小表 | 配置表 |

---

## 6. 1.2 接口兼容性

### 6.1 扩展原则

SQLRustGo 2.0 仍然使用 1.2 冻结的接口：

```rust
// 1.2 冻结接口
pub trait Operator: Send {
    fn open(&mut self);
    fn next_batch(&mut self) -> Option<RecordBatch>;
    fn close(&mut self);
}
```

### 6.2 接口扩展方式

| 1.2 接口 | 2.0 扩展 |
|----------|----------|
| Operator | + async fn |
| RecordBatch | + metadata |
| PlanNode | + Distributed variant |
| Executor | + execute_distributed |

### 6.3 分布式兼容层

```rust
// 分布式执行器（扩展自 LocalExecutor）
pub struct DistributedExecutor {
    local_executor: LocalExecutor,
    coordinator: QueryCoordinator,
}

impl DistributedExecutor {
    pub fn execute(&self, plan: DistributedPlan) -> Result<ResultSet> {
        // 1. 拆分 Stage
        let stages = self.split_stages(plan);
        
        // 2. 并行执行
        let results = self.execute_stages(stages);
        
        // 3. 聚合结果
        self.aggregate_results(results)
    }
}
```

---

## 7. 容错机制

### 7.1 故障类型

| 故障类型 | 检测方式 | 恢复策略 |
|----------|----------|----------|
| Worker 节点宕机 |心跳检测| 任务重新调度 |
| 网络分区 | 超时检测 | 重新执行 |
| 数据丢失 | Checksum | 重新获取 |

### 7.2 恢复策略

```rust
pub enum FailureRecovery {
    /// 重新执行
    Retry,
    /// 切换到备用节点
    Failover,
    /// 回滚事务
    Rollback,
}
```

---

## 8. 性能优化

### 8.1 优化策略

| 优化手段 | 效果 |
|----------|------|
| Predicate Pushdown | 减少网络传输 |
| Columnar Transfer | 减少序列化开销 |
| Data Locality | 减少跨节点访问 |
| Pipelining | 减少等待时间 |

### 8.2 性能目标

| 指标 | 1.x (单机) | 2.0 (3节点) |
|------|------------|-------------|
| 100万行 Join | <1s | <500ms |
| 1000万行聚合 | <5s | <2s |
| 水平扩展 | 1x | ~2.5x |

---

## 9. 路线图

### 9.1 2.0 发布计划

| 阶段 | 功能 | 目标版本 |
|------|------|----------|
| Phase 1 | 基础分布式执行 | 2.0 |
| Phase 2 | Sharding 支持 | 2.1 |
| Phase 3 | 容错机制 | 2.2 |
| Phase 4 | 完整事务支持 | 2.3 |

### 9.2 依赖关系

```
2.0 分布式执行
 │
 ├── 1.2 接口冻结 (✅ 已完成)
 │
 ├── 2.0.1 Exchange 算子
 ├── 2.0.2 DAG 执行器
 ├── 2.0.3 Coordinator
 │
 └── 2.1 Sharding
      │
      ├── 2.1.1 Hash Distribution
      ├── 2.1.2 Range Distribution
      └── 2.1.3 Rebalancing
```

---

## 10. 总结

### SQLRustGo 路线

```
1.x ──────► 2.x ──────► 3.x
 │             │             │
 ▼             ▼             ▼
单机执行    分布式执行    云原生数据库
引擎
```

### 2.0 核心价值

- **水平扩展**: 通过添加节点提升性能
- **容错能力**: 节点故障自动恢复
- **接口兼容**: 保持 1.x API 兼容性

---

## 附录：快速链接

- [发布白皮书](./sqlrustgo_1.2_release_whitepaper.md)
- [接口冻结](./sqlrustgo_1.2_interface_freeze.md)
- [技术债预测](./sqlrustgo_tech_debt_forecast.md)
- [2.0 路线图](../v2.0/SQLRUSTGO_2_0_ROADMAP.md)
