# Changelog

All notable changes to SQLRustGo will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.1.0-draft] - 2026-03-05

### Added

- **Architecture**: LogicalPlan/PhysicalPlan separation for query execution
- **Architecture**: ExecutionEngine trait for pluggable executors
- **Architecture**: Client-Server architecture with async network layer
- **Feature**: HashJoin implementation for efficient join operations
- **Feature**: Connection pool support for multiple clients
- **Feature**: WHERE clause AND/OR logical operators support
- **Feature**: Expression evaluation for BinaryOp (+, -, *, /)
- **Feature**: TEXT column index support (hash-based)
- **Testing**: Performance benchmark framework with Criterion
- **Testing**: Test coverage improved to 90.66%

### Changed

- **Refactor**: Replaced unwrap with proper error propagation in executor
- **Refactor**: Improved error handling with SqlResult<T>
- **Docs**: Updated gate checklist with correct branch workflow
- **Docs**: Reorganized teaching materials (student/TA separation)

### Fixed

- **Fix**: Clippy warnings resolved (zero warnings)
- **Fix**: Rust 2021 compatibility (let chains syntax)
- **Fix**: Code formatting issues

### Security

- **Audit**: Dependency audit passed
- **Audit**: No sensitive information leakage

### Documentation

- **New**: DeepSeek evaluation report
- **New**: Improvement plan for v1.1.0-draft
- **New**: AI-CLI collaboration notice
- **New**: v1.3.0 version plan with observability track
- **New**: 2.0 architecture design documents
- **New**: Distributed interface design (3.0 preview)
- **New**: Teaching practice materials (student/TA handbooks)

## [1.0.0] - 2026-02-22

### Added

- **Core**: SQL parser supporting SELECT, INSERT, UPDATE, DELETE
- **Core**: B+ tree storage engine
- **Core**: Transaction support with WAL
- **Core**: Basic query execution
- **Testing**: Unit test framework
- **Docs**: Initial documentation

### Changed

- Initial release

---

## Version History

| Version | Date | Maturity | Notes |
|---------|------|----------|-------|
| v1.1.0-draft | 2026-03-05 | L3 Draft | Architecture upgrade, Clippy passed |
| v1.0.0 | 2026-02-22 | L3 GA | Initial release |

---

## Roadmap

- **v1.1.0**: Draft → Alpha → Beta → RC → GA
- **v1.2.0**: Performance optimization (vectorization, CBO)
- **v1.3.0**: Enterprise features (observability, MVCC)
- **v2.0**: Distributed architecture

---

*This changelog is maintained by yinglichina8848*
