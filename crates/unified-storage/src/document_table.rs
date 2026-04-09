//! Document Table - Unified table structure for SQL + Vector columns

use serde::{Deserialize, Serialize};
use sqlrustgo_vector::metrics::DistanceMetric;

/// Vector column metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorColumn {
    /// Column name
    pub name: String,
    /// Vector dimension
    pub dimension: usize,
    /// Distance metric for similarity search
    pub metric: DistanceMetric,
}

impl VectorColumn {
    /// Create a new vector column
    pub fn new(name: &str, dimension: usize, metric: DistanceMetric) -> Self {
        Self {
            name: name.to_string(),
            dimension,
            metric,
        }
    }
}

/// Document column type - either SQL column or Vector column
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocumentColumn {
    /// SQL column with name and data type
    Sql { name: String, data_type: String },
    /// Vector column for embedding storage
    Vector(VectorColumn),
}

impl DocumentColumn {
    /// Get the column name
    pub fn name(&self) -> &str {
        match self {
            DocumentColumn::Sql { name, .. } => name,
            DocumentColumn::Vector(v) => &v.name,
        }
    }

    /// Check if this is a vector column
    pub fn is_vector(&self) -> bool {
        matches!(self, DocumentColumn::Vector(_))
    }
}

/// Unified Document Table supporting SQL + Vector columns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentTable {
    /// Table name
    name: String,
    /// Columns (SQL and Vector)
    columns: Vec<DocumentColumn>,
    /// Primary key column name
    primary_key: Option<String>,
    /// Table description
    description: Option<String>,
}

impl DocumentTable {
    /// Create a new document table
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            columns: Vec::new(),
            primary_key: None,
            description: None,
        }
    }

    /// Set table description
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    /// Set primary key
    pub fn with_primary_key(mut self, column: &str) -> Self {
        self.primary_key = Some(column.to_string());
        self
    }

    /// Add a SQL column
    pub fn add_sql_column(&mut self, name: &str, data_type: &str) -> &mut Self {
        self.columns.push(DocumentColumn::Sql {
            name: name.to_string(),
            data_type: data_type.to_string(),
        });
        self
    }

    /// Add a vector column
    pub fn add_vector_column(
        &mut self,
        name: &str,
        dimension: usize,
        metric: DistanceMetric,
    ) -> &mut Self {
        self.columns.push(DocumentColumn::Vector(VectorColumn::new(
            name, dimension, metric,
        )));
        self
    }

    /// Get table name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get all columns
    pub fn columns(&self) -> &[DocumentColumn] {
        &self.columns
    }

    /// Get SQL columns only
    pub fn sql_columns(&self) -> impl Iterator<Item = &DocumentColumn> {
        self.columns
            .iter()
            .filter(|c| matches!(c, DocumentColumn::Sql { .. }))
    }

    /// Get vector columns only
    pub fn vector_columns(&self) -> impl Iterator<Item = &VectorColumn> {
        self.columns.iter().filter_map(|c| match c {
            DocumentColumn::Vector(v) => Some(v),
            _ => None,
        })
    }

    /// Get primary key
    pub fn primary_key(&self) -> Option<&str> {
        self.primary_key.as_deref()
    }

    /// Get description
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Get column count
    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    /// Get SQL column count
    pub fn sql_column_count(&self) -> usize {
        self.columns
            .iter()
            .filter(|c| matches!(c, DocumentColumn::Sql { .. }))
            .count()
    }

    /// Get vector column count
    pub fn vector_column_count(&self) -> usize {
        self.columns
            .iter()
            .filter(|c| matches!(c, DocumentColumn::Vector(_)))
            .count()
    }

    /// Find column by name
    pub fn find_column(&self, name: &str) -> Option<&DocumentColumn> {
        self.columns.iter().find(|c| c.name() == name)
    }

    /// Check if column exists
    pub fn has_column(&self, name: &str) -> bool {
        self.find_column(name).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_column_creation() {
        let col = VectorColumn::new("embedding", 384, DistanceMetric::Cosine);
        assert_eq!(col.name, "embedding");
        assert_eq!(col.dimension, 384);
        assert_eq!(col.metric, DistanceMetric::Cosine);
    }

    #[test]
    fn test_document_table_mixed_columns() {
        let mut table = DocumentTable::new("products");
        table
            .add_sql_column("id", "INTEGER")
            .add_sql_column("name", "TEXT")
            .add_vector_column("embedding", 384, DistanceMetric::Cosine);

        assert_eq!(table.name(), "products");
        assert_eq!(table.column_count(), 3);
        assert_eq!(table.sql_column_count(), 2);
        assert_eq!(table.vector_column_count(), 1);
    }

    #[test]
    fn test_document_table_with_primary_key() {
        let table = DocumentTable::new("users")
            .with_primary_key("id")
            .with_description("User accounts");

        assert_eq!(table.primary_key(), Some("id"));
        assert_eq!(table.description(), Some("User accounts"));
    }

    #[test]
    fn test_find_column() {
        let mut table = DocumentTable::new("test");
        table.add_sql_column("id", "INTEGER");

        let col = table.find_column("id").unwrap();
        assert!(!col.is_vector());

        let none = table.find_column("nonexistent");
        assert!(none.is_none());
    }
}
