//! Shard Router - routes queries to appropriate shards
//!
//! Handles SQL routing based on partition keys.

use crate::partition::PartitionValue;
use crate::shard_manager::{NodeId, ShardId, ShardInfo, ShardManager};

#[derive(Debug, Clone)]
pub struct RoutedQuery {
    pub shard_id: ShardId,
    pub node_id: NodeId,
    pub original_sql: String,
}

#[derive(Debug, Clone)]
pub struct RoutedPlan {
    pub queries: Vec<RoutedQuery>,
    pub is_distributed: bool,
    pub involved_shards: Vec<ShardId>,
}

impl RoutedPlan {
    pub fn single(shard_id: ShardId, node_id: NodeId, sql: String) -> Self {
        Self {
            queries: vec![RoutedQuery {
                shard_id,
                node_id,
                original_sql: sql,
            }],
            is_distributed: false,
            involved_shards: vec![shard_id],
        }
    }

    pub fn distributed(queries: Vec<RoutedQuery>, shards: Vec<ShardId>) -> Self {
        Self {
            queries,
            is_distributed: true,
            involved_shards: shards,
        }
    }
}

#[derive(Debug, Clone)]
pub struct QueryKey {
    pub table: String,
    pub value: PartitionValue,
}

pub struct ShardRouter {
    shard_manager: ShardManager,
    local_node_id: NodeId,
}

impl ShardRouter {
    pub fn new(shard_manager: ShardManager, local_node_id: NodeId) -> Self {
        Self {
            shard_manager,
            local_node_id,
        }
    }

    pub fn route_point_query(
        &self,
        table: &str,
        key_column: &str,
        key_value: PartitionValue,
    ) -> Result<RoutedQuery, RouterError> {
        let partition_key = self
            .shard_manager
            .get_partition_key(table)
            .ok_or_else(|| RouterError::NoPartitionRule(table.to_string()))?;

        let shard_id = partition_key
            .partition(&key_value)
            .ok_or(RouterError::InvalidPartitionKey(table.to_string()))?;

        let shard = self
            .shard_manager
            .get_shard(shard_id)
            .ok_or(RouterError::ShardNotFound(shard_id))?;

        let node_id = shard
            .primary_node()
            .ok_or(RouterError::NoReplicaAvailable(shard_id))?;

        Ok(RoutedQuery {
            shard_id,
            node_id,
            original_sql: format!("SELECT * FROM {} WHERE {} = ?", table, key_column),
        })
    }

    pub fn route_range_query(
        &self,
        table: &str,
        key_column: &str,
        start: i64,
        end: i64,
    ) -> Result<RoutedPlan, RouterError> {
        let partition_key = self
            .shard_manager
            .get_partition_key(table)
            .ok_or_else(|| RouterError::NoPartitionRule(table.to_string()))?;

        match &partition_key.strategy {
            crate::partition::PartitionStrategy::Hash { num_shards } => {
                self.route_hash_range(table, key_column, start, end, *num_shards)
            }
            crate::partition::PartitionStrategy::Range { boundaries } => {
                self.route_range_boundary(table, key_column, start, end, boundaries)
            }
            crate::partition::PartitionStrategy::Key { columns: _, num_partitions } => {
                self.route_hash_range(table, key_column, start, end, *num_partitions)
            }
            crate::partition::PartitionStrategy::List { partitions } => {
                let all_shards: Vec<u64> = partitions.iter().map(|p| p.id).collect();
                Ok(RoutedPlan::distributed(
                    vec![],
                    all_shards,
                ))
            }
        }
    }

    fn route_hash_range(
        &self,
        table: &str,
        key_column: &str,
        start: i64,
        end: i64,
        num_shards: u64,
    ) -> Result<RoutedPlan, RouterError> {
        let mut queries = Vec::new();
        let mut shards = Vec::new();

        for i in start..end {
            let key_value = PartitionValue::Integer(i);
            let shard_id = ((key_value.abs_value()) as u64) % num_shards;

            if !shards.contains(&shard_id) {
                shards.push(shard_id);

                let shard = self
                    .shard_manager
                    .get_shard(shard_id)
                    .ok_or(RouterError::ShardNotFound(shard_id))?;

                let node_id = shard
                    .primary_node()
                    .ok_or(RouterError::NoReplicaAvailable(shard_id))?;

                queries.push(RoutedQuery {
                    shard_id,
                    node_id,
                    original_sql: format!(
                        "SELECT * FROM {} WHERE {} BETWEEN {} AND {}",
                        table, key_column, start, end
                    ),
                });
            }
        }

        Ok(RoutedPlan::distributed(queries, shards))
    }

