use k256::ecdsa::signature::{Signer as K256Signer, Verifier as K256Verifier};
use k256::ecdsa::{Signature, SigningKey as K256SigningKey, VerifyingKey as K256VerifyingKey};
use k256::Secp256k1;
use rand::RngCore;

use super::algorithms::SignatureAlgorithm;
use super::error::SignatureError;
use super::traits::Signer;

type EcdsaSigningKey = K256SigningKey;
type EcdsaVerifyingKey = K256VerifyingKey;

pub struct EcdsaP256Signer {
    signing_key: EcdsaSigningKey,
}

impl EcdsaP256Signer {
    pub fn new(private_key_bytes: Vec<u8>) -> Result<Self, SignatureError> {
        let key_array: [u8; 32] = private_key_bytes[..32]
            .try_into()
            .map_err(|_| SignatureError::SigningFailed("Invalid key size".to_string()))?;
        let signing_key = EcdsaSigningKey::from_bytes(&key_array.into())
            .map_err(|e| SignatureError::SigningFailed(e.to_string()))?;
        Ok(Self { signing_key })
    }

    pub fn generate() -> Self {
        let mut bytes = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut bytes);
        let signing_key = EcdsaSigningKey::from_bytes(&bytes.into())
            .map_err(|e| SignatureError::SigningFailed(e.to_string()))
            .unwrap();
        Self { signing_key }
    }

    pub fn public_key_bytes(&self) -> Vec<u8> {
        let verifying_key = EcdsaVerifyingKey::from(&self.signing_key);
        verifying_key.to_sec1_point(false).to_bytes().to_vec()
    }

    pub fn private_key_bytes(&self) -> Vec<u8> {
        self.signing_key.to_bytes().to_vec()
    }
}

impl Signer for EcdsaP256Signer {
    fn sign(&self, data: &[u8], _signer_id: &str) -> Result<Vec<u8>, SignatureError> {
        let signature: Signature = self.signing_key.sign(data);
        Ok(signature.to_bytes().to_vec())
    }

    fn algorithm(&self) -> SignatureAlgorithm {
        SignatureAlgorithm::EcdsaP256
    }
}

pub struct EcdsaP256Verifier {
    verifying_key: EcdsaVerifyingKey,
}

impl EcdsaP256Verifier {
    pub fn new(public_key_bytes: &[u8]) -> Result<Self, SignatureError> {
        use k256::elliptic_curve::sec1::Sec1Point;
        let point = Sec1Point::<Secp256k1>::from_bytes(public_key_bytes)
            .map_err(|e| SignatureError::InvalidFormat(e.to_string()))?;
        let verifying_key = EcdsaVerifyingKey::from_sec1_point(&point)
            .map_err(|e| SignatureError::InvalidFormat(e.to_string()))?;
        Ok(Self { verifying_key })
    }
}

impl super::traits::SignatureVerifier for EcdsaP256Verifier {
    fn verify(
        &self,
        data: &[u8],
        signature: &[u8],
        _public_key: &super::keys::PublicKey,
    ) -> Result<bool, SignatureError> {
        let sig = Signature::from_slice(signature)
            .map_err(|e| SignatureError::InvalidSignature(e.to_string()))?;
        Ok(self.verifying_key.verify(data, &sig).is_ok())
    }
}
