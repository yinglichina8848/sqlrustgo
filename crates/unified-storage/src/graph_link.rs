//! GraphLink - Association between document rows and graph nodes

use serde::{Deserialize, Serialize};

/// Reference to a graph node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeRef {
    /// Node label in the graph
    pub label: String,
    /// Node ID (optional, for specific node reference)
    pub id: Option<u64>,
    /// Node external key value (string representation)
    pub key: Option<String>,
}

impl NodeRef {
    /// Create a node reference by label only (matches all nodes with label)
    pub fn by_label(label: &str) -> Self {
        Self {
            label: label.to_string(),
            id: None,
            key: None,
        }
    }

    /// Create a node reference by label and key
    pub fn by_key(label: &str, key: &str) -> Self {
        Self {
            label: label.to_string(),
            id: None,
            key: Some(key.to_string()),
        }
    }

    /// Create a node reference by node ID
    pub fn by_id(label: &str, id: u64) -> Self {
        Self {
            label: label.to_string(),
            id: Some(id),
            key: None,
        }
    }

    /// Check if this is a specific node reference (has ID)
    pub fn is_specific(&self) -> bool {
        self.id.is_some() || self.key.is_some()
    }
}

/// Link between a document table/column and graph nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphLink {
    /// Source table name
    table_name: String,
    /// Column in the source table that links to graph
    link_column: String,
    /// Target graph node reference
    node_ref: NodeRef,
    /// Link description
    description: Option<String>,
}

impl GraphLink {
    /// Create a new graph link
    ///
    /// # Arguments
    /// * `table` - Source table name
    /// * `column` - Column in the table that references graph nodes
    /// * `label` - Graph node label
    pub fn new(table: &str, column: &str, label: &str) -> Self {
        Self {
            table_name: table.to_string(),
            link_column: column.to_string(),
            node_ref: NodeRef::by_label(label),
            description: None,
        }
    }

    /// Create with a specific node reference
    pub fn with_node_id(mut self, id: u64) -> Self {
        self.node_ref.id = Some(id);
        self
    }

    /// Create with a node key reference
    pub fn with_node_key(mut self, key: &str) -> Self {
        self.node_ref.key = Some(key.to_string());
        self
    }

    /// Set description
    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = Some(desc.to_string());
        self
    }

    /// Get source table name
    pub fn table_name(&self) -> &str {
        &self.table_name
    }

    /// Get link column name
    pub fn link_column(&self) -> &str {
        &self.link_column
    }

    /// Get node reference
    pub fn node_ref(&self) -> &NodeRef {
        &self.node_ref
    }

    /// Get description
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
}

/// Schema linking documents to graph
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GraphSchema {
    /// All graph links
    links: Vec<GraphLink>,
}

impl GraphSchema {
    /// Create a new empty graph schema
    pub fn new() -> Self {
        Self { links: Vec::new() }
    }

    /// Add a link
    pub fn add_link(&mut self, link: GraphLink) -> &mut Self {
        self.links.push(link);
        self
    }

    /// Get all links
    pub fn links(&self) -> &[GraphLink] {
        &self.links
    }

    /// Find links for a table
    pub fn find_links_for_table(&self, table: &str) -> Vec<&GraphLink> {
        self.links
            .iter()
            .filter(|l| l.table_name == table)
            .collect()
    }

    /// Find links for a column
    pub fn find_links_for_column(&self, table: &str, column: &str) -> Option<&GraphLink> {
        self.links
            .iter()
            .find(|l| l.table_name == table && l.link_column == column)
    }

    /// Check if table has any graph links
    pub fn table_has_graph_link(&self, table: &str) -> bool {
        self.links.iter().any(|l| l.table_name == table)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_link_creation() {
        let link = GraphLink::new("products", "id", "product_node");

        assert_eq!(link.table_name(), "products");
        assert_eq!(link.link_column(), "id");
        assert_eq!(link.node_ref().label, "product_node");
        assert!(!link.node_ref().is_specific());
    }

    #[test]
    fn test_graph_link_with_node_id() {
        let link = GraphLink::new("products", "id", "product_node").with_node_id(42);

        assert!(link.node_ref().is_specific());
        assert_eq!(link.node_ref().id, Some(42));
    }

    #[test]
    fn test_node_ref_types() {
        let by_label = NodeRef::by_label("user");
        assert!(!by_label.is_specific());

        let by_key = NodeRef::by_key("user", "user_123");
        assert!(by_key.is_specific());

        let by_id = NodeRef::by_id("user", 42);
        assert!(by_id.is_specific());
    }

    #[test]
    fn test_graph_schema() {
        let mut schema = GraphSchema::new();
        schema.add_link(GraphLink::new("products", "id", "product_node"));
        schema.add_link(GraphLink::new("users", "id", "user_node"));

        assert_eq!(schema.links().len(), 2);
        assert!(schema.table_has_graph_link("products"));
        assert!(!schema.table_has_graph_link("orders"));

        let product_links = schema.find_links_for_table("products");
        assert_eq!(product_links.len(), 1);
    }
}
