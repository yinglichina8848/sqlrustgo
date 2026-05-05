#!/usr/bin/env python3
"""Proof Coverage v2: Risk-Weighted Formal Invariant Coverage Calculator"""
import json
import sys
import re
import os

def main():
    db_path = sys.argv[1]
    changed_file = sys.argv[2]
    test_log_file = sys.argv[3]
    out_json = sys.argv[4]
    mode = sys.argv[5]

    with open(db_path) as f:
        db = json.load(f)

    with open(changed_file) as f:
        changed = set(line.strip() for line in f if line.strip())

    with open(test_log_file) as f:
        test_log = f.read()

    ci_gates = db.get('ci_gates', {})
    RISK_MIN = ci_gates.get('risk_score_min', 0.70)
    WARN_RISK = ci_gates.get('warn_risk_score', 0.50)

    impacted = []
    for inv in db['invariants']:
        matched_files = [f for f in inv.get('files', []) if f in changed]
        if not matched_files and mode != '--full':
            continue

        test_results = []
        for t in inv.get('tests', []):
            patterns = [
                r'test\s+[\w:]+' + re.escape(t) + r'\s+\.\.\.\s+(ok|FAILED)',
                r'\s' + re.escape(t) + r'\s+\.\.\.\s+(ok|FAILED)',
            ]
            matched = False
            for pat in patterns:
                m = re.search(pat, test_log)
                if m:
                    test_results.append({'name': t, 'status': m.group(1)})
                    matched = True
                    break
            if not matched:
                test_results.append({'name': t, 'status': 'missing'})

        has_tests = len(inv.get('tests', [])) > 0
        all_pass = all(r['status'] == 'ok' for r in test_results)
        any_pass = any(r['status'] == 'ok' for r in test_results)

        if not has_tests:
            cov_weight = 0.0
        elif all_pass:
            cov_weight = 1.0
        elif any_pass:
            cov_weight = 0.5
        else:
            cov_weight = 0.0

        criticality = inv.get('criticality', 0.5)
        confidence = inv.get('confidence', 0.5)
        frozen = inv.get('status') == 'frozen'
        risk_score = criticality * confidence * cov_weight

        impacted.append({
            'id': inv['id'],
            'proof': inv['proof'],
            'level': inv.get('level', 'unknown'),
            'status': inv.get('status', 'active'),
            'frozen': frozen,
            'ci_layer': inv.get('ci_layer', 'unknown'),
            'criticality': criticality,
            'confidence': confidence,
            'coverage_weight': cov_weight,
            'risk_score': risk_score,
            'tests': inv.get('tests', []),
            'test_results': test_results,
            'has_tests': has_tests,
            'matched_files': matched_files,
            'tla_invariant': inv.get('tla_invariant', ''),
            'failure_severity': inv.get('failure_severity', 'unknown'),
        })

    active = [i for i in impacted if not i['frozen']]
    frozen_list = [i for i in impacted if i['frozen']]

    if active:
        total_crit = sum(i['criticality'] for i in active)
        weighted_risk_score = sum(
            i['criticality'] * i['confidence'] * i['coverage_weight'] for i in active
        ) / sum(i['criticality'] * i['confidence'] for i in active) if active else 0.0
    else:
        total_crit = 0.0
        weighted_risk_score = 0.0

    final_risk_score = round(weighted_risk_score, 3)

    if not active:
        gate = 'SKIP'
        gate_exit = 0
    elif final_risk_score >= RISK_MIN:
        gate = 'PASS'
        gate_exit = 0
    elif final_risk_score >= WARN_RISK:
        gate = 'WARN'
        gate_exit = 2
    else:
        gate = 'FAIL'
        gate_exit = 1

    RED = '\033[0;31m'
    GREEN = '\033[0;32m'
    YELLOW = '\033[1;33m'
    BLUE = '\033[0;34m'
    NC = '\033[0m'

    print("")
    print("══════════════════════════════════════════════")
    print("  Risk-Weighted Coverage Report")
    print("══════════════════════════════════════════════")
    print("")

    print(f"  {'Invariant':<28} {'Crit':>5} {'Conf':>5} {'Cov':>5}  {'Risk':>6}  Status")
    print(f"  {'-'*27} {'-'*5} {'-'*5} {'-'*5}  {'-'*6}  ------")

    for i in impacted:
        if i['frozen']:
            icon = '─'
            status_str = f"{BLUE}[FROZEN]{NC}"
            risk_str = f"{BLUE}0.000{NC}"
        elif i['risk_score'] >= 0.8:
            icon = '✔'
            status_str = f"{GREEN}ACTIVE{NC}"
            risk_str = f"{GREEN}{i['risk_score']:.3f}{NC}"
        elif i['risk_score'] >= 0.4:
            icon = '⚠'
            status_str = f"{YELLOW}PARTIAL{NC}"
            risk_str = f"{YELLOW}{i['risk_score']:.3f}{NC}"
        elif i['risk_score'] > 0:
            icon = '⚠'
            status_str = f"{YELLOW}WEAK{NC}"
            risk_str = f"{YELLOW}{i['risk_score']:.3f}{NC}"
        else:
            icon = '✘'
            status_str = f"{RED}NOCOVER{NC}"
            risk_str = f"{RED}0.000{NC}"

        print(f"  {icon} {i['id']:<26} {i['criticality']:.2f} {i['confidence']:.2f} {i['coverage_weight']:.2f}  {risk_str}  {status_str}")

    print(f"  {'-'*27} {'-'*5} {'-'*5} {'-'*5}  {'-'*6}  ------")

    print("")
    print(f"  Risk Score:     {final_risk_score:.3f}")
    print(f"  Critical Total: {total_crit:.2f}")
    print(f"  Active:         {len(active)} invariants")
    print(f"  Frozen:         {len(frozen_list)} invariants")
    print("")

    print("──────────────────────────────────────────")
    print(f"  Gate:           {gate}")
    print(f"  Risk Score:     {final_risk_score:.3f}")
    print(f"  Threshold PASS:  {RISK_MIN:.2f}")
    print(f"  Threshold WARN: {WARN_RISK:.2f}")
    print("──────────────────────────────────────────")

    if gate == 'PASS':
        print(f"\n  {GREEN}✔ PASS — Risk Score {final_risk_score:.3f} >= {RISK_MIN}{NC}")
    elif gate == 'WARN':
        print(f"\n  {YELLOW}⚠ WARN — Risk Score {final_risk_score:.3f} ({WARN_RISK}-{RISK_MIN}){NC}")
    elif gate == 'FAIL':
        print(f"\n  {RED}✘ FAIL — Risk Score {final_risk_score:.3f} < {WARN_RISK}{NC}")
    else:
        print(f"\n  {BLUE}ℹ SKIP — No active invariants impacted{NC}")

    print("")

    gaps = [i for i in active if i['coverage_weight'] < 0.5 and not i['frozen']]
    if gaps:
        print("──────────────────────────────────────────")
        print("  Coverage Gaps (coverage_weight < 0.5)")
        print("──────────────────────────────────────────")
        for g in gaps:
            missing = [t['name'] for t in g['test_results'] if t['status'] == 'missing']
            gap_reason = "NO TESTS" if not g['has_tests'] else f"FAILING: {', '.join(missing[:2])}"
            print(f"  {RED}✘{NC} {g['id']} — {gap_reason}")
            print(f"    Severity: {g['failure_severity']}")
            if g['matched_files']:
                print(f"    Files: {', '.join(g['matched_files'][:2])}")
        print("")

    if frozen_list:
        print("──────────────────────────────────────────")
        print("  Frozen Invariants (excluded from gate)")
        print("──────────────────────────────────────────")
        for f in frozen_list:
            note = f.get('note', '')
            print(f"  {BLUE}─{NC} {f['id']} — {note}")
            print(f"    Failure: {f['failure_severity']}")
        print("")

    result = {
        'timestamp': os.popen('date +%Y-%m-%dT%H:%M:%SZ').read().strip(),
        'mode': mode,
        'risk_score': final_risk_score,
        'gate': gate,
        'gate_exit': gate_exit,
        'risk_min': RISK_MIN,
        'warn_risk': WARN_RISK,
        'total_critical': round(total_crit, 3),
        'active_count': len(active),
        'frozen_count': len(frozen_list),
        'invariants': [{
            'id': x['id'],
            'level': x['level'],
            'status': x['status'],
            'frozen': x['frozen'],
            'criticality': x['criticality'],
            'confidence': x['confidence'],
            'coverage_weight': x['coverage_weight'],
            'risk_score': x['risk_score'],
            'has_tests': x['has_tests'],
            'tests': x['tests'],
            'matched_files': x['matched_files'],
            'failure_severity': x['failure_severity'],
        } for x in impacted]
    }

    with open(out_json, 'w') as f:
        json.dump(result, f, indent=2)

    print(f"  JSON: {out_json}")

if __name__ == '__main__':
    main()
