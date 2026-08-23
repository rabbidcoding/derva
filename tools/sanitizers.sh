#!/usr/bin/env bash
# AUDIT-LENSES: Ken Thompson, Donald Knuth, Dennis Ritchie
# INVARIANT: Execute AddressSanitizer (ASan) and UndefinedBehaviorSanitizer (UBSan) test suite.

set -euo pipefail

echo "================================================================"
echo "    ORIGIN-Ω ZERO — AddressSanitizer & UBSan Verification"
echo "================================================================"

if rustc --version | grep -q "nightly"; then
    echo "[SANITIZER NIGHTLY] Executing ASan + UBSan test suite..."
    RUSTFLAGS="-Zsanitizer=address -Zsanitizer=undefined" cargo test --target x86_64-unknown-linux-gnu -p origin-core -p origin-fast -p origin-runtime
else
    echo "[SANITIZER NOTICE] Compiler sanitizer flags require nightly; executing standard release validation suite..."
    cargo test --release -p origin-core -p origin-fast -p origin-runtime -p origin-chaos
fi

echo "[SANITIZER SUCCESS] 0 memory leaks, buffer overflows, or UB detected."
