-- DDL Statement Test Cases
-- Compatibility: MySQL 5.7+

-- ============================================
-- 1. CREATE TABLE - Basic
-- ============================================

CREATE TABLE users (
    id INT PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    email VARCHAR(255) UNIQUE,
    age INT,
    status VARCHAR(20) DEFAULT 'active',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE products (
    id INT AUTO_INCREMENT PRIMARY KEY,
    name VARCHAR(200) NOT NULL,
    price DECIMAL(10, 2) NOT NULL,
    stock INT DEFAULT 0,
    category_id INT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_category (category_id)
);

-- ============================================
-- 2. CREATE TABLE - With Keys and Constraints
-- ============================================

CREATE TABLE orders (
    id INT PRIMARY KEY AUTO_INCREMENT,
    user_id INT NOT NULL,
    total DECIMAL(10, 2) NOT NULL,
    status VARCHAR(20) DEFAULT 'pending',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id),
    INDEX idx_user (user_id),
    INDEX idx_status (status)
);

CREATE TABLE order_items (
    id INT PRIMARY KEY AUTO_INCREMENT,
    order_id INT NOT NULL,
    product_id INT NOT NULL,
    quantity INT NOT NULL,
    price DECIMAL(10, 2) NOT NULL,
    FOREIGN KEY (order_id) REFERENCES orders(id) ON DELETE CASCADE,
    FOREIGN KEY (product_id) REFERENCES products(id),
    UNIQUE KEY uk_order_product (order_id, product_id)
);

CREATE TABLE categories (
    id INT PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    parent_id INT,
    FOREIGN KEY (parent_id) REFERENCES categories(id) ON DELETE SET NULL
);

-- ============================================
-- 3. CREATE TABLE - With Indexes
-- ============================================

CREATE TABLE articles (
    id INT PRIMARY KEY AUTO_INCREMENT,
    title VARCHAR(500) NOT NULL,
    content TEXT,
    author_id INT,
    published_at DATETIME,
    status VARCHAR(20) DEFAULT 'draft',
    FULLTEXT INDEX ft_title_content (title, content),
    INDEX idx_author (author_id),
    INDEX idx_published (published_at),
    INDEX idx_status_published (status, published_at)
);

CREATE TABLE logs (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    level VARCHAR(20) NOT NULL,
    message TEXT,
    context JSON,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_level (level),
    INDEX idx_created (created_at)
);

-- ============================================
-- 4. CREATE TABLE - With Different Data Types
-- ============================================

CREATE TABLE data_types_demo (
    id INT PRIMARY KEY,
    -- Numeric types
    tiny_col TINYINT,
    small_col SMALLINT,
    medium_col MEDIUMINT,
    int_col INT,
    big_col BIGINT,
    float_col FLOAT,
    double_col DOUBLE,
    decimal_col DECIMAL(15, 3),
    bit_col BIT(8),
    -- String types
    char_col CHAR(10),
    varchar_col VARCHAR(255),
    tinytext_col TINYTEXT,
    text_col TEXT,
    mediumtext_col MEDIUMTEXT,
    longtext_col LONGTEXT,
    binary_col BINARY(16),
    varbinary_col VARBINARY(255),
    blob_col BLOB,
    -- Date/Time types
    date_col DATE,
    time_col TIME,
    datetime_col DATETIME,
    timestamp_col TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    year_col YEAR,
    -- Enum and Set
    enum_col ENUM('pending', 'active', 'completed'),
    set_col SET('a', 'b', 'c'),
    -- Spatial types
    point_col POINT,
    linestring_col LINESTRING,
    polygon_col POLYGON,
    geometry_col GEOMETRY
);

-- ============================================
-- 5. CREATE TABLE - With JSON
-- ============================================

