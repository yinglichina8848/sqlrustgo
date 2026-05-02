//! Catalog - the root of the database metadata hierarchy
//!
//! Catalog -> Schema -> Table
//! Catalog -> StoredProcedure

use crate::auth::{AuthManager, ObjectType, Privilege, UserIdentity};
use crate::error::{CatalogError, CatalogResult};
use crate::schema::Schema;
use crate::stored_proc::StoredProcedure;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The root catalog containing schemas and stored procedures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Catalog {
    /// Catalog name (usually "default" or "postgres")
    pub name: String,
    /// Schemas in this catalog (name -> Schema)
    schemas: HashMap<String, Schema>,
    /// Default schema name
    default_schema: String,
    /// Stored procedures (name -> StoredProcedure)
    stored_procedures: HashMap<String, StoredProcedure>,
    #[serde(skip)]
    auth_manager: AuthManager,
}

impl Catalog {
    /// Create a new empty catalog
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            name,
            schemas: HashMap::new(),
            default_schema: "public".to_string(),
            stored_procedures: HashMap::new(),
            auth_manager: AuthManager::new(),
        }
    }

    /// Create a catalog with a default schema
    pub fn with_default_schema(name: impl Into<String>, schema_name: impl Into<String>) -> Self {
        let name = name.into();
        let schema_name = schema_name.into();
        let mut catalog = Self::new(name);
        catalog.default_schema = schema_name.clone();
        catalog
            .schemas
            .insert(schema_name.clone(), Schema::new(schema_name));
        catalog
    }

    // ============ Schema operations ============

    /// Add a schema to the catalog
    pub fn add_schema(&mut self, schema: Schema) -> CatalogResult<()> {
        if self.schemas.contains_key(&schema.name) {
            return Err(CatalogError::DuplicateTable {
                schema: self.name.clone(),
                table: schema.name.clone(),
            });
        }
        self.schemas.insert(schema.name.clone(), schema);
        Ok(())
    }

    /// Get a schema by name
    pub fn get_schema(&self, name: &str) -> Option<&Schema> {
        self.schemas.get(name)
    }

    /// Get all schema names
    pub fn schema_names(&self) -> Vec<&str> {
        self.schemas.keys().map(|s| s.as_str()).collect()
    }

    /// Check if a schema exists
    pub fn has_schema(&self, name: &str) -> bool {
        self.schemas.contains_key(name)
    }

    /// Get the default schema
    pub fn default_schema(&self) -> &str {
        &self.default_schema
    }

    /// Set the default schema
    pub fn set_default_schema(&mut self, name: String) -> CatalogResult<()> {
        if !self.schemas.contains_key(&name) {
            return Err(CatalogError::SchemaNotFound(name));
        }
        self.default_schema = name;
        Ok(())
    }

    /// Get all schemas
    pub fn schemas(&self) -> &HashMap<String, Schema> {
        &self.schemas
    }

    /// Get the number of schemas
    pub fn schema_count(&self) -> usize {
        self.schemas.len()
    }

    // ============ Stored procedure operations ============

    /// Add a stored procedure
    pub fn add_stored_procedure(&mut self, procedure: StoredProcedure) -> CatalogResult<()> {
        if self.stored_procedures.contains_key(&procedure.name) {
            return Err(CatalogError::DuplicateTable {
                schema: self.name.clone(),
                table: procedure.name.clone(),
            });
        }
        self.stored_procedures
            .insert(procedure.name.clone(), procedure);
        Ok(())
    }

    /// Get a stored procedure by name
    pub fn get_stored_procedure(&self, name: &str) -> Option<&StoredProcedure> {
        self.stored_procedures.get(name)
    }

    /// Get all stored procedure names
    pub fn stored_procedure_names(&self) -> Vec<&str> {
        self.stored_procedures.keys().map(|s| s.as_str()).collect()
    }

    /// Check if a stored procedure exists
    pub fn has_stored_procedure(&self, name: &str) -> bool {
        self.stored_procedures.contains_key(name)
    }

    /// Remove a stored procedure
    pub fn remove_stored_procedure(&mut self, name: &str) -> Option<StoredProcedure> {
        self.stored_procedures.remove(name)
    }

    /// Get the number of stored procedures
    pub fn stored_procedure_count(&self) -> usize {
        self.stored_procedures.len()
    }

    /// List all stored procedures
    pub fn stored_procedures(&self) -> Vec<&StoredProcedure> {
        self.stored_procedures.values().collect()
    }

    /// Grant a privilege to a user
    pub fn grant_privilege(
        &mut self,
        identity: &UserIdentity,
        privilege: Privilege,
        object_type: ObjectType,
        object_name: &str,
        grant_option: bool,
    ) -> CatalogResult<u64> {
        self.auth_manager
            .grant_privilege(
                identity,
                privilege,
                object_type,
                object_name,
                &UserIdentity::new("root", "%"),
                grant_option,
            )
            .map_err(|e| CatalogError::ExecutionError(e.to_string()))
    }

    /// Revoke a privilege from a user
    pub fn revoke_privilege(
        &mut self,
        identity: &UserIdentity,
        privilege: Privilege,
        object_type: ObjectType,
        object_name: &str,
    ) -> CatalogResult<()> {
        self.auth_manager
            .revoke_privilege(identity, privilege, object_type, object_name)
            .map_err(|e| CatalogError::ExecutionError(e.to_string()))
    }

    /// Get the auth manager for direct access
    pub fn auth_manager(&self) -> &AuthManager {
        &self.auth_manager
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::column::ColumnDefinition;
    use crate::data_type::DataType;

    fn create_test_catalog() -> Catalog {
        let mut catalog = Catalog::new("test_catalog");
        let schema = Schema::new("public")
            .add_table(crate::table::Table::new(
                "users",
                vec![
                    ColumnDefinition::new("id", DataType::Integer),
                    ColumnDefinition::new("name", DataType::Text),
                ],
            ))
            .unwrap();
        catalog.add_schema(schema).unwrap();
        catalog
    }

    #[test]
    fn test_catalog_creation() {
        let catalog = Catalog::new("test");
        assert_eq!(catalog.name, "test");
        assert_eq!(catalog.schema_count(), 0);
        assert_eq!(catalog.stored_procedure_count(), 0);
    }

    #[test]
    fn test_catalog_with_default_schema() {
        let catalog = Catalog::with_default_schema("test", "public");
        assert!(catalog.has_schema("public"));
        assert_eq!(catalog.default_schema(), "public");
    }

    #[test]
    fn test_add_and_get_schema() {
        let mut catalog = create_test_catalog();
        assert!(catalog.has_schema("public"));
        let schema = catalog.get_schema("public").unwrap();
        assert_eq!(schema.name, "public");
        assert!(schema.has_table("users"));
    }

    #[test]
    fn test_duplicate_schema() {
        let mut catalog = create_test_catalog();
        let result = catalog.add_schema(Schema::new("public"));
        assert!(matches!(result, Err(CatalogError::DuplicateTable { .. })));
    }

    #[test]
    fn test_add_and_get_stored_procedure() {
        let mut catalog = create_test_catalog();
        let proc = StoredProcedure::new(
            "test_proc".to_string(),
            vec![],
            vec![crate::stored_proc::StoredProcStatement::RawSql(
                "SELECT 1".to_string(),
            )],
        );
        catalog.add_stored_procedure(proc).unwrap();
        assert!(catalog.has_stored_procedure("test_proc"));
        let retrieved = catalog.get_stored_procedure("test_proc").unwrap();
        assert_eq!(retrieved.name, "test_proc");
    }

    #[test]
    fn test_duplicate_stored_procedure() {
        let mut catalog = create_test_catalog();
        let proc = StoredProcedure::new("test_proc".to_string(), vec![], vec![]);
        catalog.add_stored_procedure(proc.clone()).unwrap();
        let result = catalog.add_stored_procedure(proc);
        assert!(matches!(result, Err(CatalogError::DuplicateTable { .. })));
    }

    #[test]
    fn test_remove_stored_procedure() {
        let mut catalog = create_test_catalog();
        let proc = StoredProcedure::new("test_proc".to_string(), vec![], vec![]);
        catalog.add_stored_procedure(proc).unwrap();
        let removed = catalog.remove_stored_procedure("test_proc");
        assert!(removed.is_some());
        assert!(!catalog.has_stored_procedure("test_proc"));
    }

    #[test]
    fn test_set_default_schema() {
        let mut catalog = Catalog::new("test");
        catalog.add_schema(Schema::new("s1")).unwrap();
        catalog.add_schema(Schema::new("s2")).unwrap();
        catalog.set_default_schema("s2".to_string()).unwrap();
        assert_eq!(catalog.default_schema(), "s2");
    }

    #[test]
    fn test_set_default_schema_not_found() {
        let mut catalog = Catalog::new("test");
        let result = catalog.set_default_schema("nonexistent".to_string());
        assert!(matches!(result, Err(CatalogError::SchemaNotFound(_))));
    }
}
