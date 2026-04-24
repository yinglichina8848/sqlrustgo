.PHONY: audit test clean verify

# SQLRustGo Governance Makefile
# Stable baseline: hermes-stable-v2.8.1

AUDIT_Runs ?= 3
AUDIT_Output ?= audit_report.json

## audit — Run self-audit (independent verification)
##   Runs cargo test N times, cross-checks proof vs reality.
##   Fail if any critical failure detected.
audit:
	python3 scripts/self_audit.py --runs $(AUDIT_Runs) --output $(AUDIT_Output)
	@echo "Audit complete. See $(AUDIT_Output)"

## audit-quick — Single-run audit (fast)
audit-quick:
	python3 scripts/self_audit.py --runs 1 --output $(AUDIT_Output)

## test — Run full test suite
test:
	cargo test

## verify — Verify contract + proof consistency (no test execution)
verify:
	@echo "=== Contract ===" && python3 -c "import json; c=json.load(open('contract/v2.8.0.json')); print('Rules:', len(c['rules']), ', Baseline:', c['baseline']['test_counts'])"
	@echo "=== Proof ===" && python3 -c "import json; p=json.load(open('verification_report.json')); print('Status:', p['status'], ', Passed:', p['passed'])"

## clean — Remove audit artifacts
clean:
	rm -f $(AUDIT_Output)
	rm -f /tmp/audit_*.json
