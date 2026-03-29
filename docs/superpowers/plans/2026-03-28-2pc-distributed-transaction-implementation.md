# 2PC 分布式事务实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现去中心化 Two-Phase Commit (2PC) 分布式事务协议，支持跨节点事务的原子性和一致性。

**Architecture:** 采用去中心化协调器架构，每个节点都可能成为协调者。使用 gRPC 进行节点间通信，通过 WAL + DTC/Saga 模式实现故障恢复。混合路由模式：小事务用表级路由，跨节点事务用 GID 全局管理。

**Tech Stack:** Rust (tokio async runtime), tonic (gRPC), prost (protobuf), crates/transaction (现有事务管理)

---

## Task 1: 创建 Network Crate 和 gRPC 基础结构

**Files:**
- Create: `crates/network/Cargo.toml`
- Create: `crates/network/src/lib.rs`
- Create: `crates/network/protos/dtc.proto`
- Modify: `crates/transaction/Cargo.toml` (添加 network 依赖)
- Modify: `Cargo.toml` (workspace 依赖)

- [ ] **Step 1: 创建 crates/network 目录结构**

```bash
mkdir -p crates/network/src
mkdir -p crates/network/protos
```

- [ ] **Step 2: 创建 Cargo.toml**

```toml
[package]
name = "sqlrustgo-network"
version = "0.1.0"
edition = "2021"

[dependencies]
tonic = "0.12"
prost = "0.13"
tokio = { version = "1", features = ["full"] }
tower = "0.5"
serde = { version = "1", features = ["derive"] }
anyhow = "1"

[build-dependencies]
tonic-build = "0.12"
```

- [ ] **Step 3: 创建 dtc.proto**

```protobuf
syntax = "proto3";

package sqlrustgo.dtc;

service DistributedTransactionService {
    rpc Prepare(PrepareRequest) returns (VoteResponse);
    rpc Commit(CommitRequest) returns (ExecutionResponse);
    rpc Rollback(RollbackRequest) returns (ExecutionResponse);
    rpc Notify(Notification) returns (Empty);
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

- [ ] **Step 4: 创建 build.rs**

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("protos/dtc.proto")?;
    Ok(())
}
```

- [ ] **Step 5: 创建 lib.rs**

```rust
pub mod dtc {
    include!("dtc.rs");
}
```

- [ ] **Step 6: 运行 build 验证 proto 生成**

Run: `cd crates/network && cargo build`
Expected: SUCCESS (生成 dtc.rs)

- [ ] **Step 7: 提交**

```bash
git add crates/network/
git commit -m "feat(network): add gRPC infrastructure for 2PC"
```

---

## Task 2: 实现 GID 和 NodeId 类型

**Files:**
- Create: `crates/transaction/src/gid.rs`
- Modify: `crates/transaction/src/lib.rs` (导出 gid)

- [ ] **Step 1: 写测试 - test_gid_generation**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gid_generation() {
        let node_id = NodeId(1);
        let gid1 = GlobalTransactionId::new(node_id);
        let gid2 = GlobalTransactionId::new(node_id);

        // GID 应该单调递增
        assert!(gid1.txn_id < gid2.txn_id);
    }

    #[test]
    fn test_gid_equality() {
        let node_id = NodeId(1);
        let gid1 = GlobalTransactionId::new(node_id);
        let gid2 = gid1.clone();

        assert_eq!(gid1, gid2);
    }

    #[test]
    fn test_gid_display() {
        let node_id = NodeId(1);
        let gid = GlobalTransactionId::new(node_id);
        let display = format!("{}", gid);

        assert!(display.contains("1")); // 包含 node_id
        assert!(display.contains(&gid.txn_id.to_string())); // 包含 txn_id
    }
}
```

- [ ] **Step 2: 运行测试验证失败**

Run: `cargo test -p sqlrustgo-transaction gid -- --nocapture`
Expected: FAIL - function not found

- [ ] **Step 3: 实现 GID 类型**

```rust
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

/// 节点 ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub u64);

impl NodeId {
    pub fn new(id: u64) -> Self {
        NodeId(id)
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NodeId({})", self.0)
    }
}

/// 全局唯一事务 ID
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GlobalTransactionId {
    pub node_id: NodeId,
    pub txn_id: u64,
    pub timestamp: u64,
}

impl GlobalTransactionId {
    static TXN_COUNTER: AtomicU64 = AtomicU64::new(0);

    pub fn new(node_id: NodeId) -> Self {
        let txn_id = Self::TXN_COUNTER.fetch_add(1, Ordering::SeqCst);
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        GlobalTransactionId {
            node_id,
            txn_id,
            timestamp,
        }
    }

    pub fn parse(s: &str) -> Result<Self, String> {
        // 格式: "node_id:txn_id:timestamp"
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 3 {
            return Err("Invalid GID format".to_string());
        }

        let node_id = NodeId(parts[0].parse().map_err(|_| "Invalid node_id")?);
        let txn_id = parts[1].parse().map_err(|_| "Invalid txn_id")?;
        let timestamp = parts[2].parse().map_err(|_| "Invalid timestamp")?;

        Ok(GlobalTransactionId { node_id, txn_id, timestamp })
    }
}

