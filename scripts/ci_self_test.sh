#!/bin/bash
#
# CI Self-Test Script - Validates that CI properly rejects bad behavior
#
# This script tests the CI anti-cheat mechanisms by attempting attacks
# and verifying that CI correctly rejects them.
#

set -e

echo "=================================================="
echo "🚨 CI SELF TEST - Anti-Cheat Validation"
echo "=================================================="
echo ""

BRANCH="ci-attack-test"
MAIN_BRANCH="main"

# Cleanup any existing test branch
git branch -D $BRANCH 2>/dev/null || true

# Create fresh attack test branch
git checkout -b $BRANCH

TEST_FILE="tests/test_example.rs"

# -------------------------
# Setup: Create a minimal test file for attacks
# -------------------------
mkdir -p tests
cat > $TEST_FILE << 'TESTFILE'
#[cfg(test)]
mod tests {
    #[test]
    fn test_example() {
        assert_eq!(2 + 2, 4);
    }
}
TESTFILE

git add $TEST_FILE
git commit -m "setup: initial test file"
git push -u origin $BRANCH 2>/dev/null || true

echo "✅ Attack branch created: $BRANCH"
echo ""

# -------------------------
# Attack 1: Modify test expectation
# -------------------------
echo "=================================================="
echo "🔥 ATTACK 1: Modify test expectation"
echo "=================================================="
echo "Expected CI result: ❌ FAIL (test modification detected)"
echo ""

# Modify the test file
sed -i 's/assert_eq!(2 + 2, 4)/assert_eq!(2 + 2, 5)/' $TEST_FILE

git add $TEST_FILE
git commit -m "attack: modify test expectation"
git push -f origin $BRANCH 2>/dev/null || true

echo "⚠️  Attack 1 committed. Check CI result on PR."
echo "Expected: CI should FAIL"
echo ""
read -p "Press ENTER after confirming CI result (should be FAIL)..."

# Revert attack 1
git reset --hard HEAD~1
git push -f origin $BRANCH 2>/dev/null || true

# -------------------------
# Attack 2: Add #[ignore]
# -------------------------
echo "=================================================="
echo "🔥 ATTACK 2: Add #[ignore]"
echo "=================================================="
echo "Expected CI result: ❌ FAIL (#[ignore] detected)"
echo ""

cat > $TEST_FILE << 'TESTFILE'
#[cfg(test)]
mod tests {
    #[test]
    #[ignore]
    fn test_example() {
        assert_eq!(2 + 2, 4);
    }
}
TESTFILE

git add $TEST_FILE
git commit -m "attack: add ignore"
git push -f origin $BRANCH 2>/dev/null || true

echo "⚠️  Attack 2 committed. Check CI result on PR."
echo "Expected: CI should FAIL"
echo ""
read -p "Press ENTER after confirming CI result (should be FAIL)..."

# Revert attack 2
git reset --hard HEAD~1
git push -f origin $BRANCH 2>/dev/null || true

# -------------------------
# Attack 3: Fake test execution
# -------------------------
echo "=================================================="
echo "🔥 ATTACK 3: Fake test execution"
echo "=================================================="
echo "Expected CI result: ❌ FAIL (test not actually run)"
echo ""

# Create a fake test script
mkdir -p scripts
cat > scripts/fake_test_runner.sh << 'FAKESCRIPT'
#!/bin/bash
echo "All tests passed!"  # Fake success
exit 0
FAKESCRIPT
chmod +x scripts/fake_test_runner.sh

git add scripts/
git commit -m "attack: add fake test script"
git push -f origin $BRANCH 2>/dev/null || true

echo "⚠️  Attack 3 committed. Check CI result on PR."
echo "Expected: CI should FAIL (fake test detected)"
echo ""
read -p "Press ENTER after confirming CI result (should be FAIL)..."

# Revert attack 3
git reset --hard HEAD~1
git push -f origin $BRANCH 2>/dev/null || true

# -------------------------
# Cleanup
# -------------------------
echo "=================================================="
echo "🧹 CLEANUP"
echo "=================================================="

git checkout $MAIN_BRANCH
git branch -D $BRANCH
git push origin --delete $BRANCH 2>/dev/null || true

echo "✅ CI self-test complete!"
echo ""
echo "=================================================="
echo "📊 ATTACK SUMMARY"
echo "=================================================="
echo "Attack 1 (test modification):     CI should ❌ FAIL"
echo "Attack 2 (#[ignore]):             CI should ❌ FAIL"
echo "Attack 3 (fake test execution):   CI should ❌ FAIL"
echo ""
echo "If all attacks were correctly blocked by CI,"
echo "your Anti-Cheat CI is working correctly."
echo "=================================================="