CREATE TABLE metadata (
    id INT PRIMARY KEY,
    config JSON NOT NULL,
    tags JSON,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE api_logs (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    method VARCHAR(10) NOT NULL,
    path VARCHAR(500),
    headers JSON,
    body JSON,
    response JSON,
    duration_ms INT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_method_path (method, path(100)),
    INDEX idx_created (created_at)
);

-- ============================================
-- 6. CREATE TABLE - With Virtual Columns
-- ============================================

CREATE TABLE temperatures (
    id INT PRIMARY KEY,
    celsius DECIMAL(5, 2),
    fahrenheit DECIMAL(5, 2) AS (celsius * 9/5 + 32) STORED,
    kelvin DECIMAL(6, 2) AS (celsius + 273.15) STORED
);

CREATE TABLE products_net (
    id INT PRIMARY KEY,
    price DECIMAL(10, 2),
    tax_rate DECIMAL(5, 4) DEFAULT 0.08,
    price_with_tax DECIMAL(10, 2) AS (price * (1 + tax_rate)) STORED,
    discount_percent DECIMAL(5, 2) DEFAULT 0,
    final_price DECIMAL(10, 2) AS (price_with_tax * (1 - discount_percent / 100)) STORED
);

-- ============================================
-- 7. CREATE TABLE - LIKE (copy structure)
-- ============================================

CREATE TABLE users_backup LIKE users;

CREATE TABLE products_archive LIKE products;

-- ============================================
-- 8. CREATE TABLE - AS SELECT (copy with data)
-- ============================================

CREATE TABLE active_users AS SELECT * FROM users WHERE status = 'active';

CREATE TABLE user_orders_summary AS
SELECT
    u.id,
    u.name,
    COUNT(o.id) as order_count,
    COALESCE(SUM(o.total), 0) as total_spent
FROM users u
LEFT JOIN orders o ON u.id = o.user_id
GROUP BY u.id, u.name;

-- ============================================
-- 9. CREATE TABLE - With Partitioning
-- ============================================

CREATE TABLE sales (
    id INT PRIMARY KEY,
    sale_date DATE NOT NULL,
    amount DECIMAL(10, 2) NOT NULL,
    region VARCHAR(50)
)
PARTITION BY RANGE (YEAR(sale_date)) (
    PARTITION p2020 VALUES LESS THAN (2021),
    PARTITION p2021 VALUES LESS THAN (2022),
    PARTITION p2022 VALUES LESS THAN (2023),
    PARTITION p2023 VALUES LESS THAN (2024),
    PARTITION pmax VALUES LESS THAN MAXVALUE
);

CREATE TABLE events (
    id INT PRIMARY KEY,
    event_date DATETIME NOT NULL,
    event_type VARCHAR(50)
)
PARTITION BY RANGE (TO_DAYS(event_date)) (
    PARTITION p_old VALUES LESS THAN (TO_DAYS('2024-01-01')),
    PARTITION p_2024 VALUES LESS THAN (TO_DAYS('2025-01-01')),
    PARTITION p_future VALUES LESS THAN MAXVALUE
);

-- ============================================
-- 10. ALTER TABLE - Add columns
-- ============================================

ALTER TABLE users ADD COLUMN phone VARCHAR(20) AFTER email;

ALTER TABLE users ADD COLUMN is_verified BOOLEAN DEFAULT FALSE;

ALTER TABLE users ADD COLUMN last_login TIMESTAMP NULL;

ALTER TABLE products ADD COLUMN weight DECIMAL(8, 2) AFTER price;

ALTER TABLE orders ADD COLUMN shipped_at DATETIME AFTER status;

ALTER TABLE users ADD COLUMN (full_name VARCHAR(200), initials CHAR(3));

-- ============================================
-- 11. ALTER TABLE - Drop columns
-- ============================================

ALTER TABLE users DROP COLUMN phone;

ALTER TABLE products DROP COLUMN weight, DROP COLUMN dimensions;

ALTER TABLE orders DROP COLUMN shipped_at;

-- ============================================
-- 12. ALTER TABLE - Modify columns
-- ============================================

ALTER TABLE users MODIFY COLUMN name VARCHAR(150) NOT NULL;

ALTER TABLE users MODIFY COLUMN email VARCHAR(300) UNIQUE;

ALTER TABLE products MODIFY COLUMN price DECIMAL(12, 2) NOT NULL;

ALTER TABLE users MODIFY COLUMN age TINYINT UNSIGNED;

ALTER TABLE users MODIFY COLUMN status ENUM('pending', 'active', 'inactive', 'banned') DEFAULT 'pending';

-- ============================================
-- 13. ALTER TABLE - Rename
-- ============================================

ALTER TABLE users RENAME TO app_users;

ALTER TABLE products RENAME COLUMN name TO product_name;

ALTER TABLE orders RENAME COLUMN total TO order_total;

-- Rename back
ALTER TABLE app_users RENAME TO users;

-- ============================================
-- 14. ALTER TABLE - Add/Drop indexes
-- ============================================

ALTER TABLE users ADD INDEX idx_age_status (age, status);

ALTER TABLE products ADD FULLTEXT INDEX ft_name_desc (name, description);

ALTER TABLE users DROP INDEX idx_age_status;

ALTER TABLE users ADD UNIQUE INDEX uk_email (email);

-- ============================================
-- 15. ALTER TABLE - Add/Drop Keys/Constraints
-- ============================================

ALTER TABLE users ADD CONSTRAINT fk_department FOREIGN KEY (department_id) REFERENCES departments(id);

ALTER TABLE orders DROP FOREIGN KEY fk_user;

ALTER TABLE users ADD CONSTRAINT chk_age CHECK (age >= 0 AND age <= 150);

-- ============================================
-- 16. ALTER TABLE - Change columns
-- ============================================

ALTER TABLE users CHANGE COLUMN name full_name VARCHAR(200) NOT NULL;

ALTER TABLE products CHANGE COLUMN price price_usd DECIMAL(10, 2);

ALTER TABLE orders CHANGE COLUMN status order_status VARCHAR(30) DEFAULT 'pending';

-- ============================================
-- 17. ALTER TABLE - Set/Drop defaults
-- ============================================

ALTER TABLE users ALTER COLUMN status SET DEFAULT 'new';

ALTER TABLE users ALTER COLUMN age DROP DEFAULT;

-- ============================================
-- 18. ALTER TABLE - Engine options
-- ============================================

ALTER TABLE users ENGINE = InnoDB;

ALTER TABLE logs ENGINE = MyISAM;

ALTER TABLE users ENGINE = InnoDB ROW_FORMAT = DYNAMIC;

-- ============================================
-- 19. ALTER TABLE - Character set and collation
-- ============================================

ALTER TABLE users CONVERT TO CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;

ALTER TABLE users MODIFY COLUMN name VARCHAR(100) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;

-- ============================================
-- 20. DROP TABLE operations
-- ============================================

DROP TABLE IF EXISTS temp_data;

DROP TABLE users_backup;

-- DROP TABLE IF EXISTS old_logs, deprecated_table, unused_data;

-- ============================================
-- 21. TRUNCATE TABLE
-- ============================================

TRUNCATE TABLE logs;

TRUNCATE TABLE session_data;

-- ============================================
-- 22. CREATE INDEX
-- ============================================

CREATE INDEX idx_users_email ON users(email);

CREATE UNIQUE INDEX uk_users_email ON users(email);

CREATE FULLTEXT INDEX ft_articles ON articles(title, content);

CREATE INDEX idx_orders_user_date ON orders(user_id, created_at);

CREATE INDEX idx_products_category_price ON products(category_id, price);

-- Spatial index
-- CREATE SPATIAL INDEX sp_locations ON locations(coordinates);

-- ============================================
-- 23. DROP INDEX
-- ============================================

DROP INDEX idx_users_email ON users;

DROP INDEX ft_articles ON articles;

-- ============================================
-- 24. CREATE DATABASE
-- ============================================

CREATE DATABASE IF NOT EXISTS myapp;

CREATE DATABASE IF NOT EXISTS myapp CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;

CREATE DATABASE IF NOT EXISTS myapp CHARACTER SET latin1;

-- ============================================
-- 25. ALTER DATABASE
-- ============================================

ALTER DATABASE myapp CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;

ALTER DATABASE myapp READ ONLY = 1;

-- ============================================
-- 26. DROP DATABASE
-- ============================================

DROP DATABASE IF EXISTS myapp_backup;

-- ============================================
-- 27. CREATE VIEW
-- ============================================

CREATE VIEW active_users_view AS SELECT id, name, email FROM users WHERE status = 'active';

CREATE OR REPLACE VIEW user_orders_view AS
SELECT
    u.id,
    u.name,
    COUNT(o.id) as order_count,
    SUM(o.total) as total_spent
FROM users u
LEFT JOIN orders o ON u.id = o.user_id
GROUP BY u.id, u.name;

CREATE VIEW products_with_categories AS
SELECT
    p.id,
    p.name as product_name,
    p.price,
    c.name as category_name
FROM products p
LEFT JOIN categories c ON p.category_id = c.id;

-- ============================================
-- 28. ALTER VIEW
-- ============================================

ALTER VIEW active_users_view AS SELECT id, name, email, created_at FROM users WHERE status = 'active';

-- ============================================
-- 29. DROP VIEW
-- ============================================

DROP VIEW IF EXISTS active_users_view;

DROP VIEW IF EXISTS user_orders_view, products_with_categories;

-- ============================================
-- 30. CREATE TEMPORARY TABLE
-- ============================================

CREATE TEMPORARY TABLE temp_results (
    id INT PRIMARY KEY,
    name VARCHAR(100),
    value DECIMAL(10, 2)
);

CREATE TEMPORARY TABLE filtered_users AS SELECT * FROM users WHERE age > 30;

-- ============================================
-- 31. CREATE TRIGGER
-- ============================================

-- CREATE TRIGGER before_users_insert BEFORE INSERT ON users
-- FOR EACH ROW SET NEW.created_at = NOW();

-- CREATE TRIGGER after_orders_update AFTER UPDATE ON orders
-- FOR EACH ROW INSERT INTO audit_log (action, table_name, record_id, old_data, new_data)
-- VALUES ('UPDATE', 'orders', NEW.id, OLD.status, NEW.status);

-- ============================================
-- 32. DROP TRIGGER
-- ============================================

-- DROP TRIGGER IF EXISTS before_users_insert;

-- ============================================
-- 33. CREATE PROCEDURE
-- ============================================

-- DELIMITER //
-- CREATE PROCEDURE get_user_orders(IN user_id INT)
-- BEGIN
--     SELECT * FROM orders WHERE user_id = user_id;
-- END //
-- DELIMITER ;

-- DELIMITER //
-- CREATE PROCEDURE update_user_status(IN user_id INT, IN new_status VARCHAR(20))
-- BEGIN
--     UPDATE users SET status = new_status WHERE id = user_id;
-- END //
-- DELIMITER ;

-- ============================================
-- 34. CREATE FUNCTION
-- ============================================

-- DELIMITER //
-- CREATE FUNCTION calculate_discount(price DECIMAL(10,2), discount_percent INT)
-- RETURNS DECIMAL(10,2)
-- DETERMINISTIC
-- BEGIN
--     RETURN price * (1 - discount_percent / 100);
-- END //
-- DELIMITER ;

-- ============================================
-- 35. DROP FUNCTION/PROCEDURE
-- ============================================

-- DROP FUNCTION IF EXISTS calculate_discount;
-- DROP PROCEDURE IF EXISTS get_user_orders;

-- ============================================
-- 36. RENAME TABLE
-- ============================================

RENAME TABLE users TO app_users, products TO catalog_products;

RENAME TABLE app_users TO users;

-- ============================================
-- 37. CREATE EVENT (MySQL Event Scheduler)
-- ============================================

-- CREATE EVENT daily_cleanup ON SCHEDULE EVERY 1 DAY
-- DO DELETE FROM sessions WHERE last_activity < DATE_SUB(NOW(), INTERVAL 7 DAY);

-- CREATE EVENT weekly_report ON SCHEDULE AT CURRENT_TIMESTAMP + INTERVAL 1 WEEK
-- DO INSERT INTO reports (generated_at) VALUES (NOW());

-- ============================================
-- 38. DROP EVENT
-- ============================================

-- DROP EVENT IF EXISTS daily_cleanup;

-- ============================================
-- 39. CREATE SERVER (Federated tables)
-- ============================================

-- CREATE SERVER remote_server
-- FOREIGN DATA WRAPPER mysql
-- OPTIONS (HOST 'remote.example.com', DATABASE 'remote_db', USER 'remote_user', PASSWORD 'secret');

-- ============================================
-- 40. CREATE TABLE with specific options
-- ============================================

CREATE TABLE optimized_table (
    id INT PRIMARY KEY AUTO_INCREMENT,
    data JSON,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
) ENGINE = InnoDB
  AUTO_INCREMENT = 1000
  AVG_ROW_LENGTH = 1024
  CHECKSUM = 1
  COMMENT = 'Optimized storage table'
  ROW_FORMAT = COMPRESSED
  KEY_BLOCK_SIZE = 8;

CREATE TABLE minimal_table (
    id INT NOT NULL,
    name VARCHAR(50)
) ENGINE = Memory;

-- ============================================
-- 41. ALTER TABLE with lock options
-- ============================================

ALTER TABLE users ENGINE = InnoDB, LOCK = NONE;

ALTER TABLE large_table ENGINE = InnoDB, LOCK = SHARED;

ALTER TABLE critical_table ENGINE = InnoDB, LOCK = EXCLUSIVE;

-- ============================================
-- 42. ALTER TABLE with ALGORITHM options
-- ============================================

ALTER TABLE users ADD COLUMN nickname VARCHAR(50), ALGORITHM = INPLACE;

ALTER TABLE users ADD INDEX idx_nickname (nickname), ALGORITHM = COPY;

ALTER TABLE users MODIFY COLUMN name VARCHAR(200), ALGORITHM = DEFAULT;

-- ============================================
-- 43. Repair and analyze tables
-- ============================================

-- REPAIR TABLE users;

-- ANALYZE TABLE orders;

-- OPTIMIZE TABLE logs;

-- CHECK TABLE products;

-- ============================================
-- 44. Clone tables
-- ============================================

CREATE TABLE users_clone LIKE users;
INSERT INTO users_clone SELECT * FROM users;

CREATE TABLE orders_2023 AS SELECT * FROM orders WHERE YEAR(created_at) = 2023;
