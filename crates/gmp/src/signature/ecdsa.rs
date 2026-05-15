use k256::ecdsa::{SigningKey, VerifyingKey};
use k256::elliptic_curve::rand_core::OsRng;

use super::algorithms::SignatureAlgorithm;
use super::error::SignatureError;
use super::traits::Signer;

pub struct EcdsaP256Signer {
    signing_key: SigningKey,
}

impl EcdsaP256Signer {
    pub fn new(private_key_bytes: Vec<u8>) -> Result<Self, SignatureError> {
        let signing_key = SigningKey::from_bytes(private_key_bytes.as_slice().into())
            .map_err(|e| SignatureError::SigningFailed(e.to_string()))?;
        Ok(Self { signing_key })
    }

    pub fn generate() -> Self {
        let signing_key = SigningKey::random(&mut OsRng);
        Self { signing_key }
    }

    pub fn public_key_bytes(&self) -> Vec<u8> {
        let verifying_key = VerifyingKey::from(&self.signing_key);
        verifying_key.to_encoded_point(false).as_bytes().to_vec()
    }
}

impl Signer for EcdsaP256Signer {
    fn sign(&self, data: &[u8], _signer_id: &str) -> Result<Vec<u8>, SignatureError> {
        use k256::ecdsa::Signature;
        let signature: Signature = self.signing_key.sign(data);
        Ok(signature.to_bytes().to_vec())
    }

    fn algorithm(&self) -> SignatureAlgorithm {
        SignatureAlgorithm::EcdsaP256
    }
}

pub struct EcdsaP256Verifier {
    verifying_key: VerifyingKey,
}

impl EcdsaP256Verifier {
    pub fn new(public_key_bytes: &[u8]) -> Result<Self, SignatureError> {
        use k256::elliptic_curve::point::DecompressionError;
        let point = k256::EncodedPoint::from_bytes(public_key_bytes.to_vec())
            .map_err(|e| SignatureError::InvalidFormat(e.to_string()))?;
        let verifying_key = VerifyingKey::from_encoded_point(&point)
            .map_err(|e| SignatureError::InvalidFormat(e.to_string()))?;
        Ok(Self { verifying_key })
    }
}

impl super::traits::SignatureVerifier for EcdsaP256Verifier {
    fn verify(&self, data: &[u8], signature: &[u8], _public_key: &super::keys::PublicKey) -> Result<bool, SignatureError> {
        use k256::ecdsa::Signature;
        let sig = Signature::from_slice(signature)
            .map_err(|e| SignatureError::InvalidSignature(e.to_string()))?;
        Ok(self.verifying_key.verify(data, &sig).is_ok())
    }
}
