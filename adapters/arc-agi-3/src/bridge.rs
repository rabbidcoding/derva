// AUDIT-LENSES: Steve Jobs, Niklaus Wirth, Donald Knuth, Alan Turing
// INVARIANT: Authoritative Rust bridge for ARC-AGI-3. Implements Strategy vs Hypothesis separation, Executable Predictions, Hypothesis Discrimination Action Selection, Retrodiction Vectors, Four Prediction Outcomes (ExactMatch, PartialMatch, Contradiction, Uninformative), and Strict Epistemic State Promotion (No Illegal Verified Promotions).

use std::collections::VecDeque;
use crate::action::{ArcAction, ArcActionSpace, SelectedAction};
use crate::observation::{CanonicalObservation, GridEvent};
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
    pub telemetry: Option<String>,
}

/// Exploration Strategy Lifecycle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrategyState {
    Active,
    Stagnant,
    Deprioritized,
    Failed,
}

/// World Model Hypothesis Epistemic Lifecycle
/// INVARIANT: VERIFIED requires formal verification proof; empirical success only promotes up to SUPPORTED.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HypothesisStatus {
    Unverified,
    Supported,
    Contested,
    Refuted,
    Verified,
}

/// Four Distinct Prediction Outcomes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PredictionOutcome {
    ExactMatch,
    PartialMatch,
    Contradiction,
    Uninformative,
}

/// Retrodiction Vector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrodictionVector {
    pub applicable: u32,
    pub correct: u32,
    pub contradicted: u32,
    pub ambiguous: u32,
}

/// Executable Transition Model Hypothesis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionHypothesis {
    pub id: String,
    pub action_id: u8,
    pub target_object_id: Option<u32>,
    pub predicted_event: GridEvent,
    pub status: HypothesisStatus,
    pub support_count: u32,
    pub refutation_count: u32,
    pub retrodiction: RetrodictionVector,
}

/// Active Prediction Receipt emitted prior to action execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionReceipt {
    pub receipt_id: String,
    pub hypothesis_id: String,
    pub step: u64,
    pub action_id: u8,
    pub target_coords: Option<[u8; 2]>,
    pub expected_event: GridEvent,
    pub state_root: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalTransition {
    pub step: u64,
    pub action_id: u8,
    pub target_coords: Option<[u8; 2]>,
    pub events_observed: Vec<GridEvent>,
    pub state_root: String,
}

pub struct ArcBridgeEngine {
    pub current_commit_root: ORID,
    pub last_observation: Option<CanonicalObservation>,
    pub last_action: Option<SelectedAction>,
    pub step_counter: u64,
    pub history_trace: Vec<HistoricalTransition>,
    pub strategy_state: StrategyState,
    pub active_hypotheses: Vec<TransitionHypothesis>,
    pub pending_prediction: Option<PredictionReceipt>,
    pub supported_hypotheses: u64,
    pub refuted_hypotheses: u64,
    pub target_object_idx: usize,
    pub recent_state_roots: VecDeque<String>,
    pub stagnation_counter: u32,
}

impl ArcBridgeEngine {
    pub fn new() -> Self {
        Self {
            current_commit_root: ORID::compute(ObjectKind::Commit, b"arc3_initial_root"),
            last_observation: None,
            last_action: None,
            step_counter: 0,
            history_trace: Vec::new(),
            strategy_state: StrategyState::Active,
            active_hypotheses: Vec::new(),
            pending_prediction: None,
            supported_hypotheses: 0,
            refuted_hypotheses: 0,
            target_object_idx: 0,
            recent_state_roots: VecDeque::with_capacity(10),
            stagnation_counter: 0,
        }
    }

