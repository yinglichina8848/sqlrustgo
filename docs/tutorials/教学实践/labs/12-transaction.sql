-- =====================================================
-- 实验 12: 事务控制
-- =====================================================
-- 目标: 掌握事务的 ACID 特性

-- 12.1 开启事务
START TRANSACTION;

-- 12.2 提交事务
BEGIN;
UPDATE accounts SET balance = balance - 100 WHERE user_id = 1;
UPDATE accounts SET balance = balance + 100 WHERE user_id = 2;
COMMIT;

-- 12.3 回滚事务
BEGIN;
UPDATE products SET stock = stock - 1 WHERE id = 1;
-- 发现错误，回滚
ROLLBACK;

-- 12.4 保存点
BEGIN;
INSERT INTO orders (user_id, amount) VALUES (1, 100);
SAVEPOINT order_created;
INSERT INTO order_items (order_id, product_id) VALUES (LAST_INSERT_ID(), 1);
-- 可以回滚到保存点
ROLLBACK TO SAVEPOINT order_created;
COMMIT;

-- 12.5 设置事务隔离级别
SET TRANSACTION ISOLATION LEVEL READ COMMITTED;

-- =====================================================
-- 教学提示:
-- - 事务确保数据一致性
-- - ACID: Atomic(原子性), Consistency(一致性), 
--         Isolation(隔离性), Durability(持久性)
-- - 隔离级别: READ UNCOMMITTED, READ COMMITTED, 
--            REPEATABLE READ, SERIALIZABLE
-- - 长时间事务会增加锁竞争
-- =====================================================
