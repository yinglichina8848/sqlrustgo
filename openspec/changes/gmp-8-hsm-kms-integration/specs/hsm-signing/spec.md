## ADDED Requirements

### Requirement: Unified HSM Signing Interface
The system SHALL provide a unified interface for hardware signing.

#### Scenario: Sign via unified interface
- **WHEN** HsmSigning::new(provider).sign(key_id, data) is called
- **THEN** signing is delegated to configured provider

### Requirement: Provider Fallback
The system SHALL support fallback to alternate provider on failure.

#### Scenario: Fallback on primary failure
- **WHEN** primary provider.sign() fails
- **THEN** system attempts fallback provider if configured

### Requirement: Signing Metrics
The system SHALL track signing operation metrics.

#### Scenario: Record signing metrics
- **WHEN** sign operation completes
- **THEN** latency and provider info are recorded for monitoring
