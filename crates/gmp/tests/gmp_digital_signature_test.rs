#![allow(deprecated)]

use sqlrustgo_gmp::signature::{
    keys::{PrivateKey, PublicKey},
    SignatureAlgorithm, SignatureVerifier, Signer,
};

#[test]
fn test_ed25519_key_generation() {
    let private_key = PrivateKey::generate(SignatureAlgorithm::Ed25519).unwrap();
    let public_key = private_key.to_public();
    assert!(!public_key.to_bytes().is_empty());
}

#[test]
fn test_ed25519_sign_and_verify() {
    let private_key = PrivateKey::generate(SignatureAlgorithm::Ed25519).unwrap();
    let public_key = private_key.to_public();
    let data = b"Test data for signing";

    let signature = private_key.sign(data, "test_signer").unwrap();
    assert!(public_key.verify(data, &signature, &public_key).unwrap());
}

#[test]
fn test_ed25519_invalid_signature() {
    let private_key = PrivateKey::generate(SignatureAlgorithm::Ed25519).unwrap();
    let public_key = private_key.to_public();
    let data = b"Test data";
    let wrong_data = b"Wrong data";

    let signature = private_key.sign(data, "test_signer").unwrap();
    assert!(!public_key
        .verify(wrong_data, &signature, &public_key)
        .unwrap());
}

#[test]
fn test_ecdsa_key_generation() {
    let private_key = PrivateKey::generate(SignatureAlgorithm::EcdsaP256).unwrap();
    let public_key = private_key.to_public();
    assert!(!public_key.to_bytes().is_empty());
}

#[test]
fn test_ecdsa_sign_and_verify() {
    let private_key = PrivateKey::generate(SignatureAlgorithm::EcdsaP256).unwrap();
    let public_key = private_key.to_public();
    let data = b"Test data for signing";

    let signature = private_key.sign(data, "test_signer").unwrap();
    assert!(public_key.verify(data, &signature, &public_key).unwrap());
}

#[test]
fn test_ecdsa_invalid_signature() {
    let private_key = PrivateKey::generate(SignatureAlgorithm::EcdsaP256).unwrap();
    let public_key = private_key.to_public();
    let data = b"Test data";
    let wrong_data = b"Wrong data";

    let signature = private_key.sign(data, "test_signer").unwrap();
    assert!(!public_key
        .verify(wrong_data, &signature, &public_key)
        .unwrap());
}
