## ADDED Requirements

### Requirement: Cloud KMS Provider Interface
The system SHALL provide KMS providers for AWS/Azure/GCP.

#### Scenario: Create AWS KMS provider
- **WHEN** AwsKmsProvider::new(config) is called
- **THEN** provider connects to AWS KMS and is ready

### Requirement: Cloud KMS Signing
The system SHALL delegate signing to cloud KMS.

#### Scenario: Sign via AWS KMS
- **WHEN** provider.sign(key_id, data) is called
- **THEN** request is sent to AWS KMS and signature is returned

### Requirement: Multi-Cloud Support
The system SHALL support multiple cloud KMS providers.

#### Scenario: Support Azure KMS
- **WHEN** AzureKmsProvider::new(config) is called
- **THEN** provider connects to Azure Key Vault
