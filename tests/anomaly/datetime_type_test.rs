//! DATE/TIMESTAMP Type Tests
//!
//! P3 tests for DATE and TIMESTAMP types per TEST_PLAN.md
//! Tests date/time operations and type handling

#[cfg(test)]
mod tests {
    use sqlrustgo_types::Value;

    #[test]
    fn test_date_value_creation() {
        let date = Value::Date(19770);

        assert!(matches!(date, Value::Date(_)));
    }

    #[test]
    fn test_timestamp_value_creation() {
        let ts = Value::Timestamp(1704067200000000);

        assert!(matches!(ts, Value::Timestamp(_)));
    }

    #[test]
    fn test_date_equality() {
        let date1 = Value::Date(19770);
        let date2 = Value::Date(19770);
        let date3 = Value::Date(19771);

        assert_eq!(date1, date2);
        assert_ne!(date1, date3);
    }

    #[test]
    fn test_timestamp_equality() {
        let ts1 = Value::Timestamp(1000000);
        let ts2 = Value::Timestamp(1000000);
        let ts3 = Value::Timestamp(2000000);

        assert_eq!(ts1, ts2);
        assert_ne!(ts1, ts3);
    }

    #[test]
    fn test_null_date() {
        let null_date: Value = Value::Null;

        assert!(matches!(null_date, Value::Null));
    }

    #[test]
    fn test_timestamp_with_microseconds() {
        let ts = Value::Timestamp(1704067200000000);

        if let Value::Timestamp(us) = ts {
            assert_eq!(us, 1704067200000000);
        } else {
            panic!("Expected Timestamp");
        }
    }

    #[test]
    fn test_date_display() {
        let date = Value::Date(19770);
        let debug_str = format!("{:?}", date);

        assert!(debug_str.contains("Date"));
    }

    #[test]
    fn test_timestamp_display() {
        let ts = Value::Timestamp(1704067200000000);
        let debug_str = format!("{:?}", ts);

        assert!(debug_str.contains("Timestamp"));
    }

    #[test]
    fn test_date_in_storage() {
        use sqlrustgo_storage::engine::{ColumnDefinition, TableInfo};
        use sqlrustgo_storage::{MemoryStorage, StorageEngine};

        let mut storage = MemoryStorage::new();

        let info = TableInfo {
            name: "events".to_string(),
            columns: vec![
                ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: true,,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                },
                ColumnDefinition {
                    name: "event_date".to_string(),
                    data_type: "DATE".to_string(),
                    nullable: true,
                    is_unique: false,,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                },
            ],
        };

        storage.create_table(&info).ok();

        assert!(storage.has_table("events"));
    }

    #[test]
    fn test_timestamp_in_storage() {
        use sqlrustgo_storage::engine::{ColumnDefinition, TableInfo};
        use sqlrustgo_storage::{MemoryStorage, StorageEngine};

        let mut storage = MemoryStorage::new();

        let info = TableInfo {
            name: "logs".to_string(),
            columns: vec![
                ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: true,,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                },
                ColumnDefinition {
                    name: "created_at".to_string(),
                    data_type: "TIMESTAMP".to_string(),
                    nullable: true,
                    is_unique: false,,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                },
            ],
        };

        storage.create_table(&info).ok();

        assert!(storage.has_table("logs"));
    }
}
