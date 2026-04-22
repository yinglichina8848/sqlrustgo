-- UPDATE Statement Test Cases
-- Compatibility: MySQL 5.7+

-- ============================================
-- 1. Basic UPDATE Operations
-- ============================================

-- Simple update single column
UPDATE users SET email = 'newemail@example.com' WHERE id = 1;

-- Update multiple columns
UPDATE users SET name = 'Alice Smith', email = 'alice.smith@example.com', age = 31 WHERE id = 1;

-- Update all rows (no WHERE)
UPDATE products SET price = price * 1.1;

-- Update with WHERE clause
UPDATE users SET status = 'inactive' WHERE age > 80;

-- ============================================
-- 2. UPDATE with Expressions
-- ============================================

-- Arithmetic in SET
UPDATE products SET price = price * 0.9 WHERE category = 'sale';

UPDATE orders SET total = quantity * unit_price WHERE id > 100;

-- String concatenation
UPDATE users SET name = CONCAT(name, ' Jr.') WHERE id = 5;

-- Update with CASE
UPDATE users SET status = CASE WHEN age < 18 THEN 'minor' WHEN age < 65 THEN 'adult' ELSE 'senior' END;

-- Update with COALESCE
UPDATE users SET phone = COALESCE(phone, 'N/A') WHERE phone IS NULL;

-- ============================================
-- 3. UPDATE with Functions
-- ============================================

-- Date functions
UPDATE orders SET shipped_at = DATE_ADD(created_at, INTERVAL 3 DAY) WHERE status = 'pending';

UPDATE users SET updated_at = NOW() WHERE id = 1;

UPDATE events SET event_date = DATE_SUB(event_date, INTERVAL 1 WEEK) WHERE event_date > CURDATE();

-- String functions
UPDATE users SET name = UPPER(name) WHERE status = 'active';

UPDATE products SET description = REPLACE(description, 'old', 'new') WHERE description LIKE '%old%';

UPDATE articles SET title = TRIM(title), content = TRIM(content);

-- Math functions
UPDATE products SET price = ROUND(price * 1.05, 2) WHERE category = 'electronics';

UPDATE users SET score = FLOOR(score / 10) * 10 WHERE score > 0;

-- ============================================
-- 4. UPDATE with Subqueries
-- ============================================

-- Subquery in WHERE
UPDATE users SET status = 'vip' WHERE id IN (SELECT user_id FROM orders WHERE total > 1000);

-- Subquery in SET
UPDATE products SET price = (SELECT AVG(price) FROM products WHERE category = 'electronics') WHERE id = 1;

-- Correlated subquery
UPDATE users u SET u.order_count = (SELECT COUNT(*) FROM orders WHERE user_id = u.id);

-- Update with EXISTS
UPDATE users SET eligible = TRUE WHERE EXISTS (SELECT 1 FROM orders WHERE orders.user_id = users.id AND total > 500);

-- ============================================
-- 5. UPDATE with JOIN
-- ============================================

-- UPDATE with JOIN
UPDATE users u JOIN orders o ON u.id = o.user_id SET u.total_orders = u.total_orders + 1 WHERE o.id = 100;

-- Multi-table UPDATE
-- UPDATE users u, products p SET u.last_product = p.name, p.last_buyer = u.name WHERE u.id = 1 AND p.id = 1;

-- UPDATE with LEFT JOIN
UPDATE users u LEFT JOIN orders o ON u.id = o.user_id SET u.has_orders = (o.id IS NOT NULL) WHERE u.id = 1;

-- ============================================
-- 6. UPDATE with ORDER BY and LIMIT
-- ============================================

-- UPDATE with ORDER BY
UPDATE users SET points = points + 10 ORDER BY registration_date LIMIT 100;

-- UPDATE with ORDER BY DESC
UPDATE leaderboard SET score = score + 5 ORDER BY score DESC LIMIT 10;

-- Update top N by certain criteria
UPDATE products SET popularity = popularity + 1 ORDER BY created_at DESC LIMIT 5;

-- ============================================
-- 7. UPDATE with WHERE variations
-- ============================================

-- WHERE with multiple conditions
UPDATE users SET status = 'active', last_login = NOW() WHERE age >= 18 AND status = 'pending';

-- WHERE with OR
UPDATE users SET notifications = 'enabled' WHERE email LIKE '%@company.com%' OR email LIKE '%@partner.com%';

-- WHERE with LIKE
UPDATE products SET featured = TRUE WHERE name LIKE '%bestseller%' OR description LIKE '%popular%';

-- WHERE with IN
UPDATE users SET tier = 'gold' WHERE id IN (1, 2, 3, 5, 8, 13, 21);

-- WHERE with BETWEEN
UPDATE sales SET commission = commission * 1.1 WHERE amount BETWEEN 1000 AND 5000;

-- WHERE with NULL check
UPDATE users SET phone = 'unknown' WHERE phone IS NULL;

-- WHERE with NOT
UPDATE products SET discontinued = TRUE WHERE category NOT IN ('electronics', 'software');

-- ============================================
-- 8. UPDATE with String patterns
-- ============================================

UPDATE users SET email = REPLACE(email, '@old-domain.com', '@new-domain.com') WHERE email LIKE '%@old-domain.com%';

UPDATE articles SET slug = LOWER(REPLACE(title, ' ', '-')) WHERE published = TRUE;

