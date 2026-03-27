//! SBTest schema for sysbench-compatible benchmarks

/// SBTest schema definition
pub struct SbtestSchema;

impl SbtestSchema {
    pub fn new() -> Self {
        Self
    }

    pub fn table_name() -> &'static str {
        "sbtest"
    }
}
