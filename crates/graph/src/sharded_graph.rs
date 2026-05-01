//! Sharded Graph Store - distributed graph storage with label-based partitioning
//!
//! Provides horizontal scaling for graph data through sharding.
//! Nodes are distributed across shards based on their labels.

use crate::error::GraphError;
use crate::model::*;
use crate::store::{GraphStore, InMemoryGraphStore};
use std::collections::HashMap;

/// Shard identifier for graph storage
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct GraphShardId(pub u64);

impl GraphShardId {
    pub fn new(id: u64) -> Self {
        GraphShardId(id)
    }
}

/// Label-based partitioning strategy for graphs
/// Routes nodes to shards based on their label
pub struct LabelBasedGraphPartitioner {
    label_to_shard: HashMap<String, GraphShardId>,
    default_shard: GraphShardId,
}

impl LabelBasedGraphPartitioner {
    pub fn new() -> Self {
        LabelBasedGraphPartitioner {
            label_to_shard: HashMap::new(),
            default_shard: GraphShardId(0),
        }
    }

    pub fn register_label(&mut self, label: &str, shard_id: GraphShardId) {
        self.label_to_shard.insert(label.to_string(), shard_id);
    }

    pub fn get_shard_for_label(&self, label: &str) -> GraphShardId {
        self.label_to_shard
            .get(label)
            .copied()
            .unwrap_or(self.default_shard)
    }

    pub fn set_default_shard(&mut self, shard_id: GraphShardId) {
        self.default_shard = shard_id;
    }
}

impl Default for LabelBasedGraphPartitioner {
    fn default() -> Self {
        Self::new()
    }
}

struct GraphShard {
    store: InMemoryGraphStore,
}

impl GraphShard {
    fn new() -> Self {
        GraphShard {
            store: InMemoryGraphStore::new(),
        }
    }
}

/// Multi-shard graph store implementation
pub struct MultiShardGraphStore {
    shards: HashMap<GraphShardId, GraphShard>,
    partitioner: LabelBasedGraphPartitioner,
    node_to_shard: HashMap<NodeId, GraphShardId>,
    next_node_id: NodeId,
}

impl MultiShardGraphStore {
    /// Create a new multi-shard graph store
    pub fn new() -> Self {
        MultiShardGraphStore {
            shards: HashMap::new(),
            partitioner: LabelBasedGraphPartitioner::new(),
            node_to_shard: HashMap::new(),
            next_node_id: NodeId::MIN,
        }
    }

    /// Create a new shard
    pub fn create_shard(&mut self, shard_id: GraphShardId) {
        self.shards.entry(shard_id).or_insert_with(GraphShard::new);
    }

    /// Register a label to shard mapping
    /// Register a label to shard mapping
    pub fn register_label_sharding(&mut self, label: &str, shard_id: GraphShardId) {
        self.create_shard(shard_id);
        self.partitioner.register_label(label, shard_id);
    }

    /// Set the default shard for unmapped labels
    pub fn set_default_shard(&mut self, shard_id: GraphShardId) {
        self.create_shard(shard_id);
        self.partitioner.set_default_shard(shard_id);
    }

    fn get_shard_for_label(&self, label: &str) -> GraphShardId {
        self.partitioner.get_shard_for_label(label)
    }

    pub fn get_shard_for_node(&self, node_id: NodeId) -> Option<GraphShardId> {
        self.node_to_shard.get(&node_id).copied()
    }

    fn get_or_create_shard(&mut self, shard_id: GraphShardId) -> &mut GraphShard {
        self.shards.entry(shard_id).or_insert_with(GraphShard::new)
    }

    pub fn get_shard_ids(&self) -> Vec<GraphShardId> {
        self.shards.keys().copied().collect()
    }

    pub fn total_node_count(&self) -> usize {
        self.shards.values().map(|s| s.store.node_count()).sum()
    }

    pub fn total_edge_count(&self) -> usize {
        self.shards.values().map(|s| s.store.edge_count()).sum()
    }
}

impl Default for MultiShardGraphStore {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphStore for MultiShardGraphStore {
    fn create_node(&mut self, label: &str, props: PropertyMap) -> NodeId {
        let shard_id = self.get_shard_for_label(label);
        let node_id = self.next_node_id;
        self.next_node_id = self.next_node_id.next();

        let shard = self.get_or_create_shard(shard_id);
        shard.store.create_node_with_id(label, props, node_id);

        self.node_to_shard.insert(node_id, shard_id);
        node_id
    }

    fn get_node(&self, id: NodeId) -> Option<Node> {
        self.get_shard_for_node(id)
            .and_then(|shard_id| self.shards.get(&shard_id))
            .and_then(|shard| shard.store.get_node(id))
    }

