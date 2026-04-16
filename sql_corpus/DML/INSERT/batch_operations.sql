-- === Batch Operations Test Suite ===

-- === CASE: Batch INSERT with multiple rows ===
-- EXPECT: 5 rows affected
INSERT INTO users (id, name, email) VALUES
  (201, 'Batch1', 'batch1@example.com'),
  (202, 'Batch2', 'batch2@example.com'),
  (203, 'Batch3', 'batch3@example.com'),
  (204, 'Batch4', 'batch4@example.com'),
  (205, 'Batch5', 'batch5@example.com');

-- === CASE: Batch INSERT with SELECT ===
-- EXPECT: 5 rows affected
INSERT INTO users (id, name, email)
SELECT id + 300, name, email FROM users WHERE id < 6;

-- === CASE: Batch UPDATE with subquery ===
-- EXPECT: 5 rows affected
UPDATE users SET email = 'batch_updated@example.com' WHERE id IN (SELECT id FROM users WHERE id BETWEEN 201 AND 205);

-- === CASE: Batch DELETE with subquery ===
-- EXPECT: 5 rows affected
DELETE FROM users WHERE id IN (SELECT id FROM users WHERE id BETWEEN 201 AND 205);

-- === CASE: Batch INSERT with DEFAULT values ===
-- EXPECT: 3 rows affected
INSERT INTO users (id, name, email) VALUES
  (301, 'Default1', 'default1@example.com'),
  (302, 'Default2', DEFAULT),
  (303, 'Default3', 'default3@example.com');

-- === CASE: Batch INSERT with NULL values ===
-- EXPECT: 3 rows affected
INSERT INTO users (id, name, email) VALUES
  (304, 'Null1', NULL),
  (305, 'Null2', NULL),
  (306, 'Null3', NULL);

-- === CASE: Batch UPDATE multiple columns ===
-- EXPECT: 3 rows affected
UPDATE users SET email = 'multi_col_' || email, name = 'Updated_' || name WHERE id BETWEEN 301 AND 303;

-- === CASE: Batch DELETE with LIMIT ===
-- EXPECT: 2 rows affected
DELETE FROM users WHERE id IN (SELECT id FROM users WHERE id BETWEEN 304 AND 306 LIMIT 2);

-- === CASE: Batch INSERT with ON CONFLICT ===
-- EXPECT: success
INSERT INTO users (id, name, email) VALUES
  (1, 'Conflict1', 'conflict1@example.com'),
  (2, 'Conflict2', 'conflict2@example.com')
ON CONFLICT (id) DO UPDATE SET email = excluded.email;

-- === CASE: Batch INSERT with CTE ===
-- EXPECT: 2 rows affected
WITH new_users AS (
  SELECT 401 as id, 'CTE1' as name, 'cte1@example.com' as email
  UNION ALL
  SELECT 402, 'CTE2', 'cte2@example.com'
)
INSERT INTO users SELECT * FROM new_users;

-- === CASE: Batch UPDATE with JOIN ===
-- EXPECT: 5 rows affected
UPDATE users u
SET u.email = 'joined_' || u.email
FROM (SELECT id FROM users WHERE id BETWEEN 1 AND 5) AS upd
WHERE u.id = upd.id;

-- === CASE: Batch DELETE with JOIN ===
-- EXPECT: 3 rows affected
DELETE FROM users u
USING (SELECT id FROM users WHERE id BETWEEN 401 AND 403) AS del
WHERE u.id = del.id;

-- === CASE: Batch INSERT from multiple tables ===
-- EXPECT: 10 rows affected
INSERT INTO users (id, name, email)
SELECT id + 500, name, email FROM users WHERE id < 6
UNION ALL
SELECT id + 600, name, email FROM users WHERE id >= 6 AND id < 11;
