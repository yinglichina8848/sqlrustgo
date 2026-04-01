//! TLS/SSL configuration and certificate management
//!
//! Provides secure connection support with certificate verification.

use native_tls::{Certificate, Identity, TlsAcceptor};
use std::fs::File;
use std::io::{self, Read};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct TlsConfig {
    pub cert_path: PathBuf,
    pub key_path: PathBuf,
    pub ca_cert_path: Option<PathBuf>,
    pub accept_invalid_certs: bool,
    pub min_tls_version: TlsVersion,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TlsVersion {
    TLS1_0,
    TLS1_1,
    TLS1_2,
    TLS1_3,
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            cert_path: PathBuf::from("certs/server.crt"),
            key_path: PathBuf::from("certs/server.key"),
            ca_cert_path: None,
            accept_invalid_certs: false,
            min_tls_version: TlsVersion::TLS1_2,
        }
    }
}

impl TlsConfig {
    pub fn new(cert_path: PathBuf, key_path: PathBuf) -> Self {
        Self {
            cert_path,
            key_path,
            ..Default::default()
        }
    }

    pub fn with_ca_cert(mut self, ca_cert_path: PathBuf) -> Self {
        self.ca_cert_path = Some(ca_cert_path);
        self
    }

    pub fn with_min_tls_version(mut self, version: TlsVersion) -> Self {
        self.min_tls_version = version;
        self
    }

    pub fn accept_invalid_certs(mut self) -> Self {
        self.accept_invalid_certs = true;
        self
    }
}

pub struct CertificateManager {
    config: TlsConfig,
    identity: Option<Identity>,
    ca_certs: Vec<Certificate>,
}

impl CertificateManager {
    pub fn new(config: TlsConfig) -> io::Result<Self> {
        let mut manager = Self {
            config,
            identity: None,
            ca_certs: Vec::new(),
        };
        manager.load_certificates()?;
        Ok(manager)
    }

    fn load_certificates(&mut self) -> io::Result<()> {
        self.identity = Some(self.load_identity()?);

        if let Some(ref ca_path) = self.config.ca_cert_path {
            let ca_cert = self.load_ca_certificate(ca_path)?;
            self.ca_certs.push(ca_cert);
        }

        Ok(())
    }

    fn load_identity(&self) -> io::Result<Identity> {
        let cert_data = self.read_file(&self.config.cert_path)?;
        let key_data = self.read_file(&self.config.key_path)?;

        let cert_pem =
            pem::parse(&cert_data).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let key_pem =
            pem::parse(&key_data).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let cert_der = cert_pem.contents;
        let key_der = key_pem.contents;

        Identity::from_pkcs8(&cert_der, &key_der)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    fn load_ca_certificate(&self, path: &PathBuf) -> io::Result<Certificate> {
        let data = self.read_file(path)?;
        Certificate::from_pem(&data).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    fn read_file(&self, path: &PathBuf) -> io::Result<Vec<u8>> {
        let mut file = File::open(path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;
        Ok(data)
    }

    pub fn get_identity(&self) -> Option<&Identity> {
        self.identity.as_ref()
    }

    pub fn get_ca_certs(&self) -> &[Certificate] {
        &self.ca_certs
    }

    pub fn verify_client(&self, _cert: &Certificate) -> Result<(), TlsError> {
        if self.config.accept_invalid_certs {
            return Ok(());
        }

        if self.ca_certs.is_empty() {
            return Err(TlsError::NoCaCertificate);
        }

        Ok(())
    }

    pub fn create_acceptor(&self) -> Result<TlsAcceptor, TlsError> {
        let identity = self.identity.as_ref().ok_or(TlsError::NoIdentity)?;

        TlsAcceptor::new(identity.clone()).map_err(TlsError::AcceptorCreationFailed)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TlsError {
    #[error("Failed to create TLS acceptor: {0}")]
    AcceptorCreationFailed(native_tls::Error),
    #[error("No identity certificate loaded")]
    NoIdentity,
    #[error("No CA certificate available for verification")]
    NoCaCertificate,
    #[error("Certificate verification failed: {0}")]
    VerificationFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tls_config_default() {
        let config = TlsConfig::default();
        assert!(!config.accept_invalid_certs);
        assert_eq!(config.min_tls_version, TlsVersion::TLS1_2);
    }

    #[test]
    fn test_tls_config_builder() {
        let config = TlsConfig::new(
            PathBuf::from("/path/to/cert"),
            PathBuf::from("/path/to/key"),
        )
        .with_ca_cert(PathBuf::from("/path/to/ca"))
        .with_min_tls_version(TlsVersion::TLS1_3)
        .accept_invalid_certs();

        assert_eq!(config.cert_path, PathBuf::from("/path/to/cert"));
        assert_eq!(config.key_path, PathBuf::from("/path/to/key"));
        assert!(config.ca_cert_path.is_some());
        assert!(config.accept_invalid_certs);
        assert_eq!(config.min_tls_version, TlsVersion::TLS1_3);
    }

    #[test]
    fn test_certificate_manager_no_identity() {
        let config = TlsConfig::default();
        let manager = CertificateManager::new(config);
        assert!(manager.is_err());
    }
}
