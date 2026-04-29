//! Property map for flexible key-value storage

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Property value types
#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum PropertyValue {
    Null,
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Uuid(String), // UUID as property (not for internal IDs)
}

impl PropertyValue {
    /// Get string value if variant is String
    pub fn as_string(&self) -> Option<&String> {
        match self {
            PropertyValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Get integer value if variant is Int
    pub fn as_int(&self) -> Option<i64> {
        match self {
            PropertyValue::Int(i) => Some(*i),
            _ => None,
        }
    }

    /// Get float value if variant is Float
    pub fn as_float(&self) -> Option<f64> {
        match self {
            PropertyValue::Float(f) => Some(*f),
            _ => None,
        }
    }

    /// Get boolean value if variant is Bool
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            PropertyValue::Bool(b) => Some(*b),
            _ => None,
        }
    }
}

impl fmt::Display for PropertyValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PropertyValue::Null => write!(f, "null"),
            PropertyValue::String(s) => write!(f, "\"{}\"", s),
            PropertyValue::Int(i) => write!(f, "{}", i),
            PropertyValue::Float(fl) => write!(f, "{}", fl),
            PropertyValue::Bool(b) => write!(f, "{}", b),
            PropertyValue::Uuid(u) => write!(f, "uuid:{}", u),
        }
    }
}

impl From<&str> for PropertyValue {
    fn from(s: &str) -> Self {
        PropertyValue::String(s.to_string())
    }
}

impl From<String> for PropertyValue {
    fn from(s: String) -> Self {
        PropertyValue::String(s)
    }
}

impl From<i64> for PropertyValue {
    fn from(i: i64) -> Self {
        PropertyValue::Int(i)
    }
}

impl From<i32> for PropertyValue {
    fn from(i: i32) -> Self {
        PropertyValue::Int(i as i64)
    }
}

impl From<f64> for PropertyValue {
    fn from(f: f64) -> Self {
        PropertyValue::Float(f)
    }
}

impl From<bool> for PropertyValue {
    fn from(b: bool) -> Self {
        PropertyValue::Bool(b)
    }
}

/// Property map - flexible key-value storage for nodes and edges
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct PropertyMap {
    props: HashMap<String, PropertyValue>,
}

impl PropertyMap {
    /// Create a new empty property map
    pub fn new() -> Self {
        PropertyMap {
            props: HashMap::new(),
        }
    }

    /// Create a new property map with pre-allocated capacity
    pub fn with_capacity(capacity: usize) -> Self {
        PropertyMap {
            props: HashMap::with_capacity(capacity),
        }
    }

    /// Insert a key-value pair
    pub fn insert<K: Into<String>, V: Into<PropertyValue>>(
        &mut self,
        key: K,
        value: V,
    ) -> Option<PropertyValue> {
        self.props.insert(key.into(), value.into())
    }

    pub fn get(&self, key: &str) -> Option<&PropertyValue> {
        self.props.get(key)
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.props.contains_key(key)
    }

    pub fn remove(&mut self, key: &str) -> Option<PropertyValue> {
        self.props.remove(key)
    }

    pub fn len(&self) -> usize {
        self.props.len()
    }

    pub fn is_empty(&self) -> bool {
        self.props.is_empty()
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.props.keys()
    }

    pub fn values(&self) -> impl Iterator<Item = &PropertyValue> {
        self.props.values()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &PropertyValue)> {
        self.props.iter()
    }

    pub fn extend(&mut self, other: PropertyMap) {
        self.props.extend(other.props);
    }

    pub fn contains(&self, key: &str, value: &PropertyValue) -> bool {
        self.props.get(key) == Some(value)
    }
}

impl<K: Into<String>, V: Into<PropertyValue>> Extend<(K, V)> for PropertyMap {
    fn extend<T: IntoIterator<Item = (K, V)>>(&mut self, iter: T) {
        for (k, v) in iter {
            self.insert(k, v);
        }
    }
}

impl IntoIterator for PropertyMap {
    type Item = (String, PropertyValue);
    type IntoIter = std::collections::hash_map::IntoIter<String, PropertyValue>;

    fn into_iter(self) -> Self::IntoIter {
        self.props.into_iter()
    }
}

/// Builder for PropertyMap
pub struct PropertyMapBuilder {
    props: PropertyMap,
}

impl PropertyMapBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        PropertyMapBuilder {
            props: PropertyMap::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        PropertyMapBuilder {
            props: PropertyMap::with_capacity(capacity),
        }
    }

    pub fn insert<K: Into<String>, V: Into<PropertyValue>>(mut self, key: K, value: V) -> Self {
        self.props.insert(key, value);
        self
    }

    pub fn build(self) -> PropertyMap {
        self.props
    }
}

impl Default for PropertyMapBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_property_value() {
        let v = PropertyValue::from("hello");
        assert_eq!(v.as_string(), Some(&"hello".to_string()));

        let v = PropertyValue::from(42i64);
        assert_eq!(v.as_int(), Some(42));

        let v = PropertyValue::from(3.14f64);
        assert_eq!(v.as_float(), Some(3.14));

        let v = PropertyValue::from(true);
        assert_eq!(v.as_bool(), Some(true));
    }

    #[test]
    fn test_property_map_basics() {
        let mut props = PropertyMap::new();
        props.insert("name", "batch-001");
        props.insert("quantity", 100i64);
        props.insert("active", true);

        assert_eq!(props.len(), 3);
        assert!(props.contains_key("name"));
        assert!(!props.contains_key("missing"));
        assert_eq!(props.get("quantity").unwrap().as_int(), Some(100));
    }

    #[test]
    fn test_property_map_builder() {
        let props = PropertyMapBuilder::new()
            .insert("name", "device-001")
            .insert("model", "GMP-2000")
            .insert("year", 2024i64)
            .build();

        assert_eq!(props.len(), 3);
        assert_eq!(
            props.get("name").unwrap().as_string(),
            Some(&"device-001".to_string())
        );
    }

    #[test]
    fn test_property_map_iter() {
        let mut props = PropertyMap::new();
        props.insert("a", 1i64);
        props.insert("b", 2i64);

        let keys: Vec<_> = props.keys().collect();
        assert!(keys.contains(&&"a".to_string()));
        assert!(keys.contains(&&"b".to_string()));
    }

    #[test]
    fn test_property_map_extend() {
        let mut props1 = PropertyMap::new();
        props1.insert("a", 1i64);

        let mut props2 = PropertyMap::new();
        props2.insert("b", 2i64);
        props2.insert("c", 3i64);

        props1.extend(props2);
        assert_eq!(props1.len(), 3);
        assert!(props1.contains_key("b"));
    }
}
