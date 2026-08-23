// AUDIT-LENSES: Ada Lovelace, Alan Turing, Donald Knuth
// INVARIANT: Generic Domain-Agnostic Perception model for ARC-AGI-3 frames. Converts visual grids into canonical ORIDs & spatial distinction properties with Multi-Attribute Visual Continuity Object Matching across frame transitions.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GenericObject {
    pub id: u32,
    pub color: u8,
    pub bbox: [u8; 4], // [min_x, min_y, max_x, max_y]
    pub centroid: [u8; 2],
    pub pixel_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GridEvent {
    ObjectMoved { id: u32, dx: i8, dy: i8 },
    ObjectAppeared { id: u32, color: u8, centroid: [u8; 2] },
    ObjectDisappeared { id: u32 },
    ColorChanged { id: u32, old_color: u8, new_color: u8 },
    GridRestructured { pixel_delta: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalDiff {
    pub objects_appeared: Vec<u32>,
    pub objects_disappeared: Vec<u32>,
    pub objects_moved: Vec<u32>,
    pub color_changes: Vec<u32>,
    pub events: Vec<GridEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalObservation {
    pub step: u64,
    pub frame_width: u8,
    pub frame_height: u8,
    pub objects: Vec<GenericObject>,
    pub diff: TemporalDiff,
}

impl CanonicalObservation {
    /// Computes multi-attribute visual continuity matching between previous and current observations.
    /// Preserves persistent object identities across spatial moves and color transitions.
    pub fn compute_events(&self, prev: &CanonicalObservation) -> Vec<GridEvent> {
        let mut events = Vec::new();
        let mut matched_pairs: Vec<(u32, u32)> = Vec::new(); // (prev_id, curr_id)
        let mut used_curr_indices = std::collections::HashSet::new();

        // 1. Multi-Attribute Similarity Matching (Color, Pixel Area, Spatial Proximity)
        for prev_obj in &prev.objects {
            let mut best_match: Option<(usize, f64)> = None;

            for (c_idx, curr_obj) in self.objects.iter().enumerate() {
                if used_curr_indices.contains(&c_idx) {
                    continue;
                }

                // Feature similarities
                let color_sim = if prev_obj.color == curr_obj.color { 1.0 } else { 0.3 };

                let max_pixels = prev_obj.pixel_count.max(curr_obj.pixel_count).max(1) as f64;
                let pixel_diff = (prev_obj.pixel_count as i64 - curr_obj.pixel_count as i64).abs() as f64;
                let area_sim = (1.0 - (pixel_diff / max_pixels)).max(0.0);

                let dx = (prev_obj.centroid[0] as f64 - curr_obj.centroid[0] as f64).abs();
                let dy = (prev_obj.centroid[1] as f64 - curr_obj.centroid[1] as f64).abs();
                let dist = (dx * dx + dy * dy).sqrt();
                let pos_sim = (1.0 - (dist / 30.0)).max(0.0);

                let total_score = 0.35 * color_sim + 0.35 * area_sim + 0.30 * pos_sim;

                if total_score >= 0.55 {
                    if let Some((_, best_score)) = best_match {
                        if total_score > best_score {
                            best_match = Some((c_idx, total_score));
                        }
                    } else {
                        best_match = Some((c_idx, total_score));
                    }
                }
            }

            if let Some((c_idx, _)) = best_match {
                used_curr_indices.insert(c_idx);
                matched_pairs.push((prev_obj.id, self.objects[c_idx].id));

                let curr_obj = &self.objects[c_idx];
                let dx = curr_obj.centroid[0] as i16 - prev_obj.centroid[0] as i16;
                let dy = curr_obj.centroid[1] as i16 - prev_obj.centroid[1] as i16;

                if dx != 0 || dy != 0 {
                    events.push(GridEvent::ObjectMoved {
                        id: prev_obj.id, // Retain persistent temporal ID!
                        dx: dx.clamp(-128, 127) as i8,
                        dy: dy.clamp(-128, 127) as i8,
                    });
                }

                if curr_obj.color != prev_obj.color {
                    events.push(GridEvent::ColorChanged {
                        id: prev_obj.id, // Retain persistent temporal ID!
                        old_color: prev_obj.color,
                        new_color: curr_obj.color,
                    });
                }
            } else {
                // Object disappeared if no visual match found above threshold
                events.push(GridEvent::ObjectDisappeared { id: prev_obj.id });
            }
        }

        // 2. Unmatched Current Objects -> ObjectAppeared
        for (c_idx, curr_obj) in self.objects.iter().enumerate() {
            if !used_curr_indices.contains(&c_idx) {
                events.push(GridEvent::ObjectAppeared {
                    id: curr_obj.id,
                    color: curr_obj.color,
                    centroid: curr_obj.centroid,
                });
            }
        }

        events
    }
}
