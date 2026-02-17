# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.0-alpha] - 2026-02-16

### Added
- SQL-92 DDL support (CREATE TABLE, DROP TABLE)
- SQL-92 DML support (SELECT, INSERT, UPDATE, DELETE)
- B+ Tree index implementation
- Buffer Pool with LRU eviction
- FileStorage for persistent storage
- Write-Ahead Log (WAL) for transaction safety
- MySQL-style network protocol
- Interactive REPL (Read-Eval-Print Loop)
- Comprehensive test suite (60+ tests)

### Features
- WHERE clause with comparison operators (=, >, <, >=, <=)
- WHERE clause with logical operators (AND, OR)
- Multiple INSERT values support
- Column type support: INTEGER, FLOAT, TEXT, BOOLEAN, BLOB

### Documentation
- README.md with quick start guide
- Developer documentation (DEVELOP.md)
- Contributing guidelines (CONTRIBUTING.md)

### Limitations (Alpha)
- No JOIN support
- No subquery support
- No transaction nesting
- No constraint support (foreign key, unique key)
- No index auto-selection

[Unreleased]: https://github.com/yinglichina8848/sqlrustgo/compare/v1.0.0-alpha...HEAD
[1.0.0-alpha]: https://github.com/yinglichina8848/sqlrustgo/releases/tag/v1.0.0-alpha
