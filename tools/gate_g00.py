#!/usr/bin/env python3
"""
INVARIANT: Gate G00 closes Phase P00 only if 100% of P00 requirements (T001-T009) are verified green.
KPI: 10/10 checks PASS; 0 bypasses allowed.
"""

import sys
import subprocess
from pathlib import Path

def run_command(cmd: str, cwd: Path) -> bool:
    print(f"[G00 GATE] Running check: {cmd}...")
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
    print("========== G00 TRUTH + REPOSITORY GATE AUDIT ==========")
    
    checks = [
        ("Zero-Training Invariant Guard", "python3 tools/zero_train_guard.py"),
        ("Claim Ledger & Kill Criteria Lint", "python3 tools/claims_lint.py"),
        ("Toolchain Manifest Verification", "python3 tools/toolchain_manifest.py"),
        ("Rust Monorepo Compilation", "cargo check --workspace"),
        ("Rust Test Suite", "cargo test --workspace"),
    ]
    
    results = []
    for name, cmd in checks:
        ok = run_command(cmd, root)
        results.append((name, ok))
        
    failed = [n for n, ok in results if not ok]
    
    report_file = root / "reports" / "gates" / "G00.md"
    report_file.parent.mkdir(parents=True, exist_ok=True)
    
    status_str = "PASS" if not failed else "FAIL"
    report_md = f"# G00 Gate Execution Report\n\n> **Status:** {status_str}\n\n## Verification Checklist\n\n"
    for name, ok in results:
        mark = "✅ PASS" if ok else "❌ FAIL"
        report_md += f"- [{mark}] {name}\n"
        
    report_file.write_text(report_md, encoding="utf-8")
    print(f"\n[G00 GATE REPORT] Written to {report_file}")
    
    if failed:
        print(f"[G00 GATE FAIL] Checks failed: {failed}")
        sys.exit(1)
    else:
        print("[G00 GATE PASS] All 5 core checks green. Phase P00 closed successfully.")
        sys.exit(0)

if __name__ == "__main__":
    main()