    fn nodes_by_label(&self, label: &str) -> Vec<NodeId> {
        let shard_id = self.partitioner.get_shard_for_label(label);
        self.shards
            .get(&shard_id)
            .map(|shard| shard.store.nodes_by_label(label))
            .unwrap_or_default()
    }

    fn update_node(&mut self, id: NodeId, props: PropertyMap) -> Result<(), GraphError> {
        match self.get_shard_for_node(id) {
            Some(shard_id) => {
                if let Some(shard) = self.shards.get_mut(&shard_id) {
                    shard.store.update_node(id, props)
                } else {
                    Err(GraphError::NodeNotFound(id))
                }
            }
            None => Err(GraphError::NodeNotFound(id)),
        }
    }

    fn delete_node(&mut self, id: NodeId) -> Result<(), GraphError> {
        match self.node_to_shard.remove(&id) {
            Some(shard_id) => {
                if let Some(shard) = self.shards.get_mut(&shard_id) {
                    shard.store.delete_node(id)
                } else {
                    Err(GraphError::NodeNotFound(id))
                }
            }
            None => Err(GraphError::NodeNotFound(id)),
        }
    }

    fn create_edge(
        &mut self,
        from: NodeId,
        to: NodeId,
        label: &str,
        props: PropertyMap,
    ) -> Result<EdgeId, GraphError> {
        let from_shard = self.get_shard_for_node(from);
        let to_shard = self.get_shard_for_node(to);

        match (from_shard, to_shard) {
            (Some(fs), Some(ts)) if fs == ts => {
                if let Some(shard) = self.shards.get_mut(&fs) {
                    shard.store.create_edge(from, to, label, props)
                } else {
                    Err(GraphError::NodeNotFound(from))
                }
            }
            _ => Err(GraphError::InvalidEdge { from, to }),
        }
    }

    fn get_edge(&self, id: EdgeId) -> Option<Edge> {
        for shard in self.shards.values() {
            if let Some(edge) = shard.store.get_edge(id) {
                return Some(edge);
            }
        }
        None
    }

    fn edges_by_label(&self, label: &str) -> Vec<EdgeId> {
        let mut edges = Vec::new();
        for shard in self.shards.values() {
            edges.extend(shard.store.edges_by_label(label));
        }
        edges
    }

    fn delete_edge(&mut self, id: EdgeId) -> Result<(), GraphError> {
        for shard in self.shards.values_mut() {
            if shard.store.get_edge(id).is_some() {
                return shard.store.delete_edge(id);
            }
        }
        Err(GraphError::EdgeNotFound(id))
    }

    fn outgoing_neighbors(&self, node: NodeId) -> Vec<NodeId> {
        self.get_shard_for_node(node)
            .and_then(|shard_id| self.shards.get(&shard_id))
            .map(|shard| shard.store.outgoing_neighbors(node))
            .unwrap_or_default()
    }

    fn incoming_neighbors(&self, node: NodeId) -> Vec<NodeId> {
        self.get_shard_for_node(node)
            .and_then(|shard_id| self.shards.get(&shard_id))
            .map(|shard| shard.store.incoming_neighbors(node))
            .unwrap_or_default()
    }

    fn neighbors_by_edge_label(&self, node: NodeId, edge_label: &str) -> Vec<NodeId> {
        self.get_shard_for_node(node)
            .and_then(|shard_id| self.shards.get(&shard_id))
            .map(|shard| shard.store.neighbors_by_edge_label(node, edge_label))
            .unwrap_or_default()
    }

    fn bfs<F>(&self, start: NodeId, visitor: F)
    where
        F: FnMut(NodeId) -> bool,
    {
        if let Some(shard_id) = self.get_shard_for_node(start) {
            if let Some(shard) = self.shards.get(&shard_id) {
                shard.store.bfs(start, visitor);
            }
        }
    }

    fn dfs<F>(&self, start: NodeId, visitor: F)
    where
        F: FnMut(NodeId) -> bool,
    {
        if let Some(shard_id) = self.get_shard_for_node(start) {
            if let Some(shard) = self.shards.get(&shard_id) {
                shard.store.dfs(start, visitor);
            }
        }
    }

    fn node_count(&self) -> usize {
        self.total_node_count()
    }

    fn edge_count(&self) -> usize {
        self.total_edge_count()
    }

    fn label_registry(&self) -> &crate::store::LabelRegistry {
        if let Some(shard) = self.shards.values().next() {
            shard.store.label_registry()
        } else {
            panic!("No shards available")
        }
    }
}

/// Cross-shard traversal executor
pub struct CrossShardTraversal {
    store: MultiShardGraphStore,
}

impl CrossShardTraversal {
    pub fn new(store: MultiShardGraphStore) -> Self {
        CrossShardTraversal { store }
    }

