//! SQLRustGo Catalog Module
//!
//! Provides database metadata management including:
//! - Schema and table definitions
//! - Column and data type definitions
//! - Index information
//! - Stored procedures
//! - Authentication and authorization
//!
//! # Architecture
//!
//! Catalog -> Schema -> Table
//! Catalog -> StoredProcedure

pub mod auth;
pub mod catalog;
pub mod column;
pub mod data_type;
pub mod error;
pub mod index;
pub mod rebuild;
pub mod schema;
pub mod stored_proc;
pub mod system_tables;
pub mod table;

pub use auth::{AuthManager, Privilege, User, UserIdentity};
pub use catalog::Catalog;
pub use column::ColumnDefinition;
pub use data_type::DataType;
pub use error::{CatalogError, CatalogResult};
pub use stored_proc::{
    HandlerCondition, ParamMode, StoredProcParam, StoredProcStatement, StoredProcedure,
};
pub use table::{ForeignKeyAction, ForeignKeyRef, Table};
