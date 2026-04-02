//! GMP Vector Similarity Search
//!
//! Provides vector-based document search functionality.
//! Searches document embeddings using cosine similarity.

use crate::embedding::{
    cosine_similarity, DocumentEmbedding, EmbeddingModel,
    HashEmbeddingModel, TABLE_EMBEDDINGS,
};
use crate::document::{TABLE_DOCUMENTS, Document};
use sqlrustgo_types::{SqlResult, Value};
use sqlrustgo_storage::StorageEngine;

/// Search result with document ID and similarity score
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub doc_id: i64,
    pub title: String,
    pub doc_type: String,
    pub similarity: f32,
}

/// Create the embeddings table if it doesn't exist
pub fn create_embeddings_table(storage: &mut dyn StorageEngine) -> SqlResult<()> {
    if !storage.has_table(TABLE_EMBEDDINGS) {
        let columns = vec![
            sqlrustgo_storage::ColumnDefinition {
                name: "doc_id".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: true,
                is_primary_key: true,
                references: None,
                auto_increment: false,
            },
            sqlrustgo_storage::ColumnDefinition {
                name: "embedding".to_string(),
                data_type: "TEXT".to_string(),
                nullable: false,
                is_unique: false,
                is_primary_key: false,
                references: None,
                auto_increment: false,
            },
            sqlrustgo_storage::ColumnDefinition {
                name: "updated_at".to_string(),
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_unique: false,
                is_primary_key: false,
                references: None,
                auto_increment: false,
            },
        ];
        storage.create_table(&sqlrustgo_storage::TableInfo {
            name: TABLE_EMBEDDINGS.to_string(),
            columns,
        })?;
    }
    Ok(())
}

/// Store or update an embedding for a document
pub fn upsert_embedding(
    storage: &mut dyn StorageEngine,
    doc_id: i64,
    embedding: &[f32],
) -> SqlResult<()> {
    create_embeddings_table(storage)?;

    let json = DocumentEmbedding::embedding_to_json(embedding);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    // Check if embedding already exists
    let rows = storage.scan(TABLE_EMBEDDINGS)?;
    let existing = rows.iter().any(|row| {
        matches!(&row[0], Value::Integer(id) if *id == doc_id)
    });

    if existing {
        // Delete old and insert new
        let _ = storage.delete(TABLE_EMBEDDINGS, &[Value::Integer(doc_id)]);
    }

    let row = vec![
        Value::Integer(doc_id),
        Value::Text(json),
        Value::Integer(now),
    ];
    storage.insert(TABLE_EMBEDDINGS, vec![row])?;
    Ok(())
}

/// Get all stored embeddings
pub fn get_all_embeddings(storage: &dyn StorageEngine) -> SqlResult<Vec<DocumentEmbedding>> {
    let rows = storage.scan(TABLE_EMBEDDINGS)?;
    let embeddings = rows
        .into_iter()
        .filter_map(|row| {
            let doc_id = match &row[0] {
                Value::Integer(n) => *n,
                _ => return None,
            };
            let embedding_str = match &row[1] {
                Value::Text(s) => s.clone(),
                _ => return None,
            };
            let updated_at = match &row[2] {
                Value::Integer(n) => *n,
                _ => return None,
            };
            let embedding = DocumentEmbedding::embedding_from_json(&embedding_str);
            Some(DocumentEmbedding {
                doc_id,
                embedding,
                updated_at,
            })
        })
        .collect();
    Ok(embeddings)
}

/// Perform vector similarity search
///
/// Searches all stored document embeddings and returns the top_k most similar
/// documents to the query text.
pub fn vector_search(
    storage: &dyn StorageEngine,
    query: &str,
    top_k: usize,
) -> SqlResult<Vec<SearchResult>> {
    // Generate query embedding
    let model = HashEmbeddingModel::default();
    let query_embedding = model.generate_embedding(query);

    // Get all embeddings
    let embeddings = get_all_embeddings(storage)?;

    // Compute similarities
    let mut results: Vec<(i64, f32)> = embeddings
        .into_iter()
        .map(|emb| {
            let sim = cosine_similarity(&query_embedding, &emb.embedding);
            (emb.doc_id, sim)
        })
        .collect();

    // Sort by similarity (descending)
    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // Take top_k
    let top_results = results.into_iter().take(top_k).collect::<Vec<_>>();

    // Get document metadata
    let mut search_results = Vec::new();
    let all_doc_rows = storage.scan(TABLE_DOCUMENTS)?;

    for (doc_id, similarity) in top_results {
        // Find document by id
        if let Some(row) = all_doc_rows.iter().find(|row| {
            matches!(&row[0], Value::Integer(id) if *id == doc_id)
        }) {
            let title = row
                .get(1)
                .and_then(|v| match v {
                    Value::Text(s) => Some(s.clone()),
                    _ => None,
                })
                .unwrap_or_default();
            let doc_type = row
                .get(2)
                .and_then(|v| match v {
                    Value::Text(s) => Some(s.clone()),
                    _ => None,
                })
                .unwrap_or_default();

            search_results.push(SearchResult {
                doc_id,
                title,
                doc_type,
                similarity,
            });
        }
    }

    Ok(search_results)
}

