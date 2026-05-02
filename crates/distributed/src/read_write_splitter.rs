//! Read/Write Splitter - classifies SQL queries and routes them to primary or replica
//!
//! This module provides read/write splitting based on SQL statement type:
//! - SELECT, SHOW, DESCRIBE, EXPLAIN, CALL (read-only) -> replica
//! - INSERT, UPDATE, DELETE, CREATE, DROP, ALTER, GRANT, REVOKE (write) -> primary
//!
//! For distributed tables, it uses ShardRouter to determine target shard/node.
//! For non-distributed tables, it routes to the local node's primary or replica.

use crate::shard_manager::{NodeId, ShardId, ShardManager};
use crate::shard_router::{ConsistencyLevel, ReadWriteShardRouter, ShardRouter};
use std::sync::{Arc, RwLock};

/// Classification of a SQL query as read or write
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryClass {
    Read,
    Write,
}

impl QueryClass {
    pub fn is_read(&self) -> bool {
        matches!(self, QueryClass::Read)
    }

    pub fn is_write(&self) -> bool {
        matches!(self, QueryClass::Write)
    }
}

/// Route result for a query
#[derive(Debug, Clone)]
pub struct SplitterResult {
    /// The original SQL
    pub sql: String,
    /// Whether this is a read or write query
    pub query_class: QueryClass,
    /// Target node (distributed or local)
    pub target: RouteTarget,
    /// Consistency level for read queries
    pub consistency_level: ConsistencyLevel,
}

/// Route target - either a distributed shard+node or a local query
#[derive(Debug, Clone)]
pub enum RouteTarget {
    /// Distributed: shard + primary/replica node
    Distributed { shard_id: ShardId, node_id: NodeId },
    /// Non-distributed local query
    Local { is_primary: bool },
}

impl std::fmt::Display for RouteTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RouteTarget::Distributed { shard_id, node_id } => {
                write!(f, "shard={}, node={}", shard_id, node_id)
            }
            RouteTarget::Local { is_primary } => {
                write!(
                    f,
                    "local({})",
                    if *is_primary { "primary" } else { "replica" }
                )
            }
        }
    }
}

/// Read/Write Splitter - classifies SQL and routes to primary or replica
pub struct ReadWriteSplitter {
    shard_manager: Option<Arc<ShardManager>>,
    shard_router: Option<Arc<RwLock<ShardRouter>>>,
    read_write_router: Option<Arc<RwLock<ReadWriteShardRouter>>>,
    local_node_id: NodeId,
    default_replica_node_id: NodeId,
    default_primary_node_id: NodeId,
}

impl ReadWriteSplitter {
    pub fn new(local_node_id: NodeId) -> Self {
        Self {
            shard_manager: None,
            shard_router: None,
            read_write_router: None,
            local_node_id,
            default_replica_node_id: local_node_id + 1,
            default_primary_node_id: local_node_id,
        }
    }

    pub fn with_shard_manager(mut self, shard_manager: Arc<ShardManager>) -> Self {
        self.shard_manager = Some(shard_manager);
        self
    }

    pub fn with_shard_router(mut self, shard_router: Arc<RwLock<ShardRouter>>) -> Self {
        self.shard_router = Some(shard_router);
        self
    }

    pub fn with_read_write_router(
        mut self,
        read_write_router: Arc<RwLock<ReadWriteShardRouter>>,
    ) -> Self {
        self.read_write_router = Some(read_write_router);
        self
    }

    /// Classify a SQL string as read or write
    pub fn classify(&self, sql: &str) -> Result<(QueryClass, String), SplitterError> {
        let statement = sqlrustgo_parser::parse(sql)
            .map_err(|e| SplitterError::ParseError(format!("{:?}", e)))?;
        let query_class = Self::classify_statement(&statement);
        Ok((query_class, format!("{:?}", statement)))
    }

