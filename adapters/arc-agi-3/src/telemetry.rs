// AUDIT-LENSES: Ken Thompson, Linus Torvalds, Elon Musk
// INVARIANT: Step-level JSONL Telemetry logging for ARC-AGI-3 runs without secrets or unverified state.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepTelemetry {
    pub run_id: String,
    pub game_id: String,
    pub step: u64,
    pub derva_commit: String,
    pub state_root_before: String,
    pub state_root_after: String,
    pub hypotheses_total: usize,
    pub hypotheses_supported: usize,
    pub hypotheses_contested: usize,
    pub hypotheses_refuted: usize,
    pub active_nodes: usize,
    pub total_nodes: usize,
    pub candidate_actions: usize,
    pub selected_action: u8,
    pub cpu_us: u128,
    pub rss_mb: usize,
}
