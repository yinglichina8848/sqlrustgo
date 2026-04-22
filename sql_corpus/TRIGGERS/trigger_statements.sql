-- MySQL 5.7.0 Trigger Test Statements

DROP TABLE IF EXISTS trigger_test_users;
DROP TABLE IF EXISTS trigger_test_orders;
DROP TABLE IF EXISTS trigger_test_order_items;
DROP TABLE IF EXISTS trigger_test_products;
DROP TABLE IF EXISTS trigger_test_audit_log;
DROP TABLE IF EXISTS trigger_test_inventory;

DROP TRIGGER IF EXISTS trg_after_user_insert;
DROP TRIGGER IF EXISTS trg_after_order_insert;
DROP TRIGGER IF EXISTS trg_before_order_delete;
DROP TRIGGER IF EXISTS trg_after_inventory_update;

CREATE TABLE trigger_test_users (
    id INT PRIMARY KEY AUTO_INCREMENT,
    username VARCHAR(50) NOT NULL,
    email VARCHAR(100) NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE trigger_test_orders (
    id INT PRIMARY KEY AUTO_INCREMENT,
    user_id INT NOT NULL,
    order_number VARCHAR(50) NOT NULL,
    total_amount DECIMAL(10, 2) DEFAULT 0.00,
    status ENUM('pending', 'confirmed', 'shipped', 'delivered', 'cancelled') DEFAULT 'pending',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE trigger_test_products (
    id INT PRIMARY KEY AUTO_INCREMENT,
    name VARCHAR(100) NOT NULL,
    price DECIMAL(10, 2) NOT NULL,
    stock INT DEFAULT 0,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
);

CREATE TABLE trigger_test_order_items (
    id INT PRIMARY KEY AUTO_INCREMENT,
    order_id INT NOT NULL,
    product_id INT NOT NULL,
    quantity INT NOT NULL DEFAULT 1,
    unit_price DECIMAL(10, 2) NOT NULL
);

CREATE TABLE trigger_test_inventory (
    id INT PRIMARY KEY AUTO_INCREMENT,
    product_id INT NOT NULL,
    quantity INT DEFAULT 0,
    last_order_id INT,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
);

CREATE TABLE trigger_test_audit_log (
    id INT PRIMARY KEY AUTO_INCREMENT,
    table_name VARCHAR(50) NOT NULL,
    action VARCHAR(20) NOT NULL,
    old_value TEXT,
    new_value TEXT,
    changed_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO trigger_test_products (name, price, stock) VALUES
('Product A', 10.00, 100),
('Product B', 20.00, 50),
('Product C', 30.00, 75);

INSERT INTO trigger_test_inventory (product_id, quantity) VALUES
(1, 100),
(2, 50),
(3, 75);

-- Basic AFTER INSERT Trigger
CREATE TRIGGER trg_after_user_insert
AFTER INSERT ON trigger_test_users
FOR EACH ROW
INSERT INTO trigger_test_audit_log (table_name, action, new_value)
VALUES ('users', 'INSERT', CONCAT('New user: ', NEW.username, ', email: ', NEW.email));

-- AFTER INSERT Trigger for orders
CREATE TRIGGER trg_after_order_insert
AFTER INSERT ON trigger_test_orders
FOR EACH ROW
INSERT INTO trigger_test_audit_log (table_name, action, new_value)
VALUES ('orders', 'INSERT', CONCAT('Order: ', NEW.order_number, ', amount: ', NEW.total_amount));

-- BEFORE DELETE Trigger
CREATE TRIGGER trg_before_order_delete
BEFORE DELETE ON trigger_test_orders
FOR EACH ROW
INSERT INTO trigger_test_audit_log (table_name, action, old_value)
VALUES ('orders', 'DELETE', CONCAT('Deleted order: ', OLD.order_number));

-- AFTER UPDATE Trigger for inventory
CREATE TRIGGER trg_after_inventory_update
AFTER UPDATE ON trigger_test_inventory
FOR EACH ROW
INSERT INTO trigger_test_audit_log (table_name, action, old_value, new_value)
VALUES ('inventory', 'UPDATE',
    CONCAT('Old qty: ', OLD.quantity),
    CONCAT('New qty: ', NEW.quantity));

-- Multiple triggers on same table (MySQL 5.7 supports multiple triggers)
CREATE TRIGGER trg_after_order_status_update
AFTER UPDATE ON trigger_test_orders
FOR EACH ROW
INSERT INTO trigger_test_audit_log (table_name, action, new_value)
VALUES ('orders', 'STATUS_CHANGE', CONCAT('Order ', NEW.order_number, ' status: ', NEW.status));

-- Trigger using NEW and OLD values
DELIMITER //
CREATE TRIGGER trg_order_amount_check
BEFORE INSERT ON trigger_test_orders
FOR EACH ROW
BEGIN
    IF NEW.total_amount < 0 THEN
        SET NEW.total_amount = 0;
    END IF;
END//
DELIMITER ;

-- Trigger with conditional logic
DELIMITER //
CREATE TRIGGER trg_high_value_order_alert
AFTER INSERT ON trigger_test_orders
FOR EACH ROW
BEGIN
    IF NEW.total_amount > 1000 THEN
        INSERT INTO trigger_test_audit_log (table_name, action, new_value)
        VALUES ('orders', 'HIGH_VALUE', CONCAT('High value order: ', NEW.order_number, ' - $', NEW.total_amount));
    END IF;
END//
DELIMITER ;

-- Test triggers with actual data
INSERT INTO trigger_test_users (username, email) VALUES ('testuser1', 'test1@example.com');
INSERT INTO trigger_test_users (username, email) VALUES ('testuser2', 'test2@example.com');

INSERT INTO trigger_test_orders (user_id, order_number, total_amount, status) VALUES
(1, 'TRG001', 150.00, 'pending'),
(1, 'TRG002', 2500.00, 'pending'),
(2, 'TRG003', 75.00, 'confirmed');

-- Verify triggers fired
SELECT * FROM trigger_test_audit_log;

-- Test BEFORE DELETE trigger
DELETE FROM trigger_test_orders WHERE order_number = 'TRG003';

-- Verify audit log
SELECT * FROM trigger_test_audit_log WHERE table_name = 'orders';

-- Test trigger with NEW/OLD reference
UPDATE trigger_test_inventory SET quantity = quantity - 5 WHERE product_id = 1;

SELECT * FROM trigger_test_audit_log WHERE table_name = 'inventory';

-- Trigger referencing multiple tables
DELIMITER //
CREATE TRIGGER trg_order_item_inventory_link
AFTER INSERT ON trigger_test_order_items
FOR EACH ROW
BEGIN
    UPDATE trigger_test_inventory
    SET last_order_id = NEW.order_id
    WHERE product_id = NEW.product_id;
END//
DELIMITER ;

INSERT INTO trigger_test_orders (user_id, order_number, total_amount) VALUES (1, 'TRG004', 30.00);
INSERT INTO trigger_test_order_items (order_id, product_id, quantity, unit_price)
VALUES (LAST_INSERT_ID(), 1, 2, 10.00);

SELECT * FROM trigger_test_inventory WHERE product_id = 1;

-- Drop and recreate trigger
DROP TRIGGER IF EXISTS trg_after_user_insert;
CREATE TRIGGER trg_after_user_insert
AFTER INSERT ON trigger_test_users
FOR EACH ROW
INSERT INTO trigger_test_audit_log (table_name, action, new_value)
VALUES ('users', 'INSERT', CONCAT('User created: ', NEW.email));

-- Multiple triggers execution order
CREATE TABLE trigger_test_execution_log (
    id INT PRIMARY KEY AUTO_INCREMENT,
    trigger_name VARCHAR(50),
    execution_order INT,
    executed_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

DELIMITER //
CREATE TRIGGER trg_exec_first
AFTER INSERT ON trigger_test_users
FOR EACH ROW
BEGIN
    INSERT INTO trigger_test_execution_log (trigger_name, execution_order) VALUES ('trg_exec_first', 1);
END//

CREATE TRIGGER trg_exec_second
AFTER INSERT ON trigger_test_users
FOR EACH ROW
BEGIN
    INSERT INTO trigger_test_execution_log (trigger_name, execution_order) VALUES ('trg_exec_second', 2);
END//

CREATE TRIGGER trg_exec_third
AFTER INSERT ON trigger_test_users
FOR EACH ROW
BEGIN
    INSERT INTO trigger_test_execution_log (trigger_name, execution_order) VALUES ('trg_exec_third', 3);
END//
DELIMITER ;

INSERT INTO trigger_test_users (username, email) VALUES ('exec_test', 'exec@example.com');
SELECT * FROM trigger_test_execution_log;

-- Cleanup
DROP TABLE IF EXISTS trigger_test_execution_log;
DROP TABLE IF EXISTS trigger_test_audit_log;
DROP TABLE IF EXISTS trigger_test_order_items;
DROP TABLE IF EXISTS trigger_test_inventory;
DROP TABLE IF EXISTS trigger_test_orders;
DROP TABLE IF EXISTS trigger_test_products;
DROP TABLE IF EXISTS trigger_test_users;
