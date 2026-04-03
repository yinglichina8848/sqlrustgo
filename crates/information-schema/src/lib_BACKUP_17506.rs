//! INFORMATION_SCHEMA Implementation
//!
//! Provides standard SQL INFORMATION_SCHEMA views for metadata access.
//! This implementation is fully integrated with the Catalog system.

use serde::{Deserialize, Serialize};
use sqlrustgo_catalog::{Catalog, DataType};

/// Row representing a schema in information_schema.schemata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaRow {
    pub schema_name: String,
    pub schema_owner: String,
}

/// Row representing a table in information_schema.tables
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableRow {
    pub table_schema: String,
    pub table_name: String,
    pub table_type: String,
    pub is_insertable_into: String,
}

/// Row representing a column in information_schema.columns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnRow {
    pub table_schema: String,
    pub table_name: String,
    pub column_name: String,
    pub ordinal_position: i32,
    pub column_default: Option<String>,
    pub is_nullable: String,
    pub data_type: String,
    pub character_maximum_length: Option<i32>,
    pub numeric_precision: Option<i32>,
    pub numeric_scale: Option<i32>,
}

/// Row representing an index in information_schema.indexes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexRow {
    pub table_schema: String,
    pub table_name: String,
    pub index_name: String,
    pub column_name: String,
    pub ordinal_position: i32,
    pub is_unique: bool,
    pub is_primary: bool,
}

/// INFORMATION_SCHEMA providing standard SQL metadata views
pub struct InformationSchema<'a> {
    catalog: &'a Catalog,
}

impl<'a> InformationSchema<'a> {
    /// Create a new InformationSchema backed by the given Catalog
    pub fn new(catalog: &'a Catalog) -> Self {
        Self { catalog }
    }

    /// Get all schemata from the catalog
    pub fn get_schemata(&self) -> Vec<SchemaRow> {
        self.catalog
            .all_schemas()
            .iter()
            .map(|schema| SchemaRow {
                schema_name: schema.name.clone(),
                schema_owner: "owner".to_string(), // Default owner since Catalog doesn't track owners
            })
            .collect()
    }

    /// Get all tables from all schemas in the catalog
    pub fn get_tables(&self) -> Vec<TableRow> {
        let mut rows = Vec::new();

        for schema in self.catalog.all_schemas() {
            for table in schema.tables() {
                let table_type = "BASE TABLE".to_string();

                // Tables are generally insertable
                let is_insertable_into = "YES".to_string();

                rows.push(TableRow {
                    table_schema: schema.name.clone(),
                    table_name: table.name.clone(),
                    table_type,
                    is_insertable_into,
                });
            }
        }

        rows
    }

    /// Get all columns from all tables in all schemas
    pub fn get_columns(&self) -> Vec<ColumnRow> {
        let mut rows = Vec::new();

        for schema in self.catalog.all_schemas() {
            for table in schema.tables() {
                for (i, column) in table.columns.iter().enumerate() {
                    let (character_maximum_length, numeric_precision, numeric_scale) =
                        Self::get_type_attributes(&column.data_type);

                    rows.push(ColumnRow {
                        table_schema: schema.name.clone(),
                        table_name: table.name.clone(),
                        column_name: column.name.clone(),
                        ordinal_position: (i + 1) as i32,
                        column_default: column.default_value.as_ref().map(|v| format!("{}", v)),
                        is_nullable: if column.nullable {
                            "YES".to_string()
                        } else {
                            "NO".to_string()
                        },
                        data_type: column.data_type.sql_name().to_string(),
                        character_maximum_length,
                        numeric_precision,
                        numeric_scale,
                    });
                }
            }
        }

        rows
    }

    /// Get columns for a specific table
    pub fn get_columns_for_table(&self, table_name: &str) -> Vec<ColumnRow> {
        self.get_columns()
            .into_iter()
            .filter(|col| col.table_name == table_name)
            .collect()
    }

    /// Get all indexes from all tables in all schemas
    pub fn get_indexes(&self) -> Vec<IndexRow> {
        let mut rows = Vec::new();

        for schema in self.catalog.all_schemas() {
            for table in schema.tables() {
                for index in &table.indices {
                    for (i, column_name) in index.columns.iter().enumerate() {
                        rows.push(IndexRow {
                            table_schema: schema.name.clone(),
                            table_name: table.name.clone(),
                            index_name: index.name.clone(),
                            column_name: column_name.clone(),
                            ordinal_position: (i + 1) as i32,
                            is_unique: index.is_unique,
                            is_primary: index.is_primary_key,
                        });
                    }
                }
            }
        }

        rows
    }

