//! LabelRegistry - maps string labels to LabelIds

use crate::model::{LabelId, GMP_LABELS};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Thread-safe label registry
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct LabelRegistry {
    pub(crate) string_to_id: HashMap<String, LabelId>,
    pub(crate) id_to_string: HashMap<LabelId, String>,
    pub(crate) next_id: LabelId,
}

impl LabelRegistry {
    pub fn new() -> Self {
        let mut registry = LabelRegistry {
            string_to_id: HashMap::new(),
            id_to_string: HashMap::new(),
            next_id: LabelId::MIN,
        };
        for label in GMP_LABELS {
            let _ = registry.register(label.to_string());
        }
        registry
    }

    pub fn register(&mut self, label: String) -> LabelId {
        if let Some(&id) = self.string_to_id.get(&label) {
            return id;
        }
        let id = self.next_id;
        self.next_id = self.next_id.next();
        self.string_to_id.insert(label.clone(), id);
        self.id_to_string.insert(id, label);
        id
    }

    pub fn get_or_register(&mut self, label: &str) -> LabelId {
        self.get(label)
            .unwrap_or_else(|| self.register(label.to_string()))
    }

    pub fn get(&self, label: &str) -> Option<LabelId> {
        self.string_to_id.get(label).copied()
    }

    pub fn get_label(&self, id: LabelId) -> Option<&String> {
        self.id_to_string.get(&id)
    }

    pub fn contains(&self, label: &str) -> bool {
        self.string_to_id.contains_key(label)
    }

    pub fn len(&self) -> usize {
        self.string_to_id.len()
    }

    pub fn is_empty(&self) -> bool {
        self.string_to_id.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (LabelId, &String)> {
        self.id_to_string.iter().map(|(&id, s)| (id, s))
    }
}
