#!/usr/bin/env python3
"""
INVARIANT: Gate G01 closes Phase P01 only if 100% of P01 formal semantics & status lattice properties are verified.
KPI: 100% pass rate on formal semantics, model checking, and micro-ISA unit & property tests.
"""

import sys
import subprocess
from pathlib import Path

def run_command(cmd: str, cwd: Path) -> bool:
    print(f"[G01 GATE] Running check: {cmd}...")
    res = subprocess.run(cmd, shell=True, cwd=cwd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    if res.returncode == 0:
        print(f"  -> PASS: {cmd}")
        return True
    else:
        print(f"  -> FAIL: {cmd}")
        print(f"     stdout: {res.stdout}")
        print(f"     stderr: {res.stderr}")
        return False

def main():
    root = Path(__file__).resolve().parent.parent
    print("========== G01 FORMAL CONSISTENCY GATE AUDIT ==========")
    
    checks = [
        ("Zero-Training Guard Check", "python3 tools/zero_train_guard.py"),
        ("Formal Model Checking (origin-modelcheck)", "cargo test -p origin-modelcheck"),
        ("Rust Unit & Property Tests", "cargo test --workspace"),
        ("Workspace Build Integrity", "cargo check --workspace"),
    ]
    
    results = []
    for name, cmd in checks:
        ok = run_command(cmd, root)
        results.append((name, ok))
        
    failed = [n for n, ok in results if not ok]
    
    report_file = root / "reports" / "gates" / "G01.md"
    report_file.parent.mkdir(parents=True, exist_ok=True)
    
    status_str = "PASS" if not failed else "FAIL"
    report_md = f"# G01 Gate Execution Report\n\n> **Status:** {status_str}\n\n## Formal Verification Checklist\n\n"
    for name, ok in results:
        mark = "✅ PASS" if ok else "❌ FAIL"
        report_md += f"- [{mark}] {name}\n"
        
    report_file.write_text(report_md, encoding="utf-8")
    print(f"\n[G01 GATE REPORT] Written to {report_file}")
    
    if failed:
        print(f"[G01 GATE FAIL] Checks failed: {failed}")
        sys.exit(1)
    else:
        print("[G01 GATE PASS] All formal consistency checks green. Phase P01 closed successfully.")
        sys.exit(0)

if __name__ == "__main__":
    main()
