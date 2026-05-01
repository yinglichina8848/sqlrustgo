-- String Function Test Cases
-- Compatibility: MySQL 5.7+

-- ============================================
-- 1. CONCAT
-- ============================================

SELECT CONCAT('Hello', ' ', 'World');

SELECT CONCAT(first_name, ' ', last_name) AS full_name FROM users;

SELECT CONCAT(name, ' - ', price) AS product_label FROM products;

SELECT CONCAT('Total: ', SUM(total)) AS summary FROM orders;

SELECT CONCAT_WS(' - ', name, email, phone) AS contact FROM users;

SELECT CONCAT_WS(', ', 'a', 'b', 'c', NULL, 'd');

-- ============================================
-- 2. LENGTH and CHAR_LENGTH
-- ============================================

SELECT LENGTH('Hello');

SELECT CHAR_LENGTH('Hello');

SELECT LENGTH(name) AS name_length FROM users;

SELECT CHAR_LENGTH(name) AS char_count FROM users;

SELECT name FROM users WHERE LENGTH(name) > 20;

-- ============================================
-- 3. UPPER and LOWER
-- ============================================

SELECT UPPER('hello');

SELECT LOWER('HELLO');

SELECT UPPER(name) FROM users;

SELECT LOWER(email) FROM users;

SELECT CONCAT(UPPER(LEFT(name, 1)), LOWER(SUBSTRING(name, 2))) AS capitalized FROM users;

-- ============================================
-- 4. TRIM, LTRIM, RTRIM
-- ============================================

SELECT TRIM('  hello  ');

SELECT LTRIM('  hello');

SELECT RTRIM('hello  ');

SELECT TRIM(LEADING FROM '  hello');

SELECT TRIM(TRAILING FROM 'hello  ');

SELECT TRIM(BOTH 'x' FROM 'xxhelloxx');

SELECT TRIM(name) FROM users WHERE TRIM(name) != name;

-- ============================================
-- 5. SUBSTRING
-- ============================================

SELECT SUBSTRING('Hello World', 1, 5);

SELECT SUBSTRING('Hello World', 7);

SELECT SUBSTRING(name, 1, 1) AS initial FROM users;

SELECT SUBSTRING(email, POSITION('@' IN email) + 1) AS domain FROM users;

SELECT SUBSTRING(name, -3) AS suffix FROM users;

SELECT SUBSTRING_INDEX(email, '@', 1) AS username FROM users;

SELECT SUBSTRING_INDEX(email, '@', -1) AS domain FROM users;

-- ============================================
-- 6. REPLACE
-- ============================================

SELECT REPLACE('Hello World', 'World', 'MySQL');

SELECT REPLACE(name, 'Mr.', 'Dr.') FROM users;

SELECT REPLACE(description, 'old value', 'new value') FROM products;

SELECT REPLACE(TRIM(name), ' ', '_') AS underscored FROM users;

SELECT REPLACE(JSON_EXTRACT(data, '$.name'), '"', '') FROM metadata;

-- ============================================
-- 7. LOCATE and POSITION
-- ============================================

SELECT POSITION('World' IN 'Hello World');

SELECT LOCATE('o', 'Hello World');

SELECT LOCATE('o', 'Hello World', 6);

SELECT LOCATE('@', email) AS at_position FROM users;

SELECT name FROM users WHERE POSITION('@' IN email) > 0;

-- ============================================
-- 8. LEFT and RIGHT
-- ============================================

SELECT LEFT('Hello World', 5);

SELECT RIGHT('Hello World', 5);

SELECT LEFT(name, 1) AS first_letter FROM users;

SELECT RIGHT(email, 4) AS tld FROM users;

SELECT LEFT(name, LENGTH(name) - 3) AS name_without_suffix FROM users;

-- ============================================
-- 9. LPAD and RPAD
-- ============================================

SELECT LPAD('hi', 10, '*');

SELECT RPAD('hi', 10, '*');

SELECT LPAD(CAST(price AS CHAR), 10, '0') AS padded_price FROM products;

SELECT CONCAT(RPAD(name, 20, '.'), email) AS formatted FROM users;