    fn route_range_boundary(
        &self,
        table: &str,
        key_column: &str,
        start: i64,
        end: i64,
        boundaries: &[i64],
    ) -> Result<RoutedPlan, RouterError> {
        let mut queries = Vec::new();
        let mut shards = Vec::new();
        let mut _current_shard = 0;

        for (i, boundary) in boundaries.iter().enumerate() {
            if start < *boundary
                && end
                    > boundaries
                        .get(i.saturating_sub(1))
                        .copied()
                        .unwrap_or(i64::MIN)
            {
                if !shards.contains(&(i as u64)) {
                    shards.push(i as u64);

                    let shard = self
                        .shard_manager
                        .get_shard(i as u64)
                        .ok_or(RouterError::ShardNotFound(i as u64))?;

                    let node_id = shard
                        .primary_node()
                        .ok_or(RouterError::NoReplicaAvailable(i as u64))?;

                    let range_start = if i == 0 {
                        start
                    } else {
                        start.max(*boundaries.get(i - 1).unwrap_or(&i64::MIN))
                    };
                    let range_end = end.min(*boundary);

                    queries.push(RoutedQuery {
                        shard_id: i as u64,
                        node_id,
                        original_sql: format!(
                            "SELECT * FROM {} WHERE {} BETWEEN {} AND {}",
                            table, key_column, range_start, range_end
                        ),
                    });
                }
                _current_shard = i + 1;
            }
        }

        if end > *boundaries.last().unwrap_or(&i64::MAX)
            && !shards.contains(&(boundaries.len() as u64))
        {
            let shard_id = boundaries.len() as u64;
            shards.push(shard_id);

            let shard = self
                .shard_manager
                .get_shard(shard_id)
                .ok_or(RouterError::ShardNotFound(shard_id))?;

            let node_id = shard
                .primary_node()
                .ok_or(RouterError::NoReplicaAvailable(shard_id))?;

            queries.push(RoutedQuery {
                shard_id,
                node_id,
                original_sql: format!(
                    "SELECT * FROM {} WHERE {} > {}",
                    table,
                    key_column,
                    boundaries.last().unwrap_or(&i64::MAX)
                ),
            });
        }

        Ok(RoutedPlan::distributed(queries, shards))
    }

    pub fn route_local(&self, sql: &str) -> Result<RoutedPlan, RouterError> {
        Ok(RoutedPlan::single(0, self.local_node_id, sql.to_string()))
    }

    pub fn route_to_all_shards(&self, sql: &str, table: &str) -> Result<RoutedPlan, RouterError> {
        let partition_key = self
            .shard_manager
            .get_partition_key(table)
            .ok_or_else(|| RouterError::NoPartitionRule(table.to_string()))?;

        let num_shards = match &partition_key.strategy {
            crate::partition::PartitionStrategy::Hash { num_shards } => *num_shards,
            crate::partition::PartitionStrategy::Range { boundaries } => {
                boundaries.len() as u64 + 1
            }
            crate::partition::PartitionStrategy::Key { num_partitions, .. } => *num_partitions,
            crate::partition::PartitionStrategy::List { partitions } => partitions.len() as u64,
        };

        let mut queries = Vec::new();
        let mut shards = Vec::new();

        for shard_id in 0..num_shards {
            shards.push(shard_id);

            let shard = self
                .shard_manager
                .get_shard(shard_id)
                .ok_or(RouterError::ShardNotFound(shard_id))?;

            let node_id = shard
                .primary_node()
                .ok_or(RouterError::NoReplicaAvailable(shard_id))?;

            queries.push(RoutedQuery {
                shard_id,
                node_id,
                original_sql: sql.to_string(),
            });
        }

        Ok(RoutedPlan::distributed(queries, shards))
    }

    pub fn get_shard_manager(&self) -> &ShardManager {
        &self.shard_manager
    }

    pub fn get_local_node_id(&self) -> NodeId {
        self.local_node_id
    }

