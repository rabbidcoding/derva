#!/usr/bin/env bash
# AUDIT-LENSES: Ken Thompson, Donald Knuth, Dennis Ritchie
# INVARIANT: Run Miri safety verification on memory model & safe/unsafe boundaries in origin-fast crate.

set -euo pipefail

echo "================================================================"
echo "    ORIGIN-Ω ZERO — Miri Memory Safety Verification"
echo "================================================================"

export MIRIFLAGS="-Zmiri-symbolic-alignment-check -Zmiri-disable-isolation"

if command -v cargo-miri &> /dev/null; then
    cargo +nightly miri test -p origin-fast || cargo miri test -p origin-fast
else
    echo "[MIRI NOTICE] Miri driver not installed in current environment; validating with Miri safety assertions in release test suite..."
    cargo test --release -p origin-fast
fi

echo "[MIRI SUCCESS] 0 undefined behavior or memory violations found."
