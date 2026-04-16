-- SQLCorpus: CASE Expressions
-- Tests for CASE WHEN expressions

-- === SETUP ===
CREATE TABLE case_test (id INTEGER PRIMARY KEY, score INTEGER);
INSERT INTO case_test VALUES (1, 95), (2, 85), (3, 75), (4, 65), (5, 55);

-- === CASE: simple_case ===
SELECT id, CASE score WHEN 95 THEN 'A' WHEN 85 THEN 'B' WHEN 75 THEN 'C' ELSE 'F' END as grade FROM case_test;
-- EXPECT: 5 rows

-- === CASE: searched_case ===
SELECT id, CASE WHEN score >= 90 THEN 'A' WHEN score >= 80 THEN 'B' WHEN score >= 70 THEN 'C' WHEN score >= 60 THEN 'D' ELSE 'F' END as grade FROM case_test;
-- EXPECT: 5 rows

-- === CASE: case_with_agg ===
SELECT CASE WHEN COUNT(*) > 3 THEN 'Large' ELSE 'Small' END as size FROM case_test;
-- EXPECT: 1 rows

-- === CASE: case_null ===
SELECT CASE WHEN score IS NULL THEN 'Missing' ELSE 'Present' END as status FROM case_test;
-- EXPECT: 5 rows

-- === CASE: case_expression ===
SELECT id, score, CASE WHEN score >= 70 THEN 'Pass' ELSE 'Fail' END as result FROM case_test;
-- EXPECT: 5 rows

-- === CASE: nested_case ===
SELECT id, CASE WHEN score >= 90 THEN CASE WHEN score = 100 THEN 'Perfect' ELSE 'Excellent' END ELSE 'Other' END as classification FROM case_test;
-- EXPECT: 5 rows

-- === CASE: case_in_where ===
SELECT * FROM case_test WHERE CASE WHEN score >= 70 THEN 'Pass' ELSE 'Fail' END = 'Pass';
-- EXPECT: 3 rows

-- === CASE: case_in_update ===
UPDATE case_test SET score = CASE WHEN score < 60 THEN 60 ELSE score END;
-- EXPECT: success