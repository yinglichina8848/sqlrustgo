//! Edge (relationship) between nodes

use super::{EdgeId, LabelId, NodeId, PropertyMap};
use serde::{Deserialize, Serialize};

/// Edge direction
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Direction {
    #[default]
    Outgoing, // ->
    Incoming,      // <-
    Bidirectional, // <->
}

/// Edge entity in the graph
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Edge {
    /// Unique edge identifier
    pub id: EdgeId,
    /// Source node
    pub from: NodeId,
    /// Target node
    pub to: NodeId,
    /// Edge label (relationship type)
    pub label: LabelId,
    /// Edge properties
    pub properties: PropertyMap,
    /// Edge direction
    pub direction: Direction,
}

impl Edge {
    pub const LABEL: &'static str = "Edge";

    pub fn new(
        id: EdgeId,
        from: NodeId,
        to: NodeId,
        label: LabelId,
        properties: PropertyMap,
    ) -> Self {
        Edge {
            id,
            from,
            to,
            label,
            properties,
            direction: Direction::Outgoing,
        }
    }

    pub fn with_direction(
        id: EdgeId,
        from: NodeId,
        to: NodeId,
        label: LabelId,
        properties: PropertyMap,
        direction: Direction,
    ) -> Self {
        Edge {
            id,
            from,
            to,
            label,
            properties,
            direction,
        }
    }

    /// Check if this edge connects the given nodes (in either direction)
    pub fn connects(&self, node_a: NodeId, node_b: NodeId) -> bool {
        (self.from == node_a && self.to == node_b) || (self.from == node_b && self.to == node_a)
    }

    /// Check if this edge is from the given node
    pub fn is_from(&self, node: NodeId) -> bool {
        self.from == node
    }

    /// Check if this edge is to the given node
    pub fn is_to(&self, node: NodeId) -> bool {
        self.to == node
    }

    /// Get the other node given one endpoint
    pub fn other_endpoint(&self, node: NodeId) -> Option<NodeId> {
        if self.from == node {
            Some(self.to)
        } else if self.to == node {
            Some(self.from)
        } else {
            None
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
}

/// GMP Core Edge Labels
pub const GMP_EDGE_LABELS: &[&str] = &[
    "produced_by",    // Batch -> Device
    "calibrated_by",  // Device -> SOP
    "follows_step",   // Step -> SOP
    "deviation_from", // Deviation -> SOP
    "triggers_cap",   // Deviation -> CAPA
    "governed_by",    // Batch -> Regulation
    "uses_material",  // Batch -> Material
    "operated_by",    // Device -> Operator
    "inspected_by",   // Batch -> QA
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edge_creation() {
        let mut props = PropertyMap::new();
        props.insert("timestamp", "2024-01-01T00:00:00Z");
        props.insert("result", "pass");

        let edge = Edge::new(
            EdgeId::new(1),
            NodeId::new(100),
            NodeId::new(200),
            LabelId::new(5),
            props,
        );

        assert_eq!(edge.id, EdgeId(1));
        assert_eq!(edge.from, NodeId(100));
        assert_eq!(edge.to, NodeId(200));
        assert_eq!(edge.label, LabelId(5));
        assert_eq!(edge.direction, Direction::Outgoing);
    }

    #[test]
    fn test_edge_connects() {
        let edge = Edge::new(
            EdgeId::new(1),
            NodeId::new(100),
            NodeId::new(200),
            LabelId::new(1),
            PropertyMap::new(),
        );

        assert!(edge.connects(NodeId(100), NodeId(200)));
        assert!(edge.connects(NodeId(200), NodeId(100)));
        assert!(!edge.connects(NodeId(100), NodeId(300)));
    }

    #[test]
    fn test_edge_other_endpoint() {
        let edge = Edge::new(
            EdgeId::new(1),
            NodeId::new(100),
            NodeId::new(200),
            LabelId::new(1),
            PropertyMap::new(),
        );

        assert_eq!(edge.other_endpoint(NodeId(100)), Some(NodeId(200)));
        assert_eq!(edge.other_endpoint(NodeId(200)), Some(NodeId(100)));
        assert_eq!(edge.other_endpoint(NodeId(300)), None);
    }

    #[test]
    fn test_edge_with_direction() {
        let incoming = Edge::with_direction(
            EdgeId::new(1),
            NodeId::new(100),
            NodeId::new(200),
            LabelId::new(1),
            PropertyMap::new(),
            Direction::Incoming,
        );
        assert_eq!(incoming.direction, Direction::Incoming);

        let bidirectional = Edge::with_direction(
            EdgeId::new(2),
            NodeId::new(100),
            NodeId::new(200),
            LabelId::new(1),
            PropertyMap::new(),
            Direction::Bidirectional,
        );
        assert_eq!(bidirectional.direction, Direction::Bidirectional);
    }

    #[test]
    fn test_gmp_edge_labels() {
        assert!(GMP_EDGE_LABELS.contains(&"produced_by"));
        assert!(GMP_EDGE_LABELS.contains(&"governed_by"));
        assert_eq!(GMP_EDGE_LABELS.len(), 9);
    }
}
