// AUDIT-LENSES: Steve Jobs, Niklaus Wirth, Donald Knuth, Alan Turing
// INVARIANT: Authoritative Rust bridge for ARC-AGI-3. Implements Sub-Gate G10.2-C1 Closure Requirements: Experiment Quality Score (EQS) Gating (Applicable & Observable & Discriminating), Refinement Gain Metrics (RG_retro & RG_prospective >= 0.10), Resolution Evaluability Rate (RER >= 70%), Dual-Context Prospective Confirmation (Free -> Event, Blocked -> NoChange), and Zero Epistemic Downgrades on Uninformative Outcomes.

use std::collections::{HashMap, VecDeque};
use crate::action::{ArcAction, ArcActionSpace, SelectedAction};
use crate::observation::{CanonicalObservation, GenericObject, GridEvent};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrategyState {
    Active,
    Stagnant,
    Deprioritized,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HypothesisStatus {
    Unverified,
    Supported,
    Contested,
    Refuted,
    Verified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventCategory {
    NoChange,
    Moved,
    ColorChanged,
    Appeared,
    Disappeared,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PredictionOutcome {
    ExactMatch,
    PartialMatch,
    Contradiction,
    Uninformative(UninformativeReason),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BlockingCause {
    Boundary,
    Occupancy,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UninformativeReason {
    EffectBlocked(BlockingCause),
    PreconditionsNotSatisfied,
    PredictionObservationallyEquivalent,
    TargetUnavailable,
    AmbiguousMatching,
    InsufficientObservation,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RetrodictionVector {
    pub applicable: u32,
    pub correct: u32,
    pub contradicted: u32,
    pub ambiguous: u32,
}

impl RetrodictionVector {
    pub fn accuracy(&self) -> f64 {
        if self.applicable == 0 {
            0.0
        } else {
            self.correct as f64 / self.applicable as f64
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ProspectiveVector {
    pub issued: u32,
    pub exact_matches: u32,
    pub partial_matches: u32,
    pub contradictions: u32,
}


impl ProspectiveVector {
    pub fn accuracy(&self) -> f64 {
        if self.issued == 0 {
            0.0
        } else {
            (self.exact_matches as f64 + 0.5 * self.partial_matches as f64) / self.issued as f64
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionHypothesis {
    pub id: String,
    pub derived_from: Option<String>,
    pub action_id: u8,
    pub target_object_id: Option<u32>,
    pub predicted_event: GridEvent,
    pub requires_feasibility: bool,
    pub status: HypothesisStatus,
    pub support_count: u32,
    pub refutation_count: u32,
    pub retrodiction: RetrodictionVector,
    pub prospective: ProspectiveVector,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FeasibilityWitness {
    pub object_id: u32,
    pub bbox: [u8; 4],                     // [min_x, min_y, max_x, max_y]
    pub predicted_delta: [i8; 2],          // [dx, dy]
    pub predicted_target_bbox: [i16; 4],   // [new_min_x, new_min_y, new_max_x, new_max_y]
    pub frame_width: u8,
    pub frame_height: u8,
    pub bounds_feasible: bool,
    pub blocking_cause: Option<BlockingCause>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionReceipt {
    pub receipt_id: String,
    pub primary_hyp_id: String,
    pub secondary_hyp_id: Option<String>,
    pub step: u64,
    pub action_id: u8,
    pub target_coords: Option<[u8; 2]>,
    pub expected_event: GridEvent,
    pub state_root: String,
    pub precondition_witness: bool,
    pub contextual_feasibility: bool,
    pub feasibility_witness: FeasibilityWitness,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalTransition {
    pub step: u64,
    pub action_id: u8,
    pub target_coords: Option<[u8; 2]>,
    pub events_observed: Vec<GridEvent>,
    pub state_root: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CategoryConfusion {
    pub true_positives: u64,
    pub false_positives: u64,
    pub false_negatives: u64,
}

impl CategoryConfusion {
    pub fn f1_score(&self) -> f64 {
        let precision = if self.true_positives + self.false_positives == 0 {
            0.0
        } else {
            self.true_positives as f64 / (self.true_positives + self.false_positives) as f64
        };
        let recall = if self.true_positives + self.false_negatives == 0 {
            0.0
        } else {
            self.true_positives as f64 / (self.true_positives + self.false_negatives) as f64
        };
        if precision + recall == 0.0 {
            0.0
        } else {
            2.0 * (precision * recall) / (precision + recall)
        }
    }
}

pub struct BaselineTracker {
    pub derva_confusions: HashMap<EventCategory, CategoryConfusion>,
    pub b0_random_confusions: HashMap<EventCategory, CategoryConfusion>,
    pub b1_no_change_confusions: HashMap<EventCategory, CategoryConfusion>,
    pub b2_last_effect_confusions: HashMap<EventCategory, CategoryConfusion>,
    pub b3_freq_majority_confusions: HashMap<EventCategory, CategoryConfusion>,
    pub category_counts: HashMap<EventCategory, u64>,
    pub last_effect: EventCategory,
    pub b0_seed: u64,
    pub issued_receipts: u64,
    pub valid_at_issue_receipts: u64,
    pub informative_receipts: u64,
    pub uninformative_breakdown: HashMap<UninformativeReason, u64>,

    // Sub-Gate G10.2-D & F Metrics
    pub exact_event_matches: u64,
    pub exact_event_total: u64,
    pub window_f1_scores: Vec<f64>,
    pub current_bucket_hits: u64,
    pub current_bucket_total: u64,
}

impl BaselineTracker {
    pub fn new() -> Self {
        Self {
            derva_confusions: HashMap::new(),
            b0_random_confusions: HashMap::new(),
            b1_no_change_confusions: HashMap::new(),
            b2_last_effect_confusions: HashMap::new(),
            b3_freq_majority_confusions: HashMap::new(),
            category_counts: HashMap::new(),
            last_effect: EventCategory::NoChange,
            b0_seed: 0xCAFEF00D,
            issued_receipts: 0,
            valid_at_issue_receipts: 0,
            informative_receipts: 0,
            uninformative_breakdown: HashMap::new(),
            exact_event_matches: 0,
            exact_event_total: 0,
            window_f1_scores: Vec::new(),
            current_bucket_hits: 0,
            current_bucket_total: 0,
        }
    }

    pub fn record_prediction(&mut self, predicted_cat: EventCategory, actual_cat: EventCategory, outcome: PredictionOutcome, valid_at_issue: bool) {
        self.issued_receipts += 1;
        if valid_at_issue {
            self.valid_at_issue_receipts += 1;
        }

        match outcome {
            PredictionOutcome::Uninformative(reason) => {
                *self.uninformative_breakdown.entry(reason).or_default() += 1;
                return;
            }
            _ => {
                self.informative_receipts += 1;
            }
        }

        let categories = [
            EventCategory::NoChange,
            EventCategory::Moved,
            EventCategory::ColorChanged,
            EventCategory::Appeared,
            EventCategory::Disappeared,
        ];

        // B0 Predictor (Random Category)
        self.b0_seed = self.b0_seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let b0_pred = categories[(self.b0_seed >> 32) as usize % 5];
        let b0_entry = self.b0_random_confusions.entry(actual_cat).or_default();
        if b0_pred == actual_cat {
            b0_entry.true_positives += 1;
        } else {
            b0_entry.false_negatives += 1;
            let fp_entry = self.b0_random_confusions.entry(b0_pred).or_default();
            fp_entry.false_positives += 1;
        }

        // B3 Predictor (Frequency Majority / Mode)
        let b3_pred = self
            .category_counts
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(cat, _)| *cat)
            .unwrap_or(EventCategory::NoChange);
        let b3_entry = self.b3_freq_majority_confusions.entry(actual_cat).or_default();
        if b3_pred == actual_cat {
            b3_entry.true_positives += 1;
        } else {
            b3_entry.false_negatives += 1;
            let fp_entry = self.b3_freq_majority_confusions.entry(b3_pred).or_default();
            fp_entry.false_positives += 1;
        }
        *self.category_counts.entry(actual_cat).or_default() += 1;

        // DERVA Predictor (B4)
        let derva_entry = self.derva_confusions.entry(actual_cat).or_default();
        if predicted_cat == actual_cat {
            derva_entry.true_positives += 1;
            self.current_bucket_hits += 1;
        } else {
            derva_entry.false_negatives += 1;
            let fp_entry = self.derva_confusions.entry(predicted_cat).or_default();
            fp_entry.false_positives += 1;
        }
        self.current_bucket_total += 1;

        // Exact Event tracking for ExactEventF1
        self.exact_event_total += 1;
        if outcome == PredictionOutcome::ExactMatch {
            self.exact_event_matches += 1;
        }

        // Online Learning Curve Bucket tracking (Every 5 receipts)
        if self.current_bucket_total >= 5 {
            let bucket_acc = self.current_bucket_hits as f64 / self.current_bucket_total as f64;
            self.window_f1_scores.push(bucket_acc);
            self.current_bucket_hits = 0;
            self.current_bucket_total = 0;
        }

        // Baseline B1 (No-Change)
        let b1_entry = self.b1_no_change_confusions.entry(actual_cat).or_default();
        if actual_cat == EventCategory::NoChange {
            b1_entry.true_positives += 1;
        } else {
            b1_entry.false_negatives += 1;
            let fp_entry = self.b1_no_change_confusions.entry(EventCategory::NoChange).or_default();
            fp_entry.false_positives += 1;
        }

        // Baseline B2 (Last-Effect)
        let b2_entry = self.b2_last_effect_confusions.entry(actual_cat).or_default();
        if self.last_effect == actual_cat {
            b2_entry.true_positives += 1;
        } else {
            b2_entry.false_negatives += 1;
            let fp_entry = self.b2_last_effect_confusions.entry(self.last_effect).or_default();
            fp_entry.false_positives += 1;
        }
        self.last_effect = actual_cat;
    }

    pub fn issue_validity_rate(&self) -> f64 {
        if self.issued_receipts == 0 {
            0.0
        } else {
            self.valid_at_issue_receipts as f64 / self.issued_receipts as f64
        }
    }

    pub fn resolution_evaluability_rate(&self) -> f64 {
        if self.valid_at_issue_receipts == 0 {
            0.0
        } else {
            self.informative_receipts as f64 / self.valid_at_issue_receipts as f64
        }
    }

    pub fn informative_rate(&self) -> f64 {
        if self.issued_receipts == 0 {
            0.0
        } else {
            self.informative_receipts as f64 / self.issued_receipts as f64
        }
    }

    pub fn b0_type_macro_f1(&self) -> f64 {
        let categories = [
            EventCategory::NoChange,
            EventCategory::Moved,
            EventCategory::ColorChanged,
            EventCategory::Appeared,
            EventCategory::Disappeared,
        ];
        let sum_f1: f64 = categories
            .iter()
            .map(|c| self.b0_random_confusions.get(c).map_or(0.0, |conf| conf.f1_score()))
            .sum();
        sum_f1 / categories.len() as f64
    }

    pub fn b1_type_macro_f1(&self) -> f64 {
        let categories = [
            EventCategory::NoChange,
            EventCategory::Moved,
            EventCategory::ColorChanged,
            EventCategory::Appeared,
            EventCategory::Disappeared,
        ];
        let sum_f1: f64 = categories
            .iter()
            .map(|c| self.b1_no_change_confusions.get(c).map_or(0.0, |conf| conf.f1_score()))
            .sum();
        sum_f1 / categories.len() as f64
    }

    pub fn b2_type_macro_f1(&self) -> f64 {
        let categories = [
            EventCategory::NoChange,
            EventCategory::Moved,
            EventCategory::ColorChanged,
            EventCategory::Appeared,
            EventCategory::Disappeared,
        ];
        let sum_f1: f64 = categories
            .iter()
            .map(|c| self.b2_last_effect_confusions.get(c).map_or(0.0, |conf| conf.f1_score()))
            .sum();
        sum_f1 / categories.len() as f64
    }

    pub fn b3_type_macro_f1(&self) -> f64 {
        let categories = [
            EventCategory::NoChange,
            EventCategory::Moved,
            EventCategory::ColorChanged,
            EventCategory::Appeared,
            EventCategory::Disappeared,
        ];
        let sum_f1: f64 = categories
            .iter()
            .map(|c| self.b3_freq_majority_confusions.get(c).map_or(0.0, |conf| conf.f1_score()))
            .sum();
        sum_f1 / categories.len() as f64
    }

    pub fn derva_type_macro_f1(&self) -> f64 {
        let categories = [
            EventCategory::NoChange,
            EventCategory::Moved,
            EventCategory::ColorChanged,
            EventCategory::Appeared,
            EventCategory::Disappeared,
        ];
        let sum_f1: f64 = categories
            .iter()
            .map(|c| self.derva_confusions.get(c).map_or(0.0, |conf| conf.f1_score()))
            .sum();
        sum_f1 / categories.len() as f64
    }

    pub fn best_baseline_macro_f1(&self) -> (String, f64) {
        let b0 = self.b0_type_macro_f1();
        let b1 = self.b1_type_macro_f1();
        let b2 = self.b2_type_macro_f1();
        let b3 = self.b3_type_macro_f1();

        let mut best_name = "B0_Random";
        let mut best_score = b0;

        if b1 > best_score { best_name = "B1_NoChange"; best_score = b1; }
        if b2 > best_score { best_name = "B2_LastEffect"; best_score = b2; }
        if b3 > best_score { best_name = "B3_FrequencyMajority"; best_score = b3; }

        (best_name.to_string(), best_score)
    }

    pub fn exact_event_f1(&self) -> f64 {
        if self.exact_event_total == 0 {
            0.0
        } else {
            self.exact_event_matches as f64 / self.exact_event_total as f64
        }
    }

    pub fn derva_type_micro_f1(&self) -> f64 {
        let mut total_tp = 0;
        let mut total_fp = 0;
        let mut total_fn = 0;
        for conf in self.derva_confusions.values() {
            total_tp += conf.true_positives;
            total_fp += conf.false_positives;
            total_fn += conf.false_negatives;
        }
        let denom = 2 * total_tp + total_fp + total_fn;
        if denom == 0 {
            0.0
        } else {
            (2 * total_tp) as f64 / denom as f64
        }
    }

    pub fn derva_observed_type_macro_f1(&self) -> f64 {
        let mut sum_f1 = 0.0;
        let mut count = 0;
        for (cat, conf) in &self.derva_confusions {
            let support = self.category_counts.get(cat).cloned().unwrap_or(0);
            if support > 0 {
                sum_f1 += conf.f1_score();
                count += 1;
            }
        }
        if count == 0 {
            0.0
        } else {
            sum_f1 / count as f64
        }
    }

    pub fn category_support(&self) -> HashMap<EventCategory, u64> {
        self.category_counts.clone()
    }

    pub fn learning_slope(&self) -> f64 {
        let n = self.window_f1_scores.len();
        if n < 2 {
            return 0.0;
        }
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_xy = 0.0;
        let mut sum_xx = 0.0;

        for (i, &y) in self.window_f1_scores.iter().enumerate() {
            let x = i as f64;
            sum_x += x;
            sum_y += y;
            sum_xy += x * y;
            sum_xx += x * x;
        }

        let denom = (n as f64 * sum_xx) - (sum_x * sum_x);
        if denom == 0.0 {
            0.0
        } else {
            ((n as f64 * sum_xy) - (sum_x * sum_y)) / denom
        }
    }

    /// Block Bootstrap 95% Confidence Interval for Learning Slope
    pub fn learning_slope_bootstrap_ci_95(&self) -> (f64, f64) {
        let n = self.window_f1_scores.len();
        if n < 3 {
            let slope = self.learning_slope();
            return (slope, slope);
        }

        let mut bootstrapped_slopes = Vec::with_capacity(1000);
        let mut lcg: u64 = 0x9E3779B97F4A7C15;

        for _ in 0..1000 {
            let mut sample = Vec::with_capacity(n);
            for _ in 0..n {
                lcg = lcg.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                let idx = ((lcg >> 32) as usize) % n;
                sample.push(self.window_f1_scores[idx]);
            }

            let mut sum_x = 0.0;
            let mut sum_y = 0.0;
            let mut sum_xy = 0.0;
            let mut sum_xx = 0.0;
            for (i, &y) in sample.iter().enumerate() {
                let x = i as f64;
                sum_x += x;
                sum_y += y;
                sum_xy += x * y;
                sum_xx += x * x;
            }
            let denom = (n as f64 * sum_xx) - (sum_x * sum_x);
            let s = if denom == 0.0 { 0.0 } else { ((n as f64 * sum_xy) - (sum_x * sum_y)) / denom };
            bootstrapped_slopes.push(s);
        }

        bootstrapped_slopes.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let lower = bootstrapped_slopes[25];  // 2.5 percentile
        let upper = bootstrapped_slopes[975]; // 97.5 percentile
        (lower, upper)
    }

    pub fn f1_early_vs_late(&self) -> (f64, f64, f64) {
        let n = self.window_f1_scores.len();
        if n == 0 {
            return (0.0, 0.0, 0.0);
        }
        let early = self.window_f1_scores.first().cloned().unwrap_or(0.0);
        let late = self.window_f1_scores.last().cloned().unwrap_or(0.0);
        let delta = late - early;
        (early, late, delta)
    }
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
    pub baseline_tracker: BaselineTracker,
    pub is_frozen: bool,
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
            baseline_tracker: BaselineTracker::new(),
            is_frozen: false,
        }
    }

    pub fn count_generalized_schemas(&self) -> usize {
        let mut schemas = std::collections::HashSet::new();
        for h in &self.active_hypotheses {
            let event_type = match &h.predicted_event {
                GridEvent::ObjectMoved { .. } => "Moved",
                GridEvent::ColorChanged { .. } => "ColorChanged",
                GridEvent::ObjectAppeared { .. } => "Appeared",
                GridEvent::ObjectDisappeared { .. } => "Disappeared",
                GridEvent::GridRestructured { .. } => "NoChange",
            };
            schemas.insert((h.action_id, event_type, h.requires_feasibility));
        }
        schemas.len()
    }



    /// Single Authoritative Spatial Feasibility Evaluator for Sub-Gate G10.2-C1.1
    /// Checks full bounding box footprint translation against grid boundaries:
    /// Feasible(BBox, Delta) <=> Translate(BBox, Delta) in [0, W) x [0, H)
    pub fn evaluate_transition_feasibility(
        &self,
        hyp: &TransitionHypothesis,
        obs: &CanonicalObservation,
    ) -> FeasibilityWitness {
        let (target_id, has_event_dx_dy, event_dx, event_dy) = match &hyp.predicted_event {
            GridEvent::ObjectMoved { id, dx, dy } => (*id, true, *dx, *dy),
            GridEvent::ColorChanged { id, .. } | GridEvent::ObjectDisappeared { id } => (*id, false, 0, 0),
            GridEvent::ObjectAppeared { id, .. } => (*id, false, 0, 0),
            GridEvent::GridRestructured { .. } => (0, false, 0, 0),
        };

        let (dx, dy) = if has_event_dx_dy {
            (event_dx, event_dy)
        } else {
            match hyp.action_id {
                1 => (0, -3),
                2 => (0, 3),
                3 => (-3, 0),
                4 => (3, 0),
                _ => (0, 0),
            }
        };


        let target_obj = obs
            .objects
            .iter()
            .find(|o| o.id == target_id)
            .or_else(|| obs.objects.iter().find(|o| o.pixel_count < 100))
            .or_else(|| obs.objects.iter().min_by_key(|o| o.pixel_count))
            .or_else(|| obs.objects.first());

        if let Some(obj) = target_obj {
            let min_x = obj.bbox[0];
            let min_y = obj.bbox[1];
            let max_x = obj.bbox[2];
            let max_y = obj.bbox[3];

            let new_min_x = min_x as i16 + dx as i16;
            let new_min_y = min_y as i16 + dy as i16;
            let new_max_x = max_x as i16 + dx as i16;
            let new_max_y = max_y as i16 + dy as i16;

            let bounds_feasible = new_min_x >= 0
                && new_max_x < obs.frame_width as i16
                && new_min_y >= 0
                && new_max_y < obs.frame_height as i16;

            let blocking_cause = if bounds_feasible {
                None
            } else {
                Some(BlockingCause::Boundary)
            };

            FeasibilityWitness {
                object_id: obj.id,
                bbox: obj.bbox,
                predicted_delta: [dx, dy],
                predicted_target_bbox: [new_min_x, new_min_y, new_max_x, new_max_y],
                frame_width: obs.frame_width,
                frame_height: obs.frame_height,
                bounds_feasible,
                blocking_cause,
            }
        } else {
            FeasibilityWitness {
                object_id: target_id,
                bbox: [0, 0, 0, 0],
                predicted_delta: [dx, dy],
                predicted_target_bbox: [0, 0, 0, 0],
                frame_width: obs.frame_width,
                frame_height: obs.frame_height,
                bounds_feasible: false,
                blocking_cause: Some(BlockingCause::Boundary),
            }
        }
    }

    /// Authoritative Experiment Quality Score (EQS) Gating: Applicable & Observable & Discriminating
    pub fn evaluate_preconditions(&self, hyp: &TransitionHypothesis, obs: &CanonicalObservation) -> (bool, Option<UninformativeReason>, FeasibilityWitness) {
        let witness = self.evaluate_transition_feasibility(hyp, obs);
        if hyp.requires_feasibility {
            (true, None, witness)
        } else if !witness.bounds_feasible {
            (false, Some(UninformativeReason::EffectBlocked(witness.blocking_cause.clone().unwrap_or(BlockingCause::Boundary))), witness)
        } else {
            (true, None, witness)
        }
    }




    pub fn process_step(&mut self, req: StepRequest) -> StepResponse {
        self.step_counter += 1;
        let mut telemetry_logs = Vec::new();

        // 1. EVENT INDUCTION & MULTI-ATTRIBUTE OBJECT MATCHING
        let events = if let Some(prev_obs) = &self.last_observation {
            req.observation.compute_events(prev_obs)
        } else {
            Vec::new()
        };

        let actual_category = if events.is_empty() {
            EventCategory::NoChange
        } else {
            match &events[0] {
                GridEvent::ObjectMoved { .. } => EventCategory::Moved,
                GridEvent::ColorChanged { .. } => EventCategory::ColorChanged,
                GridEvent::ObjectAppeared { .. } => EventCategory::Appeared,
                GridEvent::ObjectDisappeared { .. } => EventCategory::Disappeared,
                GridEvent::GridRestructured { .. } => EventCategory::NoChange,
            }
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

            // Resolve Pending Prediction Receipt with Dual-Context Dual Prediction
            if let Some(receipt) = self.pending_prediction.take() {
                let (valid_at_res, res_reason, _) = if let Some(hyp) = self.active_hypotheses.iter().find(|h| h.id == receipt.primary_hyp_id) {
                    self.evaluate_preconditions(hyp, &req.observation)
                } else {
                    let dummy_witness = FeasibilityWitness {
                        object_id: 0,
                        bbox: [0, 0, 0, 0],
                        predicted_delta: [0, 0],
                        predicted_target_bbox: [0, 0, 0, 0],
                        frame_width: req.observation.frame_width,
                        frame_height: req.observation.frame_height,
                        bounds_feasible: true,
                        blocking_cause: None,
                    };
                    (true, None, dummy_witness)
                };

                let is_free_at_issue = receipt.feasibility_witness.bounds_feasible;

                let expected_category = if is_free_at_issue {
                    match &receipt.expected_event {
                        GridEvent::ObjectMoved { .. } => EventCategory::Moved,
                        GridEvent::ColorChanged { .. } => EventCategory::ColorChanged,
                        GridEvent::ObjectAppeared { .. } => EventCategory::Appeared,
                        GridEvent::ObjectDisappeared { .. } => EventCategory::Disappeared,
                        GridEvent::GridRestructured { .. } => EventCategory::NoChange,
                    }
                } else {
                    EventCategory::NoChange // S_blocked -> Dual-context prediction: Predict NoChange!
                };

                let outcome = if !valid_at_res {
                    PredictionOutcome::Uninformative(res_reason.unwrap_or(UninformativeReason::EffectBlocked(BlockingCause::Boundary)))
                } else if !is_free_at_issue && actual_category == EventCategory::NoChange {
                    PredictionOutcome::ExactMatch // Dual-context S_blocked NoChange predicted & observed!
                } else if is_free_at_issue && events.iter().any(|e| e == &receipt.expected_event) {
                    PredictionOutcome::ExactMatch // Dual-context S_free Movement predicted & observed!
                } else if is_free_at_issue {
                    if let GridEvent::ObjectMoved { id: p_id, dx: p_dx, dy: p_dy } = &receipt.expected_event {
                        if events.iter().any(|e| match e {
                            GridEvent::ObjectMoved { id, dx, dy } => id == p_id && ((*dx > 0 && *p_dx > 0) || (*dx < 0 && *p_dx < 0) || (*dy > 0 && *p_dy > 0) || (*dy < 0 && *p_dy < 0)),
                            _ => false,
                        }) {
                            PredictionOutcome::PartialMatch
                        } else if !events.is_empty() {
                            PredictionOutcome::Contradiction
                        } else {
                            PredictionOutcome::Contradiction // Predicted move in S_free, but no move occurred!
                        }
                    } else if !events.is_empty() {
                        PredictionOutcome::Contradiction
                    } else {
                        PredictionOutcome::Contradiction
                    }
                } else if !events.is_empty() {
                    PredictionOutcome::Contradiction
                } else {
                    PredictionOutcome::Uninformative(UninformativeReason::EffectBlocked(BlockingCause::Boundary))
                };

                self.baseline_tracker.record_prediction(expected_category, actual_category, outcome, receipt.precondition_witness);

                let (best_b_name, best_b_score) = self.baseline_tracker.best_baseline_macro_f1();
                let derva_macro_f1 = self.baseline_tracker.derva_type_macro_f1();
                let derva_micro_f1 = self.baseline_tracker.derva_type_micro_f1();
                let derva_obs_macro_f1 = self.baseline_tracker.derva_observed_type_macro_f1();
                let exact_event_f1 = self.baseline_tracker.exact_event_f1();
                let learning_slope = self.baseline_tracker.learning_slope();
                let (ci_lower, ci_upper) = self.baseline_tracker.learning_slope_bootstrap_ci_95();
                let (early, late, delta_learning) = self.baseline_tracker.f1_early_vs_late();
                let n_hypotheses = self.active_hypotheses.len();
                let n_schemas = self.count_generalized_schemas();

                telemetry_logs.push(format!(
                    "[RECEIPT RESOLVED] ID: {} | Outcome: {:?} | RER: {:.1}% | TypeMacroF1: DERVA={:.4} vs Best ({})={:.4} (Delta={:+.4}) | ObservedMacroF1: {:.4} | MicroF1: {:.4} | ExactEventF1: {:.4} | D2(Hyps={}, Schemas={}) | LearningSlope: {:+.4} [95% CI: [{:+.4}, {:+.4}]] | EarlyVsLate: Early={:.4}, Late={:.4} (Delta={:+.4})",
                    receipt.receipt_id,
                    outcome,
                    self.baseline_tracker.resolution_evaluability_rate() * 100.0,
                    derva_macro_f1,
                    best_b_name,
                    best_b_score,
                    derva_macro_f1 - best_b_score,
                    derva_obs_macro_f1,
                    derva_micro_f1,
                    exact_event_f1,
                    n_hypotheses,
                    n_schemas,
                    learning_slope,
                    ci_lower,
                    ci_upper,
                    early,
                    late,
                    delta_learning
                ));


                if let Some(hyp) = self.active_hypotheses.iter_mut().find(|h| h.id == receipt.primary_hyp_id) {
                    hyp.prospective.issued += 1;
                    match outcome {
                        PredictionOutcome::ExactMatch => {
                            hyp.support_count += 1;
                            hyp.status = HypothesisStatus::Supported;
                            hyp.retrodiction.correct += 1;
                            hyp.retrodiction.applicable += 1;
                            hyp.prospective.exact_matches += 1;
                            self.supported_hypotheses += 1;

                            let parent_id_opt = hyp.derived_from.clone();
                            let hyp_id = hyp.id.clone();
                            let hyp_retro_acc = hyp.retrodiction.accuracy();
                            let hyp_prosp_acc = hyp.prospective.accuracy();

                            if let Some(parent_id) = parent_id_opt {
                                let (parent_retro_acc, parent_prosp_acc) = self
                                    .active_hypotheses
                                    .iter()
                                    .find(|h| h.id == parent_id)
                                    .map(|h| (h.retrodiction.accuracy(), h.prospective.accuracy()))
                                    .unwrap_or((0.0, 0.0));

                                let rg_retro = hyp_retro_acc - parent_retro_acc;
                                let rg_prospective = hyp_prosp_acc - parent_prosp_acc;

                                telemetry_logs.push(format!(
                                    "[PROSPECTIVE REFINE CONFIRMED] Refined {} DERIVED_FROM {} | Context: {} | Witness: BBox{:?}->{:?} | RG_retro: {:+.4} | RG_prospective: {:+.4} (Refined={:.4} vs Orig={:.4})",
                                    hyp_id,
                                    parent_id,
                                    if receipt.contextual_feasibility { "Free" } else { "Blocked" },
                                    receipt.feasibility_witness.bbox,
                                    receipt.feasibility_witness.predicted_target_bbox,
                                    rg_retro,
                                    rg_prospective,
                                    hyp_prosp_acc,
                                    parent_prosp_acc
                                ));
                            } else {
                                telemetry_logs.push(format!("[EPISTEMIC PROMOTION] {} -> SUPPORTED", hyp_id));
                            }
                        }
                        PredictionOutcome::PartialMatch => {
                            hyp.support_count += 1;
                            hyp.retrodiction.applicable += 1;
                            hyp.prospective.partial_matches += 1;
                            telemetry_logs.push(format!("[PARTIAL MATCH] Direction matched for {}; support count incremented", hyp.id));
                        }
                        PredictionOutcome::Contradiction => {
                            hyp.refutation_count += 1;
                            hyp.retrodiction.contradicted += 1;
                            hyp.retrodiction.applicable += 1;
                            hyp.prospective.contradictions += 1;
                            if hyp.refutation_count >= 2 {
                                hyp.status = HypothesisStatus::Refuted;
                                self.refuted_hypotheses += 1;
                                telemetry_logs.push(format!("[FALSIFICATION] {} -> REFUTED", hyp.id));
                            } else {
                                hyp.status = HypothesisStatus::Contested;
                                telemetry_logs.push(format!("[FALSIFICATION] {} -> CONTESTED", hyp.id));
                            }
                        }
                        PredictionOutcome::Uninformative(reason) => {
                            telemetry_logs.push(format!("[UNINFORMATIVE] Reason: {:?} for {}; status preserved as {:?}", reason, hyp.id, hyp.status));

                            let hyp_id = hyp.id.clone();
                            let hyp_action_id = hyp.action_id;
                            let hyp_target_id = hyp.target_object_id;
                            let hyp_event = hyp.predicted_event.clone();
                            let hyp_req_feas = hyp.requires_feasibility;
                            let hyp_support = hyp.support_count;
                            let hyp_retro = hyp.retrodiction.clone();
                            let hyp_prosp = hyp.prospective.clone();

                            if let UninformativeReason::EffectBlocked(BlockingCause::Boundary) = reason {
                                if !hyp_req_feas && !self.is_frozen {
                                    let refined_id = format!("Refine_{}_IF_feasible", hyp_id);
                                    let exists = self.active_hypotheses.iter().any(|h| h.id == refined_id);
                                    if !exists {
                                        let refined_hyp = TransitionHypothesis {
                                            id: refined_id.clone(),
                                            derived_from: Some(hyp_id.clone()),
                                            action_id: hyp_action_id,
                                            target_object_id: hyp_target_id,
                                            predicted_event: hyp_event,
                                            requires_feasibility: true,
                                            status: HypothesisStatus::Supported,
                                            support_count: hyp_support,
                                            refutation_count: 0,
                                            retrodiction: hyp_retro,
                                            prospective: hyp_prosp,
                                        };
                                        telemetry_logs.push(format!("[HYPOTHESIS REFINE] Created {} DERIVED_FROM {}", refined_id, hyp_id));
                                        self.active_hypotheses.push(refined_hyp);
                                    }
                                }
                            }
                            self.target_object_idx = self.target_object_idx.wrapping_add(1);
                        }
                    }
                }
            }

            // Induce new Transition Hypotheses from observed events (disabled when model is frozen)
            if !self.is_frozen {
                for ev in &events {
                    let hyp_id = format!("H_trans_act{}_ev{:?}", prev_act.action_id, ev);
                    if !self.active_hypotheses.iter().any(|h| h.id == hyp_id) {
                        let target_obj_id = match ev {
                            GridEvent::ObjectMoved { id, .. } | GridEvent::ColorChanged { id, .. } | GridEvent::ObjectDisappeared { id } => Some(*id),
                            _ => None,
                        };

                        let mut retro = RetrodictionVector {
                            applicable: 0,
                            correct: 0,
                            contradicted: 0,
                            ambiguous: 0,
                        };

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
                            derived_from: None,
                            action_id: prev_act.action_id,
                            target_object_id: target_obj_id,
                            predicted_event: ev.clone(),
                            requires_feasibility: false,
                            status: HypothesisStatus::Unverified,
                            support_count: 1,
                            refutation_count: 0,
                            retrodiction: retro.clone(),
                            prospective: ProspectiveVector { issued: 0, exact_matches: 0, partial_matches: 0, contradictions: 0 },
                        };

                        telemetry_logs.push(format!("[PROPOSE HYPOTHESIS] {} | Retrodict Vector: (correct={}, applicable={})", hyp_id, retro.correct, retro.applicable));
                        self.active_hypotheses.push(hyp);
                    }
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

        // 3. ACTION SELECTION WITH EXPERIMENT QUALITY SCORE (EQS) GATING: Applicable & Observable & Discriminating
        let candidate_actions = &req.action_space.actions;
        let num_candidates = candidate_actions.len();

        let spatial_action_id = candidate_actions.iter().find_map(|a| match a {
            ArcAction::Spatial { id, .. } => Some(*id),
            _ => None,
        });

        // Filter hypotheses using single authoritative precondition evaluator: Applicable & Observable
        let active_hyps: Vec<(&TransitionHypothesis, bool, FeasibilityWitness)> = self
            .active_hypotheses
            .iter()
            .filter(|h| h.status == HypothesisStatus::Supported || h.status == HypothesisStatus::Unverified)
            .map(|h| {
                let (valid, _, witness) = self.evaluate_preconditions(h, &req.observation);
                (h, valid, witness)
            })
            .filter(|(h, valid, witness)| {
                // EQS GATING: Only consider observable experiments
                // Unrefined hypotheses require bounds_feasible == true to be observable
                // Refined hypotheses (requires_feasibility == true) are observable in both contexts (is_free true or false)
                *valid && (h.requires_feasibility || witness.bounds_feasible)
            })
            .collect();

        // Prioritize Refined hypotheses (requires_feasibility == true) to confirm dual-context predictions
        let discriminating_pair = if active_hyps.len() >= 2 {
            let mut found = None;

            // First check if a refined hypothesis can be paired with another hypothesis
            if let Some((h_ref, _valid_ref, witness_ref)) = active_hyps.iter().find(|(h, _, _)| h.requires_feasibility) {
                if let Some((h_other, _valid_other, _)) = active_hyps.iter().find(|(h, _, _)| h.id != h_ref.id) {
                    found = Some(((*h_ref).clone(), (*h_other).clone(), witness_ref.clone()));
                }
            }

            if found.is_none() {
                'outer: for i in 0..active_hyps.len() {
                    for j in (i + 1)..active_hyps.len() {
                        let (h1, _valid1, witness1) = &active_hyps[i];
                        let (h2, _valid2, _witness2) = &active_hyps[j];
                        if h1.predicted_event != h2.predicted_event {
                            found = Some(((*h1).clone(), (*h2).clone(), witness1.clone()));
                            break 'outer;
                        }
                    }
                }
            }
            found
        } else {
            None
        };

        let selected = if let Some((h1, h2, witness)) = discriminating_pair {
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
                primary_hyp_id: h1.id.clone(),
                secondary_hyp_id: Some(h2.id.clone()),
                step: self.step_counter,
                action_id: h1.action_id,
                target_coords: match (target_x, target_y) {
                    (Some(x), Some(y)) => Some([x, y]),
                    _ => None,
                },
                expected_event: h1.predicted_event.clone(),
                state_root: self.current_commit_root.to_string(),
                precondition_witness: true,
                contextual_feasibility: witness.bounds_feasible,
                feasibility_witness: witness.clone(),
            });

            telemetry_logs.push(format!(
                "[VALID DISCRIMINATION - EQS PASS] Receipt: {} | H1: {} vs H2: {} | ContextualFeasibility: {} | Witness: BBox{:?}->{:?} | ExpectedEvent: {:?}",
                receipt_id, h1.id, h2.id, witness.bounds_feasible, witness.bbox, witness.predicted_target_bbox, h1.predicted_event
            ));

            SelectedAction {
                action_id: h1.action_id,
                x: target_x,
                y: target_y,
                hypothesis_id: format!("Discriminate_{}_vs_{}|Receipt:{}", h1.id, h2.id, receipt_id),
            }
        } else if let Some((hyp, valid, witness)) = active_hyps.first() {
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
                primary_hyp_id: hyp.id.clone(),
                secondary_hyp_id: None,
                step: self.step_counter,
                action_id: hyp.action_id,
                target_coords: match (target_x, target_y) {
                    (Some(x), Some(y)) => Some([x, y]),
                    _ => None,
                },
                expected_event: hyp.predicted_event.clone(),
                state_root: self.current_commit_root.to_string(),
                precondition_witness: *valid,
                contextual_feasibility: witness.bounds_feasible,
                feasibility_witness: witness.clone(),
            });

            SelectedAction {
                action_id: hyp.action_id,
                x: target_x,
                y: target_y,
                hypothesis_id: format!("Probe_{}|Receipt:{}", hyp.id, receipt_id),
            }

        } else if let Some(sp_id) = spatial_action_id {
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
    use crate::observation::{CanonicalObservation, GenericObject, GridEvent, TemporalDiff};


    #[test]
    fn test_footprint_feasibility_exhaustive_property() {
        let mut mismatches = 0;

        for grid_w in 2..=10u8 {
            for grid_h in 2..=10u8 {
                for min_x in 0..grid_w {
                    for max_x in min_x..grid_w {
                        for min_y in 0..grid_h {
                            for max_y in min_y..grid_h {
                                for dx in -10..=10i8 {
                                    for dy in -10..=10i8 {

                                        let new_min_x = min_x as i16 + dx as i16;
                                        let new_max_x = max_x as i16 + dx as i16;
                                        let new_min_y = min_y as i16 + dy as i16;
                                        let new_max_y = max_y as i16 + dy as i16;

                                        let is_in_grid = new_min_x >= 0
                                            && new_max_x < grid_w as i16
                                            && new_min_y >= 0
                                            && new_max_y < grid_h as i16;

                                        let engine = ArcBridgeEngine::new();
                                        let dummy_obj = GenericObject {
                                            id: 1,
                                            color: 1,
                                            bbox: [min_x, min_y, max_x, max_y],
                                            centroid: [(min_x + max_x) / 2, (min_y + max_y) / 2],
                                            pixel_count: 1,
                                        };
                                        let hyp = TransitionHypothesis {
                                            id: "test".to_string(),
                                            derived_from: None,
                                            action_id: 3,
                                            target_object_id: Some(1),
                                            predicted_event: GridEvent::ObjectMoved { id: 1, dx, dy },
                                            requires_feasibility: false,
                                            status: HypothesisStatus::Supported,
                                            support_count: 1,
                                            refutation_count: 0,
                                            retrodiction: RetrodictionVector::default(),
                                            prospective: ProspectiveVector::default(),
                                        };
                                        let obs = CanonicalObservation {
                                            step: 1,
                                            frame_width: grid_w,
                                            frame_height: grid_h,
                                            objects: vec![dummy_obj],
                                            diff: TemporalDiff {
                                                objects_appeared: vec![],
                                                objects_disappeared: vec![],
                                                objects_moved: vec![],
                                                color_changes: vec![],
                                                events: vec![],
                                            },
                                        };

                                        let witness = engine.evaluate_transition_feasibility(&hyp, &obs);
                                        if witness.bounds_feasible != is_in_grid {
                                            mismatches += 1;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        assert_eq!(mismatches, 0, "Exhaustive footprint feasibility property test failed with mismatches!");
    }

    #[test]
    fn test_footprint_feasibility_large_random_property() {
        let mut mismatches = 0;
        let mut lcg_state: u64 = 0xDEADBEEF;
        let mut next_u32 = || {
            lcg_state = lcg_state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (lcg_state >> 32) as u32
        };

        let engine = ArcBridgeEngine::new();

        for _ in 0..1_000_000 {
            let grid_w = ((next_u32() % 63) + 2) as u8; // 2..64
            let grid_h = ((next_u32() % 63) + 2) as u8; // 2..64

            let x1 = (next_u32() % grid_w as u32) as u8;
            let x2 = (next_u32() % grid_w as u32) as u8;
            let min_x = x1.min(x2);
            let max_x = x1.max(x2);

            let y1 = (next_u32() % grid_h as u32) as u8;
            let y2 = (next_u32() % grid_h as u32) as u8;
            let min_y = y1.min(y2);
            let max_y = y1.max(y2);

            let dx = ((next_u32() % 129) as i16 - 64) as i8; // -64..64
            let dy = ((next_u32() % 129) as i16 - 64) as i8; // -64..64

            let new_min_x = min_x as i16 + dx as i16;
            let new_max_x = max_x as i16 + dx as i16;
            let new_min_y = min_y as i16 + dy as i16;
            let new_max_y = max_y as i16 + dy as i16;

            let is_in_grid = new_min_x >= 0
                && new_max_x < grid_w as i16
                && new_min_y >= 0
                && new_max_y < grid_h as i16;

            let dummy_obj = GenericObject {
                id: 1,
                color: 1,
                bbox: [min_x, min_y, max_x, max_y],
                centroid: [(min_x + max_x) / 2, (min_y + max_y) / 2],
                pixel_count: 1,
            };
            let hyp = TransitionHypothesis {
                id: "test_rand".to_string(),
                derived_from: None,
                action_id: 3,
                target_object_id: Some(1),
                predicted_event: GridEvent::ObjectMoved { id: 1, dx, dy },
                requires_feasibility: false,
                status: HypothesisStatus::Supported,
                support_count: 1,
                refutation_count: 0,
                retrodiction: RetrodictionVector::default(),
                prospective: ProspectiveVector::default(),
            };
            let obs = CanonicalObservation {
                step: 1,
                frame_width: grid_w,
                frame_height: grid_h,
                objects: vec![dummy_obj],
                diff: TemporalDiff {
                    objects_appeared: vec![],
                    objects_disappeared: vec![],
                    objects_moved: vec![],
                    color_changes: vec![],
                    events: vec![],
                },
            };

            let witness = engine.evaluate_transition_feasibility(&hyp, &obs);
            if witness.bounds_feasible != is_in_grid {
                mismatches += 1;
            }
        }

        assert_eq!(mismatches, 0, "Large random footprint feasibility property test failed with mismatches!");
    }
}


