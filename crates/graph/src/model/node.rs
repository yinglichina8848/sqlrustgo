//! Node entity

use super::{LabelId, NodeId, PropertyMap};
use serde::{Deserialize, Serialize};

/// Node entity in the graph
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Node {
    /// Unique node identifier
    pub id: NodeId,
    /// Node label (type)
    pub label: LabelId,
    /// Node properties
    pub properties: PropertyMap,
}

impl Node {
    pub const LABEL: &'static str = "Node";

    /// Create a new node with the given ID, label, and properties
    pub fn new(id: NodeId, label: LabelId, properties: PropertyMap) -> Self {
        Node {
            id,
            label,
            properties,
        }
    }

    /// Create a new node with pre-allocated property capacity
    pub fn with_capacity(id: NodeId, label: LabelId, capacity: usize) -> Self {
        Node {
            id,
            label,
            properties: PropertyMap::with_capacity(capacity),
        }
    }

    /// Get property value by key
    pub fn get_property(&self, key: &str) -> Option<&super::PropertyValue> {
        self.properties.get(key)
    }

    /// Set property value
    pub fn set_property<K: Into<String>, V: Into<super::PropertyValue>>(
        &mut self,
        key: K,
        value: V,
    ) {
        self.properties.insert(key, value);
    }

    /// Check if node has a property
    pub fn has_property(&self, key: &str) -> bool {
        self.properties.contains_key(key)
    }
}

/// GMP Core Labels
pub const GMP_LABELS: &[&str] = &[
    "Batch",
    "Device",
    "SOP",
    "Step",
    "Deviation",
    "CAPA",
    "Regulation",
    "Material",
    "Operator",
    "QA",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_creation() {
        let mut props = PropertyMap::new();
        props.insert("name", "batch-001");
        props.insert("quantity", 100i64);

        let node = Node::new(NodeId::new(1), LabelId::new(1), props);

        assert_eq!(node.id, NodeId(1));
        assert_eq!(node.label, LabelId(1));
        assert_eq!(
            node.get_property("name").unwrap().as_string(),
            Some(&"batch-001".to_string())
        );
        assert_eq!(node.get_property("quantity").unwrap().as_int(), Some(100));
    }

    #[test]
    fn test_node_builder() {
        let mut node = Node::with_capacity(NodeId::new(1), LabelId::new(1), 4);
        node.set_property("name", "test");
        node.set_property("active", true);

        assert!(node.has_property("name"));
        assert!(node.has_property("active"));
    }

    #[test]
    fn test_gmp_labels() {
        assert_eq!(GMP_LABELS.len(), 10);
        assert!(GMP_LABELS.contains(&"Batch"));
        assert!(GMP_LABELS.contains(&"Device"));
        assert!(GMP_LABELS.contains(&"Regulation"));
    }
}
