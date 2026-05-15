//! Digital Signature Audit Chain Module
//!
//! Provides cryptographic digital signatures (ECDSA P-256, RSA-SHA256)
//! for tamper-evident audit chains.

pub mod algorithms;
pub mod ecdsa;
pub mod error;
pub mod keys;
pub mod local_keys;
pub mod rsa;
pub mod signed_entry;
pub mod chain;
pub mod traits;

pub use algorithms::SignatureAlgorithm;
pub use error::{KeyError, SignatureError};
pub use keys::{Certificate, PrivateKey, PublicKey};
pub use traits::{KeyManager, SignatureVerifier, Signer};