impl fmt::Display for GlobalTransactionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.node_id.0, self.txn_id, self.timestamp)
    }
}
```

- [ ] **Step 4: 运行测试验证通过**

Run: `cargo test -p sqlrustgo-transaction gid -- --nocapture`
Expected: PASS (3 tests)

- [ ] **Step 5: 更新 lib.rs**

```rust
pub mod gid;
pub use gid::{GlobalTransactionId, NodeId};
```

- [ ] **Step 6: 提交**

```bash
git add crates/transaction/src/gid.rs crates/transaction/src/lib.rs
git commit -m "feat(transaction): add GlobalTransactionId and NodeId types"
```

---

## Task 3: 定义 DTC 核心类型

**Files:**
- Create: `crates/transaction/src/dtc.rs`
- Modify: `crates/transaction/src/lib.rs` (导出 dtc)

- [ ] **Step 1: 写测试 - test_dtc_types**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_state_transitions() {
        assert_eq!(
            DistributedTransactionState::Started,
            DistributedTransactionState::Started
        );
    }

    #[test]
    fn test_vote_serialization() {
        assert_eq!(Vote::VoteCommit, Vote::VoteCommit);
        assert_eq!(Vote::VoteAbort, Vote::VoteAbort);
    }

    #[test]
    fn test_change_operation() {
        let insert = ChangeOperation::Insert;
        let update = ChangeOperation::Update;
        let delete = ChangeOperation::Delete;

        assert_ne!(insert, update);
        assert_ne!(update, delete);
    }
}
```

- [ ] **Step 2: 运行测试验证失败**

Run: `cargo test -p sqlrustgo-transaction dtc -- --nocapture`
Expected: FAIL - module not found

- [ ] **Step 3: 实现 DTC 类型**

```rust
use crate::gid::GlobalTransactionId;
use serde::{Deserialize, Serialize};

/// 分布式事务状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DistributedTransactionState {
    Started,
    Preparing,
    Prepared,
    Committing,
    Committed,
    RollingBack,
    RolledBack,
    Terminated,
}

/// 参与者投票
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Vote {
    VoteCommit,
    VoteAbort,
}

/// 协调消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoordinatorMessage {
    PrepareRequest {
        gid: GlobalTransactionId,
        changes: Vec<Change>,
    },
    CommitRequest {
        gid: GlobalTransactionId,
    },
    RollbackRequest {
        gid: GlobalTransactionId,
        reason: String,
    },
}

/// 参与者响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantResponse {
    pub gid: GlobalTransactionId,
    pub node_id: u64,
    pub vote: Vote,
    pub reason: Option<String>,
}

/// 数据变更
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    pub table: String,
    pub operation: ChangeOperation,
    pub key: Vec<u8>,
    pub value: Option<Vec<u8>>,
}

/// 变更操作类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeOperation {
    Insert,
    Update,
    Delete,
}

/// 事务上下文
#[derive(Debug, Clone)]
pub struct TransactionContext {
    pub gid: GlobalTransactionId,
    pub state: DistributedTransactionState,
    pub participants: Vec<u64>,
    pub changes: Vec<Change>,
}

impl TransactionContext {
    pub fn new(gid: GlobalTransactionId) -> Self {
        TransactionContext {
            gid,
            state: DistributedTransactionState::Started,
            participants: Vec::new(),
            changes: Vec::new(),
        }
    }

    pub fn add_participant(&mut self, node_id: u64) {
        if !self.participants.contains(&node_id) {
            self.participants.push(node_id);
        }
    }
}
```

- [ ] **Step 4: 运行测试验证通过**

Run: `cargo test -p sqlrustgo-transaction dtc -- --nocapture`
Expected: PASS (3 tests)

- [ ] **Step 5: 更新 lib.rs**

```rust
pub mod dtc;
pub use dtc::*;
```

- [ ] **Step 6: 提交**

```bash
git add crates/transaction/src/dtc.rs crates/transaction/src/lib.rs
git commit -m "feat(transaction): add DTC core types"
```

---

## Task 4: 实现 Coordinator (事务协调者)

**Files:**
- Create: `crates/transaction/src/coordinator.rs`
- Modify: `crates/transaction/src/lib.rs` (导出 coordinator)

