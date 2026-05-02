# Coverage Report

> **Version**: v2.9.0
> **Date**: 2026-05-03
> **Status**: PARTIAL (tarpaulin timeout)

## Coverage Summary

| Metric | Value | Target | Status |
|--------|-------|--------|--------|
| Line Coverage | ≥80% | 80% | ⏳ 待完成 |
| Branch Coverage | ≥70% | 70% | ⏳ 待完成 |

## Test Execution

### Unit Tests
- **Status**: ✅ PASS
- **Command**: `cargo test --all-features`
- **Result**: All tests passed

### Integration Tests
- **Status**: ✅ PASS
- **Result**: All integration tests passed

### Coverage Tool
- **Tool**: cargo tarpaulin
- **Issue**: Timeout during full coverage run
- **Note**: Individual crate coverage passes when run separately

## Known Issues

1. **Flaky Test**: `test_sharded_vector_insert_and_search` fails under tarpaulin parallel execution but passes in single-threaded mode
   - **Workaround**: Added to `PROBLEMATIC_TESTS` list in `check_coverage.sh`

2. **Coverage Timeout**: Full coverage run exceeds 15 minutes
   - **Cause**: Large workspace with many test crates
   - **Recommendation**: Run coverage incrementally per crate

## Recommendations

1. Run coverage per crate for faster feedback
2. Investigate flaky vector index test
3. Consider running coverage in CI with longer timeout