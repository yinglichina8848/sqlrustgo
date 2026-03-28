# Issue #944: Phase 3 分布式能力详细设计

**Issue**: #944 Phase 3: 分布式能力 - Sharding/分布式事务
**创建日期**: 2026-03-28
**状态**: 设计完成
**架构**: Raft + 2PC 混合架构

## 1. 架构概述

### 1.1 整体架构

```
┌─────────────────────────────────────────────────────────────┐
│                        Cluster                               │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│  │   Node 1    │  │   Node 2    │  │   Node 3    │        │
│  │  ┌───────┐  │  │  ┌───────┐  │  │  ┌───────┐  │        │
│  │  │ Shard │  │  │  │ Shard │  │  │  │ Shard │  │        │
│  │  │   A    │  │  │  │   B    │  │  │  │   C    │  │        │
│  │  └───────┘  │  │  └───────┘  │  │  └───────┘  │        │
│  │  ┌───────┐  │  │  ┌───────┐  │  │  ┌───────┐  │        │
│  │  │  Raft │  │  │  │  Raft │  │  │  │  Raft │  │        │
│  │  │ State │  │  │  │ State │  │  │  │ State │  │        │
│  │  └───────┘  │  │  └───────┘  │  │  └───────┘  │        │
│  │  ┌───────┐  │  │  ┌───────┐  │  │  ┌───────┐  │        │
│  │  │ 2PC   │  │  │  │ 2PC   │  │  │  │ 2PC   │  │        │
│  │  │Participant│ │  │ │Participant│ │  │ │Participant│ │        │
│  │  └───────┘  │  │  └───────┘  │  │  └───────┘  │        │
│  └──────┬──────┘  └─────────────┘  └─────────────┘        │
│         │                                                  │
│    Raft Leader = 2PC Coordinator                           │
└─────────────────────────────────────────────────────────────┘
```

### 1.2 核心组件

| 组件 | 职责 |
|------|------|
| `ShardManager` | 分片路由、分片策略管理 |
| `ShardRouter` | SQL 路由到正确分片 |
| `RaftNode` | Leader 选举、日志复制 |
| `TwoPhaseCommit` | 事务协调器 |
| `DistributedLock` | 分布式锁管理 |
| `PartitionStrategy` | 分区策略（哈希/范围） |

## 2. 数据模型

### 2.1 ShardInfo

```rust
#[derive(Debug, Clone)]
pub struct ShardInfo {
    pub shard_id: u64,
    pub nodes: Vec<NodeId>,  // 副本节点
    pub partition_key: PartitionKey,
    pub status: ShardStatus,
}

#[derive(Debug, Clone)]
pub struct PartitionKey {
    pub column: String,
    pub strategy: PartitionStrategy,
}

#[derive(Debug, Clone, Copy)]
pub enum PartitionStrategy {
    Hash(u64),      // 哈希分区，mod 值
    Range { start: i64, end: i64 },  // 范围分区
}

#[derive(Debug, Clone, Copy)]
pub enum ShardStatus {
    Active,
    Migrating,
    Readonly,
}
```

### 2.2 DistributedTransaction

```rust
#[derive(Debug, Clone)]
pub struct DistributedTransaction {
    pub tx_id: u64,
    pub coordinator_id: NodeId,
    pub participants: Vec<Participant>,
    pub state: TransactionState,
    pub created_at: u64,
    pub timeout: Duration,
}

#[derive(Debug, Clone, Copy)]
pub enum TransactionState {
    Init,
    Preparing,
    Prepared,
    Committing,
    Committed,
    Aborting,
    Aborted,
}

#[derive(Debug, Clone)]
pub struct Participant {
    pub node_id: NodeId,
    pub shard_id: u64,
    pub prepared: bool,
    pub vote: Vote,
}

#[derive(Debug, Clone, Copy)]
pub enum Vote {
    Yes,
    No,
}
```

### 2.3 RaftMessage

```rust
#[derive(Debug, Clone)]
pub enum RaftMessage {
    RequestVote(RequestVoteArgs),
    RequestVoteResponse(RequestVoteResponse),
    AppendEntries(AppendEntriesArgs),
    AppendEntriesResponse(AppendEntriesResponse),
    InstallSnapshot(InstallSnapshotArgs),
}

#[derive(Debug, Clone)]
pub struct RequestVoteArgs {
    pub term: u64,
    pub candidate_id: NodeId,
    pub last_log_index: u64,
    pub last_log_term: u64,
}

#[derive(Debug, Clone)]
pub struct AppendEntriesArgs {
    pub term: u64,
    pub leader_id: NodeId,
    pub prev_log_index: u64,
    pub prev_log_term: u64,
    pub entries: Vec<RaftEntry>,
    pub leader_commit: u64,
}
```

