# Digital Signature Audit Chain Design

**Date**: 2026-05-15
**Issue**: #978
**Status**: Approved
**Author**: hermes-agent

## Overview

Implement a digital signature audit chain for GMP compliance, providing cryptographic tamper evidence with ECDSA P-256 and RSA-SHA256 signatures.

## Architecture

### Module Structure

```
crates/gmp/src/signature/
├── mod.rs              # Module entry, exports public types
├── traits.rs           # Signer, KeyManager, SignatureVerifier traits
├── algorithms.rs        # SignatureAlgorithm enum (EcdsaP256, RsaSha256)
├── keys.rs             # Key data structures (PublicKey, PrivateKey, Certificate)
├── ecdsa.rs            # ECDSA P-256 signature implementation
├── rsa.rs              # RSA-SHA256 signature implementation
├── signed_entry.rs     # SignedAuditEntry with signature fields
├── chain.rs            # SignedAuditChain + Signer/Verifier
├── local_keys.rs       # Local file-based KeyManager implementation
└── error.rs           # Signature-related error types
```

### Key Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Signature Algorithms | ECDSA P-256 + RSA-SHA256 | Support both modern and legacy systems |
| Key Management | Local files + KeyManager trait | MVP + future HSM/KMS (#985) |
| Identity Model | signer_id + certificate_path | Balance simplicity and security |
| Verification | Batch + incremental | Balance security and performance |
| Layer | Hybrid | Storage auto-record, application explicit sign |

## Data Structures

### SignatureAlgorithm

```rust
pub enum SignatureAlgorithm {
    EcdsaP256,
    RsaSha256,
}
```

### Signer Trait

```rust
pub trait Signer {
    fn sign(&self, data: &[u8], signer_id: &str) -> Result<Signature, SignatureError>;
    fn algorithm(&self) -> SignatureAlgorithm;
}
```

### KeyManager Trait

```rust
pub trait KeyManager: Send + Sync {
    fn get_private_key(&self, key_id: &str) -> Result<PrivateKey, KeyError>;
    fn get_public_key(&self, key_id: &str) -> Result<PublicKey, KeyError>;
    fn get_certificate(&self, key_id: &str) -> Result<Certificate, KeyError>;
    fn list_keys(&self) -> Result<Vec<String>, KeyError>;
}
```

### SignedAuditEntry

```rust
pub struct SignedAuditEntry {
    pub seq: u64,
    pub prev_hash: [u8; 32],
    pub timestamp: i64,
    pub user_id: String,
    pub action: String,
    pub table_name: String,
    pub record_id: Option<String>,
    pub old_value: Option<String>,
    pub new_value: Option<String>,

    // Signature fields
    pub signature: Vec<u8>,
    pub algorithm: SignatureAlgorithm,
    pub signer_id: String,
    pub certificate_path: Option<String>,
}
```

## Verification Strategy

### Incremental Verification
- After each append, verify the new entry's signature
- Check chain linkage (prev_hash matches last entry's hash)

### Batch Verification
- `verify_chain()` - Full chain verification
- `verify_range(seq_start, seq_end)` - Partial verification
- Returns `VerificationResult` with details

## Local Key Manager

Keys stored as PEM files:
- `keys/{key_id}/private_key.pem`
- `keys/{key_id}/public_key.pem`
- `keys/{key_id}/certificate.pem`

## Dependencies

Add to `crates/gmp/Cargo.toml`:
```toml
k256 = { version = "0.16", features = ["pem", "arithmetic"] }
rsa = "0.9"
pem = "3.0"
```

## Implementation Phases

1. **Phase 1**: Core traits and error types
2. **Phase 2**: ECDSA P-256 implementation
3. **Phase 3**: RSA-SHA256 implementation
4. **Phase 4**: Local KeyManager
5. **Phase 5**: SignedAuditChain and verifier
6. **Phase 6**: Integration tests

## Acceptance Criteria

- [ ] sign/verify API complete
- [ ] ECDSA P-256 signature support
- [ ] RSA-SHA256 signature support
- [ ] Chain verification tool
- [ ] Unit test coverage >= 80%
