# 2PC 分布式事务实现设计规格

> **版本**: v1.0.0
> **日期**: 2026-03-28
> **Issue**: #944
> **状态**: 设计完成，待实现

---

## 1. 概述

实现去中心化 Two-Phase Commit (2PC) 分布式事务协议，支持跨节点事务的原子性和一致性。

### 1.1 设计目标

- **去中心化协调器**: 无单点故障，每个节点都可能成为协调者
- **DTC/Saga 故障恢复**: 使用事务日志确保故障后能继续未完成的事务
- **gRPC 通信**: 高性能 RPC 通信，支持双向流
- **混合路由模式**: 简单事务高效，跨节点事务可靠

### 1.2 架构决策

| 决策点 | 选择 |
|--------|------|
| 协调器架构 | 去中心化 (类似 CockroachDB) |
| 故障恢复 | DTC/Saga + WAL |
| 通信层 | gRPC |
| 数据路由 | 混合模式 (表级路由 + GID 全局管理) |

---

## 2. 系统架构

### 2.1 节点架构

```
┌─────────────────────────────────────────────────────────────┐
│                    分布式节点 (DistributedNode)             │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────────────────────────────────────────────────┐ │
│  │                   Network Layer (gRPC)                 │ │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  │ │
│  │  │ gRPC Server │  │ gRPC Client │  │  Protocol   │  │ │
│  │  │             │  │             │  │  Buffers    │  │ │
│  │  └─────────────┘  └─────────────┘  └─────────────┘  │ │
│  └──────────────────────────────────────────────────────┘ │
│                            │                                │
│  ┌──────────────────────────────────────────────────────┐ │
│  │              Transaction Coordinator                   │ │
│  │  ┌───────────┐  ┌───────────┐  ┌───────────┐        │ │
│  │  │  Router   │  │ Coordinator│  │ Participant│        │ │
│  │  │           │  │  Manager   │  │  Manager   │        │ │
│  │  └───────────┘  └───────────┘  └───────────┘        │ │
│  └──────────────────────────────────────────────────────┘ │
│                            │                                │
│  ┌──────────────────────────────────────────────────────┐ │
│  │              TransactionManager                        │ │
│  │  ┌───────────┐  ┌───────────┐  ┌───────────┐        │ │
│  │  │ WAL Log   │  │ GID Pool  │  │ Lock Mgr  │        │ │
│  │  │           │  │           │  │           │        │ │
│  │  └───────────┘  └───────────┘  └───────────┘        │ │
│  └──────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 组件职责

| 组件 | 职责 |
|------|------|
| `DistributedNode` | 分布式节点入口，管理所有子组件 |
| `Coordinator` | 事务协调者，发起和协调分布式事务 |
| `Participant` | 事务参与者，执行本地事务操作 |
| `Router` | 根据 SQL 条件确定数据所在节点 |
| `GIDManager` | 全局事务 ID 管理 |
| `WALManager` | 预写日志，故障恢复 |
| `LockManager` | 分布式锁管理 |

---

## 3. 数据结构

### 3.1 全局事务 ID (GID)

```rust
// crates/transaction/src/gid.rs

/// 全局唯一事务 ID
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GlobalTransactionId {
    /// 节点 ID (发起协调的节点)
    pub node_id: NodeId,
    /// 事务编号 (本地递增)
    pub txn_id: u64,
    /// 事务开始时间戳
    pub timestamp: u64,
}

/// 节点 ID
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeId(pub u64);
```

### 3.2 事务状态

```rust
// crates/transaction/src/dtc.rs

/// 分布式事务状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DistributedTransactionState {
    /// 事务开始
    Started,
    /// Phase 1: 等待参与者投票
    Preparing,
    /// 参与者已投票，准备提交
    Prepared,
    /// Phase 2: 正在提交
    Committing,
    /// 已提交
    Committed,
    /// 正在回滚
    RollingBack,
    /// 已回滚
    RolledBack,
    /// 事务终止 (不可恢复)
    Terminated,
}

/// 参与者投票结果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Vote {
    VoteCommit,
    VoteAbort,
}

/// 事务协调消息
#[derive(Debug, Clone)]
pub enum CoordinatorMessage {
    /// Phase 1: 准备请求
    PrepareRequest {
        gid: GlobalTransactionId,
        changes: Vec<Change>,
    },
    /// Phase 2: 提交请求
    CommitRequest {
        gid: GlobalTransactionId,
    },
    /// Phase 2: 回滚请求
    RollbackRequest {
        gid: GlobalTransactionId,
    },
}

