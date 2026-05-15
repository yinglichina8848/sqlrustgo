## Context

Measurement devices (scales, thermometers, pH meters, spectrometers) require regular calibration to ensure measurement accuracy. GMP regulations mandate calibration records be maintained and measurements only be taken with calibrated equipment.

**Current State**:
- GMP-6 (Trusted Timestamp) is complete - provides timestamps
- GMP-10 (Mobile Collection) handles mobile data collection
- No device calibration tracking exists

**Requirements**:
- Track measurement devices with calibration intervals
- Record calibration events with standards used
- Verify device calibration before accepting measurements
- Provide calibration audit trail

## Goals / Non-Goals

**Goals:**
- Device registration with calibration intervals
- Calibration event recording with results
- Calibration status verification (CURRENT, DUE, EXPIRED)
- Integration with mobile collection for pre-measurement verification

**Non-Goals:**
- Automatic calibration scheduling
- Calibration certificate management
- Calibration prediction/forecasting
- Integration with external calibration labs

## Decisions

### Decision 1: Calibration Status Model

**Option A: Binary (calibrated/not)**
- Simple but insufficient
- Doesn't capture "due soon" state

**Option B: Tri-state (chosen)**
- CURRENT: Calibration within interval, results within tolerance
- DUE: Calibration interval approaching or exceeded
- EXPIRED: Past calibration interval or failed calibration

**Decision**: Tri-state model provides proper GMP compliance.

### Decision 2: Calibration Verification Timing

**Option A: At measurement time (chosen)**
- Verify device is calibrated before accepting each measurement
- Immediate feedback to operator

**Option B: At data review time
- Verify during data review rather than collection
- May accept then reject

**Decision**: At measurement time for immediate GMP compliance.

### Decision 3: Tolerance Criteria

**Option A: Binary pass/fail (chosen)**
- Calibration either passes or fails tolerance
- Simpler for GMP audit

**Option B: Range with warning
- Allow slight deviations with warning
- More complex

**Decision**: Binary pass/fail for GMP clarity.

## Risks / Trade-offs

| Risk | Impact | Mitigation |
|------|--------|------------|
| Calibration expiry during batch | Medium | Clear status indicators, batch rejection |
| Missing calibration records | High | Required before device use |
| Clock drift | Low | Trusted timestamp (GMP-6) |

## Open Questions

1. **Calibration certificate upload**: Required? (Not for v3.2.0)
2. **Automatic reminders**: Needed? (Future enhancement)
3. **Calibration standards**: Track reference standards? (Future enhancement)