## 3. 核心模块设计

### 3.1 ShardManager

```rust
pub struct ShardManager {
    shards: HashMap<u64, ShardInfo>,
    partition_rules: Vec<PartitionRule>,
    node_shards: HashMap<NodeId, Vec<u64>>,
}

impl ShardManager {
    /// 创建分片
    pub fn create_shard(&mut self, info: ShardInfo) -> Result<()>;
    
    /// 添加分区规则
    pub fn add_partition_rule(&mut self, rule: PartitionRule) -> Result<()>;
    
    /// 路由到分片
    pub fn route(&self, table: &str, key: &Value) -> Result<u64>;
    
    /// 获取分片的活跃节点
    pub fn get_active_nodes(&self, shard_id: u64) -> Result<Vec<NodeId>>;
}
```

### 3.2 ShardRouter

```rust
pub struct ShardRouter {
    shard_manager: Arc<ShardManager>,
    local_node_id: NodeId,
}

impl ShardRouter {
    /// 解析 SQL 并路由
    pub fn route_sql(&self, stmt: &Statement) -> Result<RoutedPlan>;
    
    /// 路由单个键值查询
    pub fn route_point_query(&self, table: &str, key: &Value) -> Result<NodeId>;
    
    /// 路由范围查询
    pub fn route_range_query(&self, table: &str, range: &Range<Value>) -> Result<Vec<NodeId>>;
    
    /// 路由跨分片查询 (JOIN/AGG)
    pub fn route_distributed_query(&self, stmt: &Statement) -> Result<DistributedPlan>;
}
```

### 3.3 RaftNode

```rust
pub struct RaftNode {
    node_id: NodeId,
    peers: Vec<NodeId>,
    state: RaftState,
    current_term: u64,
    voted_for: Option<NodeId>,
    log: RaftLog,
    commit_index: u64,
    last_applied: u64,
}

#[derive(Debug, Clone, Copy)]
pub enum RaftState {
    Follower,
    Candidate,
    Leader,
}

impl RaftNode {
    /// 启动 Raft 节点
    pub fn new(node_id: NodeId, peers: Vec<NodeId>) -> Self;
    
    /// 处理 Raft 消息
    pub fn handle_message(&mut self, msg: RaftMessage) -> Vec<RaftMessage>;
    
    /// 成为 Leader
    pub fn become_leader(&mut self);
    
    /// 复制日志到 follower
    pub fn replicate(&self, entries: Vec<RaftEntry>) -> Result<()>;
}
```

### 3.4 TwoPhaseCommit

```rust
pub struct TwoPhaseCommit {
    node_id: NodeId,
    transactions: HashMap<u64, DistributedTransaction>,
    raft: Arc<RaftNode>,
    lock_manager: Arc<DistributedLockManager>,
}

impl TwoPhaseCommit {
    /// 开始分布式事务
    pub fn begin_transaction(&mut self) -> u64;
    
    /// Phase 1: Prepare
    pub fn prepare(&mut self, tx_id: u64) -> Result<PrepareResult>;
    
    /// Phase 2: Commit or Abort
    pub fn commit(&mut self, tx_id: u64) -> Result<()>;
    pub fn abort(&mut self, tx_id: u64) -> Result<()>;
    
    /// 处理参与者响应
    pub fn handle_vote(&mut self, tx_id: u64, voter: NodeId, vote: Vote);
}
```

### 3.5 DistributedLockManager

```rust
pub struct DistributedLockManager {
    locks: HashMap<String, LockEntry>,
    lock_timeout: Duration,
}

#[derive(Debug, Clone)]
pub struct LockEntry {
    pub key: String,
    pub owner: NodeId,
    pub tx_id: u64,
    pub acquired_at: u64,
}

impl DistributedLockManager {
    /// 尝试获取锁
    pub fn try_lock(&mut self, key: &str, node_id: NodeId, tx_id: u64) -> bool;
    
    /// 释放锁
    pub fn unlock(&mut self, key: &str, node_id: NodeId, tx_id: u64) -> Result<()>;
    
    /// 锁超时检测
    pub fn check_timeouts(&mut self) -> Vec<String>;  // 返回超时的锁 key
}
```

