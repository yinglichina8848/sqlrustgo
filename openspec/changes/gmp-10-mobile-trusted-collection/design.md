## Context

Mobile devices (tablets, handheld scanners) are increasingly used on GMP-regulated production floors to collect data (weights, measurements, quality checks). However, data from mobile devices must meet the same integrity requirements as data from fixed systems: tamper-evident, timestamped, and traceable to device identity.

**Current State**:
- GMP-6 (Trusted Timestamp) is complete - provides RFC3161 timestamps
- GMP-9 (Workflow Engine) enables workflow-driven processes
- Audit chain exists for electronic signatures
- No mobile device trust mechanism exists

**Requirements**:
- Register mobile devices with public key infrastructure
- Each mobile collection record includes device signature
- Records are timestamped via GMP-6 trusted timestamp
- Audit chain records mobile collection events

## Goals / Non-Goals

**Goals:**
- Device registration with unique ID and public key
- Mobile data collection with device signature and timestamp
- Device trust verification before accepting data
- Integration with existing audit chain

**Non-Goals:**
- Full MDM (Mobile Device Management) functionality
- Device remote wipe or lock
- Offline mobile collection (requires network)
- Support for multiple signatures per record

## Decisions

### Decision 1: Device Registration Model

**Option A: Certificate-based (chosen)**
- Device stores X.509 certificate with public key
- Certificate fingerprint used as device ID
- Server validates certificate chain on registration

**Option B: Simple key pair**
- Generate Ed25519 key pair on device
- Send public key to server with device metadata
- Simpler but less industry-standard

**Decision**: Use X.509 certificate-based device identity (industry standard for GMP regulated environments).

### Decision 2: Mobile Collection Record Structure

**Option A: Inline with existing rows**
- Add `device_id`, `device_signature` columns to data tables
- Transparent but pollutes data schema

**Option B: Separate mobile_collection audit table (chosen)**
- Mobile collection creates audit record in separate table
- Links to main data via correlation ID
- Cleaner separation of concerns

**Decision**: Use separate `mobile_collection_audit` table for device signature and metadata.

### Decision 3: Collection Protocol

**Option A: Synchronous (chosen)**
- Mobile sends data → server timestamps → returns receipt
- Simpler consistency model

**Option B: Asynchronous with queuing**
- Mobile sends data → queued → processed → receipt
- Better for offline but more complex

**Decision**: Synchronous for Beta Gate - device sends data and waits for trusted timestamp receipt.

### Decision 4: Module Location

**Option A: New crate `sqlrustgo-mobile`**
- Clean separation
- Overkill for single feature

**Option B: New module in `sqlrustgo-gmp` (chosen)**
- Follows existing GMP module pattern
- Shares dependencies easily

**Decision**: Add `crates/gmp/src/mobile/` module.

## Risks / Trade-offs

| Risk | Impact | Mitigation |
|------|--------|------------|
| Device key compromise | High | Certificate revocation list, re-registration |
| Network failure during collection | Medium | Retry with idempotency key |
| Timestamp service unavailable | High | Queue with local timestamp, reconcile later |
| Large number of devices | Low | Device registry indexed by fingerprint |

## Open Questions

1. **Certificate issuance**: Who issues device certificates? (Self-signed vs CA)
2. **Device revocation**: How to handle lost/stolen devices?
3. **Offline tolerance**: Required for v3.2.0 or future version?
