# Issue #943: 主从复制/备份/故障转移 详细设计

**Issue**: #943 Phase 2: 高可用 - 主从复制/备份/故障转移
**创建日期**: 2026-03-28
**状态**: 设计完成

## 1. 背景与目标

现有代码已有基本的 Binlog 事件读写、MasterNode/SlaveNode 骨架，但缺少：
- 网络层面的 binlog 传输（Slave 从 Master 拉取）
- 真正的故障转移（自动检测 + Slave 升级为 Master）

**目标**: 实现完整的主从复制协议 + 自动故障转移

## 2. 架构设计

```
┌─────────────┐         Binlog          ┌─────────────┐
│  Master    │ ──────────────────────────►│   Slave    │
│  Node      │◄────────────────────────── │   Node     │
│            │      ACK / LSN            │            │
└─────────────┘                           └─────────────┘
      │                                        │
      │ Health Check                           │ Apply Events
      ▼                                        ▼
┌─────────────────────────────────────────────────────┐
│                 FailoverManager                     │
│  ┌──────────┐  ┌──────────┐  ┌──────────────────┐   │
│  │ Monitor  │  │ Election │  │ SlavePromoter    │   │
│  └──────────┘  └──────────┘  └──────────────────┘   │
└─────────────────────────────────────────────────────┘
```

## 3. 复制协议

### 3.1 协议消息类型

```rust
#[derive(Debug, Clone)]
pub enum ReplicationMessage {
    // Master -> Slave
    HandshakeOk { server_id: u32, binlog_file: String, binlog_pos: u64 },
    BinlogEvent(Vec<u8>),
    Heartbeat { lsn: u64, timestamp: u64 },
    Error { code: u16, message: String },

    // Slave -> Master
    HandshakeRequest { slave_id: u32, host: String, port: u16 },
    BinlogPosRequest { file: String, pos: u64 },
    LSNack { lsn: u64 },
    HeartbeatAck { lsn: u64 },
}
```

### 3.2 协议流程

```
Slave                        Master
  │                             │
  │──── HandshakeRequest ──────►│
  │                             │
  │◄─── HandshakeOk ────────────│
  │                             │
  │──── BinlogPosRequest ──────►│
  │                             │
  │◄─── BinlogEvent(s) ──────────│
  │                             │
  │──── LSNack ────────────────►│
  │         ...                 │
  │                             │
  │◄─── Heartbeat ───────────────│
  │──── HeartbeatAck ──────────►│
```

**端口**: Master 默认 `3333`

## 4. 核心组件

### 4.1 BinlogServer (Master 端)

```rust
pub struct BinlogServer {
    listener: TcpListener,
    server_id: u32,
    binlog_path: PathBuf,
    subscribers: Arc<RwLock<HashMap<u32, BinlogSubscriber>>>,
    binlog_writer: Arc<Mutex<BinlogWriter>>,
}

impl BinlogServer {
    pub fn start(host: &str, port: u16, binlog_path: PathBuf) -> Result<Self>;
    pub fn run(&self);  // 接受连接，启动处理线程
    pub fn broadcast(&self, event: BinlogEvent);  // 广播给所有订阅者
}

struct BinlogSubscriber {
    slave_id: u32,
    stream: TcpStream,
    sender: mpsc::Sender<BinlogEvent>,
}
```

### 4.2 BinlogClient (Slave 端)

```rust
pub struct BinlogClient {
    server_addr: SocketAddr,
    slave_id: u32,
    server_id: u32,
    stream: Option<TcpStream>,
    current_file: String,
    current_pos: u64,
    heartbeat_interval: Duration,
}

impl BinlogClient {
    pub fn connect(&mut self, master_addr: &str, port: u16) -> Result<()>;
    pub fn start_replication(&mut self) -> Result<Receiver<BinlogEvent>>;
    pub fn send_ack(&mut self, lsn: u64) -> Result<()>;
    pub fn close(&mut self);
}
```

### 4.3 FailoverManager (增强版)

```rust
pub struct FailoverManager {
    node_addr: SocketAddr,
    node_type: NodeType,
    all_nodes: Vec<NodeInfo>,
    health_check_interval: Duration,
    election_timeout: Duration,
    failover_callback: Arc<dyn FailoverCallback>,

    current_master: Arc<RwLock<Option<SocketAddr>>>,
    state: Arc<RwLock<FailoverState>>,
    is_leader: Arc<AtomicBool>,
}

#[derive(Debug, Clone, PartialEq)]
enum NodeType {
    Master,
    Slave,
}

#[derive(Debug, Clone, PartialEq)]
enum FailoverState {
    Normal,
    MasterUnreachable,
    ElectionInProgress,
    NewMasterPromoted,
}

trait FailoverCallback: Send + Sync {
    fn on_master_change(&self, new_master: SocketAddr);
    fn on_promote_to_master(&self) -> Result<()>;
    fn on_demote_to_slave(&self, new_master: SocketAddr) -> Result<()>;
}

impl FailoverManager {
    pub fn start_monitoring(&self);
    fn check_master_health(&self) -> bool;
    fn execute_failover(&self) -> Result<SocketAddr>;
    fn elect_new_master(&self, alive_nodes: &[NodeInfo]) -> Result<SocketAddr>;
}
```

### 4.4 ReplicationLagMonitor

```rust
pub struct ReplicationLagMonitor {
    master_lsn: Arc<AtomicU64>,
    master_timestamp: Arc<AtomicU64>,
    slave_applied_lsn: Arc<AtomicU64>,
    slave_applied_timestamp: Arc<AtomicU64>,
    lag_threshold_ms: u64,
}

impl ReplicationLagMonitor {
    pub fn update_master_info(&self, lsn: u64, timestamp: u64);
    pub fn report_applied(&self, lsn: u64);
    pub fn current_lag_ms(&self) -> u64;
    pub fn is_lag_exceeding_threshold(&self) -> bool;
}
```

## 5. 文件结构

```
crates/storage/src/
├── replication.rs           # 现有：BinlogEvent, MasterNode, SlaveNode (保留)
├── binlog_server.rs        # NEW: Master 网络服务
├── binlog_client.rs        # NEW: Slave 网络客户端
├── binlog_protocol.rs      # NEW: 协议消息定义
├── failover_manager.rs     # NEW: 增强版故障转移
└── replication_lag.rs     # NEW: 延迟监控

crates/storage/src/lib.rs
├── binlog_server
├── binlog_client
├── binlog_protocol
├── failover_manager
└── replication_lag
```

## 6. 错误处理

| 错误类型 | 处理策略 |
|----------|----------|
| Master 不可达 | 触发健康检查，连续失败 3 次启动选举 |
| 网络中断 | BinlogClient 自动重连，slave 从断点继续 |
| 事件应用失败 | 记录错误，重试 3 次后报警 |
| 选举平票 | 增加随机 jitter，最先响应的 slave 获胜 |
| 连接拒绝 | 指数退避重试 (1s, 2s, 4s, 8s, max 30s) |

## 7. 实现计划

1. **Phase 1**: `binlog_protocol.rs` - 定义协议消息
2. **Phase 2**: `binlog_server.rs` - Master 网络服务
3. **Phase 3**: `binlog_client.rs` - Slave 网络客户端
4. **Phase 4**: `failover_manager.rs` - 故障转移逻辑
5. **Phase 5**: `replication_lag.rs` - 延迟监控
6. **Phase 6**: 集成测试

## 8. 测试场景

1. Master-Slave 基本复制
2. 网络中断后自动重连
3. Master 宕机后自动故障转移
4. Slave 重新加入集群
5. 复制延迟监控报警
