//! SQLRustGo Compact Row v1 Format ABI
//!
//! Independent storage ABI for clustered index row format.
//! Future engines (heap, columnar, LSM) can reuse this module.

pub mod types;
pub mod encoder;
pub mod decoder;
pub mod null_bitmap;
pub mod overflow;

// Re-export for convenience
pub use types::*;
pub use encoder::encode_row;
pub use decoder::decode_row;