-- MySQL 5.7.0 Index Test Statements
-- Comprehensive coverage of index creation, usage, and hints

-- Create test tables
DROP TABLE IF EXISTS idx_test_users;
DROP TABLE IF EXISTS idx_test_products;
DROP TABLE IF EXISTS idx_test_orders;
DROP TABLE IF EXISTS idx_test_order_items;

CREATE TABLE idx_test_users (
    id INT PRIMARY KEY AUTO_INCREMENT,
    username VARCHAR(50) NOT NULL,
    email VARCHAR(100) NOT NULL,
    first_name VARCHAR(50),
    last_name VARCHAR(50),
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    status ENUM('active', 'inactive', 'pending') DEFAULT 'active',
    INDEX idx_username (username),
    INDEX idx_email (email)
);

CREATE TABLE idx_test_products (
    id INT PRIMARY KEY AUTO_INCREMENT,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    price DECIMAL(10, 2) NOT NULL,
    category_id INT,
    brand VARCHAR(50),
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FULLTEXT INDEX idx_fulltext_name_desc (name, description)
);

CREATE TABLE idx_test_orders (
    id INT PRIMARY KEY AUTO_INCREMENT,
    user_id INT NOT NULL,
    order_number VARCHAR(50) NOT NULL,
    total_amount DECIMAL(10, 2) DEFAULT 0.00,
    status ENUM('pending', 'processing', 'shipped', 'delivered', 'cancelled') DEFAULT 'pending',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    shipped_at DATETIME,
    UNIQUE INDEX idx_order_number (order_number),
    INDEX idx_user_id (user_id),
    INDEX idx_status (status),
    INDEX idx_created_at (created_at)
);

CREATE TABLE idx_test_order_items (
    id INT PRIMARY KEY AUTO_INCREMENT,
    order_id INT NOT NULL,
    product_id INT NOT NULL,
    quantity INT NOT NULL DEFAULT 1,
    unit_price DECIMAL(10, 2) NOT NULL,
    INDEX idx_order_id (order_id),
    INDEX idx_product_id (product_id),
    INDEX idx_order_product (order_id, product_id)
);

-- ============================================
-- SINGLE COLUMN INDEX TESTS
-- ============================================

-- Basic index creation
CREATE INDEX idx_users_first_name ON idx_test_users(first_name);
CREATE INDEX idx_users_last_name ON idx_test_users(last_name);
CREATE INDEX idx_users_status ON idx_test_users(status);
CREATE INDEX idx_users_created ON idx_test_users(created_at);

-- Index on VARCHAR column with prefix
CREATE INDEX idx_users_username_prefix ON idx_test_users(username(10));
CREATE INDEX idx_users_email_prefix ON idx_test_users(email(20));

-- Index on ENUM column
CREATE INDEX idx_orders_status ON idx_test_orders(status);

-- Index on DATETIME column
CREATE INDEX idx_orders_shipped ON idx_test_orders(shipped_at);

-- ============================================
-- COMPOSITE INDEX TESTS
-- ============================================

-- Composite index on multiple columns
CREATE INDEX idx_composite_user_status ON idx_test_users(status, created_at);
CREATE INDEX idx_composite_name_brand ON idx_test_products(name, brand);
CREATE INDEX idx_composite_order_user_status ON idx_test_orders(user_id, status, created_at);

-- Composite index with different column orders
CREATE INDEX idx_composite_status_created ON idx_test_orders(status, created_at);
CREATE INDEX idx_composite_created_status ON idx_test_orders(created_at, status);

-- ============================================
-- UNIQUE INDEX TESTS
-- ============================================

-- Unique index creation
CREATE UNIQUE INDEX idx_unique_product_name ON idx_test_products(name);
CREATE UNIQUE INDEX idx_unique_user_email ON idx_test_users(email);

-- Unique composite index
CREATE UNIQUE INDEX idx_unique_user_product ON idx_test_order_items(order_id, product_id);

-- ============================================
-- FULLTEXT INDEX TESTS
-- ============================================

-- Fulltext index (already created in table, testing additional)
-- Note: Each table can have only one FULLTEXT index
ALTER TABLE idx_test_users ADD FULLTEXT INDEX idx_fulltext_username (username);

-- ============================================
-- SPATIAL INDEX TESTS (MySQL 5.7)
-- ============================================

-- Note: Spatial indexes require geometry or point types
-- These are for MySQL 5.7+ GIS features
-- CREATE TABLE idx_test_geo (
--     id INT PRIMARY KEY,
--     location POINT NOT NULL SRID 0,
--     SPATIAL INDEX idx_spatial_location (location)
-- );

-- ============================================
-- ALTER TABLE INDEX OPERATIONS
-- ============================================

-- Add index
ALTER TABLE idx_test_users ADD INDEX idx_add_test (username);

-- Add unique index
ALTER TABLE idx_test_users ADD UNIQUE INDEX idx_unique_add (username);

