-- =====================================================
-- 实验 10: CREATE TABLE 表创建
-- =====================================================
-- 目标: 掌握表结构创建

-- 10.1 基础建表
CREATE TABLE users (
    id INT PRIMARY KEY AUTO_INCREMENT,
    username VARCHAR(50) NOT NULL UNIQUE,
    email VARCHAR(100) NOT NULL,
    password VARCHAR(255) NOT NULL,
    status ENUM('active', 'inactive', 'banned') DEFAULT 'active',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
);

-- 10.2 带索引的表
CREATE TABLE orders (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    user_id INT NOT NULL,
    amount DECIMAL(10, 2) NOT NULL,
    status VARCHAR(20) DEFAULT 'pending',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY (user_id) REFERENCES users(id),
    INDEX idx_user_id (user_id),
    INDEX idx_status (status),
    INDEX idx_created (created_at)
);

-- 10.3 创建临时表
CREATE TEMPORARY TABLE temp_stats (
    category VARCHAR(50),
    total_sales DECIMAL(12, 2)
);

-- 10.4 复制表结构
CREATE TABLE orders_backup LIKE orders;

-- 10.5 复制表数据
INSERT INTO orders_backup SELECT * FROM orders;

-- =====================================================
-- 教学提示:
-- - 主键用于唯一标识每条记录
-- - NOT NULL 确保字段不为空
-- - DEFAULT 设置默认值
-- - INDEX 提高查询速度
-- - FOREIGN KEY 建立表间关系
-- =====================================================
