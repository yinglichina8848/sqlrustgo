//! Test Harness

pub struct TestHarness {
    name: String,
}

impl TestHarness {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

pub struct TestFixture {
    data: std::collections::HashMap<String, Vec<Vec<sqlrustgo_types::Value>>>,
}

impl TestFixture {
    pub fn new() -> Self {
        Self {
            data: std::collections::HashMap::new(),
        }
    }

    pub fn add_table(mut self, name: &str, rows: Vec<Vec<sqlrustgo_types::Value>>) -> Self {
        self.data.insert(name.to_string(), rows);
        self
    }

    pub fn get_table(&self, name: &str) -> Option<&Vec<Vec<sqlrustgo_types::Value>>> {
        self.data.get(name)
    }

    pub fn table_names(&self) -> Vec<&String> {
        self.data.keys().collect()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl Default for TestFixture {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_harness_new() {
        let harness = TestHarness::new("test");
        assert_eq!(harness.name(), "test");
    }

    #[test]
    fn test_test_fixture_new() {
        let fixture = TestFixture::new();
        assert!(fixture.is_empty());
    }

    #[test]
    fn test_test_fixture_add_table() {
        let fixture =
            TestFixture::new().add_table("users", vec![vec![sqlrustgo_types::Value::Integer(1)]]);
        assert!(fixture.get_table("users").is_some());
    }

    #[test]
    fn test_test_fixture_table_names() {
        let fixture = TestFixture::new()
            .add_table("users", vec![])
            .add_table("orders", vec![]);
        let names = fixture.table_names();
        assert_eq!(names.len(), 2);
    }
}
