use rsa::{
    pkcs8::{DecodePrivateKey, DecodePublicKey, EncodePrivateKey, EncodePublicKey, LineEnding},
    pkcs1v15::{SigningKey, VerifyingKey},
    signature::{Signer as RsaSigner, Verifier},
    RsaPrivateKey, RsaPublicKey,
};
use sha2::{Digest, Sha256};

use super::algorithms::SignatureAlgorithm;
use super::error::SignatureError;
use super::traits::Signer;

pub struct RsaSha256Signer {
    private_key: RsaPrivateKey,
}

impl RsaSha256Signer {
    pub fn new(private_key_pem: &[u8]) -> Result<Self, SignatureError> {
        let private_key = RsaPrivateKey::from_pkcs1_pem(&String::from_utf8_lossy(private_key_pem))
            .map_err(|e| SignatureError::InvalidFormat(e.to_string()))?;
        Ok(Self { private_key })
    }

    pub fn generate(bits: usize) -> Result<Self, SignatureError> {
        let private_key = RsaPrivateKey::new(&mut rand::thread_rng(), bits)
            .map_err(|e| SignatureError::SigningFailed(e.to_string()))?;
        Ok(Self { private_key })
    }

    pub fn public_key_pem(&self) -> String {
        let public_key = RsaPublicKey::from(&self.private_key);
        public_key.to_pkcs1_pem(LineEnding::LF).unwrap_or_default()
    }
}

impl Signer for RsaSha256Signer {
    fn sign(&self, data: &[u8], _signer_id: &str) -> Result<Vec<u8>, SignatureError> {
        let signing_key = SigningKey::<Sha256>::new(&self.private_key);
        let signature = signing_key.sign(data);
        Ok(signature.to_vec())
    }

    fn algorithm(&self) -> SignatureAlgorithm {
        SignatureAlgorithm::RsaSha256
    }
}

pub struct RsaSha256Verifier {
    public_key: RsaPublicKey,
}

impl RsaSha256Verifier {
    pub fn new(public_key_pem: &[u8]) -> Result<Self, SignatureError> {
        let public_key = RsaPublicKey::from_pkcs1_pem(&String::from_utf8_lossy(public_key_pem))
            .map_err(|e| SignatureError::InvalidFormat(e.to_string()))?;
        Ok(Self { public_key })
    }
}

impl super::traits::SignatureVerifier for RsaSha256Verifier {
    fn verify(&self, data: &[u8], signature: &[u8], _public_key: &super::keys::PublicKey) -> Result<bool, SignatureError> {
        let verifying_key = VerifyingKey::<Sha256>::new(&self.public_key);
        match verifying_key.verify(data, &signature.into()) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}
