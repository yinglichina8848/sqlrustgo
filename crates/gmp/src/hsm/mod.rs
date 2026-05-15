use thiserror::Error;

pub mod software_tpm;

#[derive(Error, Debug)]
pub enum HsmError {
    #[error("Key not found: {0}")]
    KeyNotFound(String),
    #[error("Signing failed: {0}")]
    SigningFailed(String),
    #[error("Key generation failed: {0}")]
    KeyGenerationFailed(String),
    #[error("HSM not available: {0}")]
    NotAvailable(String),
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

pub trait HsmProvider: Send + Sync {
    fn generate_key(&self, key_id: &str) -> Result<(), HsmError>;
    fn sign(&self, key_id: &str, data: &[u8]) -> Result<Vec<u8>, HsmError>;
    fn verify(&self, key_id: &str, data: &[u8], signature: &[u8]) -> Result<bool, HsmError>;
    fn delete_key(&self, key_id: &str) -> Result<(), HsmError>;
    fn list_keys(&self) -> Result<Vec<String>, HsmError>;
}

#[derive(Debug, Clone, PartialEq)]
pub enum HsmProviderType {
    SoftwareTpm,
    Tpm20,
    Pkcs11,
    AwsKms,
    AzureKms,
    GcpKms,
}

impl std::fmt::Display for HsmProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HsmProviderType::SoftwareTpm => write!(f, "SoftwareTPM"),
            HsmProviderType::Tpm20 => write!(f, "TPM2.0"),
            HsmProviderType::Pkcs11 => write!(f, "PKCS#11"),
            HsmProviderType::AwsKms => write!(f, "AWS KMS"),
            HsmProviderType::AzureKms => write!(f, "Azure KMS"),
            HsmProviderType::GcpKms => write!(f, "GCP KMS"),
        }
    }
}

pub struct HsmConfig {
    pub provider_type: HsmProviderType,
    pub config_path: Option<String>,
}

impl HsmConfig {
    pub fn software_tpm() -> Self {
        Self {
            provider_type: HsmProviderType::SoftwareTpm,
            config_path: None,
        }
    }

    pub fn tpm20(config_path: &str) -> Self {
        Self {
            provider_type: HsmProviderType::Tpm20,
            config_path: Some(config_path.to_string()),
        }
    }

    pub fn pkcs11(library_path: &str) -> Self {
        Self {
            provider_type: HsmProviderType::Pkcs11,
            config_path: Some(library_path.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hsm_provider_type_display() {
        assert_eq!(format!("{}", HsmProviderType::SoftwareTpm), "SoftwareTPM");
        assert_eq!(format!("{}", HsmProviderType::Tpm20), "TPM2.0");
    }

    #[test]
    fn test_hsm_config_software_tpm() {
        let config = HsmConfig::software_tpm();
        assert_eq!(config.provider_type, HsmProviderType::SoftwareTpm);
        assert!(config.config_path.is_none());
    }

    #[test]
    fn test_hsm_error_display() {
        let err = HsmError::KeyNotFound("test_key".to_string());
        assert_eq!(format!("{}", err), "Key not found: test_key");
    }
}
