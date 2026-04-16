-- === Index Test Suite ===

-- === CASE: Create index ===
-- EXPECT: success
CREATE INDEX idx_users_email ON users(email);

-- === CASE: Create unique index ===
-- EXPECT: success
CREATE UNIQUE INDEX idx_users_name ON users(name);

-- === CASE: Create composite index ===
-- EXPECT: success
CREATE INDEX idx_orders_user_total ON orders(user_id, total);

-- === CASE: Drop index ===
-- EXPECT: success
DROP INDEX idx_users_email;

-- === CASE: Select with index hint ===
-- EXPECT: 5 rows
SELECT * FROM users USE INDEX (PRIMARY) WHERE id = 1;

-- === CASE: Select with IGNORE index hint ===
-- EXPECT: 10 rows
SELECT * FROM users IGNORE INDEX (idx_users_email) WHERE id > 0;

-- === CASE: Select with FORCE index hint ===
-- EXPECT: 8 rows
SELECT * FROM orders FORCE INDEX (idx_orders_user_total) WHERE user_id = 5;

-- === CASE: Index on expression ===
-- EXPECT: success
CREATE INDEX idx_users_email_lower ON users(LOWER(email));

-- === CASE: Index for ORDER BY optimization ===
-- EXPECT: 10 rows
SELECT * FROM orders ORDER BY user_id, total DESC;

-- === CASE: Index for range scan ===
-- EXPECT: 5 rows
SELECT * FROM orders WHERE user_id BETWEEN 1 AND 5;

-- === CASE: Index for LIKE optimization ===
-- EXPECT: 3 rows
SELECT * FROM users WHERE email LIKE '%@example.com';

-- === CASE: Composite index column order ===
-- EXPECT: 4 rows
SELECT * FROM orders WHERE user_id = 3 AND total > 100;

-- === CASE: Index on foreign key ===
-- EXPECT: success
CREATE INDEX idx_orders_user_fk ON orders(user_id);

-- === CASE: Show indexes ===
-- EXPECT: 3 rows
SELECT * FROM information_schema.indexes WHERE table_name = 'users';

-- === CASE: Index for COUNT optimization ===
-- EXPECT: 1 row
SELECT COUNT(*) FROM orders WHERE user_id = 5;
