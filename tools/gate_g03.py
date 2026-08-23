#!/usr/bin/env python3
"""
INVARIANT: Gate G03 closes Phase P03 only if 100% of P03 Evidence, Logic & Constraint Engine invariants are verified.
KPI: 100% VERIFIED claims have navigable why(claim) lineage; 0 UNSAT/CONTESTED claims promoted to VERIFIED; 0 Unsafe Rust; 1e6 Adversarial Cases PASS.
"""

import sys
import subprocess
import re
from pathlib import Path

def run_command(cmd: str, cwd: Path) -> bool:
    print(f"[G03 GATE] Running check: {cmd}...")
    res = subprocess.run(cmd, shell=True, cwd=cwd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    if res.returncode == 0:
        print(f"  -> PASS: {cmd}")
        return True
    else:
        print(f"  -> FAIL: {cmd}")
        print(f"     stdout: {res.stdout}")
        print(f"     stderr: {res.stderr}")
        return False

def check_unsafe_code_zero(root: Path) -> bool:
    print("[G03 GATE] Checking Unsafe Rust count in authoritative crates...")
    crates = [
        "origin-core",
        "origin-kernel",
        "origin-store",
        "origin-evidence",
        "origin-verify",
        "origin-logic",
        "origin-constraints",
        "origin-search",
    ]
    unsafe_pattern = re.compile(r'\bunsafe\b\s*(\{|fn|trait|impl)')
    unsafe_found = 0

    for crate in crates:
        crate_dir = root / "crates" / crate / "src"
        if not crate_dir.exists():
            continue
        for path in crate_dir.rglob("*.rs"):
            content = path.read_text(encoding="utf-8")
            for line_idx, line in enumerate(content.splitlines(), start=1):
                stripped = line.strip()
                if stripped.startswith("//") or stripped.startswith("/*") or "forbid(unsafe_code)" in stripped:
                    continue
                if unsafe_pattern.search(stripped):
                    print(f"  -> FAIL: Unsafe Rust usage detected in {path}:{line_idx}: '{line}'")
                    unsafe_found += 1

    if unsafe_found == 0:
        print("  -> PASS: 0 unsafe blocks detected in authoritative crates.")
        return True
    return False

def main():
    root = Path(__file__).resolve().parent.parent
    print("========== G03 EVIDENCE & LOGIC GATE AUDIT ==========")

    checks = [
        ("Zero-Training Guard Check", "python3 tools/zero_train_guard.py"),
        ("Unsafe Rust Block Audit (0 Unsafe)", lambda: check_unsafe_code_zero(root)),
        ("Workspace Cargo Check", "cargo check --workspace"),
        ("Strict Clippy Audit (-D warnings)", "cargo clippy --workspace -- -D warnings"),
        ("Evidence & Logic Engine Test Suite", "cargo test -p origin-evidence -p origin-verify -p origin-logic -p origin-constraints"),
        ("1e6 Epistemic Adversarial Proof Suite", "cargo test --test epistemic_adversarial"),
        ("Full Workspace Test Suite", "cargo test --workspace"),
    ]

    results = []
    for name, cmd in checks:
        if callable(cmd):
            ok = cmd()
        else:
            ok = run_command(cmd, root)
        results.append((name, ok))

    failed = [n for n, ok in results if not ok]

    report_file = root / "reports" / "gates" / "G03.md"
    report_file.parent.mkdir(parents=True, exist_ok=True)

    status_str = "PASS" if not failed else "FAIL"
    report_md = f"# G03 Gate Execution Report\n\n> **Status:** {status_str}\n\n## Formal Verification & Evidence-Logic Audit Checklist\n\n"
    for name, ok in results:
        mark = "✅ PASS" if ok else "❌ FAIL"
        report_md += f"- [{mark}] {name}\n"

    report_file.write_text(report_md, encoding="utf-8")
    print(f"\n[G03 GATE REPORT] Written to {report_file}")

    if failed:
        print(f"[G03 GATE FAIL] Checks failed: {failed}")
        sys.exit(1)
    else:
        print("[G03 GATE PASS] All evidence, logic & constraint engine checks green. Phase P03 closed successfully.")
        sys.exit(0)

if __name__ == "__main__":
    main()
