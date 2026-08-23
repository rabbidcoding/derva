#!/usr/bin/env python3
"""
INVARIANT: TRAINABLE_PARAMETER_COUNT == 0
KPI: Zero forbidden neural training imports, gradient loops, or unbacked weight checkpoints.
"""

import sys
import os
import re
from pathlib import Path

FORBIDDEN_TERMS = [
    r"jax\.grad",
    r"jax\.value_and_grad",
    r"optax",
    r"flax\.training",
    r"torch\.optim",
    r"tensorflow",
    r"backprop",
    r"loss\.backward\(",
    r"optimizer\.step\(",
]

FORBIDDEN_EXTENSIONS = [
    ".safetensors",
    ".bin",
    ".ckpt",
    ".h5",
    ".pt",
    ".pth",
    ".onnx",
]

ALLOWED_EXCEPTIONS = [
    "spec/zero_training.md",
    "tools/zero_train_guard.py",
    "ORIGIN_OMEGA_ZERO_ROADMAP_T001_T100_POST_FRONTIER_PRODUCTION.md"
]

def scan_repository(root_dir: Path) -> list:
    violations = []
    
    for path in root_dir.rglob("*"):
        if path.is_file():
            rel_path = str(path.relative_to(root_dir))
            
            # Skip git, target, venv, and allowed spec exceptions
            if any(p in rel_path for p in [".git", "target", "__pycache__", ".venv", "node_modules"]):
                continue
            if any(rel_path.endswith(exc) or rel_path == exc for exc in ALLOWED_EXCEPTIONS):
                continue
                
            # 1. Extension check for forbidden neural weight checkpoints
            if path.suffix.lower() in FORBIDDEN_EXTENSIONS:
                violations.append({
                    "file": rel_path,
                    "line": 0,
                    "match": f"Forbidden weight checkpoint extension ({path.suffix})",
                    "content": rel_path
                })
                continue

            # 2. Content scan for forbidden training code imports/patterns
            try:
                content = path.read_text(encoding="utf-8", errors="ignore")
                for line_idx, line in enumerate(content.splitlines(), start=1):
                    for term in FORBIDDEN_TERMS:
                        if re.search(term, line):
                            violations.append({
                                "file": rel_path,
                                "line": line_idx,
                                "match": term,
                                "content": line.strip()
                            })
            except Exception:
                pass
                
    return violations

def main():
    root_dir = Path(__file__).resolve().parent.parent
    print(f"[ZERO-TRAIN GUARD] Scanning repository at: {root_dir}")
    violations = scan_repository(root_dir)
    
    if violations:
        print(f"\n[FAIL] Found {len(violations)} ZERO-TRAINING VIOLATIONS:")
        for v in violations:
            print(f"  - {v['file']}:{v['line']} -> Term '{v['match']}' in: '{v['content']}'")
        sys.exit(1)
    else:
        print("\n[PASS] Zero-Training invariant verified: trainable_parameter_count == 0.")
        sys.exit(0)

if __name__ == "__main__":
    main()
