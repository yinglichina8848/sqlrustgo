-- Date/Time Function Test Cases
-- Compatibility: MySQL 5.7+

-- ============================================
-- 1. CURDATE, CURRENT_DATE
-- ============================================

SELECT CURDATE();

SELECT CURRENT_DATE;

SELECT CURDATE() + INTERVAL 1 DAY;

SELECT name FROM orders WHERE DATE(created_at) = CURDATE();

-- ============================================
-- 2. CURTIME, CURRENT_TIME
-- ============================================

SELECT CURTIME();

SELECT CURRENT_TIME;

SELECT CURTIME() + INTERVAL 1 HOUR;

SELECT name FROM sessions WHERE TIME(last_activity) > CURTIME();

-- ============================================
-- 3. NOW, CURRENT_TIMESTAMP
-- ============================================

SELECT NOW();

SELECT CURRENT_TIMESTAMP;

SELECT NOW() + INTERVAL 1 WEEK;

SELECT name FROM orders WHERE created_at > NOW() - INTERVAL 24 HOUR;

-- ============================================
-- 4. DATE
-- ============================================

SELECT DATE('2024-01-15 10:30:00');

SELECT DATE(created_at) FROM orders;

SELECT name FROM orders WHERE DATE(created_at) = '2024-01-15';

SELECT name FROM orders WHERE DATE(created_at) BETWEEN '2024-01-01' AND '2024-01-31';

-- ============================================
-- 5. TIME
-- ============================================

SELECT TIME('2024-01-15 10:30:00');

SELECT TIME(created_at) FROM orders;

SELECT name FROM sessions WHERE TIME(last_activity) > '09:00:00';

-- ============================================
-- 6. DATE_FORMAT
-- ============================================

SELECT DATE_FORMAT('2024-01-15', '%Y-%m-%d');

SELECT DATE_FORMAT(created_at, '%Y-%m') FROM orders;

SELECT DATE_FORMAT(created_at, '%W, %M %d, %Y') FROM orders;

SELECT DATE_FORMAT(created_at, '%H:%i:%s') FROM orders;

SELECT DATE_FORMAT(created_at, '%Y-%m-%d %H:%i') FROM orders;

SELECT name, DATE_FORMAT(created_at, '%Y/%m/%d') FROM users;

-- ============================================
-- 7. TIME_FORMAT
-- ============================================

SELECT TIME_FORMAT('10:30:00', '%H:%i');

SELECT TIME_FORMAT(created_at, '%h:%i %p') FROM sessions;

-- ============================================
-- 8. DATE_ADD, ADDDATE
-- ============================================

SELECT DATE_ADD('2024-01-15', INTERVAL 1 DAY);

SELECT DATE_ADD('2024-01-15', INTERVAL 1 WEEK);

SELECT DATE_ADD('2024-01-15', INTERVAL 1 MONTH);

SELECT DATE_ADD('2024-01-15', INTERVAL 1 YEAR);

SELECT DATE_ADD('2024-01-15', INTERVAL 1 HOUR);

SELECT DATE_ADD('2024-01-15 10:30:00', INTERVAL 30 MINUTE);

SELECT DATE_ADD(created_at, INTERVAL 7 DAY) FROM orders WHERE id = 1;

SELECT name FROM orders WHERE DATE_ADD(created_at, INTERVAL 30 DAY) < NOW();

-- ============================================
-- 9. DATE_SUB, SUBDATE
-- ============================================

SELECT DATE_SUB('2024-01-15', INTERVAL 1 DAY);

SELECT DATE_SUB('2024-01-15', INTERVAL 1 WEEK);

SELECT DATE_SUB('2024-01-15', INTERVAL 1 MONTH);

SELECT DATE_SUB('2024-01-15', INTERVAL 1 YEAR);

SELECT name FROM orders WHERE DATE_SUB(created_at, INTERVAL 30 DAY) > '2024-01-01';

-- ============================================
-- 10. DATEDIFF
-- ============================================

SELECT DATEDIFF('2024-01-20', '2024-01-15');

SELECT DATEDIFF(NOW(), created_at) FROM orders;

SELECT name FROM users WHERE DATEDIFF(CURDATE(), created_at) > 365;

SELECT id, DATEDIFF(CURDATE(), created_at) AS days_since_creation FROM orders;

-- ============================================
-- 11. TIMEDIFF
-- ============================================

SELECT TIMEDIFF('10:30:00', '09:00:00');

