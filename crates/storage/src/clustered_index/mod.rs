//! Clustered index implementation for SQLRustGo.
//!
//! This module provides clustered index storage using the Compact Row v1 format.
//! The clustered index stores actual row data in leaf pages, indexed by cluster key.

pub mod leaf;
pub mod overflow;
pub mod wal_integration;

#[cfg(test)]
mod invariant_tests;

pub use leaf::{ClusteredLeafIter, ClusteredLeafPage};
pub use overflow::OverflowManager;
pub use wal_integration::{ClusteredWalEntry, ClusteredWalManager};
