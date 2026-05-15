# Mobile Data Collection

## ADDED Requirements

### Requirement: Mobile data collection with signature
The system SHALL accept data collections from registered mobile devices, where each collection includes the device signature over the data payload.

#### Scenario: Successful mobile collection
- **WHEN** a registered mobile device submits COLLECT DATA with device_id, data_payload, device_signature, and collection_timestamp
- **THEN** the system verifies device signature using device's public key
- **AND** applies trusted timestamp (GMP-6) to the collection
- **AND** stores the collection in mobile_collection_audit table
- **AND** returns a collection receipt with audit_id

#### Scenario: Collection from unregistered device
- **WHEN** a device that is not registered submits COLLECT DATA
- **THEN** the system rejects the collection with error DEVICE_NOT_REGISTERED

#### Scenario: Collection from suspended device
- **WHEN** a suspended device submits COLLECT DATA
- **THEN** the system rejects the collection with error DEVICE_SUSPENDED

### Requirement: Trusted timestamp integration
The system SHALL apply GMP-6 trusted timestamps to all mobile collections.

#### Scenario: Collection with trusted timestamp
- **WHEN** a mobile device submits COLLECT DATA
- **THEN** the system obtains an RFC3161 trusted timestamp
- **AND** includes the timestamp in the mobile_collection_audit record

### Requirement: Device heartbeat
The system SHALL accept DEVICE HEARTBEAT messages to confirm device connectivity.

#### Scenario: Heartbeat from registered device
- **WHEN** a registered device submits DEVICE HEARTBEAT with device_id
- **THEN** the system updates the device's last_seen timestamp
- **AND** returns a heartbeat receipt
