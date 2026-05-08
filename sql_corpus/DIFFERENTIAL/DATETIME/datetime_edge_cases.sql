-- === DIFFERENTIAL TEST: DateTime Edge Cases ===
-- Purpose: Compare datetime edge case handling between SQLRustGo and standard SQL
-- Critical: Date/time semantics vary significantly between engines

-- === SETUP ===
CREATE TABLE datetime_test (id INT PRIMARY KEY, dt DATETIME, d DATE, t TIME);

INSERT INTO datetime_test VALUES (1, '2024-01-01 00:00:00', '2024-01-01', '00:00:00');
INSERT INTO datetime_test VALUES (2, '2024-12-31 23:59:59', '2024-12-31', '23:59:59');
INSERT INTO datetime_test VALUES (3, '2024-02-29 12:00:00', '2024-02-29', '12:30:30');
INSERT INTO datetime_test VALUES (4, '1969-12-31 23:59:59', '1969-12-31', '23:59:59');
INSERT INTO datetime_test VALUES (5, '1970-01-01 00:00:00', '1970-01-01', '00:00:00');
INSERT INTO datetime_test VALUES (6, '1000-01-01 00:00:00', '1000-01-01', '00:00:00');
INSERT INTO datetime_test VALUES (7, '9999-12-31 23:59:59', '9999-12-31', '23:59:59');

-- === CASE: Leap year boundary ===
SELECT * FROM datetime_test WHERE d = '2024-02-29';
-- EXPECT: id 3

-- === CASE: Non-leap year February 29 ===
SELECT * FROM datetime_test WHERE d = '2023-02-29';
-- EXPECT: 0 rows

-- === CASE: Year 2038 overflow (UNIX timestamp) ===
SELECT * FROM datetime_test WHERE dt >= '2038-01-19 03:14:07';
-- EXPECT: varies by engine

-- === CASE: Year 2038 exactly ===
SELECT * FROM datetime_test WHERE dt = '2038-01-19 03:14:07';
-- EXPECT: 0 rows

-- === CASE: Pre-1970 datetime ===
SELECT * FROM datetime_test WHERE dt < '1970-01-01 00:00:00';
-- EXPECT: id 4, 6

-- === CASE: Year 1970 boundary ===
SELECT * FROM datetime_test WHERE dt >= '1970-01-01 00:00:00';
-- EXPECT: id 1, 2, 3, 5, 7

-- === CASE: Date before 1000 ===
SELECT * FROM datetime_test WHERE d < '1000-01-02';
-- EXPECT: id 6

-- === CASE: Date after 9999 ===
SELECT * FROM datetime_test WHERE d > '9999-12-30';
-- EXPECT: id 7

-- === CASE: Time midnight boundary ===
SELECT * FROM datetime_test WHERE t = '00:00:00';
-- EXPECT: id 1, 5, 6

-- === CASE: Time end of day ===
SELECT * FROM datetime_test WHERE t = '23:59:59';
-- EXPECT: id 2, 4, 7

-- === CASE: DATE function with datetime ===
SELECT DATE(dt) AS date_only FROM datetime_test WHERE id = 1;
-- EXPECT: 2024-01-01

-- === CASE: TIME function with datetime ===
SELECT TIME(dt) AS time_only FROM datetime_test WHERE id = 1;
-- EXPECT: 00:00:00

-- === CASE: YEAR function ===
SELECT YEAR(dt) AS year_val FROM datetime_test WHERE id = 1;
-- EXPECT: 2024

-- === CASE: MONTH function ===
SELECT MONTH(dt) AS month_val FROM datetime_test WHERE id = 1;
-- EXPECT: 1

-- === CASE: DAY function ===
SELECT DAY(dt) AS day_val FROM datetime_test WHERE id = 1;
-- EXPECT: 1

-- === CASE: HOUR function ===
SELECT HOUR(t) AS hour_val FROM datetime_test WHERE id = 3;
-- EXPECT: 12

-- === CASE: MINUTE function ===
SELECT MINUTE(t) AS minute_val FROM datetime_test WHERE id = 3;
-- EXPECT: 30

-- === CASE: SECOND function ===
SELECT SECOND(t) AS second_val FROM datetime_test WHERE id = 3;
-- EXPECT: 30

-- === CASE: Date arithmetic - add days ===
SELECT DATE_ADD('2024-01-01', INTERVAL 1 DAY) AS next_day;
-- EXPECT: 2024-01-02

-- === CASE: Date arithmetic - subtract days ===
SELECT DATE_SUB('2024-01-01', INTERVAL 1 DAY) AS prev_day;
-- EXPECT: 2023-12-31

-- === CASE: Date difference ===
SELECT DATEDIFF('2024-12-31', '2024-01-01') AS days_diff;
-- EXPECT: 365

-- === CASE: Datetime difference with time ===
SELECT DATEDIFF('2024-12-31 23:59:59', '2024-01-01 00:00:00') AS diff_with_time;
-- EXPECT: 365

-- === CASE: NULL datetime ===
INSERT INTO datetime_test VALUES (8, NULL, NULL, NULL);
SELECT * FROM datetime_test WHERE dt IS NULL;
-- EXPECT: id 8

-- === CASE: NOW function ===
SELECT NOW() AS current_datetime;
-- EXPECT: current timestamp

-- === CASE: CURDATE function ===
SELECT CURDATE() AS current_date;
-- EXPECT: current date

-- === CASE: CURTIME function ===
SELECT CURTIME() AS current_time;
-- EXPECT: current time