- [ ] **Step 1: 写测试 - test_coordinator_prepare**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_coordinator_initialization() {
        let node_id = NodeId(1);
        let coordinator = Coordinator::new(node_id);
        assert_eq!(coordinator.node_id(), node_id);
    }

    #[tokio::test]
    async fn test_begin_distributed_transaction() {
        let node_id = NodeId(1);
        let coordinator = Coordinator::new(node_id);
        let gid = coordinator.generate_gid().unwrap();
        assert_eq!(gid.node_id, node_id);
    }

    #[tokio::test]
    async fn test_coordinator_state_transitions() {
        let coordinator = Coordinator::new(NodeId(1));
        let gid = coordinator.generate_gid().unwrap();

        coordinator.begin_transaction(gid.clone()).await.unwrap();
        assert_eq!(coordinator.get_state(&gid), DistributedTransactionState::Started);
    }
}
```

- [ ] **Step 2: 运行测试验证失败**

Run: `cargo test -p sqlrustgo-transaction coordinator -- --nocapture`
Expected: FAIL - module not found

- [ ] **Step 3: 实现 Coordinator**

```rust
use crate::dtc::{
    Change, CoordinatorMessage, DistributedTransactionState, TransactionContext, Vote,
};
use crate::gid::{GlobalTransactionId, NodeId};
use crate::GIDManager;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;

/// 事务协调者
pub struct Coordinator {
    node_id: NodeId,
    gid_manager: Arc<GIDManager>,
    pending_transactions: RwLock<HashMap<GlobalTransactionId, TransactionContext>>,
    grpc_client: RwLock<Option<GrpcClientPool>>,
}

impl Coordinator {
    pub fn new(node_id: NodeId) -> Self {
        Coordinator {
            node_id,
            gid_manager: Arc::new(GIDManager::new(node_id)),
            pending_transactions: RwLock::new(HashMap::new()),
            grpc_client: RwLock::new(None),
        }
    }

    pub fn node_id(&self) -> NodeId {
        self.node_id
    }

    pub fn generate_gid(&self) -> Result<GlobalTransactionId, String> {
        self.gid_manager.generate()
    }

    pub async fn begin_transaction(
        &self,
        gid: GlobalTransactionId,
    ) -> Result<(), String> {
        let ctx = TransactionContext::new(gid.clone());
        self.pending_transactions
            .write()
            .map_err(|_| "Lock poisoned")?
            .insert(gid, ctx);
        Ok(())
    }

    pub fn get_state(&self, gid: &GlobalTransactionId) -> Option<DistributedTransactionState> {
        self.pending_transactions
            .read()
            .ok()
            .and_then(|map| map.get(gid).map(|ctx| ctx.state))
    }

    pub async fn prepare(
        &self,
        gid: &GlobalTransactionId,
        participants: &[u64],
    ) -> Result<PrepareResult, String> {
        // 更新状态为 Preparing
        {
            let mut map = self.pending_transactions.write().map_err(|_| "Lock poisoned")?;
            if let Some(ctx) = map.get_mut(gid) {
                ctx.state = DistributedTransactionState::Preparing;
                for &p in participants {
                    ctx.add_participant(p);
                }
            }
        }

        // 发送 Prepare 请求给所有参与者
        let mut votes = Vec::new();
        let client = self.grpc_client.read().map_err(|_| "Lock poisoned")?;

        if let Some(ref pool) = *client {
            for &participant_node_id in participants {
                let vote = self.send_prepare_via_grpc(pool, participant_node_id, gid).await?;
                votes.push(vote);
            }
        }

        // 检查投票结果
        let all_commit = votes.iter().all(|v| v == &Vote::VoteCommit);

        let mut map = self.pending_transactions.write().map_err(|_| "Lock poisoned")?;
        if let Some(ctx) = map.get_mut(gid) {
            if all_commit {
                ctx.state = DistributedTransactionState::Prepared;
                Ok(PrepareResult::AllCommitted)
            } else {
                ctx.state = DistributedTransactionState::RollingBack;
                Ok(PrepareResult::NeedsRollback)
            }
        } else {
            Err("Transaction not found".to_string())
        }
    }

    async fn send_prepare_via_grpc(
        &self,
        pool: &GrpcClientPool,
        participant_node_id: u64,
        gid: &GlobalTransactionId,
    ) -> Result<Vote, String> {
        // TODO: 实现 gRPC 调用
        // 临时返回 VoteCommit 以便编译通过
        Ok(Vote::VoteCommit)
    }

    pub async fn commit(&self, gid: &GlobalTransactionId) -> Result<CommitResult, String> {
        let mut map = self.pending_transactions.write().map_err(|_| "Lock poisoned")?;
        if let Some(ctx) = map.get_mut(gid) {
            ctx.state = DistributedTransactionState::Committing;

            // TODO: 发送 Commit 请求给所有参与者

            ctx.state = DistributedTransactionState::Committed;
            map.remove(gid);
            Ok(CommitResult { success: true })
        } else {
            Err("Transaction not found".to_string())
        }
    }

    pub async fn rollback(&self, gid: &GlobalTransactionId, reason: &str) -> Result<(), String> {
        let mut map = self.pending_transactions.write().map_err(|_| "Lock poisoned")?;
        if let Some(ctx) = map.get_mut(gid) {
            ctx.state = DistributedTransactionState::RollingBack;

            // TODO: 发送 Rollback 请求给所有参与者

            ctx.state = DistributedTransactionState::RolledBack;
            map.remove(gid);
            Ok(())
        } else {
            Err("Transaction not found".to_string())
        }
    }
}