-- Add composite index
ALTER TABLE idx_test_users ADD INDEX idx_composite_add (status, username);

-- Drop index
ALTER TABLE idx_test_users DROP INDEX idx_add_test;

-- Drop unique index
ALTER TABLE idx_test_users DROP INDEX idx_unique_add;

-- Add primary key (creates index automatically)
-- ALTER TABLE idx_test_products ADD PRIMARY KEY (id);

-- Drop primary key
-- ALTER TABLE idx_test_products DROP PRIMARY KEY;

-- ============================================
-- INDEX HINTS IN SELECT
-- ============================================

-- USE INDEX hint
SELECT * FROM idx_test_users USE INDEX (idx_username) WHERE username = 'john';
SELECT * FROM idx_test_orders USE INDEX (idx_user_id) WHERE user_id = 1;

-- FORCE INDEX hint
SELECT * FROM idx_test_orders FORCE INDEX (idx_status) WHERE status = 'pending' ORDER BY created_at;

-- IGNORE INDEX hint
SELECT * FROM idx_test_orders IGNORE INDEX (idx_status) WHERE user_id = 1;

-- Multiple index hints
SELECT * FROM idx_test_orders USE INDEX (idx_user_id, idx_status) WHERE user_id = 1 AND status = 'pending';

-- Index hint with JOIN
SELECT u.username, o.order_number
FROM idx_test_users u USE INDEX (idx_username)
JOIN idx_test_orders o USE INDEX (idx_user_id) ON u.id = o.user_id
WHERE u.username = 'john';

-- ============================================
-- DROP INDEX STATEMENTS
-- ============================================

DROP INDEX idx_users_first_name ON idx_test_users;
DROP INDEX idx_users_last_name ON idx_test_users;
DROP INDEX idx_unique_user_email ON idx_test_users;
DROP INDEX idx_composite_user_status ON idx_test_users;

-- ============================================
-- SHOW INDEX STATEMENTS
-- ============================================

SHOW INDEX FROM idx_test_users;
SHOW INDEX FROM idx_test_products;
SHOW INDEX FROM idx_test_orders;
SHOW INDEX FROM idx_test_order_items;

-- Show index with verbose
SHOW INDEX FROM idx_test_users\G

-- ============================================
-- INDEX OPTIMIZATION QUERIES
-- ============================================

-- Queries that would benefit from indexes
SELECT * FROM idx_test_users WHERE username = 'john_doe' AND status = 'active';
SELECT * FROM idx_test_users WHERE email LIKE 'john%';
SELECT * FROM idx_test_orders WHERE user_id = 1 AND status = 'pending';
SELECT * FROM idx_test_orders WHERE status IN ('pending', 'processing');
SELECT * FROM idx_test_orders WHERE created_at BETWEEN '2024-01-01' AND '2024-12-31';
SELECT * FROM idx_test_products WHERE price BETWEEN 10.00 AND 100.00;
SELECT o.*, u.username
FROM idx_test_orders o
JOIN idx_test_users u ON o.user_id = u.id
WHERE o.status = 'delivered' AND u.status = 'active';

-- ============================================
-- INDEX ON EXPRESSIONS (Functional Index Concepts in MySQL 5.7)
-- ============================================

-- MySQL 5.7 supports generated columns which can be used for expression indexes
ALTER TABLE idx_test_users ADD COLUMN username_lower VARCHAR(50) GENERATED ALWAYS AS (LOWER(username)) STORED;
CREATE INDEX idx_username_lower ON idx_test_users(username_lower);

-- Using the expression index
SELECT * FROM idx_test_users WHERE LOWER(username) = 'john_doe';

-- Another example with price calculation
ALTER TABLE idx_test_products ADD COLUMN price_with_tax DECIMAL(10,2) GENERATED ALWAYS AS (price * 1.1) STORED;
CREATE INDEX idx_price_tax ON idx_test_products(price_with_tax);

-- ============================================
-- FOREIGN KEY INDEXES (Auto-created but testing verification)
-- ============================================

-- Show that foreign keys create implicit indexes
-- (user_id in orders table references users.id)
SHOW INDEX FROM idx_test_orders WHERE Column_name = 'user_id';

-- ============================================
-- HANDLER COMMANDS FOR INDEX ITERATION
-- ============================================

-- Handler commands for low-level index access
-- These are MySQL C API commands but syntax shown for testing
-- HLOW INDEX name OPEN
-- HLOW INDEX name READ name [= value] (FIRST | NEXT | PREV | LAST)
-- HLOW INDEX name CLOSE

-- ============================================
-- OPTIMIZE TABLE (Reorganizes index pages)
-- ============================================

OPTIMIZE TABLE idx_test_users;
OPTIMIZE TABLE idx_test_orders;
OPTIMIZE TABLE idx_test_products;

-- ============================================
-- ANALYZE TABLE (Updates index statistics)
-- ============================================

