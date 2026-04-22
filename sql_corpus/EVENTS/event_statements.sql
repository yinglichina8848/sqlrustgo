-- MySQL 5.7.0 Event Scheduler Test Statements

SET GLOBAL event_scheduler = ON;

DROP TABLE IF EXISTS event_test_users;
DROP TABLE IF EXISTS event_test_orders;
DROP TABLE IF EXISTS event_test_cleanup_log;
DROP TABLE IF EXISTS event_test_stats;

DROP EVENT IF EXISTS evt_daily_cleanup;
DROP EVENT IF EXISTS evt_update_stats;
DROP EVENT IF EXISTS evt_archive_orders;
DROP EVENT IF EXISTS evt_user_inactivity_check;

CREATE TABLE event_test_users (
    id INT PRIMARY KEY AUTO_INCREMENT,
    username VARCHAR(50) NOT NULL,
    email VARCHAR(100) NOT NULL,
    status ENUM('active', 'inactive', 'suspended') DEFAULT 'active',
    last_login DATETIME,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE event_test_orders (
    id INT PRIMARY KEY AUTO_INCREMENT,
    user_id INT NOT NULL,
    order_number VARCHAR(50) NOT NULL,
    total_amount DECIMAL(10, 2) DEFAULT 0.00,
    status ENUM('pending', 'confirmed', 'shipped', 'delivered', 'cancelled') DEFAULT 'pending',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    archived_at DATETIME
);

CREATE TABLE event_test_cleanup_log (
    id INT PRIMARY KEY AUTO_INCREMENT,
    event_name VARCHAR(100),
    rows_affected INT,
    executed_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE event_test_stats (
    id INT PRIMARY KEY AUTO_INCREMENT,
    stat_date DATE NOT NULL,
    active_users INT DEFAULT 0,
    total_orders INT DEFAULT 0,
    total_revenue DECIMAL(15,2) DEFAULT 0.00,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    UNIQUE KEY idx_stat_date (stat_date)
);

INSERT INTO event_test_users (username, email, status, last_login) VALUES
('alice', 'alice@example.com', 'active', NOW()),
('bob', 'bob@example.com', 'active', DATE_SUB(NOW(), INTERVAL 30 DAY)),
('charlie', 'charlie@example.com', 'inactive', DATE_SUB(NOW(), INTERVAL 90 DAY)),
('david', 'david@example.com', 'active', NOW()),
('eve', 'eve@example.com', 'suspended', DATE_SUB(NOW(), INTERVAL 60 DAY));

INSERT INTO event_test_orders (user_id, order_number, total_amount, status, created_at) VALUES
(1, 'EVT001', 100.00, 'delivered', DATE_SUB(NOW(), INTERVAL 10 DAY)),
(1, 'EVT002', 200.00, 'delivered', DATE_SUB(NOW(), INTERVAL 5 DAY)),
(2, 'EVT003', 150.00, 'pending', NOW()),
(3, 'EVT004', 300.00, 'cancelled', DATE_SUB(NOW(), INTERVAL 15 DAY)),
(4, 'EVT005', 500.00, 'shipped', DATE_SUB(NOW(), INTERVAL 2 DAY));

INSERT INTO event_test_stats (stat_date, active_users, total_orders, total_revenue) VALUES
(CURDATE(), 0, 0, 0.00);

-- Create event to clean up old sessions (runs once at specified time)
DELIMITER //
CREATE EVENT evt_one_time_cleanup
ON SCHEDULE AT CURRENT_TIMESTAMP + INTERVAL 1 HOUR
DO
BEGIN
    INSERT INTO event_test_cleanup_log (event_name, rows_affected)
    VALUES ('evt_one_time_cleanup', 0);
END//
DELIMITER ;

-- Create daily cleanup event
DELIMITER //
CREATE EVENT evt_daily_cleanup
ON SCHEDULE EVERY 1 DAY
STARTS CURRENT_TIMESTAMP
DO
BEGIN
    DECLARE v_deleted_count INT DEFAULT 0;

    DELETE FROM event_test_users
    WHERE last_login < DATE_SUB(NOW(), INTERVAL 1 YEAR)
    AND status = 'inactive';

    SET v_deleted_count = ROW_COUNT();

    INSERT INTO event_test_cleanup_log (event_name, rows_affected)
    VALUES ('evt_daily_cleanup', v_deleted_count);
END//
DELIMITER ;

-- Create event with specific time schedule
DELIMITER //
CREATE EVENT evt_midnight_report
ON SCHEDULE EVERY 1 DAY
STARTS '2024-01-01 00:00:00'
DO
BEGIN
    UPDATE event_test_stats
    SET active_users = (SELECT COUNT(*) FROM event_test_users WHERE status = 'active'),
        total_orders = (SELECT COUNT(*) FROM event_test_orders),
        total_revenue = (SELECT COALESCE(SUM(total_amount), 0) FROM event_test_orders WHERE status != 'cancelled')
    WHERE stat_date = CURDATE();
END//
DELIMITER ;

-- Create weekly event
DELIMITER //
CREATE EVENT evt_weekly_cleanup
ON SCHEDULE EVERY 1 WEEK
STARTS CURRENT_TIMESTAMP
DO
BEGIN
    DELETE FROM event_test_orders
    WHERE status = 'cancelled'
    AND created_at < DATE_SUB(NOW(), INTERVAL 30 DAY);

    INSERT INTO event_test_cleanup_log (event_name, rows_affected)
    VALUES ('evt_weekly_cleanup', ROW_COUNT());
END//
DELIMITER ;

-- Create event with monthly schedule
DELIMITER //
CREATE EVENT evt_monthly_report
ON SCHEDULE EVERY 1 MONTH
STARTS CURRENT_TIMESTAMP
DO
BEGIN
    INSERT INTO event_test_stats (stat_date, active_users, total_orders, total_revenue)
    SELECT CURDATE(), 0, 0, 0.00
    FROM DUAL
    WHERE NOT EXISTS (
        SELECT 1 FROM event_test_stats WHERE stat_date = CURDATE()
    );
END//
DELIMITER ;

-- Create event to archive old orders
DELIMITER //
CREATE EVENT evt_archive_orders
ON SCHEDULE EVERY 1 DAY
STARTS CURRENT_TIMESTAMP
DO
BEGIN
    DECLARE v_archived INT DEFAULT 0;

    UPDATE event_test_orders
    SET archived_at = NOW()
    WHERE status = 'delivered'
    AND created_at < DATE_SUB(NOW(), INTERVAL 90 DAY)
    AND archived_at IS NULL;

    SET v_archived = ROW_COUNT();

    INSERT INTO event_test_cleanup_log (event_name, rows_affected)
    VALUES ('evt_archive_orders', v_archived);
END//
DELIMITER ;

-- Create event with disable on completion
DELIMITER //
CREATE EVENT evt_disable_test
ON SCHEDULE AT CURRENT_TIMESTAMP + INTERVAL 10 MINUTE
ON COMPLETION PRESERVE
DISABLE
DO
BEGIN
    INSERT INTO event_test_cleanup_log (event_name, rows_affected)
    VALUES ('evt_disable_test', 0);
END//
DELIMITER ;

-- Create event with enable/disable
DELIMITER //
CREATE EVENT evt_status_check
ON SCHEDULE EVERY 1 HOUR
DO
BEGIN
    UPDATE event_test_users
    SET status = 'inactive'
    WHERE last_login < DATE_SUB(NOW(), INTERVAL 6 MONTH)
    AND status = 'active';
END//
DELIMITER ;

ALTER EVENT evt_status_check DISABLE;
ALTER EVENT evt_status_check ENABLE;

-- Create event that calls a stored procedure
DELIMITER //
CREATE PROCEDURE sp_calculate_daily_stats()
BEGIN
    INSERT INTO event_test_cleanup_log (event_name, rows_affected, executed_at)
    VALUES ('sp_calculate_daily_stats', 0, NOW())
    ON DUPLICATE KEY UPDATE rows_affected = rows_affected + 1;
END//

CREATE EVENT evt_call_procedure
ON SCHEDULE EVERY 1 DAY
DO
BEGIN
    CALL sp_calculate_daily_stats();
END//
DELIMITER ;

-- Create event with preservation
DELIMITER //
CREATE EVENT evt_preserve_test
ON SCHEDULE AT CURRENT_TIMESTAMP + INTERVAL 5 MINUTE
ON COMPLETION PRESERVE
DO
BEGIN
    INSERT INTO event_test_cleanup_log (event_name, rows_affected)
    VALUES ('evt_preserve_test', 1);
END//
DELIMITER ;

-- Create event with comment
DELIMITER //
CREATE EVENT evt_comment_test
ON SCHEDULE EVERY 1 DAY
COMMENT 'This event runs daily to update statistics'
DO
BEGIN
    INSERT INTO event_test_cleanup_log (event_name, rows_affected)
    VALUES ('evt_comment_test', 0);
END//
DELIMITER ;

-- Create event for user inactivity check
DELIMITER //
CREATE EVENT evt_user_inactivity_check
ON SCHEDULE EVERY 1 WEEK
STARTS CURRENT_TIMESTAMP + INTERVAL 1 DAY
DO
BEGIN
    DECLARE v_inactive_users INT DEFAULT 0;

    UPDATE event_test_users
    SET status = 'suspended'
    WHERE last_login < DATE_SUB(NOW(), INTERVAL 3 MONTH)
    AND status = 'active';

    SET v_inactive_users = ROW_COUNT();

    INSERT INTO event_test_cleanup_log (event_name, rows_affected)
    VALUES ('evt_user_inactivity_check', v_inactive_users);
END//
DELIMITER ;

-- Create event with both STARTS and ENDS
DELIMITER //
CREATE EVENT evt_temporary_event
ON SCHEDULE EVERY 1 HOUR
STARTS CURRENT_TIMESTAMP
ENDS CURRENT_TIMESTAMP + INTERVAL 7 DAY
DO
BEGIN
    INSERT INTO event_test_cleanup_log (event_name, rows_affected)
    VALUES ('evt_temporary_event', 0);
END//
DELIMITER ;

-- Create event using DO clause with single statement
CREATE EVENT evt_single_statement
ON SCHEDULE EVERY 1 DAY
DO
    INSERT INTO event_test_cleanup_log (event_name, rows_affected) VALUES ('evt_single_statement', 0);

-- Create event for order status updates
DELIMITER //
CREATE EVENT evt_order_expiration
ON SCHEDULE EVERY 1 DAY
DO
BEGIN
    UPDATE event_test_orders
    SET status = 'cancelled'
    WHERE status = 'pending'
    AND created_at < DATE_SUB(NOW(), INTERVAL 30 DAY);
END//
DELIMITER ;

-- Drop events
DROP EVENT IF EXISTS evt_daily_cleanup;
DROP EVENT IF EXISTS evt_weekly_cleanup;
DROP EVENT IF EXISTS evt_midnight_report;
DROP EVENT IF EXISTS evt_monthly_report;
DROP EVENT IF EXISTS evt_archive_orders;
DROP EVENT IF EXISTS evt_status_check;
DROP EVENT IF EXISTS evt_user_inactivity_check;
DROP EVENT IF EXISTS evt_temporary_event;
DROP EVENT IF EXISTS evt_single_statement;
DROP EVENT IF EXISTS evt_one_time_cleanup;

-- Show events
SHOW EVENTS FROM mysql;
SHOW EVENTS FROM current_database;

-- Event information from information_schema
SELECT EVENT_NAME, EVENT_TYPE, STATUS, EXECUTUTE_AT
FROM information_schema.EVENTS
WHERE EVENT_SCHEMA = DATABASE();

-- Cleanup
DROP TABLE IF EXISTS event_test_stats;
DROP TABLE IF EXISTS event_test_cleanup_log;
DROP TABLE IF EXISTS event_test_orders;
DROP TABLE IF EXISTS event_test_users;
