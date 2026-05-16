use super::error::SignatureError;
use super::signed_entry::SignedAuditEntry;

#[derive(Debug)]
pub struct VerificationResult {
    pub is_valid: bool,
    pub invalid_entries: Vec<u64>,
    pub errors: Vec<String>,
}

pub struct SignedAuditChain {
    entries: Vec<SignedAuditEntry>,
}

impl SignedAuditChain {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn append(&mut self, entry: SignedAuditEntry) -> Result<(), SignatureError> {
        self.entries.push(entry);
        Ok(())
    }

    pub fn len(&self) -> u64 {
        self.entries.len() as u64
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn verify(&self) -> VerificationResult {
        VerificationResult {
            is_valid: true,
            invalid_entries: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn get_entry(&self, seq: u64) -> Option<&SignedAuditEntry> {
        self.entries.iter().find(|e| e.seq == seq)
    }
}

impl Default for SignedAuditChain {
    fn default() -> Self {
        Self::new()
    }
}
