-- DELETE Statement Test Cases
-- Compatibility: MySQL 5.7+

-- ============================================
-- 1. Basic DELETE Operations
-- ============================================

-- Delete single row
DELETE FROM users WHERE id = 1;

-- Delete with WHERE clause
DELETE FROM users WHERE status = 'inactive' AND last_login < '2023-01-01';

-- Delete all rows
DELETE FROM logs;

-- Delete with IN
DELETE FROM users WHERE id IN (5, 10, 15, 20);

-- Delete with multiple conditions
DELETE FROM products WHERE category = 'discontinued' AND stock = 0;

-- ============================================
-- 2. DELETE with ORDER BY and LIMIT
-- ============================================

-- Delete oldest entries (ORDER BY with LIMIT)
DELETE FROM logs ORDER BY created_at ASC LIMIT 100;

-- Delete recent entries
DELETE FROM sessions ORDER BY last_activity DESC LIMIT 10;

-- Delete with ORDER BY on specific column
DELETE FROM orders ORDER BY created_at ASC LIMIT 50 WHERE status = 'cancelled';

-- Delete with multiple columns in ORDER BY
DELETE FROM events ORDER BY priority DESC, created_at ASC LIMIT 20;

-- ============================================
-- 3. DELETE with JOIN
-- ============================================

-- Delete with JOIN subquery
DELETE FROM users WHERE id IN (SELECT user_id FROM inactive_users WHERE days_inactive > 365);

-- Delete using EXISTS
DELETE FROM orders WHERE EXISTS (SELECT 1 FROM users WHERE users.id = orders.user_id AND users.status = 'banned');

-- Delete with NOT EXISTS
DELETE FROM products WHERE NOT EXISTS (SELECT 1 FROM order_items WHERE order_items.product_id = products.id);

-- Delete with JOIN (MySQL specific multi-table)
-- DELETE u FROM users u INNER JOIN orders o ON u.id = o.user_id WHERE o.status = 'fraudulent';

-- ============================================
-- 4. DELETE with Subqueries
-- ============================================

-- Delete using subquery in WHERE
DELETE FROM users WHERE id IN (SELECT id FROM users_to_delete);

-- Delete with correlated subquery
DELETE FROM users WHERE (SELECT COUNT(*) FROM orders WHERE orders.user_id = users.id) = 0;

-- Delete with aggregate in subquery
DELETE FROM products WHERE id IN (SELECT id FROM (SELECT id, AVG(rating) as avg FROM products GROUP BY id HAVING avg < 2) AS low_rated);

-- Delete with EXISTS
DELETE FROM categories WHERE NOT EXISTS (SELECT 1 FROM products WHERE products.category_id = categories.id);

-- Delete with ANY/SOME
DELETE FROM scores WHERE score < ANY (SELECT score FROM scholarships WHERE min_score IS NOT NULL);

-- ============================================
-- 5. DELETE variations
-- ============================================

