//! AST Indexer — walks Rust source files, extracts symbols, feeds to graph builder
//!
//! P0: Uses `syn` for parsing (no external C dependencies)
//! P0: Extracts modules, functions, structs, traits, enums, impl blocks, tests
//! P1: Will add call graph edges (function body analysis)

pub mod symbol_extractor;

use crate::ast::symbol_extractor::extract_symbols_from_file;
use crate::graph::{Node, NodeType};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub use symbol_extractor::Symbol;

/// AST Indexer — crawls repo, extracts symbols from Rust files
pub struct Indexer {
    repo_root: PathBuf,
}

impl Indexer {
    pub fn new(repo_root: &str) -> Result<Self, IndexerError> {
        let path = PathBuf::from(repo_root);
        if !path.exists() {
            return Err(IndexerError::RepoNotFound(repo_root.to_string()));
        }
        Ok(Self { repo_root: path })
    }

    /// Index all Rust source files under repo_root
    pub fn index_all(&self) -> Result<Vec<Node>, IndexerError> {
        let mut nodes = Vec::new();

        // Walk all Rust source files
        for entry in WalkDir::new(&self.repo_root)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            if let Some(ext) = path.extension() {
                if ext != "rs" {
                    continue;
                }
            } else {
                continue;
            }

            // Skip test files in tests/ directories (they'll be picked up as Test nodes)
            let relative = path.strip_prefix(&self.repo_root).unwrap_or(path);
            let relative_str = relative.to_string_lossy();

            // Skip target/ directories
            if relative_str.contains("target/") {
                continue;
            }

            // Extract symbols from this file
            match self.index_file(path) {
                Ok(file_nodes) => nodes.extend(file_nodes),
                Err(e) => {
                    tracing::warn!("Failed to parse {:?}: {}", path, e);
                }
            }
        }

        tracing::info!(
            "Indexed {} nodes from {}",
            nodes.len(),
            self.repo_root.display()
        );

        Ok(nodes)
    }

    /// Index a single Rust file
    fn index_file(&self, path: &Path) -> Result<Vec<Node>, IndexerError> {
        let repo_root_str = self.repo_root.to_string_lossy();
        let symbols = extract_symbols_from_file(path, &[])
            .map_err(|e| IndexerError::ParseError(e.to_string()))?;

        let nodes: Vec<Node> = symbols
            .into_iter()
            .map(|sym| {
                let file_path = path.to_string_lossy().replace(&*repo_root_str, "");
                let file_path = file_path.trim_start_matches('/');

                Node::new(
                    sym.name,
                    symbol_kind_to_node_type(&sym.kind),
                    file_path.to_string(),
                    sym.line_start,
                    sym.line_end,
                    sym.signature,
                )
            })
            .collect();

        Ok(nodes)
    }
}

fn symbol_kind_to_node_type(kind: &str) -> NodeType {
    match kind {
        "fn" => NodeType::Function,
        "struct" => NodeType::Struct,
        "trait" => NodeType::Trait,
        "enum" => NodeType::Enum,
        "impl" => NodeType::Impl,
        "mod" => NodeType::Module,
        "test" => NodeType::Test,
        _ => NodeType::Module,
    }
}

#[derive(Debug)]
pub enum IndexerError {
    RepoNotFound(String),
    ParseError(String),
    Io(String),
}

impl std::fmt::Display for IndexerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IndexerError::RepoNotFound(s) => write!(f, "Repository not found: {}", s),
            IndexerError::ParseError(s) => write!(f, "Parse error: {}", s),
            IndexerError::Io(s) => write!(f, "IO error: {}", s),
        }
    }
}

impl std::error::Error for IndexerError {}
