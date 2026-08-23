#!/usr/bin/env python3
import os
from pathlib import Path

crates = [
    "origin-kernel",
    "origin-store",
    "origin-verify",
    "origin-logic",
    "origin-constraints",
    "origin-search",
    "origin-egraph",
    "origin-reason",
    "origin-causal",
    "origin-plan",
    "origin-oir",
    "origin-codegen-rust",
    "origin-compiler",
    "origin-fast",
    "origin-runtime",
    "origin-cli",
    "origin-bench",
    "origin-chaos",
]

root = Path(__file__).resolve().parent.parent / "crates"

for c in crates:
    cdir = root / c
    src = cdir / "src"
    src.mkdir(parents=True, exist_ok=True)
    
    cargo_toml = cdir / "Cargo.toml"
    if not cargo_toml.exists():
        cargo_toml.write_text(f"""[package]
name = "{c}"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true

[dependencies]
origin-core = {{ path = "../origin-core" }}
""")
        
    lib_rs = src / "lib.rs"
    main_rs = src / "main.rs"
    
    if c == "origin-cli":
        if not main_rs.exists():
            main_rs.write_text("""fn main() {
    println!("ORIGIN-Ω ZERO CLI v0.1.0");
}
""")
    else:
        if not lib_rs.exists():
            lib_rs.write_text(f"""// ORIGIN-Ω ZERO — Subsystem: {c}

pub fn crate_name() -> &'static str {{
    "{c}"
}}
""")

print("[OK] Workspace crates initialized cleanly.")
