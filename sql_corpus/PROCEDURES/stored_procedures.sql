-- MySQL 5.7.0 Stored Procedure Test Statements

DROP TABLE IF EXISTS proc_test_users;
DROP TABLE IF EXISTS proc_test_orders;
DROP TABLE IF EXISTS proc_test_products;
DROP TABLE IF EXISTS proc_test_inventory;

DROP PROCEDURE IF EXISTS sp_get_user_count;
DROP PROCEDURE IF EXISTS sp_create_order;
DROP PROCEDURE IF EXISTS sp_update_inventory;
DROP PROCEDURE IF EXISTS sp_get_user_orders;
DROP PROCEDURE IF EXISTS sp_archive_old_orders;

CREATE TABLE proc_test_users (
    id INT PRIMARY KEY AUTO_INCREMENT,
    username VARCHAR(50) NOT NULL,
    email VARCHAR(100) NOT NULL,
    status ENUM('active', 'inactive') DEFAULT 'active',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE proc_test_orders (
    id INT PRIMARY KEY AUTO_INCREMENT,
    user_id INT NOT NULL,
    order_number VARCHAR(50) NOT NULL UNIQUE,
    total_amount DECIMAL(10, 2) DEFAULT 0.00,
    status ENUM('pending', 'confirmed', 'shipped', 'delivered', 'cancelled') DEFAULT 'pending',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE proc_test_products (
    id INT PRIMARY KEY AUTO_INCREMENT,
    name VARCHAR(100) NOT NULL,
    price DECIMAL(10, 2) NOT NULL,
    stock INT DEFAULT 0
);

CREATE TABLE proc_test_inventory (
    id INT PRIMARY KEY AUTO_INCREMENT,
    product_id INT NOT NULL,
    quantity INT DEFAULT 0,
    reserved INT DEFAULT 0,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
);

INSERT INTO proc_test_users (username, email) VALUES
('alice', 'alice@example.com'),
('bob', 'bob@example.com'),
('charlie', 'charlie@example.com');

INSERT INTO proc_test_products (name, price, stock) VALUES
('Product A', 10.00, 100),
('Product B', 20.00, 50),
('Product C', 30.00, 75);

INSERT INTO proc_test_inventory (product_id, quantity) VALUES
(1, 100),
(2, 50),
(3, 75);

-- Simple stored procedure with SELECT
DELIMITER //
CREATE PROCEDURE sp_get_user_count()
BEGIN
    SELECT COUNT(*) AS user_count FROM proc_test_users;
END//
DELIMITER ;

CALL sp_get_user_count();

-- Procedure with IN parameter
DELIMITER //
CREATE PROCEDURE sp_get_user_orders(IN p_user_id INT)
BEGIN
    SELECT * FROM proc_test_orders WHERE user_id = p_user_id;
END//
DELIMITER ;

CALL sp_get_user_orders(1);

-- Procedure with OUT parameter
DELIMITER //
CREATE PROCEDURE sp_get_total_revenue(OUT p_total DECIMAL(15,2))
BEGIN
    SELECT COALESCE(SUM(total_amount), 0) INTO p_total
    FROM proc_test_orders
    WHERE status != 'cancelled';
END//
DELIMITER ;

CALL sp_get_total_revenue(@total);
SELECT @total AS total_revenue;

-- Procedure with multiple OUT parameters
DELIMITER //
CREATE PROCEDURE sp_get_user_stats(
    IN p_user_id INT,
    OUT p_order_count INT,
    OUT p_total_spent DECIMAL(15,2)
)
BEGIN
    SELECT COUNT(*), COALESCE(SUM(total_amount), 0)
    INTO p_order_count, p_total_spent
    FROM proc_test_orders
    WHERE user_id = p_user_id AND status != 'cancelled';
END//
DELIMITER ;

CALL sp_get_user_stats(1, @count, @total);
SELECT @count AS order_count, @total AS total_spent;

-- Procedure with INOUT parameter
DELIMITER //
CREATE PROCEDURE sp_increment_counter(INOUT p_counter INT)
BEGIN
    SET p_counter = p_counter + 1;
END//
DELIMITER ;

SET @my_counter = 10;
CALL sp_increment_counter(@my_counter);
SELECT @my_counter;

-- Procedure with conditional logic
DELIMITER //
CREATE PROCEDURE sp_create_order(
    IN p_user_id INT,
    IN p_order_number VARCHAR(50),
    IN p_amount DECIMAL(10,2)
)
BEGIN
    DECLARE v_user_exists INT DEFAULT 0;

    SELECT COUNT(*) INTO v_user_exists FROM proc_test_users WHERE id = p_user_id;

    IF v_user_exists > 0 THEN
        INSERT INTO proc_test_orders (user_id, order_number, total_amount)
        VALUES (p_user_id, p_order_number, p_amount);
        SELECT 'Order created successfully' AS result;
    ELSE
        SELECT 'User not found' AS result;
    END IF;
END//
DELIMITER ;

CALL sp_create_order(1, 'PROC001', 100.00);
CALL sp_create_order(999, 'PROC002', 100.00);

-- Procedure with CASE statement
DELIMITER //
CREATE PROCEDURE sp_get_discount_rate(
    IN p_amount DECIMAL(10,2),
    OUT p_discount_rate DECIMAL(5,2)
)
BEGIN
    CASE
        WHEN p_amount > 1000 THEN SET p_discount_rate = 0.15;
        WHEN p_amount > 500 THEN SET p_discount_rate = 0.10;
        WHEN p_amount > 100 THEN SET p_discount_rate = 0.05;
        ELSE SET p_discount_rate = 0.00;
    END CASE;
END//
DELIMITER ;

CALL sp_get_discount_rate(750.00, @rate);
SELECT @rate AS discount_rate;

-- Procedure with WHILE loop
DELIMITER //
CREATE PROCEDURE sp_batch_insert_users(IN p_count INT)
BEGIN
    DECLARE v_i INT DEFAULT 1;
    WHILE v_i <= p_count DO
        INSERT INTO proc_test_users (username, email)
        VALUES (CONCAT('user_', v_i), CONCAT('user_', v_i, '@example.com'));
        SET v_i = v_i + 1;
    END WHILE;
    SELECT CONCAT('Inserted ', p_count, ' users') AS result;
END//
DELIMITER ;

-- CALL sp_batch_insert_users(5);

-- Procedure with REPEAT loop
DELIMITER //
CREATE PROCEDURE sp_generate_order_numbers(IN p_count INT)
BEGIN
    DECLARE v_i INT DEFAULT 1;
    REPEAT
        INSERT INTO proc_test_orders (user_id, order_number, total_amount)
        VALUES (1, CONCAT('ORD_REPEAT_', v_i), v_i * 10.00);
        SET v_i = v_i + 1;
    UNTIL v_i > p_count END REPEAT;
    SELECT CONCAT('Generated ', p_count, ' order numbers') AS result;
END//
DELIMITER ;

-- CALL sp_generate_order_numbers(3);

-- Procedure with cursor
DELIMITER //
CREATE PROCEDURE sp_process_user_orders(IN p_user_id INT)
BEGIN
    DECLARE v_order_number VARCHAR(50);
    DECLARE v_total DECIMAL(10,2);
    DECLARE v_done INT DEFAULT 0;

    DECLARE cur CURSOR FOR
        SELECT order_number, total_amount FROM proc_test_orders WHERE user_id = p_user_id;

    DECLARE CONTINUE HANDLER FOR NOT FOUND SET v_done = 1;

    CREATE TEMPORARY TABLE IF NOT EXISTS temp_user_orders (
        order_number VARCHAR(50),
        total DECIMAL(10,2)
    );

    OPEN cur;

    read_loop: LOOP
        FETCH cur INTO v_order_number, v_total;
        IF v_done = 1 THEN
            LEAVE read_loop;
        END IF;
        INSERT INTO temp_user_orders VALUES (v_order_number, v_total);
    END LOOP;

    CLOSE cur;

    SELECT * FROM temp_user_orders;
    DROP TEMPORARY TABLE temp_user_orders;
END//
DELIMITER ;

CALL sp_process_user_orders(1);

-- Procedure with transaction
DELIMITER //
CREATE PROCEDURE sp_transfer_funds(
    IN p_from_user INT,
    IN p_to_user INT,
    IN p_amount DECIMAL(10,2)
)
BEGIN
    DECLARE EXIT HANDLER FOR SQLEXCEPTION
    BEGIN
        ROLLBACK;
        SELECT 'Transaction failed' AS result;
    END;

    START TRANSACTION;

    UPDATE proc_test_orders
    SET total_amount = total_amount - p_amount
    WHERE user_id = p_from_user AND status = 'pending';

    UPDATE proc_test_orders
    SET total_amount = total_amount + p_amount
    WHERE user_id = p_to_user AND status = 'pending';

    COMMIT;
    SELECT 'Transfer completed' AS result;
END//
DELIMITER ;

-- Procedure with parameters and local variables
DELIMITER //
CREATE PROCEDURE sp_calculate_order_total(
    IN p_order_id INT,
    OUT p_subtotal DECIMAL(10,2),
    OUT p_tax DECIMAL(10,2),
    OUT p_total DECIMAL(10,2)
)
BEGIN
    DECLARE v_tax_rate DECIMAL(5,4) DEFAULT 0.0825;

    SELECT COALESCE(SUM(unit_price * quantity), 0) INTO p_subtotal
    FROM proc_test_orders
    WHERE id = p_order_id;

    SET p_tax = p_subtotal * v_tax_rate;
    SET p_total = p_subtotal + p_tax;
END//
DELIMITER ;

-- Procedure calling another procedure
DELIMITER //
CREATE PROCEDURE sp_log_user_activity(IN p_user_id INT)
BEGIN
    DECLARE v_username VARCHAR(50);
    DECLARE v_order_count INT;

    SELECT username INTO v_username FROM proc_test_users WHERE id = p_user_id;

    CALL sp_get_user_stats(p_user_id, v_order_count, @total);

    SELECT CONCAT('User: ', v_username, ', Orders: ', v_order_count) AS activity_log;
END//
DELIMITER ;

CALL sp_log_user_activity(1);

-- Procedure with multiple conditionals
DELIMITER //
CREATE PROCEDURE sp_update_order_status(
    IN p_order_id INT,
    IN p_new_status VARCHAR(20)
)
BEGIN
    DECLARE v_current_status VARCHAR(20);

    SELECT status INTO v_current_status FROM proc_test_orders WHERE id = p_order_id;

    IF v_current_status = 'cancelled' THEN
        SELECT 'Cannot update cancelled order' AS result;
    ELSEIF v_current_status = 'delivered' AND p_new_status != 'delivered' THEN
        SELECT 'Cannot change status of delivered order' AS result;
    ELSE
        UPDATE proc_test_orders SET status = p_new_status WHERE id = p_order_id;
        SELECT CONCAT('Order status updated to ', p_new_status) AS result;
    END IF;
END//
DELIMITER ;

CALL sp_update_order_status(1, 'shipped');
CALL sp_update_order_status(1, 'cancelled');
CALL sp_update_order_status(1, 'pending');

-- Procedure with INSERT...SELECT
DELIMITER //
CREATE PROCEDURE sp_archive_old_orders(IN p_days INT)
BEGIN
    DECLARE v_cutoff_date DATETIME;

    SET v_cutoff_date = DATE_SUB(NOW(), INTERVAL p_days DAY);

    INSERT INTO proc_test_orders (user_id, order_number, total_amount, status, created_at)
    SELECT user_id, CONCAT('ARC_', order_number), total_amount, status, created_at
    FROM proc_test_orders
    WHERE created_at < v_cutoff_date AND status = 'delivered';

    SELECT ROW_COUNT() AS archived_count;
END//
DELIMITER ;

-- DROP PROCEDURE statements
DROP PROCEDURE IF EXISTS sp_get_user_count;
DROP PROCEDURE IF EXISTS sp_get_user_orders;
DROP PROCEDURE IF EXISTS sp_create_order;
DROP PROCEDURE IF EXISTS sp_update_inventory;
DROP PROCEDURE IF EXISTS sp_get_total_revenue;
DROP PROCEDURE IF EXISTS sp_get_user_stats;

-- Cleanup
DROP TABLE IF EXISTS proc_test_inventory;
DROP TABLE IF EXISTS proc_test_orders;
DROP TABLE IF EXISTS proc_test_products;
DROP TABLE IF EXISTS proc_test_users;
