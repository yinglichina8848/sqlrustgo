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

#[derive(Debug, Clone)]
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
        assert_eq!(format!("{}", HsmProviderType::Pkcs11), "PKCS#11");
        assert_eq!(format!("{}", HsmProviderType::AwsKms), "AWS KMS");
        assert_eq!(format!("{}", HsmProviderType::AzureKms), "Azure KMS");
        assert_eq!(format!("{}", HsmProviderType::GcpKms), "GCP KMS");
    }

    #[test]
    fn test_hsm_config_software_tpm() {
        let config = HsmConfig::software_tpm();
        assert_eq!(config.provider_type, HsmProviderType::SoftwareTpm);
        assert!(config.config_path.is_none());
    }

    #[test]
    fn test_hsm_config_tpm20() {
        let config = HsmConfig::tpm20("/path/to/config");
        assert_eq!(config.provider_type, HsmProviderType::Tpm20);
        assert_eq!(config.config_path, Some("/path/to/config".to_string()));
    }

    #[test]
    fn test_hsm_config_pkcs11() {
        let config = HsmConfig::pkcs11("/path/to/library");
        assert_eq!(config.provider_type, HsmProviderType::Pkcs11);
        assert_eq!(config.config_path, Some("/path/to/library".to_string()));
    }

    #[test]
    fn test_hsm_error_display() {
        let err = HsmError::KeyNotFound("test_key".to_string());
        assert_eq!(format!("{}", err), "Key not found: test_key");

        let err = HsmError::SigningFailed("signing error".to_string());
        assert_eq!(format!("{}", err), "Signing failed: signing error");

        let err = HsmError::KeyGenerationFailed("key gen error".to_string());
        assert_eq!(format!("{}", err), "Key generation failed: key gen error");

        let err = HsmError::NotAvailable("HSM unavailable".to_string());
        assert_eq!(format!("{}", err), "HSM not available: HSM unavailable");

        let err = HsmError::InvalidConfig("invalid config".to_string());
        assert_eq!(format!("{}", err), "Invalid configuration: invalid config");
    }

    #[test]
    fn test_hsm_provider_type_equality() {
        assert_eq!(HsmProviderType::SoftwareTpm, HsmProviderType::SoftwareTpm);
        assert_eq!(HsmProviderType::Tpm20, HsmProviderType::Tpm20);
        assert_ne!(HsmProviderType::SoftwareTpm, HsmProviderType::Tpm20);
    }

    #[test]
    fn test_hsm_config_clone() {
        let config = HsmConfig::tpm20("/path/to/config");
        let cloned = config.clone();
        assert_eq!(config.provider_type, cloned.provider_type);
        assert_eq!(config.config_path, cloned.config_path);
    }
}