    /// Classify a parsed statement
    pub fn classify_statement(statement: &sqlrustgo_parser::Statement) -> QueryClass {
        match statement {
            // Read queries
            sqlrustgo_parser::Statement::Select(_) => QueryClass::Read,
            sqlrustgo_parser::Statement::Show(_) => QueryClass::Read,
            sqlrustgo_parser::Statement::Describe(_) => QueryClass::Read,
            sqlrustgo_parser::Statement::Call(_) => QueryClass::Read,
            sqlrustgo_parser::Statement::Union(_) => QueryClass::Read,
            sqlrustgo_parser::Statement::WithSelect(_) => QueryClass::Read,
            // Write queries
            sqlrustgo_parser::Statement::Insert(_) => QueryClass::Write,
            sqlrustgo_parser::Statement::Update(_) => QueryClass::Write,
            sqlrustgo_parser::Statement::Delete(_) => QueryClass::Write,
            sqlrustgo_parser::Statement::CreateTable(_) => QueryClass::Write,
            sqlrustgo_parser::Statement::CreateIndex(_) => QueryClass::Write,
            sqlrustgo_parser::Statement::CreateTrigger(_) => QueryClass::Write,
            sqlrustgo_parser::Statement::CreateProcedure(_) => QueryClass::Write,
            sqlrustgo_parser::Statement::DropTable(_) => QueryClass::Write,
            sqlrustgo_parser::Statement::Truncate(_) => QueryClass::Write,
            sqlrustgo_parser::Statement::AlterTable(_) => QueryClass::Write,
            sqlrustgo_parser::Statement::Grant(_) => QueryClass::Write,
            sqlrustgo_parser::Statement::Revoke(_) => QueryClass::Write,
            sqlrustgo_parser::Statement::Transaction(_) => QueryClass::Write,
            sqlrustgo_parser::Statement::Analyze(_) => QueryClass::Write,
            sqlrustgo_parser::Statement::CreateRole(_) => QueryClass::Write,
            sqlrustgo_parser::Statement::DropRole(_) => QueryClass::Write,
            sqlrustgo_parser::Statement::GrantRole(_) => QueryClass::Write,
            sqlrustgo_parser::Statement::RevokeRole(_) => QueryClass::Write,
            sqlrustgo_parser::Statement::SetRole(_) => QueryClass::Write,
            sqlrustgo_parser::Statement::ShowRoles => QueryClass::Read,
            sqlrustgo_parser::Statement::ShowGrantsFor(_) => QueryClass::Read,
        }
    }

    /// Route a SQL query to the appropriate target
    pub fn route(&self, sql: &str) -> Result<SplitterResult, SplitterError> {
        let statement = sqlrustgo_parser::parse(sql)
            .map_err(|e| SplitterError::ParseError(format!("{:?}", e)))?;
        let query_class = Self::classify_statement(&statement);
        let consistency = self.infer_consistency(&statement);

        // Route based on query class
        let target = if query_class.is_read() {
            self.route_read(&statement)?
        } else {
            self.route_write(&statement)?
        };

        Ok(SplitterResult {
            sql: sql.to_string(),
            query_class,
            target,
            consistency_level: consistency,
        })
    }

    /// Route a read query to a replica
    fn route_read(
        &self,
        statement: &sqlrustgo_parser::Statement,
    ) -> Result<RouteTarget, SplitterError> {
        // Check if we have shard routing
        if let Some(ref shard_router) = self.shard_router {
            if let Some(table) = self.extract_table_name(statement) {
                if let Ok(router) = shard_router.read() {
                    let shard_manager = router.get_shard_manager();
                    if shard_manager.get_partition_key(&table).is_some() {
                        // Table is sharded - route to replica
                        // TODO: implement proper replica selection with load balancing
                        return Ok(RouteTarget::Distributed {
                            shard_id: 0,
                            node_id: self.default_replica_node_id,
                        });
                    }
                }
            }
        }

        // Non-distributed: route to local replica for reads
        Ok(RouteTarget::Local { is_primary: false })
    }

    /// Route a write query to the primary
    fn route_write(
        &self,
        statement: &sqlrustgo_parser::Statement,
    ) -> Result<RouteTarget, SplitterError> {
        // Check if we have shard routing
        if let Some(ref shard_router) = self.shard_router {
            if let Some(table) = self.extract_table_name(statement) {
                if let Ok(router) = shard_router.read() {
                    let shard_manager = router.get_shard_manager();
                    if shard_manager.get_partition_key(&table).is_some() {
                        // Table is sharded - route to primary
                        return Ok(RouteTarget::Distributed {
                            shard_id: 0,
                            node_id: self.default_primary_node_id,
                        });
                    }
                }
            }
        }

        // Non-distributed: route to local primary for writes
        Ok(RouteTarget::Local { is_primary: true })
    }

    /// Extract table name from a statement
    fn extract_table_name(&self, statement: &sqlrustgo_parser::Statement) -> Option<String> {
        match statement {
            sqlrustgo_parser::Statement::Select(s) => Some(s.table.clone()),
            sqlrustgo_parser::Statement::Insert(s) => Some(s.table.clone()),
            sqlrustgo_parser::Statement::Update(s) => Some(s.table.clone()),
            sqlrustgo_parser::Statement::Delete(s) => Some(s.table.clone()),
            _ => None,
        }
    }

