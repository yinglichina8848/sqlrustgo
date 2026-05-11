# v3.1.0 Quick Start Guide

> **Version**: 3.1.0
> **Time to complete**: ~10 minutes

---

## 1. Install (2 min)

```bash
# Build from source
git clone http://192.168.0.252:3000/openclaw/sqlrustgo.git
cd sqlrustgo
git checkout develop/v3.1.0
cargo build --release --all-features
```

---

## 2. Start Server (1 min)

```bash
# Create data directory
mkdir -p /tmp/sqlrustgo_data

# Start server in background
./target/release/sqlrustgo \
  --data-dir /tmp/sqlrustgo_data \
  --bind 127.0.0.1:3306 &

# Wait for startup
sleep 2
```

---

## 3. Connect (1 min)

```bash
# Connect with MySQL client
mysql -h 127.0.0.1 -P 3306 -u root

# Or use sqlrustgo CLI
./target/release/sqlrustgo cli --data-dir /tmp/sqlrustgo_data
```

---

## 4. Create Database and Table (2 min)

```sql
-- Create database
CREATE DATABASE IF NOT EXISTS demo;

-- Use database
USE demo;

-- Create table
CREATE TABLE users (
    id INT PRIMARY KEY AUTO_INCREMENT,
    name VARCHAR(100) NOT NULL,
    email VARCHAR(255) UNIQUE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Insert data
INSERT INTO users (name, email) VALUES
    ('Alice', 'alice@example.com'),
    ('Bob', 'bob@example.com'),
    ('Charlie', 'charlie@example.com');

-- Query
SELECT * FROM users WHERE id = 1;
```

---

## 5. Try New Features (3 min)

### INFORMATION_SCHEMA

```sql
-- Check table metadata
SELECT TABLE_NAME, TABLE_ROWS
FROM information_schema.TABLES
WHERE TABLE_SCHEMA = 'demo';

-- Check column details
SELECT COLUMN_NAME, DATA_TYPE, IS_NULLABLE
FROM information_schema.COLUMNS
WHERE TABLE_SCHEMA = 'demo';
```

### MERGE Statement

```sql
-- Create source table
CREATE TABLE users_staging (
    id INT,
    name VARCHAR(100),
    email VARCHAR(255)
);

INSERT INTO users_staging VALUES
    (1, 'Alice Updated', 'alice.new@example.com'),
    (4, 'Dave', 'dave@example.com');

-- Merge
MERGE INTO users AS t
USING users_staging AS s
ON t.id = s.id
WHEN MATCHED THEN
    UPDATE SET t.name = s.name, t.email = s.email
WHEN NOT MATCHED THEN
    INSERT (id, name, email) VALUES (s.id, s.name, s.email);
```

### SAVEPOINT

```sql
BEGIN;
INSERT INTO users (name, email) VALUES ('Test', 'test@test.com');
SAVEPOINT sp1;
UPDATE users SET name = 'Updated' WHERE email = 'test@test.com';
ROLLBACK TO SAVEPOINT sp1;
COMMIT;

-- Verify: 'Test' row exists, not 'Updated'
SELECT * FROM users WHERE email = 'test@test.com';
```

### EXPLAIN with CostModel

```sql
-- Create index
CREATE INDEX idx_email ON users(email);

-- Check query plan
EXPLAIN SELECT * FROM users WHERE email = 'alice@example.com';
```

---

## 6. Run Tests (1 min)

```bash
# Run SQL corpus
cargo test -p sqlrustgo-sql-corpus

# Run specific test
cargo test --test merge_test

# Run coverage
cargo llvm-cov --all-features --lib --summary-only
```

---

## What's Next?

| Topic | Doc |
|-------|-----|
| Full installation | `INSTALL.md` |
| Configuration | `DEPLOYMENT_GUIDE.md` |
| Performance tuning | `PERFORMANCE_TARGETS.md` |
| GMP features | `GMP_COMPLIANCE_ROADMAP.md` |
| Complete feature list | `FEATURE_MATRIX.md` |

---

## Common Commands

```bash
# Stop server
pkill sqlrustgo

# Reset data
rm -rf /tmp/sqlrustgo_data
mkdir /tmp/sqlrustgo_data

# Check version
./target/release/sqlrustgo --version

# View logs
tail -f /tmp/sqlrustgo_data/log/sqlrustgo.log
```