SELECT TIMEDIFF(NOW(), created_at) FROM sessions;

SELECT TIMEDIFF(end_time, start_time) FROM events;

-- ============================================
-- 12. TIMESTAMPADD
-- ============================================

SELECT TIMESTAMPADD(DAY, 1, '2024-01-15');

SELECT TIMESTAMPADD(WEEK, 2, '2024-01-15');

SELECT TIMESTAMPADD(MONTH, 3, '2024-01-15');

SELECT TIMESTAMPADD(HOUR, 5, created_at) FROM orders WHERE id = 1;

-- ============================================
-- 13. TIMESTAMPDIFF
-- ============================================

SELECT TIMESTAMPDIFF(DAY, '2024-01-15', '2024-01-20');

SELECT TIMESTAMPDIFF(HOUR, created_at, NOW()) FROM orders;

SELECT name FROM users WHERE TIMESTAMPDIFF(YEAR, created_at, NOW()) > 1;

SELECT TIMESTAMPDIFF(MONTH, created_at, NOW()) AS membership_months FROM users;

-- ============================================
-- 14. YEAR, MONTH, DAY
-- ============================================

SELECT YEAR('2024-01-15');

SELECT MONTH('2024-01-15');

SELECT DAY('2024-01-15');

SELECT YEAR(created_at) FROM orders;

SELECT MONTH(created_at) FROM orders WHERE id = 1;

SELECT name FROM orders WHERE MONTH(created_at) = 1;

SELECT name FROM orders WHERE YEAR(created_at) = 2024;

-- ============================================
-- 15. HOUR, MINUTE, SECOND
-- ============================================

SELECT HOUR('10:30:45');

SELECT MINUTE('10:30:45');

SELECT SECOND('10:30:45');

SELECT HOUR(created_at) FROM sessions;

SELECT name FROM sessions WHERE HOUR(last_activity) BETWEEN 9 AND 17;

-- ============================================
-- 16. DAYNAME, MONTHNAME
-- ============================================

SELECT DAYNAME('2024-01-15');

SELECT MONTHNAME('2024-01-15');

SELECT DAYNAME(created_at) FROM orders;

SELECT MONTHNAME(created_at) FROM orders GROUP BY MONTHNAME(created_at);

SELECT name FROM users WHERE DAYNAME(created_at) = 'Monday';

-- ============================================
-- 17. DAYOFWEEK, DAYOFMONTH, DAYOFYEAR
-- ============================================

SELECT DAYOFWEEK('2024-01-15');

SELECT DAYOFMONTH('2024-01-15');

SELECT DAYOFYEAR('2024-01-15');

SELECT DAYOFWEEK(created_at) FROM orders;

SELECT DAYOFMONTH(created_at) FROM orders GROUP BY DAYOFMONTH(created_at);

SELECT name FROM orders WHERE DAYOFYEAR(created_at) > 300;

-- ============================================
-- 18. WEEK, WEEKDAY
-- ============================================

SELECT WEEK('2024-01-15');

SELECT WEEKDAY('2024-01-15');

SELECT WEEK(created_at) FROM orders;

SELECT name FROM orders WHERE WEEKDAY(created_at) = 0;

SELECT WEEK(created_at, 1) FROM orders;

-- ============================================
-- 19. QUARTER
-- ============================================

SELECT QUARTER('2024-01-15');

SELECT QUARTER('2024-04-15');

SELECT QUARTER('2024-07-15');

SELECT QUARTER('2024-10-15');

SELECT name FROM orders WHERE QUARTER(created_at) = 1;

-- ============================================
-- 20. YEARWEEK
-- ============================================

SELECT YEARWEEK('2024-01-15');

SELECT YEARWEEK(created_at) FROM orders;

-- ============================================
-- 21. EXTRACT
-- ============================================

SELECT EXTRACT(YEAR FROM '2024-01-15');

SELECT EXTRACT(MONTH FROM '2024-01-15');

SELECT EXTRACT(DAY FROM '2024-01-15');

SELECT EXTRACT(HOUR FROM '2024-01-15 10:30:45');

SELECT EXTRACT(MINUTE FROM '2024-01-15 10:30:45');

SELECT EXTRACT(SECOND FROM '2024-01-15 10:30:45');

SELECT name FROM orders WHERE EXTRACT(YEAR FROM created_at) = 2024;