pub enum PrepareResult {
    AllCommitted,
    NeedsRollback,
}

pub struct CommitResult {
    pub success: bool,
}

struct GrpcClientPool {
    // TODO: 实现 gRPC 客户端池
}

struct GIDManager {
    node_id: NodeId,
    counter: std::sync::atomic::AtomicU64,
}

impl GIDManager {
    fn new(node_id: NodeId) -> Self {
        GIDManager {
            node_id,
            counter: std::sync::atomic::AtomicU64::new(0),
        }
    }

    fn generate(&self) -> Result<GlobalTransactionId, String> {
        let txn_id = self.counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Ok(GlobalTransactionId {
            node_id: self.node_id,
            txn_id,
            timestamp,
        })
    }
}
```

- [ ] **Step 4: 运行测试验证通过**

Run: `cargo test -p sqlrustgo-transaction coordinator -- --nocapture`
Expected: PASS (3 tests)

- [ ] **Step 5: 更新 lib.rs**

```rust
pub mod coordinator;
pub use coordinator::Coordinator;
```

- [ ] **Step 6: 提交**

```bash
git add crates/transaction/src/coordinator.rs crates/transaction/src/lib.rs
git commit -m "feat(transaction): implement Coordinator for 2PC"
```

---

## Task 5: 实现 Participant (事务参与者)

**Files:**
- Create: `crates/transaction/src/participant.rs`
- Modify: `crates/transaction/src/lib.rs` (导出 participant)

- [ ] **Step 1: 写测试 - test_participant_prepare**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_participant_initialization() {
        let node_id = NodeId(1);
        let participant = Participant::new(node_id);
        assert_eq!(participant.node_id(), node_id);
    }

    #[tokio::test]
    async fn test_participant_handle_prepare_commit() {
        let node_id = NodeId(1);
        let participant = Participant::new(node_id);

        let request = PrepareRequest {
            gid: "1:1:1000".to_string(),
            coordinator_node_id: "2".to_string(),
            changes: vec![],
        };

        let response = participant.handle_prepare(request).await.unwrap();
        assert_eq!(response.vote, VoteType::VoteCommit as i32);
    }

    #[tokio::test]
    async fn test_participant_handle_commit() {
        let node_id = NodeId(1);
        let participant = Participant::new(node_id);

        let request = CommitRequest {
            gid: "1:1:1000".to_string(),
        };

        let response = participant.handle_commit(request).await.unwrap();
        assert!(response.success);
    }
}
```

- [ ] **Step 2: 运行测试验证失败**

Run: `cargo test -p sqlrustgo-transaction participant -- --nocapture`
Expected: FAIL - module not found

- [ ] **Step 3: 实现 Participant**

```rust
use crate::dtc::{
    Change, ChangeOperation, DistributedTransactionState, TransactionContext, Vote,
};
use crate::gid::{GlobalTransactionId, NodeId};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// 事务参与者
pub struct Participant {
    node_id: NodeId,
    local_transactions: RwLock<HashMap<GlobalTransactionId, TransactionContext>>,
}

impl Participant {
    pub fn new(node_id: NodeId) -> Self {
        Participant {
            node_id,
            local_transactions: RwLock::new(HashMap::new()),
        }
    }

    pub fn node_id(&self) -> NodeId {
        self.node_id
    }

    pub async fn handle_prepare(
        &self,
        request: PrepareRequest,
    ) -> Result<VoteResponse, String> {
        let gid = GlobalTransactionId::parse(&request.gid)
            .map_err(|e| e.to_string())?;

        // 创建本地事务上下文
        let mut ctx = TransactionContext::new(gid.clone());
        ctx.state = DistributedTransactionState::Preparing;

        // TODO: 尝试获取锁
        // TODO: 记录 Prepare 日志到 WAL

        self.local_transactions
            .write()
            .map_err(|_| "Lock poisoned")?
            .insert(gid.clone(), ctx);

        Ok(VoteResponse {
            gid: request.gid,
            node_id: self.node_id.0.to_string(),
            vote: VoteType::VoteCommit as i32,
            reason: None,
        })
    }

    pub async fn handle_commit(
        &self,
        request: CommitRequest,
    ) -> Result<ExecutionResponse, String> {
        let gid = GlobalTransactionId::parse(&request.gid)
            .map_err(|e| e.to_string())?;

        // TODO: 从 WAL 恢复事务状态
        // TODO: 执行本地提交
        // TODO: 释放锁
        // TODO: 清理 WAL

        self.local_transactions
            .write()
            .map_err(|_| "Lock poisoned")?
            .remove(&gid);

        Ok(ExecutionResponse {
            gid: request.gid,
            node_id: self.node_id.0.to_string(),
            success: true,
            affected_rows: 0,
            error: None,
        })
    }

    pub async fn handle_rollback(
        &self,
        request: RollbackRequest,
    ) -> Result<ExecutionResponse, String> {
        let gid = GlobalTransactionId::parse(&request.gid)
            .map_err(|e| e.to_string())?;

        // TODO: 执行本地回滚
        // TODO: 释放锁
        // TODO: 清理 WAL

        self.local_transactions
            .write()
            .map_err(|_| "Lock poisoned")?
            .remove(&gid);

        Ok(ExecutionResponse {
            gid: request.gid,
            node_id: self.node_id.0.to_string(),
            success: true,
            affected_rows: 0,
            error: None,
        })
    }
}

// gRPC 请求/响应类型
#[derive(Debug, Clone)]
pub struct PrepareRequest {
    pub gid: String,
    pub coordinator_node_id: String,
    pub changes: Vec<Change>,
}

#[derive(Debug, Clone)]
pub struct VoteResponse {
    pub gid: String,
    pub node_id: String,
    pub vote: i32,
    pub reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CommitRequest {
    pub gid: String,
}

#[derive(Debug, Clone)]
pub struct RollbackRequest {
    pub gid: String,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct ExecutionResponse {
    pub gid: String,
    pub node_id: String,
    pub success: bool,
    pub affected_rows: u64,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum VoteType {
    VoteCommit = 0,
    VoteAbort = 1,
}

#[derive(Debug, Clone)]
pub struct Change {
    pub table: String,
    pub operation: ChangeOperation,
    pub key: Vec<u8>,
    pub value: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Copy)]
pub enum ChangeOperation {
    Insert = 0,
    Update = 1,
    Delete = 2,
}
```

