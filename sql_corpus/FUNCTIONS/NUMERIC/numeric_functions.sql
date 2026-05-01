-- Numeric Function Test Cases
-- Compatibility: MySQL 5.7+

-- ============================================
-- 1. Basic Arithmetic
-- ============================================

SELECT 1 + 1;

SELECT 10 - 3;

SELECT 4 * 2;

SELECT 10 / 3;

SELECT 10 DIV 3;

SELECT 10 % 3;

SELECT 10 MOD 3;

SELECT price + 10 FROM products;

SELECT price * 0.9 AS discounted FROM products;

SELECT quantity * price AS total FROM order_items;

-- ============================================
-- 2. ABS
-- ============================================

SELECT ABS(-10);

SELECT ABS(10);

SELECT ABS(price - 100) FROM products;

SELECT name FROM products WHERE ABS(price - 50) < 10;

SELECT ABS(discount) FROM products;

-- ============================================
-- 3. CEIL and FLOOR
-- ============================================

SELECT CEIL(1.1);

SELECT CEIL(1.9);

SELECT FLOOR(1.1);

SELECT FLOOR(1.9);

SELECT CEIL(price) FROM products;

SELECT FLOOR(price) FROM products;

SELECT CEIL(total / 100) AS units FROM orders;

-- ============================================
-- 4. ROUND
-- ============================================

SELECT ROUND(1.5);

SELECT ROUND(1.4);

SELECT ROUND(1.23456, 2);

SELECT ROUND(1.23456, 3);

SELECT ROUND(price, 0) FROM products;

SELECT ROUND(price * 0.9, 2) AS discounted FROM products;

SELECT ROUND(AVG(price), 2) FROM products;

-- ============================================
-- 5. TRUNCATE
-- ============================================

SELECT TRUNCATE(1.23456, 2);

SELECT TRUNCATE(1.99999, 2);

SELECT TRUNCATE(123.456, -1);

SELECT TRUNCATE(123.456, -2);

SELECT TRUNCATE(price, 0) FROM products;

-- ============================================
-- 6. POW and POWER
-- ============================================

SELECT POW(2, 3);

SELECT POWER(2, 3);

SELECT POW(price, 2) FROM products;

SELECT SQRT(ABS(price - 100)) FROM products;

SELECT SQRT(ABS(discount)) FROM products;

-- ============================================
-- 7. SQRT
-- ============================================

SELECT SQRT(4);

SELECT SQRT(2);

SELECT SQRT(ABS(price)) FROM products;

SELECT name FROM products WHERE SQRT(price) < 10;

-- ============================================
-- 8. MOD (remainder)
-- ============================================

SELECT MOD(10, 3);

SELECT MOD(10, 5);

SELECT MOD(price, 3) FROM products;

SELECT name FROM products WHERE MOD(id, 2) = 0;

SELECT * FROM products WHERE id MOD 2 = 1;

-- ============================================
-- 9. GREATEST
-- ============================================

SELECT GREATEST(1, 5, 3);

SELECT GREATEST('a', 'b', 'c');

SELECT GREATEST(price, 100) FROM products;

SELECT GREATEST(price, discount, 10) FROM products;

-- ============================================
-- 10. LEAST
-- ============================================

SELECT LEAST(1, 5, 3);

SELECT LEAST('a', 'b', 'c');

SELECT LEAST(price, 100) FROM products;

SELECT LEAST(price, discount, 10) FROM products;

-- ============================================
-- 11. RAND
-- ============================================

SELECT RAND();

SELECT RAND(42);

SELECT name FROM products ORDER BY RAND();

SELECT name FROM users ORDER BY RAND() LIMIT 10;

SELECT ROUND(RAND() * 100) AS random_num;

-- ============================================
-- 12. SIGN
-- ============================================

SELECT SIGN(-10);

SELECT SIGN(0);

SELECT SIGN(10);

SELECT SIGN(price - 100) FROM products;

-- ============================================
-- 13. ABS with aggregate functions
-- ============================================

SELECT SUM(ABS(discount)) FROM products;

SELECT AVG(ABS(price - 50)) FROM products;

SELECT COUNT(DISTINCT ABS(category_id)) FROM products;

-- ============================================
-- 14. Numeric expressions in WHERE
-- ============================================

SELECT * FROM products WHERE price > 100;

SELECT * FROM products WHERE price + discount > 100;

SELECT * FROM products WHERE price * quantity > 1000;

SELECT * FROM products WHERE (price - original_price) / original_price > 0.1;

-- ============================================
-- 15. Division with different types
-- ============================================

SELECT 10 / 3;

SELECT 10 / 3.0;

SELECT CAST(10 AS DECIMAL) / 3 FROM products;

SELECT NULLIF(price, 0) FROM products;

SELECT COALESCE(price / NULLIF(discount, 0), 0) FROM products;

-- ============================================
-- 16. BIT operations
-- ============================================

SELECT 5 & 3;

SELECT 5 | 3;

SELECT 5 ^ 3;

SELECT ~5;

SELECT 5 << 1;

SELECT 5 >> 1;

-- ============================================
-- 17. LOG and LOG10
-- ============================================

SELECT LOG(10);

SELECT LOG(2, 8);

SELECT LOG10(100);

SELECT LOG10(1000);

SELECT LOG(price) FROM products;

-- ============================================
-- 18. EXP
-- ============================================

SELECT EXP(1);

SELECT EXP(2);

SELECT EXP(price / 100) FROM products;

