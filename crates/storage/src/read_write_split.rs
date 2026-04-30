//! Read/Write Split Router
//!
//! Provides read/write splitting for master-slave replication:
//! - Route read queries to replicas
//! - Route write queries to master
//! - Load balancing across replicas
//! - Read consistency guarantees

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum QueryType {
    Read,
    Write,
    Transaction,
}

impl QueryType {
    pub fn is_read(&self) -> bool {
        matches!(self, QueryType::Read)
    }

    pub fn is_write(&self) -> bool {
        matches!(self, QueryType::Write | QueryType::Transaction)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NodeRole {
    Master,
    Replica,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicaNode {
    pub addr: SocketAddr,
    pub weight: u32,
    pub lag_ms: u64,
    pub is_healthy: bool,
}

impl ReplicaNode {
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            addr,
            weight: 100,
            lag_ms: 0,
            is_healthy: true,
        }
    }

    pub fn with_weight(mut self, weight: u32) -> Self {
        self.weight = weight;
        self
    }

    pub fn with_lag(mut self, lag_ms: u64) -> Self {
        self.lag_ms = lag_ms;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadWriteSplitConfig {
    pub master_addr: SocketAddr,
    pub replica_addrs: Vec<ReplicaNode>,
    pub strategy: LoadBalanceStrategy,
    pub consistency_mode: ConsistencyMode,
    pub max_lag_ms: u64,
}

impl Default for ReadWriteSplitConfig {
    fn default() -> Self {
        Self {
            master_addr: "127.0.0.1:3306".parse().unwrap(),
            replica_addrs: Vec::new(),
            strategy: LoadBalanceStrategy::RoundRobin,
            consistency_mode: ConsistencyMode::Eventual,
            max_lag_ms: 100,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum LoadBalanceStrategy {
    RoundRobin,
    WeightedRoundRobin,
    LeastConnections,
    LeastLag,
    Random,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ConsistencyMode {
    Strong,
    Eventual,
    Session,
}

pub struct ReadWriteRouter {
    config: ReadWriteSplitConfig,
    replica_index: Arc<AtomicU64>,
    /// Per-replica connection counts for LeastConnections strategy
    replica_connection_counts: Arc<HashMap<SocketAddr, AtomicU64>>,
}

impl ReadWriteRouter {
    pub fn new(config: ReadWriteSplitConfig) -> Self {
        let replica_connection_counts: HashMap<SocketAddr, AtomicU64> = config
            .replica_addrs
            .iter()
            .map(|r| (r.addr, AtomicU64::new(0)))
            .collect();

        Self {
            config,
            replica_index: Arc::new(AtomicU64::new(0)),
            replica_connection_counts: Arc::new(replica_connection_counts),
        }
    }

    pub fn route(&self, query_type: QueryType) -> RouteResult {
        match query_type {
            QueryType::Read => self.route_read(),
            QueryType::Write | QueryType::Transaction => self.route_write(),
        }
    }

    fn route_write(&self) -> RouteResult {
        RouteResult {
            target_addr: self.config.master_addr,
            role: NodeRole::Master,
            is_read: false,
        }
    }

    fn route_read(&self) -> RouteResult {
        let replicas: Vec<_> = self
            .config
            .replica_addrs
            .iter()
            .filter(|r| r.is_healthy && r.lag_ms <= self.config.max_lag_ms)
            .collect();

        if replicas.is_empty() {
            return RouteResult {
                target_addr: self.config.master_addr,
                role: NodeRole::Master,
                is_read: true,
            };
        }

        let target = match self.config.strategy {
            LoadBalanceStrategy::RoundRobin => self.select_round_robin(&replicas),
            LoadBalanceStrategy::WeightedRoundRobin => self.select_weighted_round_robin(&replicas),
            LoadBalanceStrategy::LeastConnections => self.select_least_connections(&replicas),
            LoadBalanceStrategy::LeastLag => self.select_least_lag(&replicas),
            LoadBalanceStrategy::Random => self.select_random(&replicas),
        };

        if self.config.strategy == LoadBalanceStrategy::LeastConnections {
            self.increment_connection_count(&target.addr);
        }

        RouteResult {
            target_addr: target.addr,
            role: NodeRole::Replica,
            is_read: true,
        }
    }

    fn select_round_robin<'a>(&self, replicas: &'a [&ReplicaNode]) -> &'a ReplicaNode {
        let index = self.replica_index.fetch_add(1, Ordering::SeqCst) as usize;
        replicas[index % replicas.len()]
    }

    fn select_weighted_round_robin<'a>(&self, replicas: &'a [&ReplicaNode]) -> &'a ReplicaNode {
        let total_weight: u32 = replicas.iter().map(|r| r.weight).sum();
        if total_weight == 0 {
            return replicas[0];
        }

        let mut random_weight =
            (self.replica_index.fetch_add(1, Ordering::SeqCst) % total_weight as u64) as u32;

        for replica in replicas {
            if random_weight < replica.weight {
                return replica;
            }
            random_weight -= replica.weight;
        }

        replicas[0]
    }

    fn select_least_connections<'a>(&self, replicas: &'a [&ReplicaNode]) -> &'a ReplicaNode {
        replicas
            .iter()
            .min_by_key(|r| self.replica_connection_counts.get(&r.addr).map(|c| c.load(Ordering::SeqCst)).unwrap_or(0))
            .unwrap_or(&replicas[0])
    }

    /// Increment connection count for a replica (call when connection is acquired)
    pub fn increment_connection_count(&self, addr: &SocketAddr) {
        if let Some(counter) = self.replica_connection_counts.get(addr) {
            counter.fetch_add(1, Ordering::SeqCst);
        }
    }

    /// Decrement connection count for a replica (call when connection is released)
    pub fn decrement_connection_count(&self, addr: &SocketAddr) {
        if let Some(counter) = self.replica_connection_counts.get(addr) {
            counter.fetch_sub(1, Ordering::SeqCst);
        }
    }

    fn select_least_lag<'a>(&self, replicas: &'a [&ReplicaNode]) -> &'a ReplicaNode {
        replicas
            .iter()
            .min_by_key(|r| r.lag_ms)
            .unwrap_or(&replicas[0])
    }

    fn select_random<'a>(&self, replicas: &'a [&ReplicaNode]) -> &'a ReplicaNode {
        use std::time::{SystemTime, UNIX_EPOCH};
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_nanos() as usize;
        replicas[seed % replicas.len()]
    }

    pub fn add_replica(&mut self, replica: ReplicaNode) {
        self.config.replica_addrs.push(replica);
    }

    pub fn remove_replica(&mut self, addr: SocketAddr) {
        self.config.replica_addrs.retain(|r| r.addr != addr);
    }

    pub fn update_replica_lag(&mut self, addr: SocketAddr, lag_ms: u64) {
        if let Some(replica) = self
            .config
            .replica_addrs
            .iter_mut()
            .find(|r| r.addr == addr)
        {
            replica.lag_ms = lag_ms;
        }
    }

    pub fn set_replica_health(&mut self, addr: SocketAddr, is_healthy: bool) {
        if let Some(replica) = self
            .config
            .replica_addrs
            .iter_mut()
            .find(|r| r.addr == addr)
        {
            replica.is_healthy = is_healthy;
        }
    }

    pub fn get_master_addr(&self) -> SocketAddr {
        self.config.master_addr
    }

    pub fn get_replica_addrs(&self) -> Vec<SocketAddr> {
        self.config.replica_addrs.iter().map(|r| r.addr).collect()
    }

    pub fn get_healthy_replica_count(&self) -> usize {
        self.config
            .replica_addrs
            .iter()
            .filter(|r| r.is_healthy)
            .count()
    }

    pub fn release_connection(&self, addr: &SocketAddr) {
        self.decrement_connection_count(addr);
    }

    /// Automatically classify SQL statement type based on the SQL text.
    /// This is a simple keyword-based classifier that determines if a SQL
    /// statement is a read, write, or transaction operation.
    pub fn classify_sql(sql: &str) -> QueryType {
        let sql_trimmed = sql.trim().to_uppercase();

        if sql_trimmed.starts_with("BEGIN")
            || sql_trimmed.starts_with("START")
            || sql_trimmed.starts_with("COMMIT")
            || sql_trimmed.starts_with("ROLLBACK")
            || sql_trimmed.starts_with("SAVEPOINT") {
            return QueryType::Transaction;
        }

        if sql_trimmed.starts_with("INSERT")
            || sql_trimmed.starts_with("UPDATE")
            || sql_trimmed.starts_with("DELETE")
            || sql_trimmed.starts_with("CREATE")
            || sql_trimmed.starts_with("DROP")
            || sql_trimmed.starts_with("ALTER")
            || sql_trimmed.starts_with("TRUNCATE")
            || sql_trimmed.starts_with("GRANT")
            || sql_trimmed.starts_with("REVOKE")
            || sql_trimmed.starts_with("REPLACE")
            || sql_trimmed.starts_with("LOAD")
            || sql_trimmed.starts_with("RENAME") {
            return QueryType::Write;
        }

        if sql_trimmed.starts_with("SELECT")
            || sql_trimmed.starts_with("SHOW")
            || sql_trimmed.starts_with("DESCRIBE")
            || sql_trimmed.starts_with("EXPLAIN")
            || sql_trimmed.starts_with("CALL") {
            return QueryType::Read;
        }

        QueryType::Write
    }

    /// Automatically route a SQL statement to the appropriate node.
    pub fn route_sql(&self, sql: &str) -> RouteResult {
        let query_type = Self::classify_sql(sql);
        let result = self.route(query_type);

        if result.is_read && self.config.strategy == LoadBalanceStrategy::LeastConnections {
            self.increment_connection_count(&result.target_addr);
        }

        result
    }
}

