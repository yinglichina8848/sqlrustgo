-- === SKIP ===

-- === DateTime Operations Test Suite ===

-- === CASE: CURRENT_DATE ===
-- EXPECT: 1 row
SELECT CURRENT_DATE as today;

-- === CASE: CURRENT_TIME ===
-- EXPECT: 1 row
SELECT CURRENT_TIME as now_time;

-- === CASE: CURRENT_TIMESTAMP ===
-- EXPECT: 1 row
SELECT CURRENT_TIMESTAMP as now_timestamp;

-- === CASE: DATE function ===
-- EXPECT: 1 row
SELECT DATE('2024-01-15 10:30:45') as extracted_date;

-- === CASE: TIME function ===
-- EXPECT: 1 row
SELECT TIME('2024-01-15 10:30:45') as extracted_time;

-- === CASE: DATETIME function ===
-- EXPECT: 1 row
SELECT DATETIME('2024-01-15 10:30:45') as extracted_datetime;

-- === CASE: DATE_ADD ===
-- EXPECT: 1 row
SELECT DATE('2024-01-15', '+7 days') as next_week;

-- === CASE: DATE_SUB ===
-- EXPECT: 1 row
SELECT DATE('2024-01-15', '-7 days') as last_week;

-- === CASE: STRFTIME with %Y ===
-- EXPECT: 1 row
SELECT STRFTIME('%Y', '2024-01-15') as year_value;

-- === CASE: STRFTIME with %m ===
-- EXPECT: 1 row
SELECT STRFTIME('%m', '2024-01-15') as month_value;

-- === CASE: STRFTIME with %d ===
-- EXPECT: 1 row
SELECT STRFTIME('%d', '2024-01-15') as day_value;

-- === CASE: STRFTIME with %H ===
-- EXPECT: 1 row
SELECT STRFTIME('%H', '2024-01-15 10:30:45') as hour_value;

-- === CASE: JULIANDAY function ===
-- EXPECT: 1 row
SELECT JULIANDAY('2024-01-15') - JULIANDAY('2024-01-01') as days_in_month;

-- === CASE: DATE with INTERVAL ===
-- EXPECT: 1 row
SELECT DATE('now', '+1 month') as next_month;

-- === CASE: TIME with INTERVAL ===
-- EXPECT: 1 row
SELECT TIME('now', '+1 hour') as next_hour;

-- === CASE: Date comparison ===
-- EXPECT: 5 rows
SELECT * FROM orders WHERE order_date > '2024-01-01';

-- === CASE: Date BETWEEN ===
-- EXPECT: 5 rows
SELECT * FROM orders WHERE order_date BETWEEN '2024-01-01' AND '2024-12-31';

-- === CASE: Date extraction in WHERE ===
-- EXPECT: 5 rows
SELECT * FROM orders WHERE STRFTIME('%Y', order_date) = '2024';

-- === CASE: NOW function ===
-- EXPECT: 1 row
SELECT NOW() as current_datetime;
