use std::collections::{HashMap, HashSet};

use super::provenance::{OperationType, ProvenanceRecord, SourceType};

#[derive(Debug, Clone)]
pub struct LineageNode {
    pub record_id: String,
    pub parent_ids: Vec<String>,
    pub children_ids: Vec<String>,
    pub operation: OperationType,
    pub creator_id: String,
    pub create_time: i64,
}

#[derive(Debug, Default)]
pub struct LineageGraph {
    nodes: HashMap<String, LineageNode>,
}

impl LineageGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    pub fn add_record(&mut self, record: &ProvenanceRecord) {
        let parent_ids = if record.lineage_path.is_empty() {
            vec![]
        } else {
            record.lineage_path.clone()
        };

        let record_id = record.record_id.clone();
        let children_ids = vec![record_id.clone()];

        let mut new_node = LineageNode {
            record_id: record_id.clone(),
            parent_ids: parent_ids.clone(),
            children_ids: vec![],
            operation: record.operation_type.clone(),
            creator_id: record.creator_id.clone(),
            create_time: record.create_time,
        };

        for parent_id in &parent_ids {
            if let Some(parent) = self.nodes.get_mut(parent_id) {
                parent.children_ids.push(record_id.clone());
            } else {
                let parent_node = LineageNode {
                    record_id: parent_id.clone(),
                    parent_ids: vec![],
                    children_ids: children_ids.clone(),
                    operation: OperationType::Create,
                    creator_id: String::new(),
                    create_time: 0,
                };
                self.nodes.insert(parent_id.clone(), parent_node);
            }
        }

        self.nodes.insert(record_id, new_node);
    }

    pub fn get_lineage_path(&self, record_id: &str) -> Vec<String> {
        let mut path = vec![record_id.to_string()];
        self.get_ancestors_internal(record_id, &mut path);
        path
    }

    fn get_ancestors_internal(&self, record_id: &str, path: &mut Vec<String>) {
        if let Some(node) = self.nodes.get(record_id) {
            for parent_id in &node.parent_ids {
                if !path.contains(parent_id) {
                    path.push(parent_id.clone());
                    self.get_ancestors_internal(parent_id, path);
                }
            }
        }
    }

    pub fn get_descendants(&self, record_id: &str) -> Vec<String> {
        let mut descendants = Vec::new();
        self.get_descendants_internal(record_id, &mut descendants, &mut HashSet::new());
        descendants
    }

    fn get_descendants_internal(
        &self,
        record_id: &str,
        descendants: &mut Vec<String>,
        visited: &mut HashSet<String>,
    ) {
        if visited.contains(record_id) {
            return;
        }
        visited.insert(record_id.to_string());

        if let Some(node) = self.nodes.get(record_id) {
            for child_id in &node.children_ids {
                descendants.push(child_id.clone());
                self.get_descendants_internal(child_id, descendants, visited);
            }
        }
    }

    pub fn verify_integrity(&self) -> bool {
        for node in self.nodes.values() {
            for parent_id in &node.parent_ids {
                if !self.nodes.contains_key(parent_id) {
                    return false;
                }
            }
            for child_id in &node.children_ids {
                if !self.nodes.contains_key(child_id) {
                    return false;
                }
            }
        }
        true
    }

    pub fn get_by_creator(&self, creator_id: &str) -> Vec<&LineageNode> {
        self.nodes
            .values()
            .filter(|n| n.creator_id == creator_id)
            .collect()
    }

    pub fn get_by_time_range(&self, from: i64, to: i64) -> Vec<&LineageNode> {
        self.nodes
            .values()
            .filter(|n| n.create_time >= from && n.create_time <= to)
            .collect()
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_record(record_id: &str, lineage_path: Vec<String>) -> ProvenanceRecord {
        ProvenanceRecord::new(
            record_id.to_string(),
            SourceType::Insert,
            None,
            "user_001".to_string(),
            1000,
            OperationType::Create,
            lineage_path,
        )
    }

    #[test]
    fn test_lineage_graph_add() {
        let mut graph = LineageGraph::new();
        let record = create_test_record("rec_001", vec![]);
        graph.add_record(&record);

        assert_eq!(graph.len(), 1);
        assert!(graph.verify_integrity());
    }

    #[test]
    fn test_lineage_path() {
        let mut graph = LineageGraph::new();
        graph.add_record(&create_test_record("rec_001", vec![]));
        graph.add_record(&create_test_record("rec_002", vec!["rec_001".to_string()]));
        graph.add_record(&create_test_record(
            "rec_003",
            vec!["rec_001".to_string(), "rec_002".to_string()],
        ));

        let path = graph.get_lineage_path("rec_003");
        assert!(path.contains(&"rec_001".to_string()));
        assert!(path.contains(&"rec_002".to_string()));
        assert!(path.contains(&"rec_003".to_string()));
    }

    #[test]
    fn test_descendants() {
        let mut graph = LineageGraph::new();
        graph.add_record(&create_test_record("rec_001", vec![]));
        graph.add_record(&create_test_record("rec_002", vec!["rec_001".to_string()]));
        graph.add_record(&create_test_record("rec_003", vec!["rec_001".to_string()]));

        let descendants = graph.get_descendants("rec_001");
        assert_eq!(descendants.len(), 2);
    }
}