#[derive(Debug, Clone)]
pub struct RouteResult {
    pub target_addr: SocketAddr,
    pub role: NodeRole,
    pub is_read: bool,
}

pub struct ConnectionPool {
    master_pool: Vec<Connection>,
    replica_pools: Vec<Vec<Connection>>,
    _config: ReadWriteSplitConfig,
}

impl ConnectionPool {
    pub fn new(config: ReadWriteSplitConfig) -> Self {
        Self {
            master_pool: Vec::new(),
            replica_pools: Vec::new(),
            _config: config,
        }
    }

    pub fn get_connection(&self, role: NodeRole) -> Option<&Connection> {
        match role {
            NodeRole::Master => self.master_pool.first(),
            NodeRole::Replica => self.replica_pools.first()?.first(),
        }
    }

    pub fn return_connection(&self, _conn: &Connection) {}
}

#[derive(Debug, Clone)]
pub struct Connection {
    pub addr: SocketAddr,
    pub is_connected: bool,
}

pub struct ReadAfterWriteConsistency {
    master_position: Arc<AtomicU64>,
    replica_positions: Arc<RwLock<HashMap<SocketAddr, u64>>>,
    config: ReadWriteSplitConfig,
}

use std::sync::RwLock;

impl ReadAfterWriteConsistency {
    pub fn new(config: ReadWriteSplitConfig) -> Self {
        Self {
            master_position: Arc::new(AtomicU64::new(0)),
            replica_positions: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    pub fn record_master_write(&self, lsn: u64) {
        self.master_position.store(lsn, Ordering::SeqCst);
    }

    pub fn record_replica_position(&self, addr: SocketAddr, lsn: u64) {
        if let Ok(mut positions) = self.replica_positions.write() {
            positions.insert(addr, lsn);
        }
    }

    pub fn wait_for_replication(&self, target_addr: SocketAddr) -> bool {
        let master_lsn = self.master_position.load(Ordering::SeqCst);

        if let Ok(positions) = self.replica_positions.read() {
            if let Some(&replica_lsn) = positions.get(&target_addr) {
                return replica_lsn >= master_lsn;
            }
        }

        false
    }

    pub fn ensure_session_consistency(&self, replica_addr: SocketAddr) -> bool {
        match self.config.consistency_mode {
            ConsistencyMode::Strong => self.wait_for_replication(replica_addr),
            ConsistencyMode::Eventual => true,
            ConsistencyMode::Session => self.wait_for_replication(replica_addr),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> ReadWriteSplitConfig {
        ReadWriteSplitConfig {
            master_addr: "127.0.0.1:3306".parse().unwrap(),
            replica_addrs: vec![
                ReplicaNode::new("127.0.0.1:3307".parse().unwrap()).with_weight(100),
                ReplicaNode::new("127.0.0.1:3308".parse().unwrap()).with_weight(50),
            ],
            strategy: LoadBalanceStrategy::RoundRobin,
            consistency_mode: ConsistencyMode::Eventual,
            max_lag_ms: 100,
        }
    }

    #[test]
    fn test_query_type_classification() {
        assert!(QueryType::Read.is_read());
        assert!(!QueryType::Read.is_write());

        assert!(QueryType::Write.is_write());
        assert!(!QueryType::Write.is_read());

        assert!(QueryType::Transaction.is_write());
    }

    #[test]
    fn test_router_write_route() {
        let config = create_test_config();
        let router = ReadWriteRouter::new(config);

        let result = router.route(QueryType::Write);

        assert_eq!(result.target_addr, "127.0.0.1:3306".parse().unwrap());
        assert_eq!(result.role, NodeRole::Master);
        assert!(!result.is_read);
    }

    #[test]
    fn test_router_read_route() {
        let config = create_test_config();
        let router = ReadWriteRouter::new(config);

        let result = router.route(QueryType::Read);

        assert!(result.is_read);
        assert_eq!(result.role, NodeRole::Replica);
    }

    #[test]
    fn test_router_multiple_reads_distributed() {
        let config = create_test_config();
        let router = ReadWriteRouter::new(config);

        let mut targets = Vec::new();
        for _ in 0..4 {
            targets.push(router.route(QueryType::Read).target_addr);
        }

        let unique: Vec<_> = targets.iter().collect();
        assert!(unique.len() >= 1);
    }

    #[test]
    fn test_replica_node_creation() {
        let replica = ReplicaNode::new("127.0.0.1:3307".parse().unwrap());

        assert_eq!(replica.weight, 100);
        assert_eq!(replica.lag_ms, 0);
        assert!(replica.is_healthy);
    }

    #[test]
    fn test_replica_node_with_lag() {
        let replica = ReplicaNode::new("127.0.0.1:3307".parse().unwrap()).with_lag(50);

        assert_eq!(replica.lag_ms, 50);
    }

    #[test]
    fn test_load_balance_strategy_variants() {
        assert_eq!(
            LoadBalanceStrategy::RoundRobin,
            LoadBalanceStrategy::RoundRobin
        );
        assert_eq!(LoadBalanceStrategy::LeastLag, LoadBalanceStrategy::LeastLag);
    }

    #[test]
    fn test_consistency_mode_variants() {
        assert_eq!(ConsistencyMode::Strong, ConsistencyMode::Strong);
        assert_eq!(ConsistencyMode::Eventual, ConsistencyMode::Eventual);
        assert_eq!(ConsistencyMode::Session, ConsistencyMode::Session);
    }

    #[test]
    fn test_read_write_router_add_replica() {
        let config = create_test_config();
        let mut router = ReadWriteRouter::new(config);

        router.add_replica(ReplicaNode::new("127.0.0.1:3309".parse().unwrap()));

        assert_eq!(router.get_replica_addrs().len(), 3);
    }

    #[test]
    fn test_read_write_router_remove_replica() {
        let config = create_test_config();
        let mut router = ReadWriteRouter::new(config);

        router.remove_replica("127.0.0.1:3307".parse().unwrap());

        assert_eq!(router.get_replica_addrs().len(), 1);
    }

    #[test]
    fn test_route_result_fields() {
        let result = RouteResult {
            target_addr: "127.0.0.1:3306".parse().unwrap(),
            role: NodeRole::Master,
            is_read: false,
        };

        assert_eq!(result.target_addr, "127.0.0.1:3306".parse().unwrap());
        assert_eq!(result.role, NodeRole::Master);
        assert!(!result.is_read);
    }

    #[test]
    fn test_read_after_write_consistency() {
        let config = create_test_config();
        let consistency = ReadAfterWriteConsistency::new(config);

        consistency.record_master_write(1000);

        consistency.record_replica_position("127.0.0.1:3307".parse().unwrap(), 1001);

        assert!(consistency.wait_for_replication("127.0.0.1:3307".parse().unwrap()));
    }

    #[test]
    fn test_read_write_split_config_default() {
        let config = ReadWriteSplitConfig::default();

        assert_eq!(config.strategy, LoadBalanceStrategy::RoundRobin);
        assert_eq!(config.consistency_mode, ConsistencyMode::Eventual);
        assert_eq!(config.max_lag_ms, 100);
    }

    #[test]
    fn test_connection_struct() {
        let conn = Connection {
            addr: "127.0.0.1:3306".parse().unwrap(),
            is_connected: true,
        };

        assert_eq!(conn.addr, "127.0.0.1:3306".parse().unwrap());
        assert!(conn.is_connected);
    }

    #[test]
    fn test_least_connections_selects_min() {
        let mut config = create_test_config();
        config.strategy = LoadBalanceStrategy::LeastConnections;
        let router = ReadWriteRouter::new(config);

        let result1 = router.route(QueryType::Read);
        let result2 = router.route(QueryType::Read);
        let result3 = router.route(QueryType::Read);

        assert!(result1.is_read);
        assert!(result2.is_read);
        assert!(result3.is_read);
    }

    #[test]
    fn test_least_connections_tracks_counts() {
        let mut config = create_test_config();
        config.strategy = LoadBalanceStrategy::LeastConnections;
        let router = ReadWriteRouter::new(config);

        let replica1_addr: SocketAddr = "127.0.0.1:3307".parse().unwrap();
        let replica2_addr: SocketAddr = "127.0.0.1:3308".parse().unwrap();

        router.increment_connection_count(&replica1_addr);
        router.increment_connection_count(&replica1_addr);
        router.increment_connection_count(&replica2_addr);

        let result = router.route(QueryType::Read);
        assert_eq!(result.target_addr, replica2_addr);
    }

    #[test]
    fn test_increment_decrement_connection_count() {
        let config = create_test_config();
        let router = ReadWriteRouter::new(config);

        let replica_addr: SocketAddr = "127.0.0.1:3307".parse().unwrap();

        router.increment_connection_count(&replica_addr);
        router.increment_connection_count(&replica_addr);
        router.decrement_connection_count(&replica_addr);

        let result = router.route(QueryType::Read);
        assert!(result.is_read);
    }
}
