# ORIGIN-Ω ZERO — Release Documentation & Deployment Guide

## Overview

**ORIGIN-Ω ZERO** is a zero-training, post-frontier, authoritative, deterministic evidence and causality engine.
This document outlines the deployment, verification, and attestation procedures for release candidates and production binaries.

---

## Release Verification Protocol

Before deploying or publishing any release candidate, run the automated Release Candidate Acceptance Gate:

```bash
python3 tools/gate_rc.py
```

### Artifact Manifest

Release build artifacts are placed in `./dist/`:
- `dist/origin-cli`: Compiled CLI binary for production.
- `dist/origin-zero-sbom.spdx.json`: SPDX 2.3 Software Bill of Materials.
- `dist/SHA256SUMS`: SHA256 cryptographic digests of all release assets.
- `dist/reproducibility_manifest.json`: Independent rebuild reproducibility specification.

---

## Provenance & Attestation Verification

To independently verify the SLSA build provenance of a published release asset using the GitHub CLI:

```bash
gh attestation verify ./dist/origin-cli --repo rabbidcoding/derva
```

---

## Epistemic Debugger CLI Usage

Inspect system state, trace verified claims, or replay state execution:

```bash
# Tracing justification for a verified claim
./dist/origin-cli why orid:claim:8f3c7a --json

# Replaying commit state execution
./dist/origin-cli replay orid:commit:root01 --verify --json

# Displaying micro-op execution stats
./dist/origin-cli profile --json
```

**RELEASE STATUS: READY FOR POST-FRONTIER PRODUCTION DEPLOYMENT**
