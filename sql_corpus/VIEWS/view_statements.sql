-- MySQL 5.7.0 View Test Statements
-- Comprehensive coverage of view creation, usage, and updatability

DROP TABLE IF EXISTS view_test_orders;
DROP TABLE IF EXISTS view_test_order_items;
DROP TABLE IF EXISTS view_test_products;
DROP TABLE IF EXISTS view_test_users;
DROP TABLE IF EXISTS view_test_categories;
DROP VIEW IF EXISTS view_active_users;
DROP VIEW IF EXISTS view_pending_orders;
DROP VIEW IF EXISTS view_order_summary;
DROP VIEW IF EXISTS view_user_order_stats;
DROP VIEW IF EXISTS view_product_sales;
DROP VIEW IF EXISTS view_monthly_sales;

-- Create base tables
CREATE TABLE view_test_users (
    id INT PRIMARY KEY AUTO_INCREMENT,
    username VARCHAR(50) NOT NULL,
    email VARCHAR(100) NOT NULL,
    status ENUM('active', 'inactive', 'suspended') DEFAULT 'active',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE view_test_categories (
    id INT PRIMARY KEY AUTO_INCREMENT,
    name VARCHAR(50) NOT NULL,
    parent_id INT DEFAULT NULL
);

CREATE TABLE view_test_products (
    id INT PRIMARY KEY AUTO_INCREMENT,
    name VARCHAR(100) NOT NULL,
    category_id INT,
    price DECIMAL(10, 2) NOT NULL,
    stock INT DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE view_test_orders (
    id INT PRIMARY KEY AUTO_INCREMENT,
    user_id INT NOT NULL,
    order_number VARCHAR(50) NOT NULL UNIQUE,
    total_amount DECIMAL(10, 2) DEFAULT 0.00,
    status ENUM('pending', 'processing', 'shipped', 'delivered', 'cancelled') DEFAULT 'pending',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE view_test_order_items (
    id INT PRIMARY KEY AUTO_INCREMENT,
    order_id INT NOT NULL,
    product_id INT NOT NULL,
    quantity INT NOT NULL DEFAULT 1,
    unit_price DECIMAL(10, 2) NOT NULL
);

-- Insert sample data
INSERT INTO view_test_users (username, email, status) VALUES
('john', 'john@example.com', 'active'),
('jane', 'jane@example.com', 'active'),
('bob', 'bob@example.com', 'inactive'),
('alice', 'alice@example.com', 'active'),
('charlie', 'charlie@example.com', 'suspended');

INSERT INTO view_test_categories (name, parent_id) VALUES
('Electronics', NULL),
('Computers', 1),
('Phones', 1),
('Books', NULL),
('Fiction', 4);

INSERT INTO view_test_products (name, category_id, price, stock) VALUES
('Laptop', 2, 999.99, 50),
('Phone', 3, 599.99, 100),
('Book', 5, 19.99, 200),
('Tablet', 2, 399.99, 75),
('Headphones', 1, 99.99, 150);

INSERT INTO view_test_orders (user_id, order_number, total_amount, status) VALUES
(1, 'ORD001', 1099.98, 'delivered'),
(1, 'ORD002', 199.98, 'pending'),
(2, 'ORD003', 599.99, 'shipped'),
(3, 'ORD004', 19.99, 'cancelled'),
(4, 'ORD005', 1199.98, 'processing');

INSERT INTO view_test_order_items (order_id, product_id, quantity, unit_price) VALUES
(1, 1, 1, 999.99),
(1, 5, 1, 99.99),
(2, 3, 10, 19.99),
(3, 2, 1, 599.99),
(4, 3, 1, 19.99),
(5, 1, 1, 999.99),
(5, 5, 2, 99.99);

-- ============================================
-- Basic View Creation
-- ============================================

CREATE VIEW view_active_users AS
SELECT id, username, email, created_at
FROM view_test_users
WHERE status = 'active';

CREATE VIEW view_pending_orders AS
SELECT o.id, o.order_number, o.total_amount, o.created_at, u.username
FROM view_test_orders o
JOIN view_test_users u ON o.user_id = u.id
WHERE o.status = 'pending';

CREATE VIEW view_order_summary AS
SELECT
    o.id,
    o.order_number,
    u.username,
    u.email,
    o.total_amount,
    o.status,
    o.created_at
FROM view_test_orders o
JOIN view_test_users u ON o.user_id = u.id;

-- ============================================
-- View with Aggregates
-- ============================================

CREATE VIEW view_user_order_stats AS
SELECT
    u.id,
    u.username,
    u.email,
    COUNT(o.id) AS order_count,
    COALESCE(SUM(o.total_amount), 0) AS total_spent,
    MAX(o.created_at) AS last_order_date
FROM view_test_users u
LEFT JOIN view_test_orders o ON u.id = o.user_id
GROUP BY u.id, u.username, u.email;

CREATE VIEW view_product_sales AS
SELECT
    p.id,
    p.name,
    p.price,
    p.stock,
    c.name AS category_name,
    SUM(oi.quantity) AS total_sold,
    SUM(oi.quantity * oi.unit_price) AS total_revenue
FROM view_test_products p
LEFT JOIN view_test_order_items oi ON p.id = oi.product_id
LEFT JOIN view_test_categories c ON p.category_id = c.id
GROUP BY p.id, p.name, p.price, p.stock, c.name;

CREATE VIEW view_monthly_sales AS
SELECT
    DATE_FORMAT(o.created_at, '%Y-%m') AS month,
    COUNT(DISTINCT o.id) AS order_count,
    SUM(o.total_amount) AS total_revenue,
    COUNT(DISTINCT o.user_id) AS unique_customers
FROM view_test_orders o
WHERE o.status != 'cancelled'
GROUP BY DATE_FORMAT(o.created_at, '%Y-%m');

-- ============================================
-- View with Subqueries
-- ============================================

CREATE VIEW view_high_value_users AS
SELECT u.*
FROM view_test_users u
WHERE EXISTS (
    SELECT 1 FROM view_test_orders o
    WHERE o.user_id = u.id AND o.total_amount > 500
);

CREATE VIEW view_out_of_stock AS
SELECT
    p.id,
    p.name,
    p.stock,
    COALESCE(SUM(oi.quantity), 0) AS reserved
FROM view_test_products p
LEFT JOIN view_test_order_items oi ON p.id = oi.product_id
GROUP BY p.id, p.name, p.stock
HAVING p.stock = 0;

CREATE VIEW view_popular_products AS
SELECT p.*
FROM view_test_products p
WHERE p.id IN (
    SELECT product_id
    FROM view_test_order_items
    GROUP BY product_id
    HAVING SUM(quantity) > 5
);

-- ============================================
-- Updatable View Tests
-- ============================================

-- Simple updatable view
INSERT INTO view_active_users (username, email, created_at) VALUES ('newuser', 'new@example.com', NOW());

-- Update through view
UPDATE view_active_users SET email = 'newemail@example.com' WHERE username = 'john';

-- Delete through view
DELETE FROM view_active_users WHERE username = 'alice';

-- ============================================
-- View with CHECK OPTION
-- ============================================

DROP VIEW IF EXISTS view_check_test;

CREATE VIEW view_check_test AS
SELECT id, username, email, status
FROM view_test_users
WHERE status = 'active'
WITH CHECK OPTION;

-- This should succeed (status stays active)
INSERT INTO view_check_test (username, email, status) VALUES ('test1', 'test1@example.com', 'active');

-- This should fail (would violate CHECK OPTION)
-- INSERT INTO view_check_test (username, email, status) VALUES ('test2', 'test2@example.com', 'inactive');

-- ============================================
-- View with Join
-- ============================================

CREATE VIEW view_order_details AS
SELECT
    o.id,
    o.order_number,
    u.username,
    u.email,
    p.name AS product_name,
    oi.quantity,
    oi.unit_price,
    (oi.quantity * oi.unit_price) AS line_total
FROM view_test_orders o
JOIN view_test_users u ON o.user_id = u.id
JOIN view_test_order_items oi ON o.id = oi.order_id
JOIN view_test_products p ON oi.product_id = p.id;

-- ============================================
-- View with UNION
-- ============================================

CREATE VIEW view_all_emails AS
SELECT email FROM view_test_users
UNION
SELECT email FROM view_test_products;

-- ============================================
-- View Metadata
-- ============================================

SHOW CREATE VIEW view_active_users;
SHOW CREATE VIEW view_order_summary;
SHOW CREATE VIEW view_user_order_stats;

-- ============================================
-- Select from Views
-- ============================================

SELECT * FROM view_active_users;
SELECT * FROM view_pending_orders;
SELECT * FROM view_order_summary;
SELECT * FROM view_user_order_stats;
SELECT * FROM view_product_sales;
SELECT * FROM view_monthly_sales;
SELECT * FROM view_high_value_users;
SELECT * FROM view_out_of_stock;
SELECT * FROM view_popular_products;
SELECT * FROM view_order_details;

-- ============================================
-- View in WHERE Clause
-- ============================================

SELECT * FROM view_user_order_stats WHERE total_spent > 100;
SELECT * FROM view_product_sales WHERE total_sold > 0;
SELECT * FROM view_monthly_sales WHERE order_count > 1;

-- ============================================
-- View in JOIN
-- ============================================

SELECT
    v.username,
    v.total_spent,
    o.order_number
FROM view_user_order_stats v
JOIN view_test_orders o ON v.id = o.user_id
WHERE o.status = 'delivered';

-- ============================================
-- View with GROUP BY and HAVING
-- ============================================

SELECT * FROM view_product_sales
WHERE category_name = 'Electronics'
HAVING total_revenue > 0;

-- ============================================
-- Drop View
-- ============================================

DROP VIEW view_active_users;
DROP VIEW view_pending_orders;
DROP VIEW view_order_summary;
DROP VIEW view_user_order_stats;
DROP VIEW view_product_sales;
DROP VIEW view_monthly_sales;
DROP VIEW view_high_value_users;
DROP VIEW view_out_of_stock;
DROP VIEW view_popular_products;
DROP VIEW view_order_details;
DROP VIEW view_all_emails;
DROP VIEW view_check_test;

-- ============================================
-- Cleanup
-- ============================================

DROP TABLE IF EXISTS view_test_order_items;
DROP TABLE IF EXISTS view_test_orders;
DROP TABLE IF EXISTS view_test_products;
DROP TABLE IF EXISTS view_test_categories;
DROP TABLE IF EXISTS view_test_users;