## 4. 分区策略

### 4.1 哈希分区

```rust
fn hash_partition(key: &Value, num_shards: u64) -> u64 {
    match key {
        Value::Integer(n) => (*n as u64) % num_shards,
        Value::Text(s) => {
            let hash = calculate_hash(s);
            hash % num_shards
        }
        _ => unimplemented!(),
    }
}
```

### 4.2 范围分区

```rust
fn range_partition(key: &Value, ranges: &[(i64, i64)]) -> Option<u64> {
    let n = match key {
        Value::Integer(i) => *i,
        _ => return None,
    };
    
    for (i, (start, end)) in ranges.iter().enumerate() {
        if n >= *start && n < *end {
            return Some(i as u64);
        }
    }
    None
}
```

## 5. 2PC 协议流程

### 5.1 正常提交流程

```
Coordinator                    Participant1                Participant2
    │                               │                           │
    │──── BEGIN TX ──────────────▶│                           │
    │                               │                           │
    │◀──── OK (tx_id=100) ─────────│                           │
    │                               │                           │
    │──── PREPARE ────────────────▶│──── PREPARE ────────────▶│
    │                               │                           │
    │◀──── VOTE YES ──────────────│◀─── VOTE YES ─────────────│
    │                               │                           │
    │──── COMMIT ──────────────────▶│──── COMMIT ──────────────▶│
    │                               │                           │
    │◀──── ACK ────────────────────│◀─── ACK ──────────────────│
    │                               │                           │
```

### 5.2 回滚流程

```
Coordinator                    Participant1                Participant2
    │                               │                           │
    │──── BEGIN TX ──────────────▶│                           │
    │                               │                           │
    │──── PREPARE ────────────────▶│──── PREPARE ────────────▶│
    │                               │                           │
    │◀──── VOTE NO (原因: 锁冲突) ─│◀─── VOTE YES ─────────────│
    │                               │                           │
    │──── ABORT ───────────────────▶│──── ABORT ───────────────▶│
    │                               │                           │
```

## 6. Raft Leader 选举

```rust
fn election_timeout() -> Duration {
    // 随机超时 150-300ms
    Duration::from_millis(150 + rand::random::<u64>() % 150)
}

impl RaftNode {
    fn start_election(&mut self) {
        self.state = RaftState::Candidate;
        self.current_term += 1;
        self.voted_for = Some(self.node_id);
        
        let request = RaftMessage::RequestVote(RequestVoteArgs {
            term: self.current_term,
            candidate_id: self.node_id,
            last_log_index: self.log.last_index(),
            last_log_term: self.log.last_term(),
        });
        
        // 发送给所有 peer
        for peer in &self.peers {
            self.send_message(*peer, request.clone());
        }
    }
}
```

## 7. 文件结构

```
crates/
├── distributed/
│   ├── src/
│   │   ├── lib.rs
│   │   ├── shard_manager.rs    # 分片管理
│   │   ├── shard_router.rs    # 路由逻辑
│   │   ├── raft.rs            # Raft 协议
│   │   ├── two_phase_commit.rs # 2PC 协议
│   │   ├── distributed_lock.rs # 分布式锁
│   │   └── partition.rs       # 分区策略
│   └── Cargo.toml
```

## 8. 实现计划

### Phase 1: Sharding 基础
1. `partition.rs` - 分区策略（哈希/范围）
2. `shard_manager.rs` - 分片元数据管理
3. `shard_router.rs` - 路由逻辑

### Phase 2: Raft 协议
1. `raft.rs` - Leader 选举
2. 日志复制
3. 心跳机制

### Phase 3: 2PC 分布式事务
1. `two_phase_commit.rs` - 事务协调器
2. `distributed_lock.rs` - 分布式锁
3. 事务日志和恢复

### Phase 4: 集成测试
1. 单节点分片测试
2. Leader 选举测试
3. 2PC 提交/回滚测试
4. 跨分片查询测试

## 9. 验收标准

- [ ] 支持水平扩展（增加节点自动重平衡）
- [ ] 分布式事务 ACID（通过 2PC）
- [ ] 单表可分区（哈希/范围）
- [ ] Leader 故障自动切换
- [ ] 跨分片查询正确路由
