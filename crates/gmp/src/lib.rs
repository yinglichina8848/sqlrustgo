//! GMP Document Retrieval Extension for SQLRustGo
//!
//! This crate provides document management and vector-based similarity search
//! capabilities for SQLRustGo databases.
//!
//! # Features
//!
//! - **Document Management**: Store and version documents with typed metadata
//! - **Vector Embeddings**: Hash-based text embedding generation
//! - **Similarity Search**: Find documents by semantic similarity
//! - **Hybrid Search**: Combine text matching with vector similarity
//!
//! # Quick Start
//!
//! ```ignore
//! use sqlrustgo_gmp::{GmpExecutor, create_gmp_tables};
//! use sqlrustgo_storage::MemoryStorage;
//! use std::sync::{Arc, RwLock};
//!
//! let storage = Arc::new(RwLock::new(MemoryStorage::new()));
//! let executor = GmpExecutor::new(storage);
//!
//! executor.init().unwrap();
//!
//! let doc_id = executor
//!     .import_document(
//!         "My Document",
//!         "REPORT",
//!         "Document content here...",
//!         &["keyword1", "keyword2"],
//!     )
//!     .unwrap();
//!
//! let results = executor.search("similar content", 5).unwrap();
//! ```
//!
//! # Tables
//!
//! The GMP extension creates the following tables:
//! - `gmp_documents`: Document metadata (id, title, type, version, dates, status)
//! - `gmp_document_contents`: Document section content
//! - `gmp_document_keywords`: Document keywords for text search
//! - `gmp_embeddings`: Vector embeddings for similarity search

pub mod document;
pub mod embedding;
pub mod sql_api;
pub mod vector_search;

// Re-export commonly used types
pub use document::{
    create_gmp_tables, get_content, get_keywords, insert_document, insert_document_content,
    insert_document_keyword, query_by_effective_date, query_by_status, query_by_type, DocStatus,
    Document, DocumentContent, DocumentKeyword, TABLE_DOCUMENTS, TABLE_DOCUMENT_CONTENTS,
    TABLE_DOCUMENT_KEYWORDS,
};

pub use embedding::{
    cosine_similarity, euclidean_distance, generate_embedding, DocumentEmbedding, EmbeddingModel,
    HashEmbeddingModel, CREATE_EMBEDDINGS_TABLE, DEFAULT_MODEL, EMBEDDING_DIM, TABLE_EMBEDDINGS,
};

pub use vector_search::{
    create_embeddings_table, get_all_embeddings, hybrid_search, upsert_embedding, vector_search,
    vector_search_active, SearchResult,
};

pub use sql_api::{sql, GmpExecutor};
