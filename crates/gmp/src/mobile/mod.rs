//! Mobile device management for GMP-compliant data collection
//!
//! This module provides mobile device registration, trust verification,
//! and data collection audit capabilities.

mod device;
mod collection;
mod trust;

pub use device::{MobileDevice, DeviceStatus};
pub use collection::{MobileCollection, MobileCollectionRecord};
pub use trust::{verify_device_signature, verify_device_trust};

/// Table name for mobile devices
pub const TABLE_MOBILE_DEVICES: &str = "gmp_mobile_devices";

/// Table name for mobile collection audit records
pub const TABLE_MOBILE_COLLECTIONS: &str = "gmp_mobile_collection_audit";
