use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SignatureAlgorithm {
    EcdsaP256,
    RsaSha256,
}

impl std::fmt::Display for SignatureAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SignatureAlgorithm::EcdsaP256 => write!(f, "ECDSA P-256"),
            SignatureAlgorithm::RsaSha256 => write!(f, "RSA-SHA256"),
        }
    }
}