    /// Get type-specific attributes for columns
    fn get_type_attributes(data_type: &DataType) -> (Option<i32>, Option<i32>, Option<i32>) {
        match data_type {
            DataType::Text => (Some(65535), None, None), // TEXT max length
            DataType::Integer => (None, Some(64), Some(0)),
            DataType::Float => (None, Some(53), Some(0)),
            DataType::Boolean => (None, None, None),
            DataType::Blob => (None, None, None),
            DataType::Date => (None, None, None),
            DataType::Timestamp => (None, None, None),
            DataType::Null => (None, None, None),
<<<<<<< HEAD
            DataType::Uuid => (None, Some(128), None),
=======
            DataType::Uuid => (None, None, None),
>>>>>>> 735bce1c8e83e51d1ee89082270e35ce4c4bcca3
            DataType::Array => (None, None, None),
            DataType::Enum => (None, None, None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlrustgo_catalog::{ColumnDefinition, DataType, IndexInfo, Schema, Table};

    fn create_test_catalog() -> Catalog {
        let mut catalog = Catalog::new();

        // Add a schema
        let schema = Schema::new("test_schema");

        // Create users table
        let users_table = Table::new(
            "users",
            vec![
                ColumnDefinition::new("id", DataType::Integer).not_null(),
                ColumnDefinition::new("name", DataType::Text).not_null(),
                ColumnDefinition::new("email", DataType::Text),
                ColumnDefinition::new("created_at", DataType::Timestamp),
            ],
        )
        .primary_key(vec!["id".to_string()])
        .unwrap()
        .add_index(IndexInfo::new("idx_users_email", "users", vec!["email".to_string()]).unique());

        // Create orders table
        let orders_table = Table::new(
            "orders",
            vec![
                ColumnDefinition::new("id", DataType::Integer).not_null(),
                ColumnDefinition::new("user_id", DataType::Integer).not_null(),
                ColumnDefinition::new("total", DataType::Float),
            ],
        )
        .primary_key(vec!["id".to_string()])
        .unwrap()
        .add_foreign_key(sqlrustgo_catalog::ForeignKeyRef {
            referenced_schema: "public".to_string(),
            referenced_table: "users".to_string(),
            referenced_columns: vec!["id".to_string()],
            columns: vec!["user_id".to_string()],
            on_delete: Some(sqlrustgo_catalog::ForeignKeyAction::Cascade),
            on_update: Some(sqlrustgo_catalog::ForeignKeyAction::Cascade),
        });

        let schema = schema
            .add_table(users_table)
            .unwrap()
            .add_table(orders_table)
            .unwrap();

        catalog.add_schema(schema).unwrap();
        catalog
    }

    #[test]
    fn test_schemata_returns_all_schemas() {
        let catalog = create_test_catalog();
        let info_schema = InformationSchema::new(&catalog);

        let schemata = info_schema.get_schemata();

        // Should have 'public' (default) and 'test_schema'
        assert!(schemata.len() >= 2);
        assert!(schemata.iter().any(|s| s.schema_name == "public"));
        assert!(schemata.iter().any(|s| s.schema_name == "test_schema"));
    }

    #[test]
    fn test_tables_returns_all_tables() {
        let catalog = create_test_catalog();
        let info_schema = InformationSchema::new(&catalog);

        let tables = info_schema.get_tables();

        // Should have users and orders tables from test_schema
        let table_names: Vec<&str> = tables.iter().map(|t| t.table_name.as_str()).collect();
        assert!(table_names.contains(&"users"));
        assert!(table_names.contains(&"orders"));

        // Check that tables have correct schema
        let users_table = tables.iter().find(|t| t.table_name == "users").unwrap();
        assert_eq!(users_table.table_schema, "test_schema");
        assert_eq!(users_table.table_type, "BASE TABLE");
    }

    #[test]
    fn test_columns_returns_all_columns() {
        let catalog = create_test_catalog();
        let info_schema = InformationSchema::new(&catalog);

        let columns = info_schema.get_columns();

        // Find users table columns
        let users_columns: Vec<_> = columns.iter().filter(|c| c.table_name == "users").collect();

        assert_eq!(users_columns.len(), 4); // id, name, email, created_at

        // Check ordinal positions
        assert_eq!(users_columns[0].column_name, "id");
        assert_eq!(users_columns[0].ordinal_position, 1);
        assert_eq!(users_columns[1].column_name, "name");
        assert_eq!(users_columns[1].ordinal_position, 2);
    }

    #[test]
    fn test_columns_for_specific_table() {
        let catalog = create_test_catalog();
        let info_schema = InformationSchema::new(&catalog);

        let users_columns = info_schema.get_columns_for_table("users");

        assert_eq!(users_columns.len(), 4);
        assert!(users_columns.iter().all(|c| c.table_name == "users"));
    }

    #[test]
    fn test_column_nullable() {
        let catalog = create_test_catalog();
        let info_schema = InformationSchema::new(&catalog);

        let users_columns: Vec<_> = info_schema
            .get_columns()
            .into_iter()
            .filter(|c| c.table_name == "users")
            .collect();

        // id and name are NOT NULL
        let id_col = users_columns
            .iter()
            .find(|c| c.column_name == "id")
            .unwrap();
        assert_eq!(id_col.is_nullable, "NO");

        // email is nullable
        let email_col = users_columns
            .iter()
            .find(|c| c.column_name == "email")
            .unwrap();
        assert_eq!(email_col.is_nullable, "YES");
    }

    #[test]
    fn test_indexes_returns_all_indexes() {
        let catalog = create_test_catalog();
        let info_schema = InformationSchema::new(&catalog);

        let indexes = info_schema.get_indexes();

        // Find pk_users index
        let pk_users = indexes.iter().find(|i| i.index_name == "pk_users");
        assert!(pk_users.is_some());
        let pk_users = pk_users.unwrap();
        assert!(pk_users.is_primary);
        assert!(pk_users.is_unique);

        // Find idx_users_email index
        let idx_email = indexes.iter().find(|i| i.index_name == "idx_users_email");
        assert!(idx_email.is_some());
        let idx_email = idx_email.unwrap();
        assert!(!idx_email.is_primary);
        assert!(idx_email.is_unique);
    }

    #[test]
    fn test_empty_catalog() {
        let catalog = Catalog::new();
        let info_schema = InformationSchema::new(&catalog);

        // Should still have public schema
        let schemata = info_schema.get_schemata();
        assert_eq!(schemata.len(), 1);
        assert_eq!(schemata[0].schema_name, "public");

        // No tables or columns
        assert!(info_schema.get_tables().is_empty());
        assert!(info_schema.get_columns().is_empty());
        assert!(info_schema.get_indexes().is_empty());
    }
}
