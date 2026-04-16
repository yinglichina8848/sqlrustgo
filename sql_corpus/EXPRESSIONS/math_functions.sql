-- === SKIP ===

-- === Math and Trigonometry Functions Test Suite ===

-- === CASE: ABS - absolute value ===
-- EXPECT: 1 row
SELECT ABS(-42) as abs_val, ABS(42) as abs_val2;

-- === CASE: CEIL - ceiling ===
-- EXPECT: 1 row
SELECT CEIL(3.14) as ceil_val, CEIL(-3.14) as ceil_neg;

-- === CASE: FLOOR - floor ===
-- EXPECT: 1 row
SELECT FLOOR(3.14) as floor_val, FLOOR(-3.14) as floor_neg;

-- === CASE: ROUND - rounding ===
-- EXPECT: 1 row
SELECT ROUND(3.14159, 2) as rounded, ROUND(3.5) as rounded_int;

-- === CASE: TRUNC - truncate ===
-- EXPECT: 1 row
SELECT TRUNC(3.14159, 2) as truncated, TRUNC(3.999, 1) as trunc_dec;

-- === CASE: EXP - exponential ===
-- EXPECT: 1 row
SELECT EXP(1) as e_power_1, EXP(0) as e_power_0;

-- === CASE: LN - natural log ===
-- EXPECT: 1 row
SELECT LN(2.71828) as natural_log, LN(1) as log_one;

-- === CASE: LOG - base 10 log ===
-- EXPECT: 1 row
SELECT LOG(100) as log_100, LOG(10) as log_10;

-- === CASE: LOG2 - base 2 log ===
-- EXPECT: 1 row
SELECT LOG2(8) as log2_8, LOG2(2) as log2_2;

-- === CASE: LOG10 - base 10 log ===
-- EXPECT: 1 row
SELECT LOG10(1000) as log10_1000, LOG10(100) as log10_100;

-- === CASE: MOD - modulo ===
-- EXPECT: 1 row
SELECT MOD(10, 3) as mod_val, MOD(10, 5) as mod_zero;

-- === CASE: POWER - power ===
-- EXPECT: 1 row
SELECT POWER(2, 8) as power_2_8, POWER(3, 2) as power_3_2;

-- === CASE: SQRT - square root ===
-- EXPECT: 1 row
SELECT SQRT(16) as sqrt_16, SQRT(2) as sqrt_2;

-- === CASE: PI constant ===
-- EXPECT: 1 row
SELECT PI() as pi_value;

-- === CASE: RADIANS - degrees to radians ===
-- EXPECT: 1 row
SELECT RADIANS(180) as radians_180, RADIANS(90) as radians_90;

-- === CASE: DEGREES - radians to degrees ===
-- EXPECT: 1 row
SELECT DEGREES(3.14159) as degrees_pi, DEGREES(1.5708) as degrees_half_pi;

-- === CASE: SIN - sine ===
-- EXPECT: 1 row
SELECT SIN(0) as sin_0, SIN(3.14159 / 2) as sin_pi_2;

-- === CASE: COS - cosine ===
-- EXPECT: 1 row
SELECT COS(0) as cos_0, COS(3.14159) as cos_pi;

-- === CASE: TAN - tangent ===
-- EXPECT: 1 row
SELECT TAN(0) as tan_0, TAN(0.785398) as tan_quarter_pi;

-- === CASE: ASIN - arc sine ===
-- EXPECT: 1 row
SELECT ASIN(0) as asin_0, ASIN(1) as asin_1;

-- === CASE: ACOS - arc cosine ===
-- EXPECT: 1 row
SELECT ACOS(1) as acos_1, ACOS(0) as acos_0;

-- === CASE: ATAN - arc tangent ===
-- EXPECT: 1 row
SELECT ATAN(0) as atan_0, ATAN(1) as atan_1;

-- === CASE: ATAN2 - two argument arc tangent ===
-- EXPECT: 1 row
SELECT ATAN2(1, 1) as atan2_1_1;

-- === CASE: SINH - hyperbolic sine ===
-- EXPECT: 1 row
SELECT SINH(0) as sinh_0, SINH(1) as sinh_1;

-- === CASE: COSH - hyperbolic cosine ===
-- EXPECT: 1 row
SELECT COSH(0) as cosh_0, COSH(1) as cosh_1;

-- === CASE: TANH - hyperbolic tangent ===
-- EXPECT: 1 row
SELECT TANH(0) as tanh_0, TANH(1) as tanh_1;

-- === CASE: ACOSH - inverse hyperbolic cosine ===
-- EXPECT: 1 row
SELECT ACOSH(1) as acosh_1, ACOSH(2) as acosh_2;

-- === CASE: ASINH - inverse hyperbolic sine ===
-- EXPECT: 1 row
SELECT ASINH(0) as asinh_0, ASINH(1) as asinh_1;

-- === CASE: ATANH - inverse hyperbolic tangent ===
-- EXPECT: 1 row
SELECT ATANH(0) as atanh_0, ATANH(0.5) as atanh_half;

-- === CASE: COT - cotangent ===
-- EXPECT: 1 row
SELECT COT(1) as cot_1;

-- === CASE: SIGN - sign of number ===
-- EXPECT: 1 row
SELECT SIGN(-5) as sign_neg, SIGN(0) as sign_zero, SIGN(5) as sign_pos;