SELECT name FROM orders WHERE EXTRACT(MONTH FROM created_at) BETWEEN 1 AND 6;

-- ============================================
-- 22. MAKEDATE
-- ============================================

SELECT MAKEDATE(2024, 15);

SELECT MAKEDATE(2024, 365);

-- ============================================
-- 23. MAKETIME
-- ============================================

SELECT MAKETIME(10, 30, 45);

SELECT MAKETIME(9, 0, 0);

-- ============================================
-- 24. FROM_DAYS, TO_DAYS
-- ============================================

SELECT TO_DAYS('2024-01-15');

SELECT FROM_DAYS(738605);

SELECT name FROM orders WHERE TO_DAYS(NOW()) - TO_DAYS(created_at) > 30;

-- ============================================
-- 25. UNIX_TIMESTAMP
-- ============================================

SELECT UNIX_TIMESTAMP();

SELECT UNIX_TIMESTAMP('2024-01-15 00:00:00');

SELECT FROM_UNIXTIME(1705276800);

SELECT UNIX_TIMESTAMP(created_at) FROM orders;

-- ============================================
-- 26. UTC_DATE, UTC_TIME, UTC_TIMESTAMP
-- ============================================

SELECT UTC_DATE();

SELECT UTC_TIME();

SELECT UTC_TIMESTAMP();

-- ============================================
-- 27. LAST_DAY
-- ============================================

SELECT LAST_DAY('2024-01-15');

SELECT LAST_DAY(created_at) FROM orders;

SELECT name FROM orders WHERE created_at = LAST_DAY(created_at);

-- ============================================
-- 28. STR_TO_DATE
-- ============================================

SELECT STR_TO_DATE('15/01/2024', '%d/%m/%Y');

SELECT STR_TO_DATE('Jan 15, 2024', '%b %d, %Y');

SELECT name FROM orders WHERE created_at = STR_TO_DATE('01/15/2024', '%m/%d/%Y');

-- ============================================
-- 29. ADDTIME, SUBTIME
-- ============================================

SELECT ADDTIME('2024-01-15 10:30:00', '02:00:00');

SELECT SUBTIME('2024-01-15 10:30:00', '02:00:00');

SELECT ADDTIME(created_at, '00:30:00') FROM orders WHERE id = 1;

-- ============================================
-- 30. DATE arithmetic with different units
-- ============================================

SELECT '2024-01-15' + INTERVAL 1 DAY;

SELECT '2024-01-15' + INTERVAL 1 WEEK;

SELECT '2024-01-15' + INTERVAL 1 MONTH;

SELECT '2024-01-15' + INTERVAL 1 QUARTER;

SELECT '2024-01-15' + INTERVAL 1 YEAR;

SELECT created_at + INTERVAL 1 HOUR FROM orders;

SELECT created_at + INTERVAL 30 MINUTE FROM orders;

SELECT created_at + INTERVAL 15 SECOND FROM orders;

-- ============================================
-- 31. NOW with fractional seconds (MySQL 5.7+)
-- ============================================

SELECT NOW(6);

SELECT CURRENT_TIMESTAMP(6);

-- ============================================
-- 32. TIMESTAMP with date and time
-- ============================================

SELECT TIMESTAMP('2024-01-15');

SELECT TIMESTAMP('2024-01-15', '10:30:00');

SELECT TIMESTAMP(created_at, '00:00:00') FROM orders;

-- ============================================
-- 33. DATEIFF with different units
-- ============================================

SELECT TIMESTAMPDIFF(SECOND, created_at, NOW()) FROM orders;

SELECT TIMESTAMPDIFF(MINUTE, created_at, NOW()) FROM orders;

SELECT TIMESTAMPDIFF(HOUR, created_at, NOW()) FROM orders;

SELECT TIMESTAMPDIFF(DAY, created_at, NOW()) FROM orders;

SELECT TIMESTAMPDIFF(WEEK, created_at, NOW()) FROM orders;

SELECT TIMESTAMPDIFF(MONTH, created_at, NOW()) FROM orders;

SELECT TIMESTAMPDIFF(YEAR, created_at, NOW()) FROM users;

-- ============================================
-- 34. Combine date and time parts
-- ============================================

SELECT DATE_ADD(DATE_ADD(CAST('2024-01-15' AS DATE), INTERVAL 10 HOUR), INTERVAL 30 MINUTE);

SELECT TIMESTAMP(DATE(created_at), MAKETIME(HOUR(created_at), MINUTE(created_at), 0)) FROM orders;