/// 参与者响应
#[derive(Debug, Clone)]
pub enum ParticipantResponse {
    /// 投票结果
    VoteResponse {
        gid: GlobalTransactionId,
        node_id: NodeId,
        vote: Vote,
        reason: Option<String>,
    },
    /// 执行结果
    ExecutionResult {
        gid: GlobalTransactionId,
        node_id: NodeId,
        success: bool,
        affected_rows: u64,
    },
}

/// 数据变更描述
#[derive(Debug, Clone)]
pub struct Change {
    pub table: String,
    pub operation: ChangeOperation,
    pub key: Value,
    pub value: Option<Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeOperation {
    Insert,
    Update,
    Delete,
}
```

### 3.3 WAL 日志条目

```rust
// crates/transaction/src/wal.rs

/// WAL 日志条目
#[derive(Debug, Clone)]
pub enum WalEntry {
    /// 事务开始
    TxBegin {
        gid: GlobalTransactionId,
        timestamp: u64,
    },
    /// 事务准备 (Phase 1 完成)
    TxPrepare {
        gid: GlobalTransactionId,
        participants: Vec<NodeId>,
        timestamp: u64,
    },
    /// 事务提交 (Phase 2 完成)
    TxCommit {
        gid: GlobalTransactionId,
        timestamp: u64,
    },
    /// 事务回滚
    TxRollback {
        gid: GlobalTransactionId,
        reason: String,
        timestamp: u64,
    },
    /// 事务终止 (需要人工介入)
    TxTerminate {
        gid: GlobalTransactionId,
        reason: String,
        timestamp: u64,
    },
}
```

---

## 4. gRPC 协议定义

### 4.1 Proto 文件

```protobuf
// crates/network/protos/dtc.proto

syntax = "proto3";

package sqlrustgo.dtc;

service DistributedTransactionService {
    /// Phase 1: 协调者发送准备请求
    rpc Prepare(PrepareRequest) returns (VoteResponse);

    /// Phase 2: 协调者发送提交/回滚请求
    rpc Commit(CommitRequest) returns (ExecutionResponse);
    rpc Rollback(RollbackRequest) returns (ExecutionResponse);

    /// 参与者主动通知协调者
    rpc Notify(Notification) returns (Empty);

    /// 健康检查
    rpc Ping(NodePing) returns (NodePong);
}

message PrepareRequest {
    string gid = 1;
    string coordinator_node_id = 2;
    repeated Change changes = 3;
}

message VoteResponse {
    string gid = 1;
    string node_id = 2;
    VoteType vote = 3;
    string reason = 4;
}

enum VoteType {
    VOTE_COMMIT = 0;
    VOTE_ABORT = 1;
}

message CommitRequest {
    string gid = 1;
}

message RollbackRequest {
    string gid = 1;
    string reason = 2;
}

message ExecutionResponse {
    string gid = 1;
    string node_id = 2;
    bool success = 3;
    uint64 affected_rows = 4;
    string error = 5;
}

message Change {
    string table = 1;
    ChangeOperation operation = 2;
    bytes key = 3;
    bytes value = 4;
}

enum ChangeOperation {
    INSERT = 0;
    UPDATE = 1;
    DELETE = 2;
}

message Notification {
    string gid = 1;
    string node_id = 2;
    NotificationType type = 3;
    string details = 4;
}

enum NotificationType {
    PARTICIPANT_DOWN = 0;
    RECOVERY_COMPLETE = 1;
    LOCK_TIMEOUT = 2;
}

message NodePing {
    string node_id = 1;
}

message NodePong {
    string node_id = 1;
    bool healthy = 2;
}

message Empty {}
```

---

## 5. 核心协议实现

### 5.1 事务协调者 (Coordinator)

```rust
// crates/transaction/src/coordinator.rs

pub struct Coordinator {
    node_id: NodeId,
    gid_manager: Arc<GIDManager>,
    wal_manager: Arc<WalManager>,
    lock_manager: Arc<DistributedLockManager>,
    participant_registry: Arc<ParticipantRegistry>,
    pending_transactions: RwLock<HashMap<GlobalTransactionId, TransactionContext>>,
    grpc_client: GrpcClientPool,
}

impl Coordinator {
    /// 发起分布式事务
    pub async fn begin_distributed_tx(&self, sql: &str) -> SqlResult<DistributedTransactionId> {
        let gid = self.gid_manager.generate(self.node_id)?;

        // 解析 SQL 确定涉及的表和节点
        let plan = self.plan_sql(sql)?;
        let participants = self.router.resolve_participants(&plan)?;

        // 记录事务开始
        self.wal_manager.log(WalEntry::TxBegin { gid, timestamp: now() })?;

        // 创建事务上下文
        let ctx = TransactionContext {
            gid: gid.clone(),
            state: DistributedTransactionState::Started,
            participants,
            changes: Vec::new(),
        };

        self.pending_transactions.write().insert(gid.clone(), ctx);

        Ok(DistributedTransactionId(gid))
    }

