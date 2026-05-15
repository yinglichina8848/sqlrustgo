use ed25519_dalek::{
    Signature as DalekSignature, Signer as DalekSignerTrait, SigningKey as Ed25519SigningKey,
    Verifier, VerifyingKey as Ed25519VerifyingKey,
};
use rand::RngCore;

use super::algorithms::SignatureAlgorithm;
use super::error::SignatureError;
use super::traits::{SignatureVerifier, Signer as MySigner};

pub struct Ed25519Signer {
    signing_key: Ed25519SigningKey,
}

impl Ed25519Signer {
    pub fn new(private_key_bytes: &[u8; 32]) -> Result<Self, SignatureError> {
        let signing_key = Ed25519SigningKey::from_bytes(private_key_bytes);
        Ok(Self { signing_key })
    }

    pub fn generate() -> Self {
        let mut bytes = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut bytes);
        let signing_key = Ed25519SigningKey::from_bytes(&bytes);
        Self { signing_key }
    }

    pub fn public_key_bytes(&self) -> Vec<u8> {
        self.signing_key.verifying_key().to_bytes().to_vec()
    }

    pub fn private_key_bytes(&self) -> Vec<u8> {
        self.signing_key.to_bytes().to_vec()
    }
}

impl MySigner for Ed25519Signer {
    fn sign(&self, data: &[u8], _signer_id: &str) -> Result<Vec<u8>, SignatureError> {
        let signature: DalekSignature = DalekSignerTrait::sign(&self.signing_key, data);
        Ok(signature.to_vec())
    }

    fn algorithm(&self) -> SignatureAlgorithm {
        SignatureAlgorithm::Ed25519
    }
}

pub struct Ed25519Verifier {
    verifying_key: Ed25519VerifyingKey,
}

impl Ed25519Verifier {
    pub fn new(public_key_bytes: &[u8]) -> Result<Self, SignatureError> {
        let bytes: [u8; 32] = public_key_bytes[..32]
            .try_into()
            .map_err(|_| SignatureError::InvalidFormat("Invalid key size".to_string()))?;
        let verifying_key = Ed25519VerifyingKey::from_bytes(&bytes)
            .map_err(|e| SignatureError::InvalidFormat(e.to_string()))?;
        Ok(Self { verifying_key })
    }
}

impl SignatureVerifier for Ed25519Verifier {
    fn verify(
        &self,
        data: &[u8],
        signature: &[u8],
        _public_key: &super::keys::PublicKey,
    ) -> Result<bool, SignatureError> {
        let sig = DalekSignature::from_slice(signature)
            .map_err(|e| SignatureError::InvalidSignature(e.to_string()))?;
        Ok(self.verifying_key.verify(data, &sig).is_ok())
    }
}