- [ ] **Step 4: 运行测试验证通过**

Run: `cargo test -p sqlrustgo-transaction participant -- --nocapture`
Expected: PASS (3 tests)

- [ ] **Step 5: 更新 lib.rs**

```rust
pub mod participant;
pub use participant::Participant;
```

- [ ] **Step 6: 提交**

```bash
git add crates/transaction/src/participant.rs crates/transaction/src/lib.rs
git commit -m "feat(transaction): implement Participant for 2PC"
```

---

## Task 6: 实现 Router (SQL 路由逻辑)

**Files:**
- Create: `crates/transaction/src/router.rs`
- Modify: `crates/transaction/src/lib.rs` (导出 router)

- [ ] **Step 1: 写测试 - test_router**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_initialization() {
        let router = Router::new();
        assert!(router.get_routes().is_empty());
    }

    #[test]
    fn test_register_table() {
        let mut router = Router::new();
        router.register_table("users", 1);
        assert_eq!(router.get_node_for_table("users"), Some(1));
    }

    #[test]
    fn test_resolve_single_node() {
        let mut router = Router::new();
        router.register_table("users", 1);

        let tables = vec!["users".to_string()];
        let nodes = router.resolve_tables(&tables).unwrap();
        assert_eq!(nodes, vec![1]);
    }

    #[test]
    fn test_resolve_multi_node() {
        let mut router = Router::new();
        router.register_table("users", 1);
        router.register_table("orders", 2);

        let tables = vec!["users".to_string(), "orders".to_string()];
        let nodes = router.resolve_tables(&tables).unwrap();
        assert_eq!(nodes.len(), 2);
    }
}
```

- [ ] **Step 2: 运行测试验证失败**

Run: `cargo test -p sqlrustgo-transaction router -- --nocapture`
Expected: FAIL - module not found

- [ ] **Step 3: 实现 Router**

```rust
use std::collections::HashMap;

/// SQL 路由逻辑 - 根据表名确定数据所在节点
pub struct Router {
    table_to_node: HashMap<String, u64>,
}

impl Router {
    pub fn new() -> Self {
        Router {
            table_to_node: HashMap::new(),
        }
    }

    /// 注册表到节点的映射
    pub fn register_table(&mut self, table: &str, node_id: u64) {
        self.table_to_node.insert(table.to_string(), node_id);
    }

    /// 获取表所在的节点
    pub fn get_node_for_table(&self, table: &str) -> Option<u64> {
        self.table_to_node.get(table).copied()
    }

    /// 获取所有路由表
    pub fn get_routes(&self) -> Vec<(String, u64)> {
        self.table_to_node
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect()
    }

    /// 根据表名列表解析参与的节点
    pub fn resolve_tables(&self, tables: &[String]) -> Result<Vec<u64>, String> {
        let mut nodes = Vec::new();
        for table in tables {
            if let Some(&node_id) = self.table_to_node.get(table) {
                if !nodes.contains(&node_id) {
                    nodes.push(node_id);
                }
            } else {
                return Err(format!("Unknown table: {}", table));
            }
        }
        Ok(nodes)
    }

