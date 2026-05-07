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

/// Row representing a trigger in information_schema.triggers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerRow {
    pub trigger_name: String,
    pub trigger_schema: String,
    pub event_manipulation: String,
    pub action_order: i32,
    pub action_condition: Option<String>,
    pub action_statement: String,
    pub orientid: String,
    pub schema_uid: String,
    pub body_catalog: String,
    pub schema_catalog: String,
}

/// Row representing a routine in information_schema.routines
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutineRow {
    pub specific_name: String,
    pub routine_schema: String,
    pub routine_name: String,
    pub routine_type: String,
    pub data_type: String,
    pub routine_definition: Option<String>,
    pub external_name: Option<String>,
    pub external_language: String,
}

/// Row representing a parameter in information_schema.parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterRow {
    pub specific_schema: String,
    pub specific_name: String,
    pub ordinal_position: i32,
    pub parameter_name: Option<String>,
    pub data_type: String,
    pub parameter_mode: Option<String>,
}

/// Row representing user privileges in information_schema.user_privileges
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPrivilegeRow {
    pub grantee: String,
    pub table_catalog: String,
    pub privilege_type: String,
    pub is_grantable: String,
}

/// Row representing schema privileges in information_schema.schema_privileges
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaPrivilegeRow {
    pub grantee: String,
    pub table_catalog: String,
    pub schema_name: String,
    pub privilege_type: String,
    pub is_grantable: String,
}

/// Row representing table privileges in information_schema.table_privileges
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TablePrivilegeRow {
    pub grantee: String,
    pub table_catalog: String,
    pub table_schema: String,
    pub table_name: String,
    pub privilege_type: String,
    pub is_grantable: String,
}