    /// Phase 1: 准备阶段
    pub async fn prepare(&self, gid: &GlobalTransactionId) -> SqlResult<PrepareResult> {
        let mut ctx = self.pending_transactions.write()
            .get_mut(gid)
            .ok_or(SqlError::TransactionNotFound)?;

        ctx.state = DistributedTransactionState::Preparing;

        // 向所有参与者发送 Prepare 请求
        let votes = self.send_prepare_requests(gid, &ctx.changes).await?;

        // 检查所有投票
        let all_commit = votes.iter().all(|v| v == &Vote::VoteCommit);

        if all_commit {
            ctx.state = DistributedTransactionState::Prepared;
            self.wal_manager.log(WalEntry::TxPrepare {
                gid: gid.clone(),
                participants: ctx.participants.clone(),
                timestamp: now(),
            })?;
            Ok(PrepareResult::AllCommitted)
        } else {
            ctx.state = DistributedTransactionState::RollingBack;
            self.wal_manager.log(WalEntry::TxRollback {
                gid: gid.clone(),
                reason: "Participant voted abort".to_string(),
                timestamp: now(),
            })?;
            Ok(PrepareResult::NeedsRollback)
        }
    }

    /// Phase 2: 提交阶段
    pub async fn commit(&self, gid: &GlobalTransactionId) -> SqlResult<CommitResult> {
        let mut ctx = self.pending_transactions.write()
            .get_mut(gid)
            .ok_or(SqlError::TransactionNotFound)?;

        ctx.state = DistributedTransactionState::Committing;

        // 向所有参与者发送 Commit 请求
        let results = self.send_commit_requests(gid).await?;

        ctx.state = DistributedTransactionState::Committed;
        self.wal_manager.log(WalEntry::TxCommit {
            gid: gid.clone(),
            timestamp: now(),
        })?;

        // 清理本地状态
        self.pending_transactions.write().remove(gid);

        Ok(CommitResult { results })
    }

    /// 回滚事务
    pub async fn rollback(&self, gid: &GlobalTransactionId, reason: &str) -> SqlResult<()> {
        let mut ctx = self.pending_transactions.write()
            .get_mut(gid)
            .ok_or(SqlError::TransactionNotFound)?;

        ctx.state = DistributedTransactionState::RollingBack;

        // 向所有参与者发送 Rollback 请求
        self.send_rollback_requests(gid).await?;

        ctx.state = DistributedTransactionState::RolledBack;
        self.wal_manager.log(WalEntry::TxRollback {
            gid: gid.clone(),
            reason: reason.to_string(),
            timestamp: now(),
        })?;

        self.pending_transactions.write().remove(gid);

        Ok(())
    }
}
```

### 5.2 事务参与者 (Participant)

```rust
// crates/transaction/src/participant.rs

pub struct Participant {
    node_id: NodeId,
    storage: Arc<dyn StorageEngine>,
    wal_manager: Arc<WalManager>,
    lock_manager: Arc<DistributedLockManager>,
    grpc_server: GrpcServer,
}

impl Participant {
    /// 处理协调者的 Prepare 请求
    pub async fn handle_prepare(&self, request: PrepareRequest) -> SqlResult<VoteResponse> {
        let gid = parse_gid(&request.gid)?;

        // 尝试获取所有需要的锁
        let lock_result = self.lock_manager.try_lock(&gid, &request.changes).await;

        match lock_result {
            Ok(_) => {
                // 记录本地 Prepare 日志
                self.wal_manager.log(WalEntry::TxPrepare {
                    gid: gid.clone(),
                    participants: vec![self.node_id],
                    timestamp: now(),
                })?;

                Ok(VoteResponse {
                    gid: request.gid,
                    node_id: self.node_id.to_string(),
                    vote: VoteType::VoteCommit as i32,
                    reason: None,
                })
            }
            Err(LockError::Conflict) => {
                Ok(VoteResponse {
                    gid: request.gid,
                    node_id: self.node_id.to_string(),
                    vote: VoteType::VoteAbort as i32,
                    reason: Some("Lock conflict".to_string()),
                })
            }
            Err(e) => Err(e.into()),
        }
    }

    /// 处理协调者的 Commit 请求
    pub async fn handle_commit(&self, request: CommitRequest) -> SqlResult<ExecutionResponse> {
        let gid = parse_gid(&request.gid)?;

        // 从 WAL 恢复事务状态
        let prepared = self.wal_manager.read_prepared(&gid)?;

        if !prepared {
            return Err(SqlError::Internal("Transaction not in prepared state".to_string()));
        }

        // 执行提交
        let affected_rows = self.storage.commit_transaction(&gid)?;

        // 释放锁
        self.lock_manager.unlock(&gid).await;

        // 清理 WAL
        self.wal_manager.mark_committed(&gid)?;

        Ok(ExecutionResponse {
            gid: request.gid,
            node_id: self.node_id.to_string(),
            success: true,
            affected_rows,
            error: None,
        })
    }

