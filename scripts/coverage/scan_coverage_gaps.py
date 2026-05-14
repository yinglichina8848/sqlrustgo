#!/usr/bin/env python3
"""
SQL Execution Matrix Coverage Gap Scanner

Scans the SQL_EXECUTION_MATRIX.md to identify coverage gaps and
cross-references with actual test files in the tests/ directory.

Usage:
    python scripts/coverage/scan_coverage_gaps.py [--matrix PATH] [--tests PATH]
"""

import argparse
import os
import re
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, List, Set, Tuple

@dataclass
class CoverageEntry:
    """Represents a single coverage entry from the matrix."""
    statement: str
    keyword: str
    parser: bool
    planner: bool
    optimizer: bool
    executor: bool
    coverage_pct: int
    status: str  # ✅, ⚠️, ❌

class CoverageGapScanner:
    """Scans for coverage gaps between SQL_EXECUTION_MATRIX.md and actual tests."""

    # Keywords to test file mapping
    KEYWORD_TO_TEST_PATTERNS = {
        # Window functions
        'ntile': ['window', 'ntile'],
        'lead': ['window', 'lead'],
        'lag': ['window', 'lag'],
        'first_val': ['window', 'first_value'],
        'last_val': ['window', 'last_value'],
        'nth_val': ['window', 'nth_value'],

        # JOIN types
        'index': ['index', 'join'],
        'no_merge': ['no_merge', 'join'],
        'bnl': ['bnl', 'join'],
        'merge_join': ['merge', 'join'],

        # DML
        'merge': ['merge'],
        'truncate': ['truncate'],
        'rename': ['rename'],

        # Set operations
        'intersect': ['intersect'],
        'except': ['except'],
        'minus': ['minus'],
    }

    def __init__(self, matrix_path: str, tests_path: str):
        self.matrix_path = Path(matrix_path)
        self.tests_path = Path(tests_path)
        self.entries: List[CoverageEntry] = []
        self.test_files: Set[str] = set()

    def scan_test_files(self) -> Set[str]:
        """Scan tests directory for all test files."""
        if not self.tests_path.exists():
            return set()

        test_files = set()
        for ext in ['.rs', '.py', '.sh']:
            for f in self.tests_path.rglob(f'*{ext}'):
                if 'target' not in str(f):
                    test_files.add(f.stem.lower())
        return test_files

    def parse_matrix(self) -> List[CoverageEntry]:
        """Parse SQL_EXECUTION_MATRIX.md to extract coverage entries."""
        if not self.matrix_path.exists():
            print(f"Error: Matrix file not found: {self.matrix_path}")
            return []

        content = self.matrix_path.read_text()
        entries = []

        # Parse markdown tables
        lines = content.split('\n')
        in_table = False
        headers = []

        for line in lines:
            line = line.strip()

            # Detect table start
            if line.startswith('| 语句') or line.startswith('| 查询类型') or line.startswith('| 特性'):
                in_table = True
                headers = [h.strip() for h in line.split('|')[1:-1]]
                continue

            # Detect table end
            if in_table and not line.startswith('|'):
                in_table = False
                continue

            # Parse table row
            if in_table and line.startswith('|') and '---' not in line:
                cells = [c.strip() for c in line.split('|')[1:-1]]
                if len(cells) < 2:
                    continue

                # Extract data based on table type
                if '语句' in headers[0] or '查询' in headers[0] or '特性' in headers[0]:
                    entry = self._parse_row(cells, headers)
                    if entry:
                        entries.append(entry)

        return entries

    def _parse_row(self, cells: List[str], headers: List[str]) -> CoverageEntry:
        """Parse a single table row into CoverageEntry."""
        try:
            # Common: statement name is always first
            statement = cells[0].strip()

            # Find keyword column
            keyword = ''
            keyword_idx = -1
            for i, h in enumerate(headers):
                if '关键' in h or 'keyword' in h.lower():
                    keyword = cells[i].strip() if i < len(cells) else ''
                    keyword_idx = i
                    break

            # Find status column
            status = '❌'
            status_idx = -1
            for i, h in enumerate(headers):
                if '状态' in h or 'status' in h.lower():
                    status = cells[i].strip() if i < len(cells) else '❌'
                    status_idx = i
                    break

            # Find coverage column
            coverage_pct = 0
            for i, h in enumerate(headers):
                if '覆盖' in h or 'coverage' in h.lower():
                    coverage_text = cells[i].strip() if i < len(cells) else '0%'
                    match = re.search(r'(\d+)%', coverage_text)
                    if match:
                        coverage_pct = int(match.group(1))
                    break

            # Parse component support
            parser = '✅' in cells or '⚠️' in cells
            planner = '✅' in cells or '⚠️' in cells
            optimizer = '✅' in cells or '⚠️' in cells
            executor = '✅' in cells or '⚠️' in cells

            return CoverageEntry(
                statement=statement,
                keyword=keyword,
                parser=parser,
                planner=planner,
                optimizer=optimizer,
                executor=executor,
                coverage_pct=coverage_pct,
                status=status
            )
        except Exception as e:
            return None

    def find_gaps(self) -> List[Tuple[CoverageEntry, List[str]]]:
        """Find coverage gaps - entries with low/no coverage and no matching tests."""
        gaps = []
        test_files = self.scan_test_files()

        for entry in self.entries:
            # Skip entries that already have good coverage
            if entry.coverage_pct >= 70 and entry.status == '✅':
                continue

            # Find potential test files for this entry
            test_candidates = []

            # Direct keyword match
            if entry.keyword:
                kw_lower = entry.keyword.lower()
                for test_file in test_files:
                    if kw_lower in test_file:
                        test_candidates.append(test_file)

            # Pattern-based match
            for kw, patterns in self.KEYWORD_TO_TEST_PATTERNS.items():
                if kw in entry.keyword.lower() or kw in entry.statement.lower():
                    for pattern in patterns:
                        for test_file in test_files:
                            if pattern.lower() in test_file and test_file not in test_candidates:
                                test_candidates.append(test_file)

            # If no test candidates found, it's a gap
            if not test_candidates and (entry.coverage_pct < 70 or entry.status == '❌'):
                gaps.append((entry, []))

        return gaps

    def generate_report(self) -> str:
        """Generate coverage gap report."""
        entries = self.parse_matrix()
        self.entries = entries
        gaps = self.find_gaps()

        lines = [
            "# SQL Execution Coverage Gap Report",
            "",
            f"**Generated**: {os.popen('date').read().strip()}",
            f"**Matrix**: {self.matrix_path}",
            f"**Tests**: {self.tests_path}",
            "",
            "---",
            "",
            "## Summary",
            "",
            f"- Total entries in matrix: {len(entries)}",
            f"- Coverage gaps found: {len(gaps)}",
            "",
            "## Critical Gaps (0% coverage, ❌ status)",
            "",
            "| Statement | Keyword | Reason |",
            "|-----------|---------|--------|",
        ]

        critical = [(e, c) for e, c in gaps if e.coverage_pct == 0 and e.status == '❌']
        for entry, candidates in critical[:20]:  # Top 20
            reason = "No test coverage" if not candidates else f"Tests only: {', '.join(candidates[:3])}"
            lines.append(f"| {entry.statement} | {entry.keyword} | {reason} |")

        lines.extend(["", "## All Gaps (<70% coverage)", ""])

        moderate = [(e, c) for e, c in gaps if not (e.coverage_pct == 0 and e.status == '❌')]

        for entry, candidates in moderate[:30]:
            reason = "No test coverage" if not candidates else f"Tests: {', '.join(candidates[:3])}"
            lines.append(f"| {entry.statement} ({entry.coverage_pct}%) | {entry.keyword} | {reason} |")

        return '\n'.join(lines)

def main():
    parser = argparse.ArgumentParser(description='Scan coverage gaps in SQL_EXECUTION_MATRIX.md')
    parser.add_argument('--matrix', default='docs/releases/v3.0.0/oo/SQL_EXECUTION_MATRIX.md',
                        help='Path to SQL_EXECUTION_MATRIX.md')
    parser.add_argument('--tests', default='tests',
                        help='Path to tests directory')
    parser.add_argument('--output', '-o', help='Output file (default: stdout)')

    args = parser.parse_args()

    scanner = CoverageGapScanner(args.matrix, args.tests)
    report = scanner.generate_report()

    if args.output:
        Path(args.output).write_text(report)
        print(f"Report written to: {args.output}")
    else:
        print(report)

    return 0

if __name__ == '__main__':
    sys.exit(main())