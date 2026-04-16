-- === Schema and Metadata Test Suite ===

-- === CASE: SELECT from sqlite_master ===
-- EXPECT: 10 rows
SELECT * FROM sqlite_master WHERE type='table';

-- === CASE: SELECT from sqlite_master with name filter ===
-- EXPECT: 1 row
SELECT * FROM sqlite_master WHERE name='users' AND type='table';

-- === CASE: SELECT from information_schema ===
-- EXPECT: 5 rows
SELECT * FROM information_schema.tables WHERE table_schema='main';

-- === CASE: SELECT column info from information_schema ===
-- EXPECT: 3 rows
SELECT * FROM information_schema.columns WHERE table_name='users';

-- === CASE: SELECT index info ===
-- EXPECT: 3 rows
SELECT * FROM information_schema.indexes WHERE table_name='users';

-- === CASE: SELECT view definitions ===
-- EXPECT: 2 rows
SELECT * FROM sqlite_master WHERE type='view';

-- === CASE: SELECT trigger definitions ===
-- EXPECT: 1 row
SELECT * FROM sqlite_master WHERE type='trigger';

-- === CASE: TABLE INFO pragma ===
-- EXPECT: 5 rows
PRAGMA table_info('users');

-- === === CASE: INDEX LIST pragma ===
-- EXPECT: 3 rows
PRAGMA index_list('users');

-- === CASE: INDEX INFO pragma ===
-- EXPECT: 2 rows
PRAGMA index_info('idx_users_email');

-- === CASE: DATABASE LIST pragma ===
-- EXPECT: 1 row
PRAGMA database_list;

-- === CASE: FUNCTION LIST pragma ===
-- EXPECT: 10 rows
PRAGMA function_list;

-- === CASE: MODULE LIST pragma ===
-- EXPECT: 5 rows
PRAGMA module_list;

-- === CASE: COLLATION LIST pragma ===
-- EXPECT: 5 rows
PRAGMA collation_list;

-- === CASE: FOREIGN KEY LIST pragma ===
-- EXPECT: 2 rows
PRAGMA foreign_key_list('orders');

-- === CASE: TABLE XINFO (with hidden columns) ===
-- EXPECT: 5 rows
PRAGMA table_xinfo('users');
