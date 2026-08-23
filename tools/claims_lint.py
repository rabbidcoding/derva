#!/usr/bin/env python3
"""
KPI: Every claim in spec/claims.yaml must have metric, baseline, target, gate, owner, benchmark_id, and kill_condition.
"""

import sys
from pathlib import Path

def parse_claims_simple(text: str) -> list:
    claims = []
    current = None
    for line in text.splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        if line.startswith("- id:"):
            if current:
                claims.append(current)
            current = {"id": line.split(":", 1)[1].strip().strip('"')}
        elif current and ":" in line:
            k, v = line.split(":", 1)
            current[k.strip()] = v.strip().strip('"')
    if current:
        claims.append(current)
    return claims

def main():
    spec_file = Path(__file__).resolve().parent.parent / "spec" / "claims.yaml"
    print(f"[CLAIMS LINT PY] Validating claim ledger at: {spec_file}")
    
    if not spec_file.exists():
        print(f"[FAIL] Missing {spec_file}")
        sys.exit(1)
        
    text = spec_file.read_text(encoding="utf-8")
    claims = parse_claims_simple(text)
        
    required_keys = ["id", "description", "metric", "baseline", "target", "gate", "owner", "benchmark_id", "kill_condition"]
    errors = []
    
    for claim in claims:
        cid = claim.get("id", "UNKNOWN")
        for k in required_keys:
            if k not in claim or claim[k] is None:
                errors.append(f"Claim '{cid}' is missing required field '{k}'")
        if "POST-FRONTIER" in cid and not claim.get("benchmark_id"):
            errors.append(f"Post-frontier claim '{cid}' must resolve to a valid benchmark_id!")
                
    if errors:
        print(f"[FAIL] {len(errors)} errors found in claim ledger:")
        for err in errors:
            print(f"  - {err}")
        sys.exit(1)
    else:
        print(f"[PASS] All {len(claims)} claims in ledger are valid and falsable.")
        sys.exit(0)

if __name__ == "__main__":
    main()