-- ============================================
-- 10. REVERSE
-- ============================================

SELECT REVERSE('Hello');

SELECT REVERSE(name) AS reversed_name FROM users;

SELECT REVERSE(SUBSTRING_INDEX(email, '@', 1)) AS reversed_username FROM users;

-- ============================================
-- 11. INSERT
-- ============================================

SELECT INSERT('Hello World', 7, 5, 'MySQL');

SELECT INSERT(name, 1, 0, 'Mr. ') FROM users WHERE id = 1;

-- ============================================
-- 12. REPEAT
-- ============================================

SELECT REPEAT('*', 10);

SELECT REPEAT(name, 2) FROM users WHERE id = 1;

SELECT RPAD(name, LENGTH(name) * 3, ' ') AS repeated FROM users;

-- ============================================
-- 13. SPACE
-- ============================================

SELECT SPACE(10);

SELECT CONCAT('Hello', SPACE(5), 'World');

SELECT CONCAT(name, SPACE(20 - LENGTH(name)), email) AS aligned FROM users;

-- ============================================
-- 14. STRCMP
-- ============================================

SELECT STRCMP('abc', 'abd');

SELECT STRCMP('abc', 'abc');

SELECT STRCMP('abd', 'abc');

SELECT name FROM users WHERE STRCMP(LEFT(name, 3), 'Joh') = 0;

-- ============================================
-- 15. SUBSTRING_INDEX with different delimiters
-- ============================================

SELECT SUBSTRING_INDEX('www.example.com', '.', 1);

SELECT SUBSTRING_INDEX('www.example.com', '.', 2);

SELECT SUBSTRING_INDEX('www.example.com', '.', -1);

SELECT SUBSTRING_INDEX('www.example.com', '.', -2);

SELECT SUBSTRING_INDEX('a,b,c,d,e', ',', 3);

-- ============================================
-- 16. MAKE_SET
-- ============================================

