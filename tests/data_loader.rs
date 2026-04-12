use serde_json::Value;
use std::path::Path;

pub struct TestDataLoader;

impl TestDataLoader {
    fn get_data_path(file: &str) -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("data")
            .join(file)
    }

    pub fn load_json(file: &str) -> Value {
        let path = Self::get_data_path(file);
        let content = std::fs::read_to_string(path)
            .expect(&format!("Failed to read test data file: {}", file));
        serde_json::from_str(&content).expect(&format!("Failed to parse test data: {}", file))
    }

    pub fn load_string(file: &str) -> String {
        let path = Self::get_data_path(file);
        std::fs::read_to_string(path).expect(&format!("Failed to read test data file: {}", file))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_json() {
        let data = TestDataLoader::load_json("t.json");
        assert!(data.is_object());
    }
}
