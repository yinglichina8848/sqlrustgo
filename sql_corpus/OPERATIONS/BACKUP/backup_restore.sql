-- === SQLRustGo Backup and Restore Test Suite ===
-- Tests for backup and restore functionality

-- === SETUP ===
CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, email TEXT);
INSERT INTO users VALUES (1, 'Alice', 'alice@example.com');
INSERT INTO users VALUES (2, 'Bob', 'bob@example.com');
INSERT INTO users VALUES (3, 'Charlie', 'charlie@example.com');

CREATE TABLE orders (id INTEGER PRIMARY KEY, user_id INTEGER, amount REAL);
INSERT INTO orders VALUES (1, 1, 100.50);
INSERT INTO orders VALUES (2, 1, 250.00);
INSERT INTO orders VALUES (3, 2, 75.25);

-- === CASE: Full Backup ===
-- EXPECT: success
BACKUP DATABASE TO '/backup/test_backup_001';

-- === CASE: List Backups ===
-- EXPECT: 1 row
-- Note: This would be a meta-query to list available backups

-- === CASE: Restore from Backup ===
-- EXPECT: success
RESTORE DATABASE FROM '/backup/test_backup_001';

-- === CASE: Incremental Backup ===
-- EXPECT: success
BACKUP DATABASE TO '/backup/incremental_001' INCREMENTAL;

-- === CASE: Differential Backup ===
-- EXPECT: success
BACKUP DATABASE TO '/backup/differential_001' DIFFERENTIAL;

-- === CASE: Backup with Compression ===
-- EXPECT: success
BACKUP DATABASE TO '/backup/compressed_001' COMPRESSED;

-- === CASE: Backup Verification ===
-- EXPECT: 3 rows (original table row count)
SELECT COUNT(*) FROM users;

-- === CASE: Point-in-time Restore ===
-- EXPECT: success
RESTORE DATABASE FROM '/backup/test_backup_001' POINT IN TIME '2026-05-07 10:00:00';

-- === CASE: Backup Metadata Check ===
-- EXPECT: success
-- Note: Would query backup metadata

-- === CASE: Cross-database Restore ===
-- EXPECT: success
RESTORE DATABASE FROM '/backup/test_backup_001' TO 'restored_db';

-- === TEARDOWN ===
DROP TABLE orders;
DROP TABLE users;