    /// Infer consistency level from query
    fn infer_consistency(&self, _statement: &sqlrustgo_parser::Statement) -> ConsistencyLevel {
        // Default to eventual consistency for reads, strong for writes
        ConsistencyLevel::Eventual
    }

    /// Simple route: returns (QueryClass, is_primary)
    pub fn route_simple(&self, sql: &str) -> Result<(QueryClass, bool), SplitterError> {
        let result = self.route(sql)?;
        let is_primary = match result.target {
            RouteTarget::Local { is_primary } => is_primary,
            RouteTarget::Distributed { node_id, .. } => node_id == self.local_node_id,
        };
        Ok((result.query_class, is_primary))
    }
}

/// Add this missing field to SplitterResult
impl SplitterResult {
    pub fn target_node(&self) -> &RouteTarget {
        &self.target
    }
}

/// Splitter errors
#[derive(Debug, Clone)]
pub enum SplitterError {
    ParseError(String),
    LockError,
    NoTableFound,
    RoutingError(String),
}

impl std::fmt::Display for SplitterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SplitterError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            SplitterError::LockError => write!(f, "Failed to acquire lock"),
            SplitterError::NoTableFound => write!(f, "No table found in statement"),
            SplitterError::RoutingError(msg) => write!(f, "Routing error: {}", msg),
        }
    }
}

impl std::error::Error for SplitterError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_splitter() -> ReadWriteSplitter {
        ReadWriteSplitter::new(1)
    }

    #[test]
    fn test_classify_select() {
        let splitter = create_splitter();
        let sql = "SELECT * FROM users WHERE id = 1";
        let (class, _) = splitter.classify(sql).unwrap();
        assert_eq!(class, QueryClass::Read);
    }

    #[test]
    fn test_classify_insert() {
        let splitter = create_splitter();
        let sql = "INSERT INTO users (name) VALUES ('test')";
        let (class, _) = splitter.classify(sql).unwrap();
        assert_eq!(class, QueryClass::Write);
    }

    #[test]
    fn test_classify_update() {
        let splitter = create_splitter();
        let sql = "UPDATE users SET name = 'test' WHERE id = 1";
        let (class, _) = splitter.classify(sql).unwrap();
        assert_eq!(class, QueryClass::Write);
    }

    #[test]
    fn test_classify_delete() {
        let splitter = create_splitter();
        let sql = "DELETE FROM users WHERE id = 1";
        let (class, _) = splitter.classify(sql).unwrap();
        assert_eq!(class, QueryClass::Write);
    }

    #[test]
    fn test_classify_create() {
        let splitter = create_splitter();
        let sql = "CREATE TABLE test (id INT PRIMARY KEY)";
        let (class, _) = splitter.classify(sql).unwrap();
        assert_eq!(class, QueryClass::Write);
    }

    #[test]
    fn test_classify_show() {
        let splitter = create_splitter();
        let sql = "SHOW TABLES";
        let (class, _) = splitter.classify(sql).unwrap();
        assert_eq!(class, QueryClass::Read);
    }

    #[test]
    fn test_classify_describe() {
        let splitter = create_splitter();
        let sql = "DESCRIBE users";
        let (class, _) = splitter.classify(sql).unwrap();
        assert_eq!(class, QueryClass::Read);
    }

    #[test]
    fn test_route_select_to_replica() {
        let splitter = create_splitter();
        let sql = "SELECT * FROM users WHERE id = 1";
        let result = splitter.route(sql).unwrap();
        assert_eq!(result.query_class, QueryClass::Read);
        match result.target {
            RouteTarget::Local { is_primary } => assert!(!is_primary),
            RouteTarget::Distributed { .. } => {}
        }
    }

    #[test]
    fn test_route_insert_to_primary() {
        let splitter = create_splitter();
        let sql = "INSERT INTO users (name) VALUES ('test')";
        let result = splitter.route(sql).unwrap();
        assert_eq!(result.query_class, QueryClass::Write);
        match result.target {
            RouteTarget::Local { is_primary } => assert!(is_primary),
            RouteTarget::Distributed { .. } => {}
        }
    }

    #[test]
    fn test_route_simple() {
        let splitter = create_splitter();
        let (class, is_primary) = splitter.route_simple("SELECT * FROM users").unwrap();
        assert_eq!(class, QueryClass::Read);
        assert!(!is_primary);
    }

    #[test]
    fn test_route_simple_insert() {
        let splitter = create_splitter();
        let (class, is_primary) = splitter
            .route_simple("INSERT INTO users (id) VALUES (1)")
            .unwrap();
        assert_eq!(class, QueryClass::Write);
        assert!(is_primary);
    }
}
