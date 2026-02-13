pub fn greet() -> String {
    "SQLRustGo Database System".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greet() {
        assert_eq!(greet(), "SQLRustGo Database System");
    }
}
