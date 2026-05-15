use super::algorithms::SignatureAlgorithm;

#[derive(Debug, Clone)]
pub enum PrivateKey {
    EcdsaP256(Vec<u8>),
    RsaSha256(Vec<u8>),
}

impl PrivateKey {
    pub fn algorithm(&self) -> SignatureAlgorithm {
        match self {
            PrivateKey::EcdsaP256(_) => SignatureAlgorithm::EcdsaP256,
            PrivateKey::RsaSha256(_) => SignatureAlgorithm::RsaSha256,
        }
    }
}

#[derive(Debug, Clone)]
pub enum PublicKey {
    EcdsaP256(Vec<u8>),
    RsaSha256(Vec<u8>),
}

impl PublicKey {
    pub fn algorithm(&self) -> SignatureAlgorithm {
        match self {
            PublicKey::EcdsaP256(_) => SignatureAlgorithm::EcdsaP256,
            PublicKey::RsaSha256(_) => SignatureAlgorithm::RsaSha256,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Certificate {
    pub data: Vec<u8>,
    pub subject: String,
}
