use super::algorithms::SignatureAlgorithm;
use super::error::SignatureError;
use super::traits::{SignatureVerifier, Signer};
use super::{ecdsa::EcdsaP256Signer, ed25519::Ed25519Signer};

#[derive(Debug, Clone)]
pub enum PrivateKey {
    Ed25519(Vec<u8>),
    EcdsaP256(Vec<u8>),
}

impl PrivateKey {
    pub fn generate(algorithm: SignatureAlgorithm) -> Result<Self, SignatureError> {
        match algorithm {
            SignatureAlgorithm::Ed25519 => {
                let signer = Ed25519Signer::generate();
                Ok(Self::Ed25519(signer.private_key_bytes()))
            }
            SignatureAlgorithm::EcdsaP256 => {
                let signer = EcdsaP256Signer::generate();
                Ok(Self::EcdsaP256(signer.private_key_bytes()))
            }
        }
    }

    pub fn algorithm(&self) -> SignatureAlgorithm {
        match self {
            PrivateKey::Ed25519(_) => SignatureAlgorithm::Ed25519,
            PrivateKey::EcdsaP256(_) => SignatureAlgorithm::EcdsaP256,
        }
    }

    pub fn to_public(&self) -> PublicKey {
        match self {
            PrivateKey::Ed25519(scalar) => {
                let signer = Ed25519Signer::new(&scalar.clone().try_into().unwrap()).unwrap();
                PublicKey::Ed25519(signer.public_key_bytes())
            }
            PrivateKey::EcdsaP256(scalar) => {
                let signer = EcdsaP256Signer::new(scalar.clone()).unwrap();
                PublicKey::EcdsaP256(signer.public_key_bytes())
            }
        }
    }
}

impl Signer for PrivateKey {
    fn sign(&self, data: &[u8], signer_id: &str) -> Result<Vec<u8>, SignatureError> {
        match self {
            PrivateKey::Ed25519(bytes) => {
                let signer = Ed25519Signer::new(&bytes.clone().try_into().unwrap())?;
                signer.sign(data, signer_id)
            }
            PrivateKey::EcdsaP256(bytes) => {
                let signer = EcdsaP256Signer::new(bytes.clone())?;
                signer.sign(data, signer_id)
            }
        }
    }

    fn algorithm(&self) -> SignatureAlgorithm {
        match self {
            PrivateKey::Ed25519(_) => SignatureAlgorithm::Ed25519,
            PrivateKey::EcdsaP256(_) => SignatureAlgorithm::EcdsaP256,
        }
    }
}

#[derive(Debug, Clone)]
pub enum PublicKey {
    Ed25519(Vec<u8>),
    EcdsaP256(Vec<u8>),
}

impl PublicKey {
    pub fn algorithm(&self) -> SignatureAlgorithm {
        match self {
            PublicKey::Ed25519(_) => SignatureAlgorithm::Ed25519,
            PublicKey::EcdsaP256(_) => SignatureAlgorithm::EcdsaP256,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            PublicKey::Ed25519(bytes) => bytes.clone(),
            PublicKey::EcdsaP256(bytes) => bytes.clone(),
        }
    }
}

impl SignatureVerifier for PublicKey {
    fn verify(
        &self,
        data: &[u8],
        signature: &[u8],
        _public_key: &super::keys::PublicKey,
    ) -> Result<bool, SignatureError> {
        match self {
            PublicKey::Ed25519(bytes) => {
                use super::ed25519::Ed25519Verifier;
                let verifier = Ed25519Verifier::new(bytes)?;
                verifier.verify(data, signature, _public_key)
            }
            PublicKey::EcdsaP256(bytes) => {
                use super::ecdsa::EcdsaP256Verifier;
                let verifier = EcdsaP256Verifier::new(bytes)?;
                verifier.verify(data, signature, _public_key)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Certificate {
    pub data: Vec<u8>,
    pub subject: String,
}
