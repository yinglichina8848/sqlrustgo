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

    #[test]
    fn test_is_single_node_transaction() {
        let mut router = Router::new();
        router.register_table("users", 1);
        router.register_table("orders", 2);

        let single_table = vec!["users".to_string()];
        assert!(router.is_single_node_transaction(&single_table));

        let multi_table = vec!["users".to_string(), "orders".to_string()];
        assert!(!router.is_single_node_transaction(&multi_table));
    }

    #[test]
    fn test_unknown_table_error() {
        let mut router = Router::new();
        router.register_table("users", 1);

        let tables = vec!["users".to_string(), "unknown".to_string()];
        let result = router.resolve_tables(&tables);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown table"));
    }
}