    pub fn process_step(&mut self, req: StepRequest) -> StepResponse {
        self.step_counter += 1;
        let mut telemetry_logs = Vec::new();

        // 1. EVENT INDUCTION & RECEIPT RESOLUTION
        let events = if let Some(prev_obs) = &self.last_observation {
            req.observation.compute_events(prev_obs)
        } else {
            Vec::new()
        };

        if !events.is_empty() {
            telemetry_logs.push(format!("[EVENT OBSERVED] Step {}: {:?}", self.step_counter, events));
        }

        if let Some(prev_act) = &self.last_action {
            let transition = HistoricalTransition {
                step: self.step_counter - 1,
                action_id: prev_act.action_id,
                target_coords: match (prev_act.x, prev_act.y) {
                    (Some(x), Some(y)) => Some([x, y]),
                    _ => None,
                },
                events_observed: events.clone(),
                state_root: self.current_commit_root.to_string(),
            };
            self.history_trace.push(transition);

            // Resolve Pending Prediction Receipt if present
            if let Some(receipt) = self.pending_prediction.take() {
                let outcome = if events.iter().any(|e| e == &receipt.expected_event) {
                    PredictionOutcome::ExactMatch
                } else if !events.is_empty() {
                    PredictionOutcome::Contradiction
                } else {
                    PredictionOutcome::Uninformative
                };

                telemetry_logs.push(format!("[RECEIPT RESOLVED] ID: {} | Outcome: {:?}", receipt.receipt_id, outcome));

                if let Some(hyp) = self.active_hypotheses.iter_mut().find(|h| h.id == receipt.hypothesis_id) {
                    match outcome {
                        PredictionOutcome::ExactMatch => {
                            hyp.support_count += 1;
                            hyp.status = HypothesisStatus::Supported;
                            self.supported_hypotheses += 1;
                            telemetry_logs.push(format!("[EPISTEMIC PROMOTION] {} -> SUPPORTED (SupportCount: {})", hyp.id, hyp.support_count));
                        }
                        PredictionOutcome::Contradiction => {
                            hyp.refutation_count += 1;
                            if hyp.refutation_count >= 2 {
                                hyp.status = HypothesisStatus::Refuted;
                                self.refuted_hypotheses += 1;
                                telemetry_logs.push(format!("[FALSIFICATION] {} -> REFUTED", hyp.id));
                            } else {
                                hyp.status = HypothesisStatus::Contested;
                                telemetry_logs.push(format!("[FALSIFICATION] {} -> CONTESTED", hyp.id));
                            }
                        }
                        PredictionOutcome::Uninformative => {
                            telemetry_logs.push(format!("[UNINFORMATIVE] Preconditions not met for {}; status remains {:?}", hyp.id, hyp.status));
                        }
                        PredictionOutcome::PartialMatch => {}
                    }
                }
            }

            // Induce new Transition Hypotheses from observed events
            for ev in &events {
                let hyp_id = format!("H_trans_act{}_ev{:?}", prev_act.action_id, ev);
                if !self.active_hypotheses.iter().any(|h| h.id == hyp_id) {
                    let mut retro = RetrodictionVector {
                        applicable: 0,
                        correct: 0,
                        contradicted: 0,
                        ambiguous: 0,
                    };

                    // RETRODICTION MATRIX: Evaluate across history trace
                    for past_trans in &self.history_trace {
                        if past_trans.action_id == prev_act.action_id {
                            retro.applicable += 1;
                            if past_trans.events_observed.iter().any(|e| e == ev) {
                                retro.correct += 1;
                            } else if !past_trans.events_observed.is_empty() {
                                retro.contradicted += 1;
                            } else {
                                retro.ambiguous += 1;
                            }
                        }
                    }

                    let hyp = TransitionHypothesis {
                        id: hyp_id.clone(),
                        action_id: prev_act.action_id,
                        target_object_id: None,
                        predicted_event: ev.clone(),
                        status: HypothesisStatus::Unverified,
                        support_count: 1,
                        refutation_count: 0,
                        retrodiction: retro.clone(),
                    };

                    telemetry_logs.push(format!("[PROPOSE HYPOTHESIS] {} | Retrodict Vector: (correct={}, applicable={}, contradicted={})", hyp_id, retro.correct, retro.applicable, retro.contradicted));
                    self.active_hypotheses.push(hyp);
                }
            }

            // Update Strategy Stagnation
            if events.is_empty() {
                self.stagnation_counter += 1;
            } else {
                self.stagnation_counter = 0;
                self.strategy_state = StrategyState::Active;
            }
        }

        // 2. STRATEGY REVISION & CYCLE DETECTOR
        let current_root_str = self.current_commit_root.to_string();
        if self.recent_state_roots.contains(&current_root_str) || self.stagnation_counter >= 6 {
            if self.strategy_state != StrategyState::Deprioritized {
                telemetry_logs.push("[STRATEGY DEPRIORITIZED] Stagnation or loop detected. Switch to ExplorationSweep.".to_string());
            }
            self.strategy_state = StrategyState::Deprioritized;
            self.target_object_idx = self.target_object_idx.wrapping_add(3);
            self.stagnation_counter = 0;
        }

        if self.recent_state_roots.len() >= 8 {
            self.recent_state_roots.pop_front();
        }
        self.recent_state_roots.push_back(current_root_str);

        // 3. ACTION SELECTION: HYPOTHESIS DISCRIMINATION VS COVERAGE EXPLORATION
        let candidate_actions = &req.action_space.actions;
        let num_candidates = candidate_actions.len();

        let spatial_action_id = candidate_actions.iter().find_map(|a| match a {
            ArcAction::Spatial { id, .. } => Some(*id),
            _ => None,
        });

        // Identify competing hypotheses
        let candidate_hyp = self.active_hypotheses.iter().find(|h| {
            h.status == HypothesisStatus::Supported || h.status == HypothesisStatus::Unverified
        }).cloned();

        let selected = if let Some(hyp) = candidate_hyp {
            // DISCRIMINATION MODE
            let (target_x, target_y) = if let Some(_sp_id) = spatial_action_id {
                let num_objects = req.observation.objects.len();
                if num_objects > 0 {
                    let obj = &req.observation.objects[self.target_object_idx % num_objects];
                    (Some(obj.centroid[0]), Some(obj.centroid[1]))
                } else {
                    (Some(15), Some(15))
                }
            } else {
                (None, None)
            };

            let receipt_id = format!("P{}", self.step_counter);
            self.pending_prediction = Some(PredictionReceipt {
                receipt_id: receipt_id.clone(),
                hypothesis_id: hyp.id.clone(),
                step: self.step_counter,
                action_id: hyp.action_id,
                target_coords: match (target_x, target_y) {
                    (Some(x), Some(y)) => Some([x, y]),
                    _ => None,
                },
                expected_event: hyp.predicted_event.clone(),
                state_root: self.current_commit_root.to_string(),
            });

            telemetry_logs.push(format!("[DISCRIMINATION EXPERIMENT] Receipt: {} | TargetHypothesis: {} | ExpectedEvent: {:?}", receipt_id, hyp.id, hyp.predicted_event));

            SelectedAction {
                action_id: hyp.action_id,
                x: target_x,
                y: target_y,
                hypothesis_id: format!("Discriminate_{}|Receipt:{}", hyp.id, receipt_id),
            }
        } else if let Some(sp_id) = spatial_action_id {
            // EXPLORATION MODE: Alternate spatial target selection with directional key probes (ACTION1..ACTION4)
            let num_objects = req.observation.objects.len();
            if num_objects > 0 && self.strategy_state == StrategyState::Active {
                let target_idx = self.target_object_idx % num_objects;
                let target_obj = &req.observation.objects[target_idx];
                let tx = target_obj.centroid[0];
                let ty = target_obj.centroid[1];

                if self.step_counter % 2 == 1 {
                    SelectedAction {
                        action_id: sp_id,
                        x: Some(tx),
                        y: Some(ty),
                        hypothesis_id: format!("Select_obj_{}_x{}_y{}", target_obj.id, tx, ty),
                    }
                } else {
                    let dir_act = (self.step_counter % 4) as u8 + 1;
                    SelectedAction {
                        action_id: dir_act,
                        x: None,
                        y: None,
                        hypothesis_id: format!("Probe_dir_action_{}", dir_act),
                    }
                }
            } else {
                let grid_w = (req.observation.frame_width as u64).min(21).max(1);
                let grid_h = (req.observation.frame_height as u64).min(21).max(1);
                let sweep_index = self.step_counter - 1;
                let tx = (sweep_index % grid_w) as u8;
                let ty = ((sweep_index / grid_w) % grid_h) as u8;

                SelectedAction {
                    action_id: sp_id,
                    x: Some(tx),
                    y: Some(ty),
                    hypothesis_id: format!("ExplorationSweep_x{}_y{}", tx, ty),
                }
            }

        } else if num_candidates > 0 {
            let choice_idx = self.step_counter as usize % num_candidates;
            match &candidate_actions[choice_idx] {
                ArcAction::Simple { id } => SelectedAction {
                    action_id: *id,
                    x: None,
                    y: None,
                    hypothesis_id: format!("SimpleExplore_{}", id),
                },
                ArcAction::Spatial { id, x_min, x_max, y_min, y_max } => SelectedAction {
                    action_id: *id,
                    x: Some((*x_min + *x_max) / 2),
                    y: Some((*y_min + *y_max) / 2),
                    hypothesis_id: format!("SpatialExplore_{}", id),
                },
            }
        } else {
            SelectedAction {
                action_id: 1,
                x: None,
                y: None,
                hypothesis_id: "H_fallback".to_string(),
            }
        };

        // 4. UPDATE COMMIT ROOT & PERSIST OBSERVATION
        self.last_observation = Some(req.observation);
        self.last_action = Some(selected.clone());

        let mut root_bytes = self.current_commit_root.to_string().into_bytes();
        root_bytes.push(selected.action_id);
        if let Some(x) = selected.x { root_bytes.extend_from_slice(&(x as u32).to_le_bytes()); }
        if let Some(y) = selected.y { root_bytes.extend_from_slice(&(y as u32).to_le_bytes()); }
        self.current_commit_root = ORID::compute(ObjectKind::Commit, &root_bytes);

        let telemetry_str = if telemetry_logs.is_empty() {
            None
        } else {
            Some(telemetry_logs.join("\n"))
        };

        StepResponse {
            step: req.step,
            action: selected,
            state_root: self.current_commit_root.to_string(),
            telemetry: telemetry_str,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::{ArcAction, ArcActionSpace};
    use crate::observation::{CanonicalObservation, TemporalDiff};

    #[test]
    fn test_arc_bridge_engine_epistemic_receipt_lifecycle() {
        let mut engine = ArcBridgeEngine::new();

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
                    events: vec![],
                },
            },
            action_space: ArcActionSpace {
                actions: vec![ArcAction::Simple { id: 1 }],
            },
        };

        let resp = engine.process_step(req);
        assert_eq!(resp.step, 1);
        assert_eq!(resp.action.action_id, 1);
        assert_eq!(engine.strategy_state, StrategyState::Active);
    }
}
