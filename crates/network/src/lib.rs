//! Network module for SQLRustGo
//!
//! Provides network communication capabilities including DTC (Distributed Transaction Coordinator).

/// DTC (Distributed Transaction Coordinator) module
/// Generated from protobuf definitions
pub mod dtc {
    include!(concat!(env!("OUT_DIR"), "/sqlrustgo.dtc.rs"));
}