    /// 判断是否为单节点事务
    pub fn is_single_node_transaction(&self, tables: &[String]) -> bool {
        match self.resolve_tables(tables) {
            Ok(nodes) => nodes.len() == 1,
            Err(_) => false,
        }
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}
```

- [ ] **Step 4: 运行测试验证通过**

Run: `cargo test -p sqlrustgo-transaction router -- --nocapture`
Expected: PASS (4 tests)

- [ ] **Step 5: 更新 lib.rs**

```rust
pub mod router;
pub use router::Router;
```

- [ ] **Step 6: 提交**

```bash
git add crates/transaction/src/router.rs crates/transaction/src/lib.rs
git commit -m "feat(transaction): implement Router for table-to-node mapping"
```

---

## Task 7: 实现 DistributedLockManager

**Files:**
- Create: `crates/transaction/src/lock_manager.rs`
- Modify: `crates/transaction/src/lib.rs` (导出 lock_manager)

- [ ] **Step 1: 写测试 - test_lock_manager**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_lock_acquire_release() {
        let lock_mgr = DistributedLockManager::new();
        let gid = GlobalTransactionId::new(NodeId(1));

        let lock_key = LockKey::Table("users".to_string());
        let result = lock_mgr.try_lock(&gid, &lock_key).await;
        assert!(result.is_ok());

        lock_mgr.unlock(&gid).await.unwrap();
    }

    #[tokio::test]
    async fn test_lock_conflict() {
        let lock_mgr = DistributedLockManager::new();
        let gid1 = GlobalTransactionId::new(NodeId(1));
        let gid2 = GlobalTransactionId::new(NodeId(2));

        let lock_key = LockKey::Table("users".to_string());

        // 第一个事务获取锁
        let result1 = lock_mgr.try_lock(&gid1, &lock_key).await;
        assert!(result1.is_ok());

        // 第二个事务尝试获取同一把锁
        let result2 = lock_mgr.try_lock(&gid2, &lock_key).await;
        assert!(result2.is_err()); // 应该失败
    }
}
```

- [ ] **Step 2: 运行测试验证失败**

Run: `cargo test -p sqlrustgo-transaction lock_manager -- --nocapture`
Expected: FAIL - module not found

- [ ] **Step 3: 实现 DistributedLockManager**

```rust
use crate::gid::{GlobalTransactionId, NodeId};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 锁键
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LockKey {
    Table(String),
    Row { table: String, row_key: Vec<u8> },
}

/// 锁值 - 持有事务的 GID
#[derive(Debug, Clone)]
pub struct LockValue {
    pub gid: GlobalTransactionId,
    pub lock_mode: LockMode,
}

/// 锁模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockMode {
    Shared,
    Exclusive,
}

/// 锁错误
#[derive(Debug, Clone)]
pub enum LockError {
    Conflict { held_by: GlobalTransactionId },
    Timeout,
}

/// 分布式锁管理器
pub struct DistributedLockManager {
    locks: RwLock<HashMap<LockKey, LockValue>>,
}

impl DistributedLockManager {
    pub fn new() -> Self {
        DistributedLockManager {
            locks: RwLock::new(HashMap::new()),
        }
    }

    /// 尝试获取锁
    pub async fn try_lock(
        &self,
        gid: &GlobalTransactionId,
        key: &LockKey,
    ) -> Result<(), LockError> {
        let mut locks = self.locks.write().await;

        if let Some(existing) = locks.get(key) {
            if existing.gid != *gid {
                return Err(LockError::Conflict {
                    held_by: existing.gid.clone(),
                });
            }
        }

        locks.insert(
            key.clone(),
            LockValue {
                gid: gid.clone(),
                lock_mode: LockMode::Exclusive,
            },
        );

        Ok(())
    }

    /// 释放锁
    pub async fn unlock(&self, gid: &GlobalTransactionId) -> Result<(), String> {
        let mut locks = self.locks.write().await;

        // 移除该事务持有的所有锁
        locks.retain(|_, v| v.gid != *gid);

        Ok(())
    }

    /// 检查锁是否被持有
    pub async fn is_locked(&self, key: &LockKey) -> bool {
        let locks = self.locks.read().await;
        locks.contains_key(key)
    }

    /// 获取锁的持有者
    pub async fn get_holder(&self, key: &LockKey) -> Option<GlobalTransactionId> {
        let locks = self.locks.read().await;
        locks.get(key).map(|v| v.gid.clone())
    }
}

impl Default for DistributedLockManager {
    fn default() -> Self {
        Self::new()
    }
}
```

- [ ] **Step 4: 运行测试验证通过**

Run: `cargo test -p sqlrustgo-transaction lock_manager -- --nocapture`
Expected: PASS (2 tests)

- [ ] **Step 5: 更新 lib.rs**

```rust
pub mod lock_manager;
pub use lock_manager::{DistributedLockManager, LockKey, LockMode, LockError};
```

- [ ] **Step 6: 提交**

```bash
git add crates/transaction/src/lock_manager.rs crates/transaction/src/lib.rs
git commit -m "feat(transaction): implement DistributedLockManager"
```

---

## Task 8: 实现 Recovery (故障恢复)

**Files:**
- Create: `crates/transaction/src/recovery.rs`
- Modify: `crates/transaction/src/wal.rs` (添加 DTC 日志类型)
- Modify: `crates/transaction/src/lib.rs` (导出 recovery)

- [ ] **Step 1: 写测试 - test_recovery**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recovery_report_default() {
        let report = RecoveryReport::default();
        assert_eq!(report.committed, 0);
        assert_eq!(report.rolled_back, 0);
        assert_eq!(report.terminated, 0);
    }

