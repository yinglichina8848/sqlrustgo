#!/bin/bash
cd /home/ai/sqlrustgo
mkdir -p test_results_72h

echo "Starting 3 x 72h tests in parallel..."
echo "Started at: $(date)"

nohup cargo test --test long_run_stability_72h_test -- --ignored test_sustained_write_72h > test_results_72h/test1_write.log 2>&1 &
echo "Test 1 (write): PID $!"

nohup cargo test --test long_run_stability_72h_test -- --ignored test_sustained_read_72h > test_results_72h/test2_read.log 2>&1 &
echo "Test 2 (read): PID $!"

nohup cargo test --test long_run_stability_72h_test -- --ignored test_concurrent_read_write_72h > test_results_72h/test3_concurrent.log 2>&1 &
echo "Test 3 (concurrent): PID $!"

echo ""
echo "All 3 tests started. Check progress with:"
echo "  tail -f test_results_72h/test1_write.log"
echo "  tail -f test_results_72h/test2_read.log"
echo "  tail -f test_results_72h/test3_concurrent.log"
echo ""
echo "Expected completion: 72 hours from now"
