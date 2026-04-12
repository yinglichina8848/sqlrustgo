//! GMP SQL API Functions
//!
//! Provides SQL-compatible API functions for the GMP extension.
//! These functions work with the sqlrustgo ExecutionEngine.

use crate::document::{
    create_gmp_tables, insert_document, insert_document_content, insert_document_keyword,
    DocStatus, NewDocument, TABLE_DOCUMENTS, TABLE_DOCUMENT_CONTENTS,
};
use crate::embedding::generate_embedding;
use crate::vector_search::{
    create_embeddings_table, hybrid_search, upsert_embedding, vector_search, SearchResult,
};
use sqlrustgo_storage::StorageEngine;
use sqlrustgo_types::{SqlResult, Value};
use std::sync::{Arc, RwLock};

/// GMP Executor - provides a convenient API for GMP operations
///
/// Wraps a storage engine reference and provides GMP-specific operations.
pub struct GmpExecutor {
    storage: Arc<RwLock<dyn StorageEngine>>,
}

impl GmpExecutor {
    /// Create a new GmpExecutor wrapping a storage engine
    pub fn new(storage: Arc<RwLock<dyn StorageEngine>>) -> Self {
        Self { storage }
    }

    /// Initialize all GMP tables (documents, keywords, contents, embeddings)
    pub fn init(&self) -> SqlResult<()> {
        let mut storage = self.storage.write().unwrap();
        create_gmp_tables(&mut *storage)?;
        create_embeddings_table(&mut *storage)?;
        Ok(())
    }

    /// Import a document with its content, keywords, and embedding
    ///
    /// This is a convenience method that:
    /// 1. Inserts the document record
    /// 2. Inserts the content sections
    /// 3. Inserts the keywords
    /// 4. Generates and stores an embedding from the content
    pub fn import_document(
        &self,
        title: &str,
        doc_type: &str,
        content: &str,
        keywords: &[&str],
    ) -> SqlResult<i64> {
        let mut storage = self.storage.write().unwrap();

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        // Insert document
        let doc_id = insert_document(
            &mut *storage,
            NewDocument {
                title,
                doc_type,
                version: 1,
                created_at: now,
                updated_at: now,
                effective_date: (now / 86400) as i32,
                status: DocStatus::Active,
            },
        )?;

        // Insert content as a single section
        insert_document_content(&mut *storage, doc_id, "main", content)?;

        // Insert keywords
        for keyword in keywords {
            insert_document_keyword(&mut *storage, doc_id, keyword)?;
        }

        // Generate and store embedding from content
        let embedding = generate_embedding(content);
        upsert_embedding(&mut *storage, doc_id, &embedding)?;

        Ok(doc_id)
    }

    /// Search documents by vector similarity
    pub fn search(&self, query: &str, top_k: usize) -> SqlResult<Vec<SearchResult>> {
        let storage = self.storage.read().unwrap();
        vector_search(&*storage, query, top_k)
    }

    /// Hybrid search (text + vector)
    pub fn hybrid_search(&self, query: &str, top_k: usize) -> SqlResult<Vec<SearchResult>> {
        let storage = self.storage.read().unwrap();
        hybrid_search(&*storage, query, top_k)
    }

    /// Generate an embedding for text
    pub fn vector_embed(&self, text: &str) -> Vec<f32> {
        generate_embedding(text)
    }

    /// Bulk import multiple documents
    pub fn bulk_import(
        &self,
        documents: &[(&str, &str, &str, Vec<&str>)], // (title, doc_type, content, keywords)
    ) -> SqlResult<Vec<i64>> {
        let mut ids = Vec::new();
        for (title, doc_type, content, keywords) in documents {
            let id = self.import_document(title, doc_type, content, keywords)?;
            ids.push(id);
        }
        Ok(ids)
    }

    /// Get document by ID
    pub fn get_document(&self, doc_id: i64) -> SqlResult<Option<crate::document::Document>> {
        use crate::document::Document;
        let storage = self.storage.read().unwrap();
        let rows = storage.scan(TABLE_DOCUMENTS)?;
        let doc = rows
            .into_iter()
            .find(|row| matches!(&row[0], Value::Integer(id) if *id == doc_id))
            .and_then(|row| Document::from_row(&row));
        Ok(doc)
    }