/// Row representing column privileges in information_schema.column_privileges
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnPrivilegeRow {
    pub grantee: String,
    pub table_catalog: String,
    pub table_schema: String,
    pub table_name: String,
    pub column_name: String,
    pub privilege_type: String,
    pub is_grantable: String,
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
            .schemas()
            .values()
            .map(|schema| SchemaRow {
                schema_name: schema.name.clone(),
                schema_owner: "owner".to_string(),
            })
            .collect()
    }

    /// Get all tables from all schemas in the catalog
    pub fn get_tables(&self) -> Vec<TableRow> {
        let mut rows = Vec::new();

        for schema in self.catalog.schemas().values() {
            for table_ref in schema.tables() {
                let table_type = "BASE TABLE".to_string();
                let is_insertable_into = "YES".to_string();

                rows.push(TableRow {
                    table_schema: schema.name.clone(),
                    table_name: table_ref.name.clone(),
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

        for schema in self.catalog.schemas().values() {
            for table_ref in schema.tables() {
                for (i, column) in table_ref.columns.iter().enumerate() {
                    let (character_maximum_length, numeric_precision, numeric_scale) =
                        Self::get_type_attributes(&column.data_type);

                    rows.push(ColumnRow {
                        table_schema: schema.name.clone(),
                        table_name: table_ref.name.clone(),
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

        for schema in self.catalog.schemas().values() {
            for table_ref in schema.tables() {
                for index in &table_ref.indices {
                    for (i, column_name) in index.columns.iter().enumerate() {
                        rows.push(IndexRow {
                            table_schema: schema.name.clone(),
                            table_name: table_ref.name.clone(),
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

    fn get_type_attributes(data_type: &DataType) -> (Option<i32>, Option<i32>, Option<i32>) {
        match data_type {
            DataType::Text => (Some(65535), None, None),
            DataType::Integer => (None, Some(64), Some(0)),
            DataType::Float => (None, Some(53), Some(0)),
            DataType::Boolean => (None, None, None),
            DataType::Blob => (None, None, None),
            DataType::Date => (None, None, None),
            DataType::Timestamp => (None, None, None),
            DataType::Null => (None, None, None),
            DataType::Uuid => (None, Some(128), None),
            DataType::Array => (None, None, None),
            DataType::Enum => (None, None, None),
        }
    }

    pub fn get_triggers(&self) -> Vec<TriggerRow> {
        // Triggers not yet implemented in catalog
        Vec::new()
    }

    pub fn get_routines(&self) -> Vec<RoutineRow> {
        self.catalog
            .stored_procedures()
            .iter()
            .map(|proc| {
                let data_type = proc
                    .params
                    .iter()
                    .find(|p| matches!(p.mode, sqlrustgo_catalog::stored_proc::ParamMode::Out))
                    .map(|p| p.data_type.clone())
                    .unwrap_or_default();

                RoutineRow {
                    specific_name: proc.name.clone(),
                    routine_schema: "public".to_string(),
                    routine_name: proc.name.clone(),
                    routine_type: "PROCEDURE".to_string(),
                    data_type,
                    routine_definition: Some(format!("{:?}", proc.body)),
                    external_name: None,
                    external_language: "SQL".to_string(),
                }
            })
            .collect()
    }

    pub fn get_parameters(&self) -> Vec<ParameterRow> {
        let mut rows = Vec::new();

        for proc in self.catalog.stored_procedures() {
            for (i, param) in proc.params.iter().enumerate() {
                let parameter_mode = match param.mode {
                    sqlrustgo_catalog::stored_proc::ParamMode::In => "IN",
                    sqlrustgo_catalog::stored_proc::ParamMode::Out => "OUT",
                    sqlrustgo_catalog::stored_proc::ParamMode::InOut => "INOUT",
                };

                rows.push(ParameterRow {
                    specific_schema: "public".to_string(),
                    specific_name: proc.name.clone(),
                    ordinal_position: (i + 1) as i32,
                    parameter_name: Some(param.name.clone()),
                    data_type: param.data_type.clone(),
                    parameter_mode: Some(parameter_mode.to_string()),
                });
            }
        }

        rows
    }

    pub fn get_user_privileges(&self) -> Vec<UserPrivilegeRow> {
        let mut rows = Vec::new();

        for (identity, grants) in self.catalog.auth_manager().all_privileges() {
            for grant in grants {
                rows.push(UserPrivilegeRow {
                    grantee: format!("{}@{}", identity.username, identity.host),
                    table_catalog: "def".to_string(),
                    privilege_type: grant.privilege.to_string(),
                    is_grantable: if grant.grant_option {
                        "YES".to_string()
                    } else {
                        "NO".to_string()
                    },
                });
            }
        }

        rows
    }

    pub fn get_schema_privileges(&self) -> Vec<SchemaPrivilegeRow> {
        let mut rows = Vec::new();

        for (identity, grants) in self.catalog.auth_manager().all_privileges() {
            for grant in grants {
                if matches!(
                    grant.object_type,
                    sqlrustgo_catalog::auth::ObjectType::Database
                ) {
                    rows.push(SchemaPrivilegeRow {
                        grantee: format!("{}@{}", identity.username, identity.host),
                        table_catalog: "def".to_string(),
                        schema_name: grant.object_name.clone(),
                        privilege_type: grant.privilege.to_string(),
                        is_grantable: if grant.grant_option {
                            "YES".to_string()
                        } else {
                            "NO".to_string()
                        },
                    });
                }
            }
        }

        rows
    }

    pub fn get_table_privileges(&self) -> Vec<TablePrivilegeRow> {
        let mut rows = Vec::new();

        for (identity, grants) in self.catalog.auth_manager().all_privileges() {
            for grant in grants {
                if matches!(
                    grant.object_type,
                    sqlrustgo_catalog::auth::ObjectType::Table
                ) {
                    rows.push(TablePrivilegeRow {
                        grantee: format!("{}@{}", identity.username, identity.host),
                        table_catalog: "def".to_string(),
                        table_schema: "public".to_string(),
                        table_name: grant.object_name.clone(),
                        privilege_type: grant.privilege.to_string(),
                        is_grantable: if grant.grant_option {
                            "YES".to_string()
                        } else {
                            "NO".to_string()
                        },
                    });
                }
            }
        }

        rows
    }

    pub fn get_column_privileges(&self) -> Vec<ColumnPrivilegeRow> {
        let mut rows = Vec::new();

        for (identity, grants) in self.catalog.auth_manager().all_privileges() {
            for grant in grants {
                if matches!(
                    grant.object_type,
                    sqlrustgo_catalog::auth::ObjectType::Column
                ) {
                    if let Some(ref column_name) = grant.column_name {
                        rows.push(ColumnPrivilegeRow {
                            grantee: format!("{}@{}", identity.username, identity.host),
                            table_catalog: "def".to_string(),
                            table_schema: "public".to_string(),
                            table_name: grant.object_name.clone(),
                            column_name: column_name.clone(),
                            privilege_type: grant.privilege.to_string(),
                            is_grantable: if grant.grant_option {
                                "YES".to_string()
                            } else {
                                "NO".to_string()
                            },
                        });
                    }
                }
            }
        }

        rows
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlrustgo_catalog::{index::IndexInfo, schema::Schema, ColumnDefinition, DataType, Table};

    fn create_test_catalog() -> Catalog {
        let mut catalog = Catalog::new("test_catalog");

        let public_schema = Schema::new("public");
        catalog.add_schema(public_schema).unwrap();

        let schema = Schema::new("test_schema");

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

        assert!(schemata.len() >= 2);
        assert!(schemata.iter().any(|s| s.schema_name == "public"));
        assert!(schemata.iter().any(|s| s.schema_name == "test_schema"));
    }

    #[test]
    fn test_tables_returns_all_tables() {
        let catalog = create_test_catalog();
        let info_schema = InformationSchema::new(&catalog);

        let tables = info_schema.get_tables();

        let table_names: Vec<&str> = tables.iter().map(|t| t.table_name.as_str()).collect();
        assert!(table_names.contains(&"users"));
        assert!(table_names.contains(&"orders"));

        let users_table = tables.iter().find(|t| t.table_name == "users").unwrap();
        assert_eq!(users_table.table_schema, "test_schema");
        assert_eq!(users_table.table_type, "BASE TABLE");
    }

    #[test]
    fn test_columns_returns_all_columns() {
        let catalog = create_test_catalog();
        let info_schema = InformationSchema::new(&catalog);

        let columns = info_schema.get_columns();

        let users_columns: Vec<_> = columns.iter().filter(|c| c.table_name == "users").collect();

        assert_eq!(users_columns.len(), 4);

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

        let id_col = users_columns
            .iter()
            .find(|c| c.column_name == "id")
            .unwrap();
        assert_eq!(id_col.is_nullable, "NO");

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

        let pk_users = indexes.iter().find(|i| i.index_name == "pk_users");
        assert!(pk_users.is_some());
        let pk_users = pk_users.unwrap();
        assert!(pk_users.is_primary);
        assert!(pk_users.is_unique);

        let idx_email = indexes.iter().find(|i| i.index_name == "idx_users_email");
        assert!(idx_email.is_some());
        let idx_email = idx_email.unwrap();
        assert!(!idx_email.is_primary);
        assert!(idx_email.is_unique);
    }

    #[test]
    fn test_empty_catalog() {
        let catalog = Catalog::new("test");
        let info_schema = InformationSchema::new(&catalog);

        let schemata = info_schema.get_schemata();
        assert_eq!(schemata.len(), 0);

        assert!(info_schema.get_tables().is_empty());
        assert!(info_schema.get_columns().is_empty());
        assert!(info_schema.get_indexes().is_empty());
    }

    #[test]
    fn test_triggers_returns_empty() {
        let catalog = create_test_catalog();
        let info_schema = InformationSchema::new(&catalog);

        // Triggers not yet implemented - should return empty
        let triggers = info_schema.get_triggers();
        assert!(triggers.is_empty());
    }

    #[test]
    fn test_routines_with_stored_procedure() {
        let mut catalog = create_test_catalog();

        // Add a stored procedure
        let proc = sqlrustgo_catalog::StoredProcedure::new(
            "test_proc".to_string(),
            vec![sqlrustgo_catalog::StoredProcParam {
                name: "param1".to_string(),
                mode: sqlrustgo_catalog::ParamMode::In,
                data_type: "INTEGER".to_string(),
            }],
            vec![],
        );
        catalog.add_stored_procedure(proc).unwrap();

        let info_schema = InformationSchema::new(&catalog);
        let routines = info_schema.get_routines();

        assert!(!routines.is_empty());
        let proc_routine = routines
            .iter()
            .find(|r| r.routine_name == "test_proc")
            .unwrap();
        assert_eq!(proc_routine.routine_type, "PROCEDURE");
    }

    #[test]
    fn test_parameters_with_stored_procedure() {
        let mut catalog = create_test_catalog();

        // Add a stored procedure with parameters
        let proc = sqlrustgo_catalog::StoredProcedure::new(
            "test_proc".to_string(),
            vec![
                sqlrustgo_catalog::StoredProcParam {
                    name: "p1".to_string(),
                    mode: sqlrustgo_catalog::ParamMode::In,
                    data_type: "INTEGER".to_string(),
                },
                sqlrustgo_catalog::StoredProcParam {
                    name: "p2".to_string(),
                    mode: sqlrustgo_catalog::ParamMode::Out,
                    data_type: "TEXT".to_string(),
                },
            ],
            vec![],
        );
        catalog.add_stored_procedure(proc).unwrap();

        let info_schema = InformationSchema::new(&catalog);
        let params = info_schema.get_parameters();

        assert_eq!(params.len(), 2);
        assert_eq!(params[0].specific_name, "test_proc");
        assert_eq!(params[0].parameter_name, Some("p1".to_string()));
        assert_eq!(params[0].parameter_mode, Some("IN".to_string()));
        assert_eq!(params[1].parameter_name, Some("p2".to_string()));
        assert_eq!(params[1].parameter_mode, Some("OUT".to_string()));
    }

    #[test]
    fn test_empty_catalog_new_tables() {
        let catalog = Catalog::new("test");
        let info_schema = InformationSchema::new(&catalog);

        // New tables should return empty
        assert!(info_schema.get_triggers().is_empty());
        assert!(info_schema.get_routines().is_empty());
        assert!(info_schema.get_parameters().is_empty());
        assert!(info_schema.get_user_privileges().is_empty());
        assert!(info_schema.get_schema_privileges().is_empty());
        assert!(info_schema.get_table_privileges().is_empty());
        assert!(info_schema.get_column_privileges().is_empty());
    }
}
