# ORIGIN-Ω ZERO — Main Branch & Merge Queue Production Lock Policy

## Epistemic Constitution & Branch Governance

- **Target Branch**: `refs/heads/main`
- **Target Release Tags**: `refs/tags/v*`
- **Enforcement Status**: **ACTIVE (Zero-Bypass Policy)**
- **Audit Lenses**: **Linus Torvalds**, **Bill Gates**, **Ken Thompson**

---

## Production Ruleset Contract (`.github/ruleset.production.json`)

### 1. Zero Direct Pushes (`no_direct_push`)
- Direct `git push origin main` is strictly prohibited.
- All state changes must be proposed via GitHub Pull Request.

### 2. Linear History Enforcement (`non_fast_forward`)
- Non-linear merge commits are rejected by non-fast-forward protection.
- Merges must occur via linear rebase or squash merge strategy.

### 3. Required Status Checks Pipeline (`required_status_checks`)
The following status checks are required before any merge into `main`:
- `zero-training` (Task T001 / T010 / T090 / T099 invariant)
- `ci` (Continuous Integration test & lint suite)
- `security-audit` (Red-team & SAST verification)
- `gate-g00` through `gate-g09` (Phase Certification Gates P00–P09)

### 4. Code Ownership & Stale Review Invalidation (`pull_request`)
- Required approving reviews: $\ge 1$.
- `require_code_owner_review: true`: Mandatory approval from CODEOWNERS for TCB paths (`crates/origin-core`, `crates/origin-kernel`, `asm/`, `.github/`).
- `dismiss_stale_reviews_on_push: true`: Any push to a PR branch automatically invalidates previous approvals.

### 5. GitHub Merge Queue (`merge_queue`)
- Merge queue strategy `ALL_GREEN` ensures PRs are tested in combined speculative queue order before committing to `main`.
- Minimum entries to build: 1; Maximum parallel queue build entries: 5.

### 6. Release Tag Protection (`deletion`)
- Release tags matching `v*` cannot be deleted, modified, or force-pushed.

---

## Ruleset Verification Script

Run `python3 tools/ruleset_audit.py` to audit compliance against `.github/ruleset.production.json`.
