//! SQLRustGo Compact Row v1 Format ABI
//!
//! Independent storage ABI for clustered index row format.
//! Future engines (heap, columnar, LSM) can reuse this module.

pub mod decoder;
pub mod encoder;
pub mod null_bitmap;
pub mod overflow;
pub mod types;

#[cfg(test)]
pub mod integration_tests;

// Re-export for convenience
pub use decoder::decode_row;
pub use encoder::encode_row;
pub use types::*;
