-- SQLCorpus: Window Functions
-- Tests for SQL window functions

-- === SETUP ===
CREATE TABLE sales (id INTEGER PRIMARY KEY, employee TEXT, region TEXT, amount INTEGER);
INSERT INTO sales VALUES (1, 'Alice', 'North', 100);
INSERT INTO sales VALUES (2, 'Alice', 'South', 150);
INSERT INTO sales VALUES (3, 'Bob', 'North', 200);
INSERT INTO sales VALUES (4, 'Bob', 'South', 175);
INSERT INTO sales VALUES (5, 'Charlie', 'North', 125);

-- === CASE: row_number ===
SELECT ROW_NUMBER() OVER (ORDER BY amount DESC) as rn, employee, amount FROM sales;
-- EXPECT: 5 rows

-- === CASE: rank ===
SELECT RANK() OVER (ORDER BY amount DESC) as rk, employee, amount FROM sales;
-- EXPECT: 5 rows

-- === CASE: dense_rank ===
SELECT DENSE_RANK() OVER (ORDER BY amount DESC) as dr, employee, amount FROM sales;
-- EXPECT: 5 rows

-- === CASE: percent_rank ===
SELECT PERCENT_RANK() OVER (ORDER BY amount) as pr, employee, amount FROM sales;
-- EXPECT: 5 rows

-- === CASE: cume_dist ===
SELECT CUME_DIST() OVER (ORDER BY amount) as cd, employee, amount FROM sales;
-- EXPECT: 5 rows

-- === CASE: ntile ===
SELECT NTILE(2) OVER (ORDER BY amount) as quartile, employee, amount FROM sales;
-- EXPECT: 5 rows

-- === CASE: lag ===
SELECT LAG(amount) OVER (ORDER BY id) as prev_amount, employee, amount FROM sales;
-- EXPECT: 5 rows

-- === CASE: lead ===
SELECT LEAD(amount) OVER (ORDER BY id) as next_amount, employee, amount FROM sales;
-- EXPECT: 5 rows

-- === CASE: first_value ===
SELECT FIRST_VALUE(amount) OVER (ORDER BY id) as first, employee, amount FROM sales;
-- EXPECT: 5 rows

-- === CASE: last_value ===
SELECT LAST_VALUE(amount) OVER (ORDER BY id) as last, employee, amount FROM sales;
-- EXPECT: 5 rows

-- === CASE: nth_value ===
SELECT NTH_VALUE(amount, 2) OVER (ORDER BY id) as second, employee, amount FROM sales;
-- EXPECT: 5 rows

-- === CASE: sum_over ===
SELECT employee, amount, SUM(amount) OVER (PARTITION BY employee) as emp_total FROM sales;
-- EXPECT: 5 rows

-- === CASE: count_over ===
SELECT employee, COUNT(*) OVER (PARTITION BY employee) as emp_count FROM sales;
-- EXPECT: 5 rows

-- === CASE: avg_over ===
SELECT region, AVG(amount) OVER (PARTITION BY region) as region_avg FROM sales;
-- EXPECT: 5 rows