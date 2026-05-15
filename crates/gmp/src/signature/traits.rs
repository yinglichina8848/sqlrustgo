use super::algorithms::SignatureAlgorithm;
use super::error::{KeyError, SignatureError};
use super::keys::{Certificate, PrivateKey, PublicKey};

pub trait Signer: Send + Sync {
    fn sign(&self, data: &[u8], signer_id: &str) -> Result<Vec<u8>, SignatureError>;
    fn algorithm(&self) -> SignatureAlgorithm;
}

pub trait KeyManager: Send + Sync {
    fn get_private_key(&self, key_id: &str) -> Result<PrivateKey, KeyError>;
    fn get_public_key(&self, key_id: &str) -> Result<PublicKey, KeyError>;
    fn get_certificate(&self, key_id: &str) -> Result<Certificate, KeyError>;
    fn list_keys(&self) -> Result<Vec<String>, KeyError>;
}

pub trait SignatureVerifier: Send + Sync {
    fn verify(&self, data: &[u8], signature: &[u8], public_key: &PublicKey) -> Result<bool, SignatureError>;
}
