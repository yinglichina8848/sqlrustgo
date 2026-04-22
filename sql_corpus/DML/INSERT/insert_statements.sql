-- INSERT Statement Test Cases
-- Compatibility: MySQL 5.7+

-- ============================================
-- 1. Basic INSERT Operations
-- ============================================

-- Single row insert
INSERT INTO users (id, name, email, age, status) VALUES (1, 'Alice', 'alice@example.com', 30, 'active');

-- Multiple rows in single INSERT
INSERT INTO users (id, name, email, age, status) VALUES
    (2, 'Bob', 'bob@example.com', 25, 'active'),
    (3, 'Carol', 'carol@example.com', 28, 'inactive'),
    (4, 'David', 'david@example.com', 35, 'active');

-- Insert with DEFAULT values
INSERT INTO users (id, name, email) VALUES (5, 'Eve', 'eve@example.com');

-- Insert without column list
INSERT INTO users VALUES (6, 'Frank', 'frank@example.com', 40, 'active');

-- ============================================
-- 2. INSERT with Expressions
-- ============================================

-- Insert with calculation
INSERT INTO products (id, name, price, stock, category) VALUES (1, 'Widget', 19.99 * 1.1, 100, 'electronics');

-- Insert with string concatenation
INSERT INTO users (id, name, email, age, status) VALUES (7, CONCAT('Ge', 'orge'), CONCAT('ge', 'orge@test.com'), 33, 'active');

-- Insert with current timestamp
INSERT INTO orders (id, user_id, total, created_at) VALUES (1, 1, 99.99, NOW());

-- Insert with DATE_ADD
INSERT INTO subscriptions (id, user_id, start_date, end_date) VALUES (1, 1, CURDATE(), DATE_ADD(CURDATE(), INTERVAL 1 YEAR));

-- ============================================
-- 3. INSERT with Functions
-- ============================================

-- Insert with UPPER/LOWER
INSERT INTO users (id, name, email, age, status) VALUES (8, UPPER('helen'), LOWER('HELEN@TEST.COM'), 29, 'active');

-- Insert with mathematical functions
INSERT INTO products (id, name, price, stock, category) VALUES (2, 'Gadget', ROUND(29.99 * 0.9, 2), ABS(-50), 'electronics');

-- Insert with NULLIF
INSERT INTO products (id, name, price, stock, category) VALUES (3, 'Thing', NULLIF(0, 0), 75, 'misc');

-- Insert with COALESCE
INSERT INTO users (id, name, email, age, status) VALUES (9, 'Ivan', COALESCE(NULL, 'ivan@test.com'), COALESCE(NULL, 45), 'active');

-- Insert with IF function
INSERT INTO products (id, name, price, stock, category) VALUES (4, 'Special', IF(1 > 0, 99.99, 49.99), 10, 'premium');

-- ============================================
-- 4. INSERT with Subqueries
-- ============================================

-- Insert from SELECT
INSERT INTO users (id, name, email, age, status)
SELECT 10, 'John', 'john@example.com', 32, 'active';

-- Insert with subquery in VALUES
INSERT INTO order_items (order_id, product_id, quantity, price)
SELECT 1, (SELECT id FROM products WHERE name = 'Widget'), 2, (SELECT price FROM products WHERE name = 'Widget');

-- Insert multiple rows from SELECT
INSERT INTO top_users (id, name, email)
SELECT id, name, email FROM users WHERE age > 30;

-- Insert with aggregate subquery
INSERT INTO stats (category, total_count, avg_value)
SELECT 'users', COUNT(*), AVG(age) FROM users;

-- ============================================
-- 5. INSERT with ON DUPLICATE KEY UPDATE
-- ============================================

-- Basic upsert
INSERT INTO users (id, name, email, age, status) VALUES (1, 'Alice', 'alice@example.com', 31, 'active')
ON DUPLICATE KEY UPDATE age = 31, email = 'alice@example.com';

-- Upsert with VALUES reference
INSERT INTO products (id, name, price, stock) VALUES (1, 'Widget', 19.99, 101)
ON DUPLICATE KEY UPDATE stock = stock + 1;

-- Multiple upsert
INSERT INTO inventory (product_id, quantity) VALUES (1, 10), (2, 20)
ON DUPLICATE KEY UPDATE quantity = quantity + VALUES(quantity);

-- ============================================
-- 6. INSERT with IGNORE
-- ============================================

-- Insert with ignore (silences errors)
INSERT IGNORE INTO users (id, name, email) VALUES (1, 'Alice', 'alice@example.com');

-- Insert ignore on duplicate
INSERT IGNORE INTO products (id, name, price) VALUES (1, 'Widget', 19.99);

