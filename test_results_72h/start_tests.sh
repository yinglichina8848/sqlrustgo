#!/bin/bash
cd /home/ai/sqlrustgo
TEST_BIN="/home/ai/sqlrustgo/target/release/deps/long_run_stability_72h_test-395926ed30e8a7f6"

$TEST_BIN --ignored test_sustained_write_72h > test_results_72h/test1_write.log 2>&1 &
echo "Test 1: $!"

$TEST_BIN --ignored test_sustained_read_72h > test_results_72h/test2_read.log 2>&1 &
echo "Test 2: $!"

$TEST_BIN --ignored test_concurrent_read_write_72h > test_results_72h/test3_concurrent.log 2>&1 &
echo "Test 3: $!"
