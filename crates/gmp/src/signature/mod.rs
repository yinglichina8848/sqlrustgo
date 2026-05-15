//! Digital Signature Audit Chain Module
//!
//! Provides cryptographic digital signatures (Ed25519, ECDSA P-256)
//! for tamper-evident audit chains.

pub mod algorithms;
pub mod chain;
pub mod ecdsa;
pub mod ed25519;
pub mod error;
pub mod keys;
pub mod local_keys;
pub mod signed_entry;
pub mod traits;

pub use algorithms::SignatureAlgorithm;
pub use error::{KeyError, SignatureError};
pub use keys::{Certificate, PrivateKey, PublicKey};
pub use traits::{KeyManager, SignatureVerifier, Signer};
