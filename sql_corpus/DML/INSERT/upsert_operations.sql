-- === Upsert Operations Test Suite ===

-- === CASE: INSERT ON CONFLICT DO NOTHING ===
-- EXPECT: success
INSERT INTO users (id, name, email) VALUES (1, 'Conflict', 'conflict@example.com')
ON CONFLICT (id) DO NOTHING;

-- === CASE: INSERT ON CONFLICT DO UPDATE ===
-- EXPECT: success
INSERT INTO users (id, name, email) VALUES (1, 'Updated', 'updated@example.com')
ON CONFLICT (id) DO UPDATE SET email = excluded.email;

-- === CASE: INSERT ON CONFLICT with WHERE ===
-- EXPECT: success
INSERT INTO users (id, name, email) VALUES (2, 'Conditional', 'cond@example.com')
ON CONFLICT (id) DO UPDATE SET email = excluded.email WHERE excluded.id > 10;

-- === CASE: INSERT ON CONFLICT multiple columns ===
-- EXPECT: success
INSERT INTO users (id, name, email) VALUES (3, 'MultiCol', 'multicol@example.com')
ON CONFLICT (id, email) DO UPDATE SET name = excluded.name;

-- === CASE: INSERT OR ROLLBACK ===
-- EXPECT: success
INSERT OR ROLLBACK INTO users (id, name, email) VALUES (100, 'Rollback', 'rollback@example.com');

-- === CASE: INSERT OR ABORT ===
-- EXPECT: success
INSERT OR ABORT INTO users (id, name, email) VALUES (101, 'Abort', 'abort@example.com');

-- === CASE: INSERT OR FAIL ===
-- EXPECT: success
INSERT OR FAIL INTO users (id, name, email) VALUES (102, 'Fail', 'fail@example.com');

-- === CASE: INSERT OR IGNORE ===
-- EXPECT: success
INSERT OR IGNORE INTO users (id, name, email) VALUES (1, 'Ignore', 'ignore@example.com');

-- === CASE: INSERT OR REPLACE ===
-- EXPECT: success
INSERT OR REPLACE INTO users (id, name, email) VALUES (1, 'Replace', 'replace@example.com');

-- === CASE: UPSERT with increment ===
-- EXPECT: success
INSERT INTO order_counts (user_id, order_count) VALUES (1, 1)
ON CONFLICT (user_id) DO UPDATE SET order_count = order_count + 1;

-- === CASE: UPSERT with default ===
-- EXPECT: success
INSERT INTO users (id, name) VALUES (200, 'DefaultEmail')
ON CONFLICT (id) DO UPDATE SET name = excluded.name;

-- === CASE: UPSERT with multiple conflicts ===
-- EXPECT: success
INSERT INTO users (id, name, email) VALUES (5, 'MultiConflict', 'multi@example.com')
ON CONFLICT (id) DO UPDATE SET email = excluded.email
ON CONFLICT (email) DO UPDATE SET name = excluded.name;

-- === CASE: UPSERT with NULL handling ===
-- EXPECT: success
INSERT INTO users (id, name, email) VALUES (6, 'NullEmail', NULL)
ON CONFLICT (id) DO UPDATE SET email = COALESCE(excluded.email, users.email);
