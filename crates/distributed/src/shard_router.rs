//! Shard Router - routes queries to appropriate shards
//!
//! Handles SQL routing based on partition keys.

use crate::partition::{PartitionKey, PartitionValue};
use crate::shard_manager::{NodeId, ShardId, ShardInfo, ShardManager};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
            .ok_or_else(|| RouterError::InvalidPartitionKey(table.to_string()))?;

        let shard = self
            .shard_manager
            .get_shard(shard_id)
            .ok_or_else(|| RouterError::ShardNotFound(shard_id))?;

        let node_id = shard
            .primary_node()
            .ok_or_else(|| RouterError::NoReplicaAvailable(shard_id))?;

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
                    .ok_or_else(|| RouterError::ShardNotFound(shard_id))?;

                let node_id = shard
                    .primary_node()
                    .ok_or_else(|| RouterError::NoReplicaAvailable(shard_id))?;

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
        let mut current_shard = 0;

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
                        .ok_or_else(|| RouterError::ShardNotFound(i as u64))?;

                    let node_id = shard
                        .primary_node()
                        .ok_or_else(|| RouterError::NoReplicaAvailable(i as u64))?;

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
                current_shard = i + 1;
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
                .ok_or_else(|| RouterError::ShardNotFound(shard_id))?;

            let node_id = shard
                .primary_node()
                .ok_or_else(|| RouterError::NoReplicaAvailable(shard_id))?;

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
        };

        let mut queries = Vec::new();
        let mut shards = Vec::new();

        for shard_id in 0..num_shards {
            shards.push(shard_id);

            let shard = self
                .shard_manager
                .get_shard(shard_id)
                .ok_or_else(|| RouterError::ShardNotFound(shard_id))?;

            let node_id = shard
                .primary_node()
                .ok_or_else(|| RouterError::NoReplicaAvailable(shard_id))?;

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
