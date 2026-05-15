## Why

GMP-compliant manufacturing requires trusted data collection from mobile devices (tablets, handheld scanners) used on the production floor. Data collected must be cryptographically signed, timestamped, and tamper-evident to meet regulatory requirements. This enables mobile-first GMP workflows while maintaining data integrity.

## What Changes

- **New**: `MobileDevice` entity for registering and managing trusted mobile devices
- **New**: `MobileCollection` module for handling data collection from mobile devices
- **New**: Device trust chain verification (device registration → data collection → audit chain)
- **New**: Integration with GMP-6 Trusted Timestamp for collection timestamping
- **New**: Mobile collection statement types (REGISTER DEVICE, COLLECT DATA, DEVICE HEARTBEAT)
- **Modified**: Audit chain to support mobile collection records

## Capabilities

### New Capabilities

- `mobile-device-registration`: Register mobile devices with unique device IDs, public keys, and trust credentials for GMP-compliant data collection
- `mobile-data-collection`: Collect data from registered mobile devices with cryptographic signatures and trusted timestamps
- `device-trust-chain`: Verify device identity and trust status before accepting mobile-collected data

### Modified Capabilities

- `audit-chain`: Audit chain will be extended to store mobile collection records with device signature verification

## Impact

- **New crate module**: `crates/gmp/src/mobile/` with device registration and collection logic
- **Parser**: New statement types for mobile device operations
- **Executor**: New execution engine handlers for mobile collection
- **Audit chain**: New record type for mobile collection entries
- **Tests**: `tests/gmp/mobile_integration_test.rs`
