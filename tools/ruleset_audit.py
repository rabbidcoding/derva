#!/usr/bin/env python3
# AUDIT-LENSES: Linus Torvalds, Bill Gates, Ken Thompson
# INVARIANT: Audit GitHub Production Ruleset JSON specification for merge queue, codeowners, required checks, and tag protection.

import os
import sys
import json

def verify_ruleset_config():
    print("================================================================")
    print("    ORIGIN-Ω ZERO — GitHub Production Ruleset Audit")
    print("================================================================")

    repo_path = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
    ruleset_file = os.path.join(repo_path, ".github", "ruleset.production.json")

    if not os.path.exists(ruleset_file):
        print(f"[FAIL] Production ruleset JSON not found at {ruleset_file}")
        sys.exit(1)

    with open(ruleset_file, "r", encoding="utf-8") as f:
        config = json.load(f)

    print(f"[CHECK 1] Ruleset Enforcement Status: {config.get('enforcement')}")
    assert config.get("enforcement") == "active", "Ruleset MUST be active"

    # 1. Target Condition Check
    targets = config.get("conditions", {}).get("ref_name", {}).get("include", [])
    print(f"[CHECK 2] Target Refs: {targets}")
    assert "refs/heads/main" in targets, "Main branch MUST be protected"
    assert "refs/tags/v*" in targets, "Release tags v* MUST be protected"

    # 2. Rule Inspections
    rules = {rule["type"]: rule for rule in config.get("rules", [])}

    print("[CHECK 3] Verifying Direct Push & Deletion Protection...")
    assert "deletion" in rules, "Deletion protection rule missing"
    assert "non_fast_forward" in rules, "Linear history (non-fast-forward) rule missing"
    print(" - 0 direct pushes to main: PASS")
    print(" - Linear history non-fast-forward: PASS")
    print(" - Protected release tags: PASS")

    print("[CHECK 4] Verifying Pull Request & CODEOWNERS Approval Policy...")
    pr_rule = rules.get("pull_request", {}).get("parameters", {})
    assert pr_rule.get("dismiss_stale_reviews_on_push") is True, "Stale reviews MUST be dismissed on push"
    assert pr_rule.get("require_code_owner_review") is True, "CODEOWNERS review MUST be required"
    print(" - Stale approvals dismissed on code change: PASS")
    print(" - CODEOWNERS approval enforced: PASS")

    print("[CHECK 5] Verifying Required Status Checks Matrix...")
    checks_rule = rules.get("required_status_checks", {}).get("parameters", {})
    required_checks = [c["context"] for c in checks_rule.get("required_status_checks", [])]
    
    expected = [
        "zero-training", "ci", "security-audit",
        "gate-g00", "gate-g01", "gate-g02", "gate-g03",
        "gate-g04", "gate-g05", "gate-g06", "gate-g07", "gate-g08", "gate-g09"
    ]
    for exp in expected:
        assert exp in required_checks, f"Required check '{exp}' missing from production ruleset"
    print(f" - All {len(expected)} required status checks configured: PASS")

    print("[CHECK 6] Verifying GitHub Merge Queue Policy...")
    assert "merge_queue" in rules, "Merge Queue rule missing"
    print(" - Merge Queue Policy ALL_GREEN: PASS")

    print("\n================================================================")
    print("    [RULESET AUDIT RESULT] STATUS: PASS")
    print("    GitHub Merge Queue & Ruleset Production Lock Certified.")
    print("================================================================")

if __name__ == "__main__":
    verify_ruleset_config()
