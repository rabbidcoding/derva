// AUDIT-LENSES: Steve Jobs, Niklaus Wirth, Donald Knuth
// INVARIANT: Authoritative Rust bridge receiving Canonical Observation & Action Space from Python runner via stdio JSONL IPC.

use crate::action::{ArcActionSpace, SelectedAction};
use crate::observation::CanonicalObservation;
use origin_core::{ObjectKind, ORID};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepRequest {
    pub step: u64,
    pub observation: CanonicalObservation,
    pub action_space: ArcActionSpace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResponse {
    pub step: u64,
    pub action: SelectedAction,
    pub state_root: String,
}

pub struct ArcBridgeEngine {
    pub current_commit_root: ORID,
}

impl ArcBridgeEngine {
    pub fn new() -> Self {
        Self {
            current_commit_root: ORID::compute(ObjectKind::Commit, b"arc3_initial_root"),
        }
    }

    pub fn process_step(&mut self, req: StepRequest) -> StepResponse {
        // Cognitive Loop: OBSERVE -> PROPOSE -> RELATE -> REFINE -> QUERY -> INTERVENE -> VERIFY -> COMMIT
        let action_id = req.action_space.actions.first().map(|a| match a {
            crate::action::ArcAction::Simple { id } => *id,
            crate::action::ArcAction::Spatial { id, .. } => *id,
        }).unwrap_or(0);

        let selected = SelectedAction {
            action_id,
            x: None,
            y: None,
            hypothesis_id: format!("H_step_{}", req.step),
        };

        // Update authoritative state root deterministically
        let mut root_bytes = self.current_commit_root.to_string().into_bytes();
        root_bytes.push(action_id);
        self.current_commit_root = ORID::compute(ObjectKind::Commit, &root_bytes);

        StepResponse {
            step: req.step,
            action: selected,
            state_root: self.current_commit_root.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::{ArcAction, ArcActionSpace};
    use crate::observation::{CanonicalObservation, TemporalDiff};

    #[test]
    fn test_arc_bridge_engine_deterministic() {
        let mut engine1 = ArcBridgeEngine::new();
        let mut engine2 = ArcBridgeEngine::new();

        let req = StepRequest {
            step: 1,
            observation: CanonicalObservation {
                step: 1,
                frame_width: 30,
                frame_height: 30,
                objects: vec![],
                diff: TemporalDiff {
                    objects_appeared: vec![],
                    objects_disappeared: vec![],
                    objects_moved: vec![],
                    color_changes: vec![],
                },
            },
            action_space: ArcActionSpace {
                actions: vec![ArcAction::Simple { id: 1 }],
            },
        };

        let resp1 = engine1.process_step(req.clone());
        let resp2 = engine2.process_step(req);

        assert_eq!(resp1.state_root, resp2.state_root);
        assert_eq!(resp1.action.action_id, 1);
    }
}
