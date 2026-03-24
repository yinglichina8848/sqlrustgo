# v1.8.0 Release Goals & Planning

> **Version**: v1.8.0  
> **Release Date**: 2026-03-24  
> **Status**: In Development  
> **Branch**: `develop/v1.8.0`

---

## Release Focus

SQL-92 Compliance Enhancement & Test Suite Building

---

## Core Features

### SQL-92 Features

| Feature | Issue | Status |
|---------|-------|--------|
| ALTER TABLE support | #761 | ✅ Merged |
| DECIMAL data type | #762 | ✅ Merged |
| JSON data type | #763 | ✅ Merged |
| Stored procedure tokens | #764 | ✅ Merged |
| Trigger tokens | #765 | ✅ Merged |
| CREATE INDEX / DROP INDEX | #766 | ✅ Merged |
| String & datetime functions | #767 | ✅ Merged |
| LIMIT/OFFSET pagination | #760 | ✅ Merged |
| INSERT SET syntax | #768 | ✅ Open |

### SQL-92 Test Suite

| Component | Status |
|-----------|--------|
| Test runner framework | ✅ Complete |
| DDL test cases | ✅ 5/5 |
| DML test cases | ✅ 2/2 |
| Query test cases | ✅ 2/2 |
| Types test cases | ✅ 2/2 |
| **Total** | **11/11 (100%)** |

---

## Technical Improvements

### Parser Enhancements

- SQL comment support (`--` and `/* */`)
- Enhanced error messages
- Tokenizer improvements

---

## Test Coverage

| Module | Coverage |
|--------|----------|
| sqlrustgo-parser | 137 tests ✅ |
| sqlrustgo (lib) | 13 tests ✅ |
| SQL-92 compliance | 11/11 ✅ |

---

## Milestones

| Date | Milestone |
|------|-----------|
| 2026-03-22 | SQL-92 core features merged |
| 2026-03-23 | Test suite framework |
| 2026-03-24 | Comment support + formatting |

---

## Next Steps

1. Complete INSERT SET syntax (#768)
2. Add more SQL-92 test cases
3. Begin Epic-05: Constraints & Foreign Keys

---

**Document Status**: In Progress  
**Last Updated**: 2026-03-24
