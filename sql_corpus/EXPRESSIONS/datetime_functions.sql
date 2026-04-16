-- === SKIP ===

-- SQLCorpus: Date/Time Functions
-- Tests for date and time functions

-- === SETUP ===
CREATE TABLE dates (id INTEGER PRIMARY KEY, d TEXT);
INSERT INTO dates VALUES (1, '2024-01-15'), (2, '2024-02-20'), (3, '2024-03-25');

-- === CASE: date ===
SELECT DATE('now');
-- EXPECT: 1 rows

-- === CASE: time ===
SELECT TIME('now');
-- EXPECT: 1 rows

-- === CASE: datetime ===
SELECT DATETIME('now');
-- EXPECT: 1 rows

-- === CASE: strftime ===
SELECT STRFTIME('%Y-%m-%d', '2024-01-15');
-- EXPECT: 1 rows

-- === CASE: strftime_format ===
SELECT STRFTIME('%Y', '2024-01-15');
-- EXPECT: 1 rows

-- === CASE: date_add ===
SELECT DATE('2024-01-15', '+1 day');
-- EXPECT: 1 rows

-- === CASE: date_sub ===
SELECT DATE('2024-01-15', '-1 day');
-- EXPECT: 1 rows

-- === CASE: date_diff ===
SELECT JULIANDAY('2024-01-20') - JULIANDAY('2024-01-15');
-- EXPECT: 1 rows

-- === CASE: date_with_time ===
SELECT DATE('2024-01-15 12:30:45');
-- EXPECT: 1 rows