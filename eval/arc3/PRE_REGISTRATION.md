# DERVA / ORIGIN-Ω ZERO — ARC-AGI-3 Evaluation Pre-Registration

## System Snapshot
- **DERVA Version**: `v1.0.0-rc1` (Post-Frontier Candidate Certified)
- **Git Commit**: `920e798` (or current tag `derva-v1-arc3-pretest`)
- **Execution Date**: 2026-08-23

## System Invariants & Profile
- **Trainable Parameters**: `0`
- **Pretrained Weights**: `0`
- **Gradient Updates**: `0`
- **External LLM / Heuristics**: `0`
- **Game-Specific Rules / Walkthroughs**: `0`
- **Kernel Autonomy**: DERVA TCB is 100% domain-agnostic and non-trainable.

## Evaluation Metrics Matrix

### Primary Metric
- **ARC-AGI-3 RHAE (Relative Human Action Efficiency)**:
  $$Score_l = \left(\frac{\text{Actions}_{\text{human}}}{\text{Actions}_{\text{AI}}}\right)^2$$

### Secondary Metrics
- Levels Completed / Completion Rate (%)
- Environment Actions Executed
- Hypotheses Generated / Refuted / Supported
- Active-Slice Graph Ratio ($|G_{\text{active}}| / |G|$)
- Planning Depth & E-graph sat iterations
- CPU Latency (p50, p99 per action)
- Memory Growth (RAM & VRAM bounds)
- Bitwise Deterministic Replay Parity (100% Root Match)

---

## Authoritative Pre-Registration Signatures
- **Architecture**: DERVA Post-Frontier Candidate
- **Zero-Training Invariant Guard**: `python3 tools/zero_train_guard.py` (PASS)
