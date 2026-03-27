//! View Tests
//!
//! P2 tests for view operations per TEST_PLAN.md
//! Tests CREATE VIEW, query view, and view metadata

#[cfg(test)]
mod tests {
    use sqlrustgo_storage::engine::{StorageEngine, TableInfo, ViewInfo};
    use sqlrustgo_storage::MemoryStorage;
    use sqlrustgo_types::Value;

    #[test]
    fn test_create_and_get_view() {
        let mut storage = MemoryStorage::new();

        let base_table = TableInfo {
            name: "users".to_string(),
            columns: vec![
                sqlrustgo_storage::ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: true,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                },
                sqlrustgo_storage::ColumnDefinition {
                    name: "name".to_string(),
                    data_type: "VARCHAR".to_string(),
                    nullable: false,
                    is_unique: false,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                },
            ],
        };

        storage.create_table(&base_table).unwrap();
        storage
            .insert(
                "users",
                vec![vec![Value::Integer(1), Value::Text("Alice".to_string())]],
            )
            .ok();
        storage
            .insert(
                "users",
                vec![vec![Value::Integer(2), Value::Text("Bob".to_string())]],
            )
            .ok();

        let view_info = ViewInfo {
            name: "user_view".to_string(),
            query: "SELECT id, name FROM users".to_string(),
            schema: base_table.clone(),
            records: vec![],
        };

        storage.create_view(view_info).unwrap();

        let retrieved = storage.get_view("user_view");

        assert!(retrieved.is_some(), "View should be retrieved");
    }

    #[test]
    fn test_has_view() {
        let mut storage = MemoryStorage::new();

        let base_table = TableInfo {
            name: "test".to_string(),
            columns: vec![],
        };

        storage.create_table(&base_table).ok();

        let view_info = ViewInfo {
            name: "my_view".to_string(),
            query: "SELECT * FROM test".to_string(),
            schema: base_table,
            records: vec![],
        };

        storage.create_view(view_info).ok();

        assert!(
            storage.has_view("my_view"),
            "has_view should return true for existing view"
        );
        assert!(
            !storage.has_view("nonexistent"),
            "has_view should return false for non-existing view"
        );
    }

    #[test]
    fn test_list_views() {
        let mut storage = MemoryStorage::new();

        let base_table = TableInfo {
            name: "t".to_string(),
            columns: vec![],
        };

        storage.create_table(&base_table).ok();

        let view1 = ViewInfo {
            name: "view1".to_string(),
            query: "SELECT 1".to_string(),
            schema: base_table.clone(),
            records: vec![],
        };

        let view2 = ViewInfo {
            name: "view2".to_string(),
            query: "SELECT 2".to_string(),
            schema: base_table.clone(),
            records: vec![],
        };

        storage.create_view(view1).ok();
        storage.create_view(view2).ok();

        let views = storage.list_views();

        assert_eq!(views.len(), 2);
        assert!(views.contains(&"view1".to_string()));
        assert!(views.contains(&"view2".to_string()));
    }

    #[test]
    fn test_view_persists_after_operations() {
        let mut storage = MemoryStorage::new();

        let base_table = TableInfo {
            name: "data".to_string(),
            columns: vec![sqlrustgo_storage::ColumnDefinition {
                name: "id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: true,
                is_primary_key: false,
                auto_increment: false,
                references: None,
            }],
        };

        storage.create_table(&base_table).ok();

        let view_info = ViewInfo {
            name: "data_view".to_string(),
            query: "SELECT * FROM data".to_string(),
            schema: base_table.clone(),
            records: vec![],
        };

        storage.create_view(view_info).ok();

        storage.insert("data", vec![vec![Value::Integer(1)]]).ok();
        storage.insert("data", vec![vec![Value::Integer(2)]]).ok();

        assert!(
            storage.has_view("data_view"),
            "View should persist after insert"
        );
    }

    #[test]
    fn test_view_query_storage() {
        let mut storage = MemoryStorage::new();

        let base_table = TableInfo {
            name: "orders".to_string(),
            columns: vec![
                sqlrustgo_storage::ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: true,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                },
                sqlrustgo_storage::ColumnDefinition {
                    name: "total".to_string(),
                    data_type: "REAL".to_string(),
                    nullable: false,
                    is_unique: false,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                },
            ],
        };

        storage.create_table(&base_table).ok();
        storage
            .insert("orders", vec![vec![Value::Integer(1), Value::Float(100.0)]])
            .ok();
        storage
            .insert("orders", vec![vec![Value::Integer(2), Value::Float(200.0)]])
            .ok();

        let view_info = ViewInfo {
            name: "expensive_orders".to_string(),
            query: "SELECT * FROM orders WHERE total > 150".to_string(),
            schema: base_table,
            records: vec![vec![Value::Integer(2), Value::Float(200.0)]],
        };

        storage.create_view(view_info).ok();

        let retrieved = storage.get_view("expensive_orders").unwrap();

        assert_eq!(retrieved.query, "SELECT * FROM orders WHERE total > 150");
    }

    #[test]
    fn test_multiple_views_same_base_table() {
        let mut storage = MemoryStorage::new();

        let base_table = TableInfo {
            name: "products".to_string(),
            columns: vec![
                sqlrustgo_storage::ColumnDefinition {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    is_unique: true,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                },
                sqlrustgo_storage::ColumnDefinition {
                    name: "category".to_string(),
                    data_type: "VARCHAR".to_string(),
                    nullable: false,
                    is_unique: false,
                    is_primary_key: false,
                    auto_increment: false,
                    references: None,
                },
            ],
        };

        storage.create_table(&base_table).ok();

        let view1 = ViewInfo {
            name: "electronics".to_string(),
            query: "SELECT * FROM products WHERE category = 'electronics'".to_string(),
            schema: base_table.clone(),
            records: vec![],
        };

        let view2 = ViewInfo {
            name: "clothing".to_string(),
            query: "SELECT * FROM products WHERE category = 'clothing'".to_string(),
            schema: base_table.clone(),
            records: vec![],
        };

        storage.create_view(view1).ok();
        storage.create_view(view2).ok();

        let views = storage.list_views();

        assert_eq!(views.len(), 2);
    }
}