-- ============================================
-- 7. INSERT with SET syntax
-- ============================================

INSERT INTO users SET
    id = 11,
    name = 'Karen',
    email = 'karen@example.com',
    age = 27,
    status = 'active';

INSERT INTO products SET
    id = 5,
    name = 'Super Gadget',
    price = 299.99,
    stock = 5,
    category = 'premium';

-- ============================================
-- 8. Bulk INSERT Performance Tests
-- ============================================

-- 100 row bulk insert
INSERT INTO users (id, name, email, age, status) VALUES
    (20, 'User20', 'user20@test.com', 20, 'active'),
    (21, 'User21', 'user21@test.com', 21, 'active'),
    (22, 'User22', 'user22@test.com', 22, 'active'),
    (23, 'User23', 'user23@test.com', 23, 'active'),
    (24, 'User24', 'user24@test.com', 24, 'active'),
    (25, 'User25', 'user25@test.com', 25, 'active'),
    (26, 'User26', 'user26@test.com', 26, 'active'),
    (27, 'User27', 'user27@test.com', 27, 'active'),
    (28, 'User28', 'user28@test.com', 28, 'active'),
    (29, 'User29', 'user29@test.com', 29, 'active'),
    (30, 'User30', 'user30@test.com', 30, 'active'),
    (31, 'User31', 'user31@test.com', 31, 'active'),
    (32, 'User32', 'user32@test.com', 32, 'active'),
    (33, 'User33', 'user33@test.com', 33, 'active'),
    (34, 'User34', 'user34@test.com', 34, 'active'),
    (35, 'User35', 'user35@test.com', 35, 'active'),
    (36, 'User36', 'user36@test.com', 36, 'active'),
    (37, 'User37', 'user37@test.com', 37, 'active'),
    (38, 'User38', 'user38@test.com', 38, 'active'),
    (39, 'User39', 'user39@test.com', 39, 'active'),
    (40, 'User40', 'user40@test.com', 40, 'active'),
    (41, 'User41', 'user41@test.com', 41, 'active'),
    (42, 'User42', 'user42@test.com', 42, 'active'),
    (43, 'User43', 'user43@test.com', 43, 'active'),
    (44, 'User44', 'user44@test.com', 44, 'active'),
    (45, 'User45', 'user45@test.com', 45, 'active'),
    (46, 'User46', 'user46@test.com', 46, 'active'),
    (47, 'User47', 'user47@test.com', 47, 'active'),
    (48, 'User48', 'user48@test.com', 48, 'active'),
    (49, 'User49', 'user49@test.com', 49, 'active'),
    (50, 'User50', 'user50@test.com', 50, 'active'),
    (51, 'User51', 'user51@test.com', 51, 'active'),
    (52, 'User52', 'user52@test.com', 52, 'active'),
    (53, 'User53', 'user53@test.com', 53, 'active'),
    (54, 'User54', 'user54@test.com', 54, 'active'),
    (55, 'User55', 'user55@test.com', 55, 'active'),
    (56, 'User56', 'user56@test.com', 56, 'active'),
    (57, 'User57', 'user57@test.com', 57, 'active'),
    (58, 'User58', 'user58@test.com', 58, 'active'),
    (59, 'User59', 'user59@test.com', 59, 'active'),
    (60, 'User60', 'user60@test.com', 60, 'active'),
    (61, 'User61', 'user61@test.com', 61, 'active'),
    (62, 'User62', 'user62@test.com', 62, 'active'),
    (63, 'User63', 'user63@test.com', 63, 'active'),
    (64, 'User64', 'user64@test.com', 64, 'active'),
    (65, 'User65', 'user65@test.com', 65, 'active'),
    (66, 'User66', 'user66@test.com', 66, 'active'),
    (67, 'User67', 'user67@test.com', 67, 'active'),
    (68, 'User68', 'user68@test.com', 68, 'active'),
    (69, 'User69', 'user69@test.com', 69, 'active'),
    (70, 'User70', 'user70@test.com', 70, 'active'),
    (71, 'User71', 'user71@test.com', 71, 'active'),
    (72, 'User72', 'user72@test.com', 72, 'active'),
    (73, 'User73', 'user73@test.com', 73, 'active'),
    (74, 'User74', 'user74@test.com', 74, 'active'),
    (75, 'User75', 'user75@test.com', 75, 'active'),
    (76, 'User76', 'user76@test.com', 76, 'active'),
    (77, 'User77', 'user77@test.com', 77, 'active'),
    (78, 'User78', 'user78@test.com', 78, 'active'),
    (79, 'User79', 'user79@test.com', 79, 'active'),
    (80, 'User80', 'user80@test.com', 80, 'active'),
    (81, 'User81', 'user81@test.com', 81, 'active'),
    (82, 'User82', 'user82@test.com', 82, 'active'),
    (83, 'User83', 'user83@test.com', 83, 'active'),
    (84, 'User84', 'user84@test.com', 84, 'active'),
    (85, 'User85', 'user85@test.com', 85, 'active'),
    (86, 'User86', 'user86@test.com', 86, 'active'),
    (87, 'User87', 'user87@test.com', 87, 'active'),
    (88, 'User88', 'user88@test.com', 88, 'active'),
    (89, 'User89', 'user89@test.com', 89, 'active'),
    (90, 'User90', 'user90@test.com', 90, 'active'),
    (91, 'User91', 'user91@test.com', 91, 'active'),
    (92, 'User92', 'user92@test.com', 92, 'active'),
    (93, 'User93', 'user93@test.com', 93, 'active'),
    (94, 'User94', 'user94@test.com', 94, 'active'),
    (95, 'User95', 'user95@test.com', 95, 'active'),
    (96, 'User96', 'user96@test.com', 96, 'active'),
    (97, 'User97', 'user97@test.com', 97, 'active'),
    (98, 'User98', 'user98@test.com', 98, 'active'),
    (99, 'User99', 'user99@test.com', 99, 'active'),
    (100, 'User100', 'user100@test.com', 100, 'active');

