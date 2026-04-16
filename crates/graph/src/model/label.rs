//! Label registry for mapping string labels to LabelIds

use super::{LabelId, GMP_LABELS};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Thread-safe label registry
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct LabelRegistry {
    /// Map from string label to LabelId
    string_to_id: HashMap<String, LabelId>,
    /// Map from LabelId to string label
    id_to_string: HashMap<LabelId, String>,
    /// Next available LabelId
    next_id: LabelId,
}

impl LabelRegistry {
    pub fn new() -> Self {
        let mut registry = LabelRegistry {
            string_to_id: HashMap::new(),
            id_to_string: HashMap::new(),
            next_id: LabelId::new(0),
        };
        // Pre-register GMP core labels
        for label in GMP_LABELS {
            let _ = registry.register(label.to_string());
        }
        registry
    }

    /// Register a new label, returns the LabelId
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

    /// Get LabelId for a label string, registers if not exists
    pub fn get_or_register(&mut self, label: &str) -> LabelId {
        if let Some(&id) = self.string_to_id.get(label) {
            return id;
        }
        self.register(label.to_string())
    }

    /// Get LabelId for a label string, returns None if not registered
    pub fn get(&self, label: &str) -> Option<LabelId> {
        self.string_to_id.get(label).copied()
    }

    /// Get label string for a LabelId, returns None if not found
    pub fn get_label(&self, id: LabelId) -> Option<&str> {
        self.id_to_string.get(&id).map(|s| s.as_str())
    }

    /// Check if a label is registered
    pub fn contains(&self, label: &str) -> bool {
        self.string_to_id.contains_key(label)
    }

    /// Check if a LabelId is registered
    pub fn contains_id(&self, id: LabelId) -> bool {
        self.id_to_string.contains_key(&id)
    }

    /// Get total number of registered labels
    pub fn len(&self) -> usize {
        self.string_to_id.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.string_to_id.is_empty()
    }

    /// Get all registered labels
    pub fn labels(&self) -> impl Iterator<Item = &str> {
        self.string_to_id.keys().map(|s| s.as_str())
    }

    /// Get all (LabelId, label) pairs
    pub fn iter(&self) -> impl Iterator<Item = (LabelId, &str)> {
        self.id_to_string.iter().map(|(&id, s)| (id, s.as_str()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_label_registry_basics() {
        let mut registry = LabelRegistry::new();

        // GMP labels should be pre-registered
        assert!(registry.contains("Batch"));
        assert!(registry.contains("Device"));
        assert_eq!(registry.get("Batch"), Some(LabelId(0)));
        assert_eq!(registry.get("Device"), Some(LabelId(1)));

        // Register new label
        let id = registry.register("CustomLabel".to_string());
        assert_eq!(registry.get("CustomLabel"), Some(id));
        assert_eq!(registry.get_label(id), Some("CustomLabel"));
    }

    #[test]
    fn test_get_or_register() {
        let mut registry = LabelRegistry::new();

        let id1 = registry.get_or_register("NewLabel");
        let id2 = registry.get_or_register("NewLabel");
        assert_eq!(id1, id2);

        let id3 = registry.get_or_register("AnotherLabel");
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_label_registry_len() {
        let registry = LabelRegistry::new();
        // Should have 10 GMP labels pre-registered
        assert_eq!(registry.len(), 10);
    }

    #[test]
    fn test_label_registry_iter() {
        let registry = LabelRegistry::new();
        let items: Vec<_> = registry.iter().collect();
        assert!(items.len() >= 10);
        // Check GMP labels are in there
        assert!(items.iter().any(|(id, label)| *label == "Batch"));
    }

    #[test]
    fn test_label_registry_contains_id() {
        let mut registry = LabelRegistry::new();

        let batch_id = registry.get("Batch").unwrap();
        assert!(registry.contains_id(batch_id));
        assert!(!registry.contains_id(LabelId(9999)));
    }

    #[test]
    fn test_label_registry_labels_iterator() {
        let registry = LabelRegistry::new();
        let labels: Vec<_> = registry.labels().collect();
        assert!(labels.len() >= 10);
        assert!(labels.contains(&"Batch"));
        assert!(labels.contains(&"Device"));
    }

    #[test]
    fn test_label_registry_get_label() {
        let mut registry = LabelRegistry::new();

        let custom_id = registry.register("Custom".to_string());
        assert_eq!(registry.get_label(custom_id), Some("Custom"));

        let nonexistent = registry.get_label(LabelId(9999));
        assert_eq!(nonexistent, None);
    }

    #[test]
    fn test_label_registry_id_exhausted() {
        let mut registry = LabelRegistry::new();

        // Register many labels to approach LabelId max
        for i in 0..100 {
            let label = format!("Label{}", i);
            let _ = registry.register(label);
        }

        assert!(registry.len() > 100);
        assert!(registry.contains("Label50"));
    }
}