ANALYZE TABLE idx_test_users;
ANALYZE TABLE idx_test_orders;
ANALYZE TABLE idx_test_products;
ANALYZE TABLE idx_test_order_items;

-- ============================================
-- CHECK TABLE (Verifies index integrity)
-- ============================================

CHECK TABLE idx_test_users;
CHECK TABLE idx_test_orders;

-- ============================================
-- REPAIR TABLE (Repairs index issues)
-- ============================================

-- REPAIR TABLE idx_test_users;

-- ============================================
-- ADVANCED INDEX PATTERNS
-- ============================================

-- Index on calculated field
ALTER TABLE idx_test_orders ADD COLUMN order_year INT GENERATED ALWAYS AS (YEAR(created_at)) STORED;
CREATE INDEX idx_order_year ON idx_test_orders(order_year);

-- Index for NULL handling
CREATE INDEX idx_shipped_at ON idx_test_orders(shipped_at);
SELECT * FROM idx_test_orders WHERE shipped_at IS NULL;
SELECT * FROM idx_test_orders WHERE shipped_at IS NOT NULL;

-- Index for string operations
CREATE INDEX idx_brand ON idx_test_products(brand);
SELECT * FROM idx_test_products WHERE LEFT(brand, 3) = 'App';

-- Composite index column order optimization
-- Testing different column orders for same data
CREATE INDEX idx_status_date ON idx_test_orders(status, created_at);
CREATE INDEX idx_date_status ON idx_test_orders(created_at, status);

-- ============================================
-- INDEX FOR JOIN OPTIMIZATION
-- ============================================

SELECT o.id, o.order_number, u.username, u.email
FROM idx_test_orders o
INNER JOIN idx_test_users u ON o.user_id = u.id
WHERE o.status = 'shipped';

SELECT oi.*, p.name, p.price
FROM idx_test_order_items oi
INNER JOIN idx_test_products p ON oi.product_id = p.id
WHERE oi.quantity > 5;

SELECT u.username, COUNT(o.id) as order_count
FROM idx_test_users u
LEFT JOIN idx_test_orders o ON u.id = o.user_id
WHERE u.status = 'active'
GROUP BY u.id, u.username
HAVING COUNT(o.id) > 0;

-- ============================================
-- COVERING INDEX QUERIES
-- ============================================

-- Query that can be satisfied using only the index (covering index)
SELECT username, email FROM idx_test_users WHERE status = 'active';
SELECT order_number, status, created_at FROM idx_test_orders WHERE user_id = 1;
SELECT name, price FROM idx_test_products WHERE category_id = 5;

-- ============================================
-- INDEX FOR ORDER BY OPTIMIZATION
-- ============================================

SELECT * FROM idx_test_orders ORDER BY created_at DESC LIMIT 10;
SELECT * FROM idx_test_orders WHERE status = 'pending' ORDER BY created_at ASC;
SELECT * FROM idx_test_users ORDER BY created_at DESC LIMIT 100;

-- ============================================
-- INDEX FOR GROUP BY OPTIMIZATION
-- ============================================

SELECT status, COUNT(*) as count FROM idx_test_users GROUP BY status;
SELECT user_id, SUM(total_amount) as total FROM idx_test_orders GROUP BY user_id;
SELECT DATE(created_at) as order_date, COUNT(*) as orders FROM idx_test_orders GROUP BY DATE(created_at);

-- ============================================
-- MULTI-TABLE INDEX HINTS
-- ============================================

SELECT * FROM idx_test_users u
USE INDEX (idx_status)
JOIN idx_test_orders o USE INDEX (idx_user_id) ON u.id = o.user_id
WHERE u.status = 'active' AND o.status = 'pending';

SELECT /*+ NO_JOIN_COND_PUSHDOWN(u) */ * FROM idx_test_users u;

-- MySQL 5.7 optimizer hints
SELECT /*+ MAX_EXECUTION_TIME(1000) */ * FROM idx_test_users;

-- ============================================
-- CONDITIONAL INDEXES (Via Generated Columns)
-- ============================================

-- Index only active orders
ALTER TABLE idx_test_orders ADD COLUMN is_active_order TINYINT(1) GENERATED ALWAYS AS (
    CASE WHEN status IN ('pending', 'processing', 'shipped') THEN 1 ELSE 0 END
) STORED;
CREATE INDEX idx_active_orders ON idx_test_orders(is_active_order);

-- Index for high-value orders
ALTER TABLE idx_test_orders ADD COLUMN is_high_value TINYINT(1) GENERATED ALWAYS AS (
    CASE WHEN total_amount > 1000 THEN 1 ELSE 0 END
) STORED;
CREATE INDEX idx_high_value_orders ON idx_test_orders(is_high_value);

-- ============================================
-- DROP ALL TEST TABLES
-- ============================================

DROP TABLE IF EXISTS idx_test_order_items;
DROP TABLE IF EXISTS idx_test_orders;
DROP TABLE IF EXISTS idx_test_products;
DROP TABLE IF EXISTS idx_test_users;