-- ============================================
-- 9. INSERT with Different Data Types
-- ============================================

-- Boolean
INSERT INTO settings (key, value, enabled) VALUES ('notifications', 'email', TRUE);

-- Hex values
INSERT INTO binary_data (id, data) VALUES (1, 0x48656C6C6F);

-- Bit values
INSERT INTO flags (id, flag) VALUES (1, b'1010');

-- Scientific notation
INSERT INTO measurements (id, value, unit) VALUES (1, 1.23E+5, 'meters');

-- ============================================
-- 10. INSERT with JSON Values
-- ============================================

INSERT INTO metadata (id, data) VALUES (1, '{"name": "test", "active": true}');

INSERT INTO metadata (id, data) VALUES (2, JSON_OBJECT('key', 'value', 'num', 42));

INSERT INTO metadata (id, data) VALUES (3, JSON_ARRAY(1, 2, 3, 'four'));

-- ============================================
-- 11. INSERT with Spatial Values
-- ============================================

-- INSERT INTO locations (id, name, coords) VALUES (1, 'HQ', ST_GeomFromText('POINT(0 0)'));

INSERT INTO areas (id, name, boundary) VALUES (1, 'Zone A', ST_GeomFromText('POLYGON((0 0, 10 0, 10 10, 0 10, 0 0))'));

-- ============================================
-- 12. INSERT with ENUM/SET
-- ============================================

-- Assuming users has a 'role' ENUM column
-- INSERT INTO users (id, name, email, age, status, role) VALUES (101, 'Admin', 'admin@test.com', 30, 'active', 'admin');

-- Assuming permissions has a 'perms' SET column
-- INSERT INTO permissions (id, name, perms) VALUES (1, 'Editor', 'read,write,delete');

-- ============================================
-- 13. INSERT with TIMESTAMP/DATETIME special values
-- ============================================

INSERT INTO logs (id, event, created_at) VALUES (1, 'login', '2024-01-15 10:30:00');

INSERT INTO logs (id, event, created_at) VALUES (2, 'logout', '2024-01-15 18:45:00');

INSERT INTO schedules (id, name, start_time, end_time) VALUES (1, 'Morning', '08:00:00', '12:00:00');

-- ============================================
-- 14. REPLACE (MySQL extension - delete + insert)
-- ============================================

REPLACE INTO users (id, name, email, age, status) VALUES (1, 'Alice', 'alice@new.com', 31, 'active');

REPLACE INTO products (id, name, price, stock) VALUES (2, 'Updated Gadget', 39.99, 150);

-- ============================================
-- 15. INSERT ... SELECT with joins
-- ============================================

-- INSERT INTO user_orders (user_name, product_name, order_date)
-- SELECT u.name, p.name, o.created_at
-- FROM users u
-- JOIN orders o ON u.id = o.user_id
-- JOIN products p ON o.product_id = p.id;

-- ============================================
-- 16. Conditional INSERT
-- ============================================

-- Insert only if condition is met
INSERT INTO premium_users (id, name, email)
SELECT id, name, email FROM users WHERE age > 65 AND status = 'active';

-- Insert with IIF (MySQL 8.0+)
-- INSERT INTO categories (id, name, type) VALUES (1, 'Electronics', IIF(1=1, 'Tech', 'Other'));