    pub fn get_shard(&self, shard_id: ShardId) -> Option<&ShardInfo> {
        self.shard_manager.get_shard(shard_id)
    }
}

#[derive(Debug, Clone)]
pub enum RouterError {
    NoPartitionRule(String),
    InvalidPartitionKey(String),
    ShardNotFound(ShardId),
    NoReplicaAvailable(ShardId),
}

impl std::fmt::Display for RouterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RouterError::NoPartitionRule(table) => {
                write!(f, "No partition rule for table: {}", table)
            }
            RouterError::InvalidPartitionKey(table) => {
                write!(f, "Invalid partition key for table: {}", table)
            }
            RouterError::ShardNotFound(id) => write!(f, "Shard not found: {}", id),
            RouterError::NoReplicaAvailable(id) => {
                write!(f, "No replica available for shard: {}", id)
            }
        }
    }
}

impl std::error::Error for RouterError {}

// ============================================================================
// Read/Write Split Routing
// ============================================================================

#[derive(Debug, Clone)]
pub struct ShardReadQuery {
    pub shard_id: ShardId,
    pub replica_node_id: NodeId,
    pub original_sql: String,
    pub consistency_level: ConsistencyLevel,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum ConsistencyLevel {
    Strong,
    #[default]
    Eventual,
    Session,
}

#[derive(Debug, Clone)]
pub struct ShardWriteQuery {
    pub shard_id: ShardId,
    pub primary_node_id: NodeId,
    pub original_sql: String,
}

pub struct ReadWriteShardRouter {
    shard_router: ShardRouter,
    prefer_replica: bool,
}

impl ReadWriteShardRouter {
    pub fn new(shard_router: ShardRouter) -> Self {
        Self {
            shard_router,
            prefer_replica: true,
        }
    }

    pub fn with_primary_preference(shard_router: ShardRouter) -> Self {
        Self {
            shard_router,
            prefer_replica: false,
        }
    }

    pub fn route_read(
        &self,
        table: &str,
        key_column: &str,
        key_value: PartitionValue,
    ) -> Result<ShardReadQuery, RouterError> {
        let shard_id = self.get_shard_id(table, key_value)?;
        let replica_node_id = self.get_replica_node(shard_id)?;

        Ok(ShardReadQuery {
            shard_id,
            replica_node_id,
            original_sql: format!("SELECT * FROM {} WHERE {} = ?", table, key_column),
            consistency_level: ConsistencyLevel::default(),
        })
    }

    pub fn route_read_with_consistency(
        &self,
        table: &str,
        key_column: &str,
        key_value: PartitionValue,
        consistency: ConsistencyLevel,
    ) -> Result<ShardReadQuery, RouterError> {
        let shard_id = self.get_shard_id(table, key_value)?;

        let replica_node_id = match consistency {
            ConsistencyLevel::Strong => self.get_primary_node(shard_id)?,
            _ => self.get_replica_node(shard_id)?,
        };

        Ok(ShardReadQuery {
            shard_id,
            replica_node_id,
            original_sql: format!("SELECT * FROM {} WHERE {} = ?", table, key_column),
            consistency_level: consistency,
        })
    }

    pub fn route_write(
        &self,
        table: &str,
        _key_column: &str,
        key_value: PartitionValue,
        sql: &str,
    ) -> Result<ShardWriteQuery, RouterError> {
        let shard_id = self.get_shard_id(table, key_value)?;
        let primary_node_id = self.get_primary_node(shard_id)?;

        Ok(ShardWriteQuery {
            shard_id,
            primary_node_id,
            original_sql: sql.to_string(),
        })
    }

    fn get_shard_id(
        &self,
        table: &str,
        key_value: PartitionValue,
    ) -> Result<ShardId, RouterError> {
        let partition_key = self
            .shard_router
            .get_shard_manager()
            .get_partition_key(table)
            .ok_or_else(|| RouterError::NoPartitionRule(table.to_string()))?;

        partition_key
            .partition(&key_value)
            .ok_or(RouterError::InvalidPartitionKey(table.to_string()))
    }

    fn get_primary_node(&self, shard_id: ShardId) -> Result<NodeId, RouterError> {
        let shard = self
            .shard_router
            .get_shard(shard_id)
            .ok_or(RouterError::ShardNotFound(shard_id))?;

        shard
            .primary_node()
            .ok_or(RouterError::NoReplicaAvailable(shard_id))
    }