-- ============================================
-- 19. PI
-- ============================================

SELECT PI();

SELECT 2 * PI() * 10;

SELECT ACOS(0.5);

SELECT COS(PI());

-- ============================================
-- 20. DEGREES and RADIANS
-- ============================================

SELECT DEGREES(PI());

SELECT RADIANS(180);

SELECT DEGREES(price) FROM products;

-- ============================================
-- 21. SIN, COS, TAN
-- ============================================

SELECT SIN(PI() / 2);

SELECT COS(0);

SELECT TAN(PI() / 4);

-- ============================================
-- 22. ASIN, ACOS, ATAN
-- ============================================

SELECT ASIN(1);

SELECT ACOS(1);

SELECT ATAN(1);

-- ============================================
-- 23. COT
-- ============================================

SELECT COT(1);

SELECT COT(PI() / 4);

-- ============================================
-- 24. CONV (base conversion)
-- ============================================

SELECT CONV(10, 10, 2);

SELECT CONV('A', 16, 10);

SELECT CONV('FF', 16, 10);

SELECT CONV(price, 10, 16) FROM products;

-- ============================================
-- 25. BIN, OCT, HEX (base representations)
-- ============================================

SELECT BIN(10);

SELECT OCT(10);

SELECT HEX(10);

-- ============================================
-- 26. INTERVAL
-- ============================================

SELECT INTERVAL(5, 1, 3, 5, 7);

SELECT INTERVAL(1, 1, 3, 5, 7);

-- ============================================
-- 27. Numeric CASE
-- ============================================

SELECT CASE WHEN price > 100 THEN 'expensive' WHEN price > 50 THEN 'moderate' ELSE 'cheap' END FROM products;

SELECT name, price, CASE MOD(id, 3) WHEN 0 THEN 'group_a' WHEN 1 THEN 'group_b' ELSE 'group_c' END AS `group` FROM products;

-- ============================================
-- 28. Percentage calculations
-- ============================================

SELECT price, price * 0.1 AS ten_percent, price * 0.2 AS twenty_percent FROM products;

SELECT total, total * 0.08 AS tax, total * 1.08 AS total_with_tax FROM orders;

SELECT (price - cost) / cost * 100 AS margin_percent FROM products;

-- ============================================
-- 29. Running totals
-- ============================================

-- SELECT id, total, SUM(total) OVER (ORDER BY id) AS running_total FROM orders;

-- SELECT id, price, SUM(price) OVER (PARTITION BY category_id ORDER BY id) AS category_running FROM products;

-- ============================================
-- 30. ROW_NUMBER alternatives (without window functions)
-- ============================================

-- SELECT @rownum := @rownum + 1 AS row_num, name FROM products, (SELECT @rownum := 0) r;

-- ============================================
-- 31. Division by zero handling
-- ============================================

SELECT IF(denominator != 0, numerator / denominator, 0) FROM calculations;

SELECT NULLIF(price, 0) FROM products;

SELECT COALESCE(price / NULLIF(discount, 0), 0) FROM products;

SELECT IFNULL(price / NULLIF(discount, 0), 0) FROM products;

-- ============================================
-- 32. Aggregate with HAVING
-- ============================================

SELECT user_id, SUM(total) AS total_spent
FROM orders
GROUP BY user_id
HAVING SUM(total) > 1000;

SELECT category_id, AVG(price) AS avg_price
FROM products
GROUP BY category_id
HAVING AVG(price) > 50;

-- ============================================
-- 33. COUNT variations
-- ============================================

SELECT COUNT(*) FROM products;

SELECT COUNT(DISTINCT category_id) FROM products;

SELECT COUNT(price) FROM products;

SELECT COUNT(*) FROM products WHERE price > 100;

-- ============================================
-- 34. MIN, MAX with expressions
-- ============================================

SELECT MIN(price), MAX(price) FROM products;

SELECT MIN(ABS(price - 100)), MAX(ABS(price - 100)) FROM products;

SELECT name FROM products WHERE price = (SELECT MAX(price) FROM products);

-- ============================================
-- 35. AVG with expressions
-- ============================================

SELECT AVG(price) FROM products;

SELECT AVG(price * 0.9) FROM products;

SELECT AVG(price - cost) FROM products;

SELECT category_id, AVG(price - cost) AS avg_profit FROM products GROUP BY category_id;

-- ============================================
-- 36. STDDEV
-- ============================================

SELECT STDDEV(price) FROM products;

SELECT STDDEV_POP(price) FROM products;

SELECT STDDEV_SAMP(price) FROM products;

-- ============================================
-- 37. VARIANCE
-- ============================================

SELECT VARIANCE(price) FROM products;

SELECT VAR_POP(price) FROM products;

SELECT VAR_SAMP(price) FROM products;

-- ============================================
-- 38. COALESCE with numerics
-- ============================================

SELECT COALESCE(price, 0) FROM products;

SELECT COALESCE(discount, price * 0.1) FROM products;

SELECT COALESCE(NULLIF(price, 0), 10) FROM products;

-- ============================================
-- 39. Arithmetic with NULL
-- ============================================

SELECT NULL + 1;

SELECT NULL - 1;

SELECT NULL * 10;

SELECT NULL / 1;

SELECT price + COALESCE(discount, 0) FROM products;

-- ============================================
-- 40. Scientific notation
-- ============================================

SELECT 1e10;

SELECT 1.23e+5;

SELECT 1.23e-3;

SELECT price * 1e-6 FROM products;