SELECT MAKE_SET(1 | 4 | 8, 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h');

SELECT MAKE_SET(0, 'a', 'b');

SELECT MAKE_SET(5, 'a', 'b', 'c', 'd', 'e');

-- ============================================
-- 17. EXPORT_SET
-- ============================================

SELECT EXPORT_SET(5, '1', '0', '', 4);

SELECT EXPORT_SET(5, 'Y', 'N', ',', 8);

-- ============================================
-- 18. FIELD
-- ============================================

SELECT FIELD('b', 'a', 'b', 'c', 'd');

SELECT FIELD('z', 'a', 'b', 'c');

SELECT name FROM users ORDER BY FIELD(status, 'pending', 'active', 'completed');

-- ============================================
-- 19. FIND_IN_SET
-- ============================================

SELECT FIND_IN_SET('b', 'a,b,c,d');

SELECT FIND_IN_SET('z', 'a,b,c');

SELECT name FROM users WHERE FIND_IN_SET('admin', roles) > 0;

-- ============================================
-- 20. ASCII and CHAR
-- ============================================

SELECT ASCII('A');

SELECT ASCII('a');

SELECT CHAR(65, 66, 67);

SELECT name FROM users WHERE ASCII(LEFT(name, 1)) BETWEEN 65 AND 90;

-- ============================================
-- 21. ORD
-- ============================================

SELECT ORD('A');

SELECT ORD('a');

SELECT name FROM users WHERE ORD(LEFT(name, 1)) > 127;

-- ============================================
-- 22. BIN and BINARY
-- ============================================

SELECT BIN(10);

SELECT BINARY 'Hello';

SELECT name FROM users WHERE BINARY name = 'admin';

-- ============================================
-- 23. CHAR_LENGTH with Unicode
-- ============================================

SELECT CHAR_LENGTH('日本語');

SELECT LENGTH('日本語');

SELECT name FROM users WHERE CHAR_LENGTH(name) > 10;

-- ============================================
-- 24. SOUNDEX
-- ============================================

SELECT SOUNDEX('Hello');

SELECT SOUNDEX('Helo');

SELECT name FROM users WHERE SOUNDEX(name) = SOUNDEX('John');

-- ============================================
-- 25. DIFFERENCE (if available)
-- ============================================

-- SELECT DIFFERENCE('Hello', 'Helo');

-- SELECT name FROM users WHERE DIFFERENCE(name, 'John') > 3;

-- ============================================
-- 26. ELT
-- ============================================

SELECT ELT(1, 'a', 'b', 'c');

SELECT ELT(3, 'a', 'b', 'c');

SELECT ELT(status, 'Pending', 'Active', 'Completed') FROM users;

-- ============================================
-- 27. FORMAT
-- ============================================

SELECT FORMAT(12345.6789, 2);

SELECT FORMAT(1234567.89, 0);

SELECT FORMAT(price, 2) FROM products;

-- ============================================
-- 28. QUOTE
-- ============================================

SELECT QUOTE('Hello\'s World');

SELECT QUOTE(NULL);

SELECT name, QUOTE(name) FROM users;

-- ============================================
-- 29. UNHEX
-- ============================================

SELECT UNHEX('48656C6C6F');

SELECT name, UNHEX(HEX(name)) FROM users WHERE id = 1;

-- ============================================
-- 30. HEX
-- ============================================

SELECT HEX('Hello');

SELECT HEX(name) FROM users;

SELECT name FROM users WHERE HEX(name) = '414C494345';

-- ============================================
-- 31. COMPRESS and UNCOMPRESS
-- ============================================

-- SELECT COMPRESS('Hello World');

-- SELECT UNCOMPRESS(COMPRESS('Hello World'));

-- ============================================
-- 32. MD5 and SHA
-- ============================================

SELECT MD5('Hello');

SELECT SHA('Hello');

SELECT SHA1('Hello');

SELECT MD5(email) FROM users;

SELECT SHA1(CONCAT(name, email)) FROM users;

-- ============================================
-- 33. INSTR
-- ============================================

SELECT INSTR('Hello World', 'World');

SELECT INSTR('Hello World', 'xyz');

SELECT name FROM users WHERE INSTR(name, 'John') > 0;

-- ============================================
-- 34. LCASE and UCASE (synonyms for LOWER/UPPER)
-- ============================================

SELECT LCASE('HELLO');

SELECT UCASE('hello');

-- ============================================
-- 35. LIKE with wildcards
-- ============================================

SELECT name FROM users WHERE name LIKE 'J%';

SELECT name FROM users WHERE email LIKE '%@example.com';

SELECT name FROM users WHERE name LIKE '%John%';

SELECT name FROM products WHERE description LIKE '%sale%';

SELECT name FROM users WHERE name LIKE '___';

-- ============================================
-- 36. NOT LIKE
-- ============================================

SELECT name FROM users WHERE name NOT LIKE 'A%';

SELECT name FROM products WHERE description NOT LIKE '%deprecated%';

-- ============================================
-- 37. REGEXP
-- ============================================

SELECT name FROM users WHERE name REGEXP '^J';

SELECT name FROM users WHERE email REGEXP '^[a-z]+@';

SELECT name FROM users WHERE name REGEXP 'John|Mike';

SELECT name FROM products WHERE name REGEXP '^[0-9]+';

-- ============================================
-- 38. NOT REGEXP
-- ============================================

SELECT name FROM users WHERE name NOT REGEXP '^A';

SELECT name FROM users WHERE email NOT REGEXP '[0-9]';

-- ============================================
-- 39. RLIKE (synonym for REGEXP)
-- ============================================

SELECT name FROM users WHERE name RLIKE '^A';

-- ============================================
-- 40. CHAR to number conversion
-- ============================================

SELECT CAST('123' AS UNSIGNED);

SELECT CAST('123.45' AS DECIMAL(5,2));

SELECT name FROM users WHERE CAST(id AS CHAR) LIKE '%1';

-- ============================================
-- 41. String aggregation
-- ============================================

SELECT GROUP_CONCAT(name SEPARATOR ', ') FROM users;

SELECT GROUP_CONCAT(DISTINCT category ORDER BY category SEPARATOR '; ') FROM products;

SELECT user_id, GROUP_CONCAT(product_id ORDER BY quantity DESC SEPARATOR ',') FROM order_items GROUP BY user_id;
