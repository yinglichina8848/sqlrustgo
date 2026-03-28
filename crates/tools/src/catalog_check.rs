//! Catalog Invariant Checker CLI
//!
//! Validates that the catalog invariants are satisfied in a storage engine.
//! This tool checks foreign key constraints, column references, and schema consistency.

use anyhow::{Context, Result};
use sqlrustgo_catalog::Catalog;
use sqlrustgo_storage::{MemoryStorage, StorageEngine};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "catalog_check", about = "Validate catalog invariants")]
pub struct Opt {
    /// Storage file path (for file-based storage)
    #[structopt(short = "d", long = "data-dir", default_value = "./data")]
    pub data_dir: PathBuf,

    /// Use memory storage instead of file storage
    #[structopt(short = "m", long = "memory")]
    pub memory: bool,
}

/// Open storage engine based on options
fn open_storage(opt: &Opt) -> Result<Box<dyn StorageEngine>> {
    if opt.memory {
        log::info!("Using in-memory storage");
        Ok(Box::new(MemoryStorage::new()))
    } else {
        // For now, use memory storage
        // TODO: Implement file-based storage
        log::info!("Using in-memory storage (file storage not yet implemented)");
        Ok(Box::new(MemoryStorage::new()))
    }
}

pub fn run() -> Result<()> {
    // Initialize logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Parse command line options
    let opt = Opt::from_args();

    run_with_opt(opt)
}

pub fn run_with_opt(opt: Opt) -> Result<()> {
    // Open storage engine
    let storage = open_storage(&opt).context("Failed to open storage engine")?;

    // Rebuild catalog from storage and check invariants
    match Catalog::rebuild(&*storage) {
        Ok(catalog) => match catalog.check_invariants() {
            Ok(()) => {
                println!("✅ Catalog invariants OK");
                println!("   Schema: {}", catalog.default_schema_name());
                if let Some(schema) = catalog.default_schema() {
                    println!("   Tables: {}", schema.table_count());
                }
                std::process::exit(0);
            }
            Err(e) => {
                eprintln!("❌ Catalog invariant violation: {}", e);
                std::process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("❌ Failed to rebuild catalog: {}", e);
            std::process::exit(1);
        }
    }
}