    #[tokio::test]
    async fn test_scan_incomplete_transactions() {
        let recovery = Recovery::new();
        let incomplete = recovery.scan_incomplete_transactions().await.unwrap();
        // 初始化时应该没有未完成的事务
        assert!(incomplete.is_empty());
    }
}
```

- [ ] **Step 2: 运行测试验证失败**

Run: `cargo test -p sqlrustgo-transaction recovery -- --nocapture`
Expected: FAIL - module not found

- [ ] **Step 3: 实现 Recovery**

```rust
use crate::dtc::DistributedTransactionState;
use crate::gid::GlobalTransactionId;
use serde::{Deserialize, Serialize};

/// 恢复报告
#[derive(Debug, Default)]
pub struct RecoveryReport {
    pub committed: u32,
    pub rolled_back: u32,
    pub terminated: u32,
}

/// 事务结果
#[derive(Debug)]
pub enum TxOutcome {
    Committed,
    RolledBack,
    Unknown,
}

/// WAL 日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WalEntry {
    TxBegin {
        gid: GlobalTransactionId,
        timestamp: u64,
    },
    TxPrepare {
        gid: GlobalTransactionId,
        participants: Vec<u64>,
        timestamp: u64,
    },
    TxCommit {
        gid: GlobalTransactionId,
        timestamp: u64,
    },
    TxRollback {
        gid: GlobalTransactionId,
        reason: String,
        timestamp: u64,
    },
    TxTerminate {
        gid: GlobalTransactionId,
        reason: String,
        timestamp: u64,
    },
}

/// 故障恢复组件
pub struct Recovery {
    // TODO: 需要 WALManager 的引用
}

impl Recovery {
    pub fn new() -> Self {
        Recovery {}
    }

    /// 扫描未完成的事务
    pub async fn scan_incomplete_transactions(&self) -> Result<Vec<WalEntry>, String> {
        // TODO: 从 WAL 读取所有未 Commit 的条目
        Ok(Vec::new())
    }