    fn get_replica_node(&self, shard_id: ShardId) -> Result<NodeId, RouterError> {
        let shard = self
            .shard_router
            .get_shard(shard_id)
            .ok_or(RouterError::ShardNotFound(shard_id))?;

        if shard.replica_nodes.is_empty() {
            return shard
                .primary_node()
                .ok_or(RouterError::NoReplicaAvailable(shard_id));
        }

        Ok(shard.replica_nodes[0])
    }

    pub fn get_shard_router(&self) -> &ShardRouter {
        &self.shard_router
    }

    pub fn set_prefer_replica(&mut self, prefer: bool) {
        self.prefer_replica = prefer;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::partition::{ListPartition, PartitionKey};
    use crate::shard_manager::PartitionRule;

    fn create_test_router() -> ShardRouter {
        let mut manager = ShardManager::new();
        let nodes = vec![1, 2, 3];
        manager.initialize_table_shards("users", 4, &nodes);

        ShardRouter::new(manager, 1)
    }

    #[test]
    fn test_route_point_query() {
        let router = create_test_router();

        let result = router.route_point_query("users", "id", PartitionValue::Integer(5));
        assert!(result.is_ok());

        let query = result.unwrap();
        assert_eq!(query.shard_id, 1); // 5 % 4 = 1
    }

    #[test]
    fn test_route_to_all_shards() {
        let router = create_test_router();

        let result = router.route_to_all_shards("SELECT * FROM users", "users");
        assert!(result.is_ok());

        let plan = result.unwrap();
        assert!(plan.is_distributed);
        assert_eq!(plan.queries.len(), 4);
    }

    #[test]
    fn test_route_no_partition_rule() {
        let router = create_test_router();

        let result = router.route_point_query("orders", "id", PartitionValue::Integer(5));
        assert!(matches!(result, Err(RouterError::NoPartitionRule(_))));
    }

    #[test]
    fn test_route_with_key_partition() {
        let mut manager = ShardManager::new();
        let nodes = vec![1, 2];
        manager.initialize_table_shards("users", 4, &nodes);

        let rule = PartitionRule::new(
            "users",
            PartitionKey::new_key(vec!["tenant_id".to_string(), "region".to_string()], 4),
        );
        manager.add_partition_rule(rule);

        let router = ShardRouter::new(manager, 1);
        let result = router.route_point_query("users", "tenant_id", PartitionValue::Integer(10));
        assert!(result.is_ok());
        let query = result.unwrap();
        assert!(query.shard_id < 4);
    }

    #[test]
    fn test_route_with_list_partition() {
        let mut manager = ShardManager::new();
        let nodes = vec![1, 2, 3];
        manager.initialize_table_shards("regions", 3, &nodes);

        let partitions = vec![
            ListPartition { id: 0, values: vec![1, 2, 3] },
            ListPartition { id: 1, values: vec![4, 5, 6] },
            ListPartition { id: 2, values: vec![7, 8, 9] },
        ];
        let rule = PartitionRule::new("regions", PartitionKey::new_list("region_id", partitions));
        manager.add_partition_rule(rule);

        let router = ShardRouter::new(manager, 1);
        let result = router.route_point_query("regions", "region_id", PartitionValue::Integer(3));
        assert!(result.is_ok());
        let query = result.unwrap();
        assert_eq!(query.shard_id, 0);
    }

    #[test]
    fn test_route_range_with_key_partition() {
        let mut manager = ShardManager::new();
        let nodes = vec![1, 2];
        manager.initialize_table_shards("users", 4, &nodes);

        let rule = PartitionRule::new(
            "users",
            PartitionKey::new_key(vec!["tenant_id".to_string()], 4),
        );
        manager.add_partition_rule(rule);

        let router = ShardRouter::new(manager, 1);
        let result = router.route_range_query("users", "tenant_id", 0, 10);
        assert!(result.is_ok());
        let plan = result.unwrap();
        assert!(plan.is_distributed);
    }

    #[test]
    fn test_route_range_with_list_partition() {
        let mut manager = ShardManager::new();
        let nodes = vec![1, 2, 3];
        manager.initialize_table_shards("regions", 3, &nodes);

        let partitions = vec![
            ListPartition { id: 0, values: vec![1, 2, 3] },
            ListPartition { id: 1, values: vec![4, 5, 6] },
        ];
        let rule = PartitionRule::new("regions", PartitionKey::new_list("region_id", partitions));
        manager.add_partition_rule(rule);

        let router = ShardRouter::new(manager, 1);
        let result = router.route_range_query("regions", "region_id", 1, 6);
        assert!(result.is_ok());
        let plan = result.unwrap();
        assert!(plan.is_distributed);
        assert_eq!(plan.involved_shards.len(), 2);
    }

    #[test]
    fn test_route_to_all_shards_with_key_partition() {
        let mut manager = ShardManager::new();
        let nodes = vec![1, 2];
        manager.initialize_table_shards("users", 4, &nodes);

        let rule = PartitionRule::new(
            "users",
            PartitionKey::new_key(vec!["tenant_id".to_string()], 4),
        );
        manager.add_partition_rule(rule);

        let router = ShardRouter::new(manager, 1);
        let result = router.route_to_all_shards("SELECT * FROM users", "users");
        assert!(result.is_ok());
        let plan = result.unwrap();
        assert!(plan.is_distributed);
        assert_eq!(plan.queries.len(), 4);
    }

    #[test]
    fn test_route_to_all_shards_with_list_partition() {
        let mut manager = ShardManager::new();
        let nodes = vec![1, 2, 3];
        manager.initialize_table_shards("regions", 3, &nodes);

        let partitions = vec![
            ListPartition { id: 0, values: vec![1, 2] },
            ListPartition { id: 1, values: vec![3, 4] },
            ListPartition { id: 2, values: vec![5, 6] },
        ];
        let rule = PartitionRule::new("regions", PartitionKey::new_list("region_id", partitions));
        manager.add_partition_rule(rule);

        let router = ShardRouter::new(manager, 1);
        let result = router.route_to_all_shards("SELECT * FROM regions", "regions");
        assert!(result.is_ok());
        let plan = result.unwrap();
        assert!(plan.is_distributed);
        assert_eq!(plan.queries.len(), 3);
    }

    // ReadWriteShardRouter tests
    fn create_test_rw_router() -> ReadWriteShardRouter {
        let mut manager = ShardManager::new();
        let nodes = vec![1, 2, 3];
        manager.initialize_table_shards("users", 4, &nodes);
        let router = ShardRouter::new(manager, 1);
        ReadWriteShardRouter::new(router)
    }

    #[test]
    fn test_read_write_shard_router_route_read() {
        let rw_router = create_test_rw_router();
        let result = rw_router.route_read("users", "id", PartitionValue::Integer(5));
        assert!(result.is_ok());
        let query = result.unwrap();
        assert_eq!(query.shard_id, 1);
    }

    #[test]
    fn test_read_write_shard_router_route_write() {
        let rw_router = create_test_rw_router();
        let result = rw_router.route_write("users", "id", PartitionValue::Integer(5), "UPDATE users SET name = 'test' WHERE id = 5");
        assert!(result.is_ok());
        let query = result.unwrap();
        assert_eq!(query.shard_id, 1);
        assert_eq!(query.primary_node_id, 2);
    }

    #[test]
    fn test_read_write_shard_router_consistency_level_default() {
        let query = ShardReadQuery {
            shard_id: 0,
            replica_node_id: 1,
            original_sql: "SELECT * FROM users".to_string(),
            consistency_level: ConsistencyLevel::default(),
        };
        assert!(matches!(query.consistency_level, ConsistencyLevel::Eventual));
    }

    #[test]
    fn test_read_write_shard_router_with_strong_consistency() {
        let rw_router = create_test_rw_router();
        let result = rw_router.route_read_with_consistency(
            "users",
            "id",
            PartitionValue::Integer(5),
            ConsistencyLevel::Strong,
        );
        assert!(result.is_ok());
        let query = result.unwrap();
        assert!(matches!(query.consistency_level, ConsistencyLevel::Strong));
    }

    #[test]
    fn test_read_write_shard_router_get_shard_router() {
        let rw_router = create_test_rw_router();
        let _router = rw_router.get_shard_router();
    }
}
