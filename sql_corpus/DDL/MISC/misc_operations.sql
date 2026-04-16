-- === Miscellaneous DDL Operations Test Suite ===

-- === CASE: VACUUM command ===
-- EXPECT: success
VACUUM;

-- === CASE: ANALYZE command ===
-- EXPECT: success
ANALYZE;

-- === CASE: PRAGMA table_info ===
-- EXPECT: 5 rows
PRAGMA table_info('users');

-- === CASE: PRAGMA index_list ===
-- EXPECT: 3 rows
PRAGMA index_list('users');

-- === CASE: PRAGMA foreign_key_list ===
-- EXPECT: 2 rows
PRAGMA foreign_key_list('orders');

-- === CASE: DETACH database ===
-- EXPECT: success
DETACH DATABASE auxiliary;

-- === CASE: ATTACH database ===
-- EXPECT: success
ATTACH DATABASE 'test.db' AS auxiliary;

-- === CASE: REINDEX ===
-- EXPECT: success
REINDEX;

-- === CASE: REINDEX specific table ===
-- EXPECT: success
REINDEX users;

-- === CASE: CLOSE database ===
-- EXPECT: success
CLOSE;

-- === CASE: RELOAD schema ===
-- EXPECT: success
reload_schema();

-- === CASE: BEGIN DEFERRED transaction ===
-- EXPECT: success
BEGIN DEFERRED;

-- === CASE: BEGIN EXCLUSIVE transaction ===
-- EXPECT: success
BEGIN EXCLUSIVE;

-- === CASE: SAVEPOINT ===
-- EXPECT: success
SAVEPOINT sp1;

-- === CASE: RELEASE savepoint ===
-- EXPECT: success
RELEASE SAVEPOINT sp1;

-- === CASE: ROLLBACK to savepoint ===
-- EXPECT: success
ROLLBACK TO SAVEPOINT sp1;

-- === CASE: PRAGMA database_list ===
-- EXPECT: 2 rows
PRAGMA database_list;

-- === CASE: PRAGMA version ===
-- EXPECT: 1 row
PRAGMA version;

-- === CASE: PRAGMA compile_options ===
-- EXPECT: 5 rows
PRAGMA compile_options;

-- === CASE: DROP INDEX IF EXISTS ===
-- EXPECT: success
DROP INDEX IF EXISTS idx_temp;
