-- === Window Functions Advanced Test Suite ===

-- === CASE: ROW_NUMBER ===
-- EXPECT: 10 rows
SELECT id, name, ROW_NUMBER() OVER (ORDER BY id) as row_num FROM users;

-- === CASE: RANK ===
-- EXPECT: 10 rows
SELECT id, name, RANK() OVER (ORDER BY id) as rank_val FROM users;

-- === CASE: DENSE_RANK ===
-- EXPECT: 10 rows
SELECT id, name, DENSE_RANK() OVER (ORDER BY id) as dense_rank_val FROM users;

-- === CASE: PERCENT_RANK ===
-- EXPECT: 10 rows
SELECT id, name, PERCENT_RANK() OVER (ORDER BY id) as percent_rank_val FROM users;

-- === CASE: CUME_DIST ===
-- EXPECT: 10 rows
SELECT id, name, CUME_DIST() OVER (ORDER BY id) as cume_dist_val FROM users;

-- === CASE: NTILE ===
-- EXPECT: 10 rows
SELECT id, name, NTILE(3) OVER (ORDER BY id) as tile_num FROM users;

-- === CASE: LAG ===
-- EXPECT: 10 rows
SELECT id, name, LAG(name) OVER (ORDER BY id) as prev_name FROM users;

-- === CASE: LEAD ===
-- EXPECT: 10 rows
SELECT id, name, LEAD(name) OVER (ORDER BY id) as next_name FROM users;

-- === CASE: FIRST_VALUE ===
-- EXPECT: 10 rows
SELECT id, name, FIRST_VALUE(name) OVER (ORDER BY id) as first_val FROM users;

-- === CASE: LAST_VALUE ===
-- EXPECT: 10 rows
SELECT id, name, LAST_VALUE(name) OVER (ORDER BY id) as last_val FROM users;

-- === CASE: NTH_VALUE ===
-- EXPECT: 10 rows
SELECT id, name, NTH_VALUE(name, 3) OVER (ORDER BY id) as third_val FROM users;

-- === CASE: Window with PARTITION BY ===
-- EXPECT: 10 rows
SELECT user_id, total, ROW_NUMBER() OVER (PARTITION BY user_id ORDER BY total) as row_num FROM orders;

-- === CASE: Window with multiple functions ===
-- EXPECT: 10 rows
SELECT id, name,
  ROW_NUMBER() OVER (ORDER BY id) as row_num,
  RANK() OVER (ORDER BY id) as rank_val,
  LAG(name) OVER (ORDER BY id) as prev_name,
  LEAD(name) OVER (ORDER BY id) as next_name
FROM users;

-- === CASE: Window with aggregate in frame ===
-- EXPECT: 10 rows
SELECT id, name,
  COUNT(*) OVER (ORDER BY id ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW) as running_count
FROM users;

-- === CASE: Window with FIRST_VALUE and IGNORE NULLS ===
-- EXPECT: 10 rows
SELECT id, name, FIRST_VALUE(name IGNORE NULLS) OVER (ORDER BY id) as first_non_null FROM users;