    pub fn distributed_bfs(&self, start: NodeId, max_depth: usize) -> Vec<NodeId> {
        let mut result = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut queue = vec![(start, 0)];

        while let Some((node_id, depth)) = queue.pop() {
            if depth > max_depth {
                continue;
            }

            if visited.contains(&node_id) {
                continue;
            }
            visited.insert(node_id);
            result.push(node_id);

            if let Some(neighbors) = self.store.get_shard_for_node(node_id) {
                if let Some(shard) = self.store.shards.get(&neighbors) {
                    let outs = shard.store.outgoing_neighbors(node_id);
                    for neighbor in outs {
                        if !visited.contains(&neighbor) {
                            queue.push((neighbor, depth + 1));
                        }
                    }
                }
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_label_based_partitioning() {
        let mut partitioner = LabelBasedGraphPartitioner::new();
        partitioner.register_label("User", GraphShardId(0));
        partitioner.register_label("Product", GraphShardId(1));
        partitioner.set_default_shard(GraphShardId(2));

        assert_eq!(partitioner.get_shard_for_label("User"), GraphShardId(0));
        assert_eq!(partitioner.get_shard_for_label("Product"), GraphShardId(1));
        assert_eq!(partitioner.get_shard_for_label("Order"), GraphShardId(2));
    }

    #[test]
    fn test_multi_shard_store() {
        let mut store = MultiShardGraphStore::new();
        store.register_label_sharding("User", GraphShardId(0));
        store.register_label_sharding("Order", GraphShardId(1));

        let user1 = store.create_node("User", PropertyMap::new());
        let user2 = store.create_node("User", PropertyMap::new());
        let order1 = store.create_node("Order", PropertyMap::new());

        assert_eq!(store.node_count(), 3);
        assert_eq!(store.total_node_count(), 3);

        assert_eq!(store.get_shard_for_node(user1), Some(GraphShardId(0)));
        assert_eq!(store.get_shard_for_node(user2), Some(GraphShardId(0)));
        assert_eq!(store.get_shard_for_node(order1), Some(GraphShardId(1)));

        let users = store.nodes_by_label("User");
        assert_eq!(users.len(), 2);

        let orders = store.nodes_by_label("Order");
        assert_eq!(orders.len(), 1);
    }

    #[test]
    fn test_cross_shard_edge_creation_fails() {
        let mut store = MultiShardGraphStore::new();
        store.register_label_sharding("User", GraphShardId(0));
        store.register_label_sharding("Order", GraphShardId(1));

        let user1 = store.create_node("User", PropertyMap::new());
        let order1 = store.create_node("Order", PropertyMap::new());

        let result = store.create_edge(user1, order1, "PLACED", PropertyMap::new());
        assert!(result.is_err());
    }

    #[test]
    fn test_same_shard_edge_creation() {
        let mut store = MultiShardGraphStore::new();
        store.register_label_sharding("User", GraphShardId(0));

        let user1 = store.create_node("User", PropertyMap::new());
        let user2 = store.create_node("User", PropertyMap::new());

        let edge_id = store.create_edge(user1, user2, "KNOWS", PropertyMap::new());
        assert!(edge_id.is_ok());
        assert_eq!(store.edge_count(), 1);
    }

    #[test]
    fn test_cross_shard_traversal() {
        let mut store = MultiShardGraphStore::new();
        store.register_label_sharding("User", GraphShardId(0));
        store.set_default_shard(GraphShardId(0));

        let node1 = store.create_node("User", PropertyMap::new());
        let _node2 = store.create_node("User", PropertyMap::new());

        let traversal = CrossShardTraversal::new(store);
        let result = traversal.distributed_bfs(node1, 2);
        assert!(result.contains(&node1));
    }

    #[test]
    fn test_delete_node() {
        let mut store = MultiShardGraphStore::new();
        store.register_label_sharding("User", GraphShardId(0));

        let user1 = store.create_node("User", PropertyMap::new());
        assert_eq!(store.node_count(), 1);

        let result = store.delete_node(user1);
        assert!(result.is_ok());
        assert_eq!(store.node_count(), 0);
        assert!(store.get_node(user1).is_none());
    }

    #[test]
    fn test_default_shard_creation() {
        let mut store = MultiShardGraphStore::new();
        store.set_default_shard(GraphShardId(5));

        let node1 = store.create_node("Unknown", PropertyMap::new());
        assert_eq!(store.get_shard_for_node(node1), Some(GraphShardId(5)));
    }
}