    /// Re-index all documents (regenerate embeddings from content)
    pub fn reindex_all(&self) -> SqlResult<usize> {
        let mut storage = self.storage.write().unwrap();

        // Get all documents
        let doc_rows = storage.scan(TABLE_DOCUMENTS)?;
        let docs: Vec<_> = doc_rows
            .into_iter()
            .filter_map(|row| crate::document::Document::from_row(&row))
            .collect();

        // Get all content
        let content_rows = storage.scan(TABLE_DOCUMENT_CONTENTS)?;

        let mut count = 0;
        for doc in docs {
            // Concatenate all content sections
            let content: String = content_rows
                .iter()
                .filter(|row| matches!(&row[0], Value::Integer(id) if *id == doc.id))
                .filter_map(|row| match &row[2] {
                    Value::Text(s) => Some(s.clone()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join(" ");

            if !content.is_empty() {
                let embedding = generate_embedding(&content);
                upsert_embedding(&mut *storage, doc.id, &embedding)?;
                count += 1;
            }
        }

        Ok(count)
    }
}

/// SQL statement builders for GMP operations
///
/// These return SQL strings that can be executed by the ExecutionEngine.
pub mod sql {
    use crate::document::{TABLE_DOCUMENTS, TABLE_DOCUMENT_KEYWORDS};

    /// SQL to initialize GMP tables
    pub const INIT_TABLES: &str = r#"
CREATE TABLE IF NOT EXISTS gmp_documents (
    id INTEGER PRIMARY KEY,
    title TEXT NOT NULL,
    doc_type TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    effective_date INTEGER NOT NULL,
    status TEXT NOT NULL DEFAULT 'DRAFT'
);

CREATE TABLE IF NOT EXISTS gmp_document_contents (
    doc_id INTEGER NOT NULL,
    section TEXT NOT NULL,
    content TEXT NOT NULL,
    PRIMARY KEY (doc_id, section)
);

CREATE TABLE IF NOT EXISTS gmp_document_keywords (
    doc_id INTEGER NOT NULL,
    keyword TEXT NOT NULL,
    PRIMARY KEY (doc_id, keyword)
);

CREATE TABLE IF NOT EXISTS gmp_embeddings (
    doc_id INTEGER PRIMARY KEY,
    embedding TEXT NOT NULL,
    updated_at INTEGER NOT NULL
)
"#;

    /// Build a SELECT statement to find documents by keyword
    pub fn select_by_keyword(keyword: &str) -> String {
        format!(
            "SELECT d.* FROM {} d \
             JOIN {} dk ON d.id = dk.doc_id \
             WHERE dk.keyword = '{}'",
            TABLE_DOCUMENTS, TABLE_DOCUMENT_KEYWORDS, keyword
        )
    }

    /// Build a SELECT statement to find documents by type
    pub fn select_by_type(doc_type: &str) -> String {
        format!(
            "SELECT * FROM {} WHERE doc_type = '{}'",
            TABLE_DOCUMENTS, doc_type
        )
    }

    /// Build a SELECT statement to find documents by status
    pub fn select_by_status(status: &str) -> String {
        format!(
            "SELECT * FROM {} WHERE status = '{}'",
            TABLE_DOCUMENTS, status
        )
    }

    /// Build a SELECT statement to find documents by effective date
    pub fn select_by_date_range(from_date: i32, to_date: i32) -> String {
        format!(
            "SELECT * FROM {} WHERE effective_date BETWEEN {} AND {}",
            TABLE_DOCUMENTS, from_date, to_date
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlrustgo_storage::MemoryStorage;

    #[test]
    fn test_gmp_executor_import() {
        let storage = Arc::new(RwLock::new(MemoryStorage::new()));
        let executor = GmpExecutor::new(storage.clone());

        executor.init().unwrap();

        let doc_id = executor
            .import_document(
                "Test Document",
                "TEST",
                "This is a test document about Rust programming",
                &["rust", "test", "programming"],
            )
            .unwrap();

        assert!(doc_id > 0);

        // Verify document was inserted
        let doc = executor.get_document(doc_id).unwrap();
        assert!(doc.is_some());
        assert_eq!(doc.unwrap().title, "Test Document");
    }

    #[test]
    fn test_gmp_executor_search() {
        let storage = Arc::new(RwLock::new(MemoryStorage::new()));
        let executor = GmpExecutor::new(storage.clone());

        executor.init().unwrap();

        executor
            .import_document(
                "Rust Guide",
                "GUIDE",
                "Learn Rust programming language with this comprehensive guide",
                &["rust", "guide"],
            )
            .unwrap();

        executor
            .import_document(
                "Python Tips",
                "TIPS",
                "Python programming tips and tricks for developers",
                &["python", "tips"],
            )
            .unwrap();

        let results = executor.search("Rust programming", 5).unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].title, "Rust Guide");
    }

    #[test]
    fn test_bulk_import() {
        let storage = Arc::new(RwLock::new(MemoryStorage::new()));
        let executor = GmpExecutor::new(storage.clone());

        executor.init().unwrap();

        let docs = vec![
            (
                "Doc 1",
                "TYPE1",
                "Content for document 1",
                vec!["key1", "key2"],
            ),
            (
                "Doc 2",
                "TYPE2",
                "Content for document 2",
                vec!["key2", "key3"],
            ),
            (
                "Doc 3",
                "TYPE1",
                "Content for document 3",
                vec!["key1", "key3"],
            ),
        ];

        let ids = executor.bulk_import(&docs).unwrap();
        assert_eq!(ids.len(), 3);
    }

    #[test]
    fn test_sql_statement_builders() {
        assert!(sql::select_by_keyword("rust").contains("gmp_document_keywords"));
        assert!(sql::select_by_type("GUIDE").contains("GUIDE"));
        assert!(sql::select_by_status("ACTIVE").contains("ACTIVE"));
        assert!(sql::select_by_date_range(19000, 20000).contains("19000"));
    }
}