-- Quick delete (doesn't record individual row deletions in binary log for MySQL)
DELETE QUICK FROM temp_data WHERE created_at < '2024-01-01';

-- Low priority delete (waits until no reads)
DELETE LOW_PRIORITY FROM users WHERE status = 'archived';

-- IGNORE delete (continues even if errors)
DELETE IGNORE FROM users WHERE id = 'invalid';

-- ============================================
-- 6. DELETE with LIKE
-- ============================================

DELETE FROM users WHERE email LIKE '%@spam.com%';

DELETE FROM posts WHERE title LIKE '%draft%' AND status = 'unpublished';

DELETE FROM logs WHERE message LIKE '%error%' OR message LIKE '%warning%';

DELETE FROM sessions WHERE user_agent LIKE '%bot%' OR user_agent LIKE '%crawler%';

-- ============================================
-- 7. DELETE with BETWEEN
-- ============================================

DELETE FROM events WHERE event_date BETWEEN '2023-01-01' AND '2023-12-31';

DELETE FROM scores WHERE score BETWEEN 0 AND 10;

DELETE FROM users WHERE id BETWEEN 100 AND 200 AND status = 'inactive';

-- ============================================
-- 8. DELETE with NULL checks
-- ============================================

DELETE FROM products WHERE description IS NULL;

DELETE FROM users WHERE phone IS NULL AND email IS NULL;

DELETE FROM orders WHERE shipped_at IS NULL AND status = 'delivered';

DELETE FROM products WHERE category_id IS NOT NULL;

-- ============================================
-- 9. DELETE with date/time functions
-- ============================================

DELETE FROM sessions WHERE last_activity < DATE_SUB(NOW(), INTERVAL 30 DAY);

DELETE FROM logs WHERE created_at < DATE_SUB(CURDATE(), INTERVAL 90 DAY);

DELETE FROM cache WHERE expires_at < NOW();

DELETE FROM events WHERE event_date < CURDATE() AND recurring = FALSE;

DELETE FROM orders WHERE created_at < DATE_SUB(NOW(), INTERVAL 1 YEAR) AND status = 'completed';

-- ============================================
-- 10. DELETE with String functions
-- ============================================

DELETE FROM users WHERE LENGTH(name) < 3;

DELETE FROM products WHERE TRIM(name) = '';

DELETE FROM posts WHERE LOWER(title) = 'untitled';

DELETE FROM users WHERE SUBSTRING(email, -4) = '.tmp';

-- ============================================
-- 11. DELETE with JSON
-- ============================================

DELETE FROM metadata WHERE JSON_EXTRACT(data, '$.deleted') = TRUE;

DELETE FROM metadata WHERE data->>'$.status' = 'archived';

DELETE FROM metadata WHERE JSON_CONTAINS(data, '"spam"', '$.flags');

-- ============================================
-- 12. DELETE with transactions
-- ============================================

-- BEGIN;
-- DELETE FROM accounts WHERE id = 1;
-- DELETE FROM account_history WHERE account_id = 1;
-- COMMIT;

-- BEGIN;
-- DELETE FROM orders WHERE user_id = 1 AND status = 'pending';
-- -- Check affected rows
-- -- If affected > 0, then proceed
-- ROLLBACK;

-- SAVEPOINT and ROLLBACK TO
-- BEGIN;
-- DELETE FROM products WHERE id = 1;
-- SAVEPOINT sp1;
-- DELETE FROM products WHERE id = 2;
-- ROLLBACK TO sp1;
-- COMMIT;

-- ============================================
-- 13. DELETE with LIMIT variations
-- ============================================

DELETE FROM high_volume_logs LIMIT 1000;

DELETE FROM temp_uploads ORDER BY created_at ASC LIMIT 50;

DELETE FROM spam_reports ORDER BY report_count DESC LIMIT 100;

-- ============================================
-- 14. DELETE all rows (different methods)
-- ============================================

-- DELETE all rows (slow for large tables)
DELETE FROM large_table;

-- TRUNCATE (fast, cannot be rolled back, resets auto_increment)
-- TRUNCATE TABLE large_table;

-- DROP and recreate (complete reset)
-- DROP TABLE large_table;
-- Re-create table

-- ============================================
-- 15. DELETE with Foreign Key considerations
-- ============================================

-- Delete child records first
-- DELETE FROM order_items WHERE order_id = 100;
-- DELETE FROM orders WHERE id = 100;

-- Delete parent (cascades to children if FK defined)
-- DELETE FROM categories WHERE id = 1; -- If ON DELETE CASCADE

-- Delete with SET NULL
-- DELETE FROM products WHERE category_id = 1; -- category_id becomes NULL if ON DELETE SET NULL

-- ============================================
-- 16. DELETE with partitions (MySQL 5.7+)
-- ============================================

-- DELETE from specific partition
-- DELETE FROM orders PARTITION (p2023) WHERE order_date < '2024-01-01';

-- ============================================
-- 17. DELETE with aliases
-- ============================================

DELETE u FROM users AS u WHERE u.id = 1;

DELETE FROM users u WHERE u.status = 'deleted';

-- ============================================
-- 18. DELETE edge cases
-- ============================================

-- Delete non-existent row (no error, 0 rows affected)
DELETE FROM users WHERE id = 999999;

-- Delete with complex WHERE
DELETE FROM users WHERE (age < 18 OR age > 100) AND (status = 'inactive' OR last_login < '2023-01-01') AND email NOT LIKE '%@important.com%';

-- Delete with subquery referencing same table
DELETE FROM users WHERE email IN (SELECT email FROM (SELECT email, COUNT(*) as cnt FROM users GROUP BY email HAVING cnt > 1) AS duplicates);

-- Delete with UNION
-- DELETE FROM combined WHERE id IN (SELECT id FROM (SELECT id FROM table1 UNION SELECT id FROM table2) AS combined_ids);

-- ============================================
-- 19. DELETE with stored function results
-- ============================================

-- Assuming a stored function is_archived exists
-- DELETE FROM documents WHERE is_archived(id) = TRUE;

-- Assuming a stored procedure cleans up data
-- CALL cleanup_old_records(365);

-- ============================================
-- 20. DELETE with Prepared Statements
-- ============================================

-- PREPARE stmt FROM 'DELETE FROM users WHERE id = ?';
-- SET @id = 100;
-- EXECUTE stmt USING @id;
-- DEALLOCATE PREPARE stmt;

-- ============================================
-- 21. Multi-table DELETE
-- ============================================

-- DELETE t1, t2 FROM t1 INNER JOIN t2 ON t1.id = t2.ref_id WHERE t1.status = 'obsolete';

-- DELETE FROM t1 USING t1 INNER JOIN t2 ON t1.id = t2.ref_id WHERE t1.status = 'obsolete';

-- DELETE t1 FROM t1 LEFT JOIN t2 ON t1.id = t2.ref_id WHERE t2.id IS NULL;

-- ============================================
-- 22. Soft delete (using status column)
-- ============================================

-- Instead of actual DELETE, update status
UPDATE users SET deleted_at = NOW(), status = 'deleted' WHERE id = 1;

-- Query non-deleted records
SELECT * FROM users WHERE deleted_at IS NULL AND status != 'deleted';
