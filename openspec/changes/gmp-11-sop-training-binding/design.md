## Context

GMP-regulated production requires that operators demonstrate completion of required training before performing critical operations. An SOP (Standard Operating Procedure) defines a critical operation, and operators must have completed the corresponding training to be authorized.

**Current State**:
- GMP-2 (Electronic Signature) is complete - provides identity verification
- GMP-9 (Workflow Engine) is being developed - enables workflow-driven processes
- No SOP or training management exists

**Requirements**:
- Define SOPs with version control
- Track operator training completion
- Bind workflow steps to required SOP qualifications
- Verify training before allowing workflow step execution

## Goals / Non-Goals

**Goals:**
- SOP CRUD with version tracking
- Training record management linked to SOPs
- Workflow step → SOP binding
- Training verification gate in workflow execution

**Non-Goals:**
- Full LMS (Learning Management System) functionality
- Training content management
- Recertification tracking
- Classroom/training session scheduling

## Decisions

### Decision 1: SOP Binding Granularity

**Option A: Workflow-level binding**
- Entire workflow requires specific training
- Coarse-grained, simpler

**Option B: Step-level binding (chosen)**
- Each workflow step can require specific SOP qualification
- More flexible, aligns with GMP requirements

**Decision**: Step-level binding allows fine-grained control per GMP requirement.

### Decision 2: Training Verification Timing

**Option A: At workflow start (chosen)**
- Verify all training before any step executes
- All-or-nothing approach

**Option B: At each step execution
- Verify training when step is about to execute
- More granular but more complex

**Decision**: At workflow start for Beta Gate - simpler and sufficient.

### Decision 3: SOP/Training Storage

**Option A: Separate GMP tables (chosen)**
- `sop` table for SOP definitions
- `training_record` table for completions
- Clean separation from operational data

**Option B: Extend existing workflow tables
- Pollutes workflow schema

**Decision**: Separate tables following GMP audit trail pattern.

## Risks / Trade-offs

| Risk | Impact | Mitigation |
|------|--------|------------|
| Training expires mid-workflow | Medium | Expiry date check, workflow restart |
| SOP version mismatch | Medium | Version pinning in binding |
| Large training records | Low | Indexed by operator and SOP |

## Open Questions

1. **Recertification**: How to handle expired training? (Future enhancement)
2. **Training equivalence**: Can training from one SOP count for another? (No for v3.2.0)
3. **Offline verification**: Required? (No for v3.2.0)