UPDATE comments SET status = 'flagged' WHERE content LIKE '%spam%' OR content LIKE '%scam%';

-- ============================================
-- 9. UPDATE with Timestamps
-- ============================================

UPDATE orders SET status = 'shipped', shipped_at = CURRENT_TIMESTAMP WHERE status = 'paid' AND shipping_address IS NOT NULL;

UPDATE sessions SET last_activity = UNIX_TIMESTAMP() WHERE user_id = 1;

UPDATE logs SET created_at = FROM_UNIXTIME(1700000000) WHERE id = 1;

-- ============================================
-- 10. UPDATE with JSON
-- ============================================

UPDATE metadata SET data = JSON_SET(data, '$.verified', TRUE) WHERE id = 1;

UPDATE metadata SET data = JSON_INSERT(data, '$.new_field', 'value') WHERE id = 1;

UPDATE metadata SET data = JSON_REPLACE(data, '$.name', 'new_name') WHERE id = 1;

UPDATE metadata SET data = JSON_REMOVE(data, '$.temporary') WHERE id = 1;

UPDATE metadata SET data = JSON_MERGE(data, '{"extra": true}') WHERE id = 1;

-- ============================================
-- 11. UPDATE with Conditional Logic
-- ============================================

-- IF function
UPDATE products SET discount = IF(price > 100, 10, IF(price > 50, 5, 0));

-- CASE in SET
UPDATE orders SET priority = CASE status WHEN 'pending' THEN 1 WHEN 'processing' THEN 2 WHEN 'shipped' THEN 3 ELSE 0 END;

-- IIF (MySQL 8.0+)
-- UPDATE users SET category = IIF(points > 1000, 'VIP', IIF(points > 500, 'Premium', 'Standard'));

-- ============================================
-- 12. UPDATE with Aggregate Functions
-- ============================================

-- Update with COUNT in subquery
UPDATE users SET order_count = (SELECT COUNT(*) FROM orders WHERE orders.user_id = users.id);

-- Update with SUM in subquery
UPDATE users SET total_spent = (SELECT SUM(total) FROM orders WHERE orders.user_id = users.id);

-- Update with AVG in subquery
UPDATE products SET avg_rating = (SELECT AVG(rating) FROM reviews WHERE reviews.product_id = products.id);

-- Update with MAX in subquery
UPDATE users SET highest_order = (SELECT MAX(total) FROM orders WHERE orders.user_id = users.id);

-- ============================================
-- 13. UPDATE with LIMIT variations
-- ============================================

UPDATE users SET notifications = TRUE LIMIT 50;

UPDATE posts SET views = views + 1 ORDER BY created_at DESC LIMIT 10;

UPDATE inventory SET reserved = reserved + 1 WHERE product_id IN (SELECT product_id FROM available LIMIT 5);

-- ============================================
-- 14. UPDATE with Transactions (within transaction)
-- ============================================

-- BEGIN;
-- UPDATE accounts SET balance = balance - 100 WHERE id = 1;
-- UPDATE accounts SET balance = balance + 100 WHERE id = 2;
-- COMMIT;

-- ROLLBACK example (if balance would go negative)
-- BEGIN;
-- UPDATE accounts SET balance = balance - 5000 WHERE id = 1 AND balance >= 5000;
-- -- If affected rows = 0, then insufficient funds
-- ROLLBACK;

-- ============================================
-- 15. UPDATE with Low Priority (MySQL specific)
-- ============================================

UPDATE LOW_PRIORITY users SET last_seen = NOW() WHERE online = TRUE;

-- DELAYED (MySQL 5.7 and earlier)
-- INSERT DELAYED ... (deprecated in MySQL 8.0)

-- ============================================
-- 16. UPDATE withIGNORE
-- ============================================

UPDATE IGNORE users SET id = id + 1000 WHERE id > 100;

-- ============================================
-- 17. Multi-table UPDATE
-- ============================================

-- UPDATE products p, categories c SET p.category_name = c.name WHERE p.category_id = c.id;

UPDATE users u, stats s SET u.profile_views = s.views WHERE u.id = s.user_id;

-- ============================================
-- 18. UPDATE with Row Values (MySQL 8.0+)
-- ============================================

-- UPDATE orders SET (product_id, quantity) = (1, 5) WHERE id = 100; -- NOT VALID
-- MySQL doesn't support tuple assignment in UPDATE

-- ============================================
-- 19. UPDATE with Spatial Functions
-- ============================================

-- Assuming location table with POINT column
-- UPDATE locations SET address = ST_AsText(geometry) WHERE id = 1;

-- ============================================
-- 20. UPDATE edge cases
-- ============================================

-- Update to same value (no change)
UPDATE users SET name = name WHERE id = 1;

-- Update with 0
UPDATE products SET stock = 0 WHERE discontinued = TRUE;

-- Update with negative
UPDATE temperatures SET value = -ABS(value) WHERE unit = 'celsius';

-- Update with very long string
UPDATE articles SET content = REPEAT('x', 10000) WHERE id = 1;

-- Update with special characters
UPDATE users SET name = 'O\'Brien', email = 'obrien@example.com' WHERE id = 1;

-- Update with Unicode
UPDATE users SET name = '日本語ユーザー', email = 'japanese@example.com' WHERE id = 1;
