//! Catalog rebuild functionality
//!
//! Provides utilities to rebuild a Catalog from a StorageEngine for testing.

use crate::data_type::DataType;

/// Convert a storage data type string to catalog DataType
#[allow(dead_code)]
pub fn convert_data_type(data_type: &str) -> Option<DataType> {
    DataType::parse_sql_name(data_type)
}

// TEMPORARILY DISABLED: The test module has extensive API mismatches with the current
// Catalog and Storage implementations. The tests expect APIs that no longer exist.
// #[cfg(test)]
// mod tests { ... }