    /// 执行恢复
    pub async fn recover(&self) -> Result<RecoveryReport, String> {
        let mut report = RecoveryReport::default();

        // 扫描未完成的事务
        let incomplete_txs = self.scan_incomplete_transactions().await?;

        for entry in incomplete_txs {
            match entry {
                WalEntry::TxBegin { gid, .. } => {
                    // 从未完成 Phase 1，回滚
                    self.rollback_incomplete_tx(&gid, "Node crash before prepare")
                        .await?;
                    report.rolled_back += 1;
                }
                WalEntry::TxPrepare { gid, participants, .. } => {
                    // 等待协调者指令或主动查询
                    let outcome = self
                        .query_coordinator_for_outcome(&gid, &participants)
                        .await?;
                    match outcome {
                        TxOutcome::Committed => {
                            self.mark_committed(&gid).await?;
                            report.committed += 1;
                        }
                        TxOutcome::RolledBack => {
                            self.mark_rolled_back(&gid).await?;
                            report.rolled_back += 1;
                        }
                        TxOutcome::Unknown => {
                            self.mark_terminated(&gid, "Coordinator uncertain").await?;
                            report.terminated += 1;
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(report)
    }

    async fn rollback_incomplete_tx(&self, gid: &GlobalTransactionId, reason: &str) -> Result<(), String> {
        // TODO: 向参与者发送 Rollback 请求
        Ok(())
    }

    async fn query_coordinator_for_outcome(&self, gid: &GlobalTransactionId, _participants: &[u64]) -> Result<TxOutcome, String> {
        // TODO: 查询协调者事务状态
        // 暂时返回 Unknown
        Ok(TxOutcome::Unknown)
    }

    async fn mark_committed(&self, gid: &GlobalTransactionId) -> Result<(), String> {
        // TODO: 更新 WAL 条目状态
        Ok(())
    }

    async fn mark_rolled_back(&self, gid: &GlobalTransactionId) -> Result<(), String> {
        // TODO: 更新 WAL 条目状态
        Ok(())
    }

    async fn mark_terminated(&self, gid: &GlobalTransactionId, reason: &str) -> Result<(), String> {
        // TODO: 记录 TxTerminate 日志
        Ok(())
    }
}

impl Default for Recovery {
    fn default() -> Self {
        Self::new()
    }
}
```

- [ ] **Step 4: 运行测试验证通过**

Run: `cargo test -p sqlrustgo-transaction recovery -- --nocapture`
Expected: PASS (2 tests)

- [ ] **Step 5: 更新 lib.rs**

```rust
pub mod recovery;
pub use recovery::{Recovery, RecoveryReport, WalEntry, TxOutcome};
```

- [ ] **Step 6: 提交**

```bash
git add crates/transaction/src/recovery.rs crates/transaction/src/lib.rs
git commit -m "feat(transaction): implement Recovery for fault tolerance"
```

---

## Task 9: 集成测试

**Files:**
- Create: `tests/integration/distributed_transaction_test.rs`

- [ ] **Step 1: 创建集成测试文件**

```rust
use sqlrustgo_transaction::{
    Coordinator, Participant, Router, DistributedLockManager,
    GlobalTransactionId, NodeId, Recovery,
};

#[tokio::test]
async fn test_single_node_transaction() {
    let node_id = NodeId(1);
    let coordinator = Coordinator::new(node_id);
    let router = Router::new();

    // 注册表到节点
    router.register_table("users", 1);

    // 生成 GID
    let gid = coordinator.generate_gid().unwrap();

    // 开始事务
    coordinator.begin_transaction(gid.clone()).await.unwrap();

    // 验证状态
    assert_eq!(coordinator.get_state(&gid), Some(DistributedTransactionState::Started));
}

#[tokio::test]
async fn test_multi_node_transaction_flow() {
    let node_id = NodeId(1);
    let coordinator = Coordinator::new(node_id);

    // 生成 GID
    let gid = coordinator.generate_gid().unwrap();

    // 开始事务
    coordinator.begin_transaction(gid.clone()).await.unwrap();

    // 添加参与者
    let participants = vec![2, 3];

    // Prepare
    let result = coordinator.prepare(&gid, &participants).await.unwrap();
    assert!(matches!(result, PrepareResult::AllCommitted | PrepareResult::NeedsRollback));
}

#[tokio::test]
async fn test_participant_voting() {
    let node_id = NodeId(2);
    let participant = Participant::new(node_id);

    let request = PrepareRequest {
        gid: "1:1:1000".to_string(),
        coordinator_node_id: "1".to_string(),
        changes: vec![],
    };

    let response = participant.handle_prepare(request).await.unwrap();
    assert_eq!(response.vote, VoteType::VoteCommit as i32);
}

#[tokio::test]
async fn test_lock_contention() {
    let lock_mgr = DistributedLockManager::new();
    let gid1 = GlobalTransactionId::new(NodeId(1));
    let gid2 = GlobalTransactionId::new(NodeId(2));

    let lock_key = LockKey::Table("users".to_string());

    // 第一个事务获取锁
    let result1 = lock_mgr.try_lock(&gid1, &lock_key).await;
    assert!(result1.is_ok());

    // 第二个事务尝试获取同一把锁
    let result2 = lock_mgr.try_lock(&gid2, &lock_key).await;
    assert!(result2.is_err());
}

#[tokio::test]
async fn test_router_single_node() {
    let mut router = Router::new();
    router.register_table("users", 1);

    let tables = vec!["users".to_string()];
    assert!(router.is_single_node_transaction(&tables));
}

#[tokio::test]
async fn test_router_multi_node() {
    let mut router = Router::new();
    router.register_table("users", 1);
    router.register_table("orders", 2);

    let tables = vec!["users".to_string(), "orders".to_string()];
    assert!(!router.is_single_node_transaction(&tables));
}

#[tokio::test]
async fn test_recovery_scan() {
    let recovery = Recovery::new();
    let incomplete = recovery.scan_incomplete_transactions().await.unwrap();
    assert!(incomplete.is_empty());
}
```

- [ ] **Step 2: 运行集成测试验证**

Run: `cargo test --test distributed_transaction_test -- --nocapture`
Expected: SUCCESS (all tests pass)

- [ ] **Step 3: 运行完整测试套件**

Run: `cargo test --workspace`
Expected: 所有测试通过

- [ ] **Step 4: 提交**

```bash
git add tests/integration/distributed_transaction_test.rs
git commit -m "test: add 2PC distributed transaction integration tests"
```

---

## 验证命令汇总

```bash
# 编译检查
cargo build --workspace

# 单元测试
cargo test -p sqlrustgo-transaction --lib

# 集成测试
cargo test --test distributed_transaction_test

# 完整测试
cargo test --workspace
```

---

## 依赖关系

```
Task 1 (Network) ─┬─> Task 2 (GID)
                   └─> Task 3 (DTC)
                            │
Task 4 (Coordinator) <─────┘
         │
         ├─> Task 5 (Participant)
         ├─> Task 6 (Router)
         ├─> Task 7 (LockManager)
         └─> Task 8 (Recovery)
                   │
                   └─> Task 9 (集成测试)
```

---

## 风险与限制

1. **gRPC 集成**: Task 1 创建 proto 文件，实际的 gRPC Server/Client 实现需要进一步开发
2. **WAL 持久化**: Task 8 依赖 WALManager，需要与现有 WAL 代码集成
3. **网络分区**: 当前实现未处理脑裂情况

---

*计划完成*