-- ============================================
-- 35. Get first/last day of month
-- ============================================

SELECT DATE_FORMAT(created_at, '%Y-%m-01') FROM orders;

SELECT LAST_DAY(created_at) FROM orders;

SELECT name FROM orders WHERE DATE_FORMAT(created_at, '%Y-%m') = '2024-01';

-- ============================================
-- 36. Age calculation
-- ============================================

SELECT TIMESTAMPDIFF(YEAR, birth_date, CURDATE()) FROM users;

SELECT name, TIMESTAMPDIFF(YEAR, birth_date, CURDATE()) AS age FROM users WHERE age > 18;

-- ============================================
-- 37. Time period queries
-- ============================================

SELECT name FROM orders WHERE created_at >= DATE_SUB(NOW(), INTERVAL 7 DAY);

SELECT name FROM orders WHERE created_at >= DATE_SUB(NOW(), INTERVAL 1 MONTH);

SELECT name FROM orders WHERE created_at >= DATE_SUB(NOW(), INTERVAL 3 MONTH);

SELECT name FROM orders WHERE created_at >= DATE_SUB(NOW(), INTERVAL 1 YEAR);

-- ============================================
-- 38. GROUP BY date period
-- ============================================

SELECT DATE(created_at) AS date, COUNT(*) AS count FROM orders GROUP BY DATE(created_at);

SELECT DATE_FORMAT(created_at, '%Y-%m') AS month, SUM(total) AS total FROM orders GROUP BY DATE_FORMAT(created_at, '%Y-%m');

SELECT WEEK(created_at) AS week, AVG(total) AS avg_order FROM orders GROUP BY WEEK(created_at);

SELECT MONTHNAME(created_at) AS month, COUNT(*) AS count FROM orders GROUP BY MONTHNAME(created_at);

-- ============================================
-- 39. Case with date
-- ============================================

SELECT name,
    CASE
        WHEN TIMESTAMPDIFF(DAY, created_at, NOW()) = 0 THEN 'Today'
        WHEN TIMESTAMPDIFF(DAY, created_at, NOW()) = 1 THEN 'Yesterday'
        WHEN TIMESTAMPDIFF(DAY, created_at, NOW()) < 7 THEN 'This Week'
        WHEN TIMESTAMPDIFF(DAY, created_at, NOW()) < 30 THEN 'This Month'
        ELSE 'Older'
    END AS time_period
FROM orders;

-- ============================================
-- 40. Is date functions
-- ============================================

SELECT ISNULL(created_at) FROM orders;

SELECT name FROM orders WHERE created_at IS NOT NULL;

SELECT name FROM orders WHERE created_at IS NULL;

-- ============================================
-- 41. Date validation
-- ============================================

SELECT ISDATE('2024-01-15');

SELECT ISDATE('2024-02-30');

SELECT name FROM users WHERE ISDATE(birth_date) = 1;

-- ============================================
-- 42. Business days calculation
-- ============================================

-- Excludes weekends (simplified)
-- SELECT COUNT(*) FROM (
--     SELECT DATE_ADD('2024-01-01', INTERVAL n DAY) AS d
--     FROM numbers
--     WHERE n <= DATEDIFF('2024-01-31', '2024-01-01')
--     AND DAYOFWEEK(DATE_ADD('2024-01-01', INTERVAL n DAY)) NOT IN (1, 7)
-- ) business_days;

-- ============================================
-- 43. Compare dates
-- ============================================

SELECT name FROM orders WHERE created_at < '2024-01-15';

SELECT name FROM orders WHERE created_at <= '2024-01-15 23:59:59';

SELECT name FROM orders WHERE DATE(created_at) <= '2024-01-15';

SELECT name FROM orders WHERE created_at BETWEEN '2024-01-01' AND '2024-01-31';

-- ============================================
-- 44. Extract parts with arithmetic
-- ============================================

SELECT YEAR(created_at) * 100 + MONTH(created_at) AS year_month FROM orders;

SELECT DAYOFWEEK(created_at) + 1 FROM orders;

SELECT MONTH(created_at) - 1 FROM orders;

-- ============================================
-- 45. Time zones (if supported)
-- ============================================

-- SELECT CONVERT_TZ(created_at, 'UTC', 'America/New_York') FROM orders;

-- SELECT CONVERT_TZ(NOW(), 'UTC', 'Europe/London');
