//! SBTest schema for sysbench-compatible benchmarks

/// SBTest table schema definition (sysbench-compatible)
pub const SBTEST_SCHEMA: &str = r#"
CREATE TABLE sbtest (
    id BIGINT PRIMARY KEY,
    k BIGINT NOT NULL,
    c TEXT NOT NULL,
    pad TEXT NOT NULL
)
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sbtest_schema() {
        // Verify schema contains required columns
        assert!(SBTEST_SCHEMA.contains("id BIGINT PRIMARY KEY"));
        assert!(SBTEST_SCHEMA.contains("k BIGINT NOT NULL"));
        assert!(SBTEST_SCHEMA.contains("c TEXT NOT NULL"));
        assert!(SBTEST_SCHEMA.contains("pad TEXT NOT NULL"));
    }
}