    /// 处理协调者的 Rollback 请求
    pub async fn handle_rollback(&self, request: RollbackRequest) -> SqlResult<ExecutionResponse> {
        let gid = parse_gid(&request.gid)?;

        // 执行回滚
        let affected_rows = self.storage.rollback_transaction(&gid)?;

        // 释放锁
        self.lock_manager.unlock(&gid).await;

        // 清理 WAL
        self.wal_manager.mark_rolled_back(&gid)?;

        Ok(ExecutionResponse {
            gid: request.gid,
            node_id: self.node_id.to_string(),
            success: true,
            affected_rows,
            error: None,
        })
    }
}
```

### 5.3 故障恢复 (Recovery)

```rust
// crates/transaction/src/recovery.rs

impl WALManager {
    /// 故障恢复：扫描未完成的事务并恢复
    pub async fn recover(&self) -> SqlResult<RecoveryReport> {
        let mut report = RecoveryReport::default();

        // 扫描所有未 Commit 的日志条目
        let incomplete_txs = self.scan_incomplete_transactions()?;

        for entry in incomplete_txs {
            match entry {
                WalEntry::TxBegin { gid, .. } => {
                    // 事务从未完成 Phase 1，回滚
                    self.rollback_incomplete_tx(&gid, "Node crash before prepare")?;
                    report.rolled_back += 1;
                }
                WalEntry::TxPrepare { gid, participants, .. } => {
                    // 等待协调者指令或主动查询协调者状态
                    let outcome = self.query_coordinator_for_outcome(&gid, &participants).await?;
                    match outcome {
                        TxOutcome::Committed => {
                            self.mark_committed(&gid)?;
                            report.committed += 1;
                        }
                        TxOutcome::RolledBack => {
                            self.mark_rolled_back(&gid)?;
                            report.rolled_back += 1;
                        }
                        TxOutcome::Unknown => {
                            // 协调者也挂了，需要人工介入
                            self.mark_terminated(&gid, "Coordinator and participant both uncertain")?;
                            report.terminated += 1;
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(report)
    }
}
```

---

## 6. 文件清单

| 文件 | 操作 | 说明 |
|------|------|------|
| `crates/network/Cargo.toml` | 新建 | Network crate 依赖 |
| `crates/network/protos/dtc.proto` | 新建 | gRPC 协议定义 |
| `crates/network/src/lib.rs` | 新建 | Network crate 入口 |
| `crates/network/src/server.rs` | 新建 | gRPC Server 实现 |
| `crates/network/src/client.rs` | 新建 | gRPC Client 实现 |
| `crates/transaction/src/gid.rs` | 新建 | GID 和 NodeId 定义 |
| `crates/transaction/src/dtc.rs` | 新建 | 分布式事务核心类型 |
| `crates/transaction/src/coordinator.rs` | 新建 | 协调者实现 |
| `crates/transaction/src/participant.rs` | 新建 | 参与者实现 |
| `crates/transaction/src/router.rs` | 新建 | 路由逻辑 |
| `crates/transaction/src/lock_manager.rs` | 新建 | 分布式锁管理 |
| `crates/transaction/src/recovery.rs` | 新建 | 故障恢复实现 |
| `crates/transaction/src/wal.rs` | 修改 | 添加 DTC 日志类型 |
| `crates/transaction/src/lib.rs` | 修改 | 导出新模块 |
| `crates/transaction/src/manager.rs` | 修改 | 集成分布式事务 |

---

## 7. 验收标准

- [ ] gRPC 网络层能正常启动和通信
- [ ] 能生成全局唯一事务 ID (GID)
- [ ] Phase 1 (Prepare) 能正确收集所有参与者的投票
- [ ] Phase 2 (Commit/Rollback) 能正确通知所有参与者
- [ ] 节点故障后能通过 WAL 恢复未完成的事务
- [ ] 去中心化协调：任何节点都能发起事务
- [ ] 分布式锁能正确防止死锁
- [ ] 单元测试覆盖核心逻辑
- [ ] 集成测试验证端到端流程

---

## 8. 风险与限制

1. **gRPC 依赖**: 需要引入 tonic 等 crates
2. **网络分区**: 脑裂情况需要更复杂的共识协议 (如 Raft)
3. **性能**: 2PC 有额外的网络延迟，需要优化批处理
4. **锁粒度**: 当前实现为表级锁，可扩展到行级锁

---

*设计完成，等待用户批准后进入实现阶段*
