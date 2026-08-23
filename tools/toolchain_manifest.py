#!/usr/bin/env python3
"""
INVARIANT: Pinned reproducible toolchain manifest generator with SHA-256 lockfile hashes.
KPI: 100% lockfile hash reproducibility on clean checkout.
"""

import sys
import subprocess
import json
import hashlib
from pathlib import Path

def compute_sha256(path: Path) -> str:
    if not path.exists():
        return "MISSING"
    return hashlib.sha256(path.read_bytes()).hexdigest()

def get_toolchain_versions(root: Path):
    manifest = {
        "rustc": None,
        "cargo": None,
        "python": sys.version.split()[0],
        "lockfile_hashes": {
            "rust_toolchain_toml": compute_sha256(root / "rust-toolchain.toml"),
            "cargo_lock": compute_sha256(root / "Cargo.lock"),
            "uv_lock": compute_sha256(root / "uv.lock"),
        }
    }
    
    try:
        manifest["rustc"] = subprocess.check_output(["rustc", "--version"], text=True).strip()
    except Exception as e:
        manifest["rustc"] = f"Error: {e}"
        
    try:
        manifest["cargo"] = subprocess.check_output(["cargo", "--version"], text=True).strip()
    except Exception as e:
        manifest["cargo"] = f"Error: {e}"
        
    return manifest

def main():
    root = Path(__file__).resolve().parent.parent
    manifest = get_toolchain_versions(root)
    out_file = root / "spec" / "toolchain_manifest.json"
    
    out_file.parent.mkdir(parents=True, exist_ok=True)
    out_file.write_text(json.dumps(manifest, indent=2), encoding="utf-8")
    
    print(f"[TOOLCHAIN MANIFEST] Generated at {out_file}:")
    print(json.dumps(manifest, indent=2))

if __name__ == "__main__":
    main()
