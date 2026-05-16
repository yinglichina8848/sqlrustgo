//! Mobile device management for GMP-compliant data collection
//!
//! This module provides mobile device registration, trust verification,
//! and data collection audit capabilities.

pub mod collection;
pub mod collection_manager;
pub mod device;
pub mod trust;

pub use collection::{CollectionStatus, MobileCollection, MobileCollectionRecord};
pub use collection_manager::MobileTrustedCollection;
pub use device::{DeviceStatus, MobileDevice};
pub use trust::{verify_device_signature, verify_device_trust, TrustVerificationResult};

/// Table name for mobile devices
pub const TABLE_MOBILE_DEVICES: &str = "gmp_mobile_devices";

/// Table name for mobile collection audit records
pub const TABLE_MOBILE_COLLECTIONS: &str = "gmp_mobile_collection_audit";
