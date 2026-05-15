## Why

GMP-regulated operations require that operators complete required training before performing critical procedures (SOPs). There must be a verifiable link between training completion records and the ability to execute workflow steps that require specific SOP qualifications. This ensures only trained personnel can perform regulated operations.

## What Changes

- **New**: `SOP` entity for Standard Operating Procedure definitions
- **New**: `TrainingRecord` entity for tracking operator training completion
- **New**: `SOPBinding` to link workflow steps to required SOP qualifications
- **New**: Training verification in workflow execution (GMP-9 integration)
- **New**: SQL statements for SOP and training management
- **Modified**: Workflow step execution to verify training before allowing operation

## Capabilities

### New Capabilities

- `sop-management`: Define Standard Operating Procedures with version control and required qualifications
- `training-record`: Track operator training completion for specific SOPs
- `sop-binding`: Bind workflow steps (from GMP-9) to required SOP qualifications
- `training-verification`: Verify operator has completed required training before workflow step execution

### Modified Capabilities

- `workflow-engine`: Workflow engine (GMP-9) will check operator training qualifications before executing bound steps

## Impact

- **New module**: `crates/gmp/src/sop/` with SOP management and training verification
- **Parser**: New statement types for SOP and training CRUD operations
- **Executor**: Training verification in workflow step execution
- **Planner**: Integration with workflow planning for SOP binding
- **Tests**: `tests/gmp/sop_test.rs`