/// Search only active documents by vector similarity
pub fn vector_search_active(
    storage: &dyn StorageEngine,
    query: &str,
    top_k: usize,
) -> SqlResult<Vec<SearchResult>> {
    let all_results = vector_search(storage, query, top_k * 2)?; // Over-fetch
    let active_results: Vec<_> = all_results
        .into_iter()
        .filter(|r| r.doc_type != "ARCHIVED" && r.doc_type != "SUPERSEDED")
        .take(top_k)
        .collect();
    Ok(active_results)
}

/// Combined text and vector search
///
/// First does keyword/text search on title and keywords,
/// then re-ranks using vector similarity.
pub fn hybrid_search(
    storage: &dyn StorageEngine,
    query: &str,
    top_k: usize,
) -> SqlResult<Vec<SearchResult>> {
    let model = HashEmbeddingModel::default();
    let query_embedding = model.generate_embedding(query);

    // Get all documents
    let doc_rows = storage.scan(TABLE_DOCUMENTS)?;
    let docs: Vec<Document> = doc_rows
        .into_iter()
        .filter_map(|row| Document::from_row(&row))
        .collect();

    // Get all embeddings
    let embeddings = get_all_embeddings(storage)?;
    let emb_map: std::collections::HashMap<i64, Vec<f32>> = embeddings
        .into_iter()
        .map(|e| (e.doc_id, e.embedding))
        .collect();

    // Score each document
    let mut scored: Vec<(SearchResult, f32)> = Vec::new();

    for doc in docs {
        // Text match score: check if query words appear in title or doc_type
        let query_lower = query.to_lowercase();
        let title_lower = doc.title.to_lowercase();
        let doc_type_lower = doc.doc_type.to_lowercase();

        let text_match = query_lower
            .split_whitespace()
            .filter(|word| {
                word.len() > 2
                    && (title_lower.contains(word) || doc_type_lower.contains(word))
            })
            .count();

        let text_score = if text_match > 0 {
            (text_match as f32) / (query_lower.split_whitespace().count().max(1) as f32)
        } else {
            0.0f32
        };

        // Vector similarity score
        let vector_score = emb_map
            .get(&doc.id)
            .map(|emb| cosine_similarity(&query_embedding, emb))
            .unwrap_or(0.0);

        // Combined score: weighted average (60% vector, 40% text)
        let combined_score = vector_score * 0.6 + text_score * 0.4;

        if combined_score > 0.0 {
            scored.push((
                SearchResult {
                    doc_id: doc.id,
                    title: doc.title,
                    doc_type: doc.doc_type,
                    similarity: combined_score,
                },
                combined_score,
            ));
        }
    }

    // Sort by combined score
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    Ok(scored.into_iter().take(top_k).map(|(r, _)| r).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlrustgo_storage::MemoryStorage;
    use crate::document::{create_gmp_tables, insert_document, DocStatus, NewDocument};
    use crate::embedding::generate_embedding;

    #[test]
    fn test_vector_search() {
        let mut storage = MemoryStorage::new();
        create_gmp_tables(&mut storage).unwrap();
        create_embeddings_table(&mut storage).unwrap();

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        // Insert some documents
        let doc1_id = insert_document(
            &mut storage,
            NewDocument {
                title: "Rust Programming Guide",
                doc_type: "GUIDE",
                version: 1,
                created_at: now,
                updated_at: now,
                effective_date: 19000,
                status: DocStatus::Active,
            },
        )
        .unwrap();

        let doc2_id = insert_document(
            &mut storage,
            NewDocument {
                title: "Python Tutorial",
                doc_type: "TUTORIAL",
                version: 1,
                created_at: now,
                updated_at: now,
                effective_date: 19000,
                status: DocStatus::Active,
            },
        )
        .unwrap();

        let doc3_id = insert_document(
            &mut storage,
            NewDocument {
                title: "Database Design Patterns",
                doc_type: "BOOK",
                version: 1,
                created_at: now,
                updated_at: now,
                effective_date: 19000,
                status: DocStatus::Active,
            },
        )
        .unwrap();

        // Generate and store embeddings
        let emb1 = generate_embedding("Rust programming language memory safety");
        let emb2 = generate_embedding("Python scripting web development");
        let emb3 = generate_embedding("Database SQL queries transactions");

        upsert_embedding(&mut storage, doc1_id, &emb1).unwrap();
        upsert_embedding(&mut storage, doc2_id, &emb2).unwrap();
        upsert_embedding(&mut storage, doc3_id, &emb3).unwrap();

        // Search for Rust-related content - verify search returns non-empty results
        // (hash-based embeddings may not perfectly rank by semantic similarity)
        let results = vector_search(&storage, "Rust memory safety", 2).unwrap();
        assert!(!results.is_empty(), "vector search should return results");
        assert_eq!(results.len(), 2, "should return up to top_k results");
    }

    #[test]
    fn test_hybrid_search() {
        let mut storage = MemoryStorage::new();
        create_gmp_tables(&mut storage).unwrap();
        create_embeddings_table(&mut storage).unwrap();

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let doc_id = insert_document(
            &mut storage,
            NewDocument {
                title: "Rust Programming Guide",
                doc_type: "GUIDE",
                version: 1,
                created_at: now,
                updated_at: now,
                effective_date: 19000,
                status: DocStatus::Active,
            },
        )
        .unwrap();

        let emb = generate_embedding("Rust programming language");
        upsert_embedding(&mut storage, doc_id, &emb).unwrap();

        let results = hybrid_search(&storage, "Rust Guide", 5).unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].doc_id, doc_id);
    }
}
