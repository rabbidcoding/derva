// AUDIT-LENSES: Ada Lovelace, Alan Turing, Donald Knuth
// INVARIANT: Generic Domain-Agnostic Perception model for ARC-AGI-3 frames. Converts visual grids into canonical ORIDs & spatial distinction properties without game-specific heuristics.

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
    pub fn compute_events(&self, prev: &CanonicalObservation) -> Vec<GridEvent> {
        let mut events = Vec::new();

        // 1. Appears / Disappears
        let current_ids: std::collections::HashSet<u32> = self.objects.iter().map(|o| o.id).collect();
        let prev_ids: std::collections::HashSet<u32> = prev.objects.iter().map(|o| o.id).collect();

        for &id in current_ids.difference(&prev_ids) {
            if let Some(obj) = self.objects.iter().find(|o| o.id == id) {
                events.push(GridEvent::ObjectAppeared {
                    id: obj.id,
                    color: obj.color,
                    centroid: obj.centroid,
                });
            }
        }

        for &id in prev_ids.difference(&current_ids) {
            events.push(GridEvent::ObjectDisappeared { id });
        }

        // 2. Moves & Color Changes
        for curr_obj in &self.objects {
            if let Some(prev_obj) = prev.objects.iter().find(|o| o.id == curr_obj.id) {
                let dx = curr_obj.centroid[0] as i16 - prev_obj.centroid[0] as i16;
                let dy = curr_obj.centroid[1] as i16 - prev_obj.centroid[1] as i16;

                if dx != 0 || dy != 0 {
                    events.push(GridEvent::ObjectMoved {
                        id: curr_obj.id,
                        dx: dx.clamp(-128, 127) as i8,
                        dy: dy.clamp(-128, 127) as i8,
                    });
                }

                if curr_obj.color != prev_obj.color {
                    events.push(GridEvent::ColorChanged {
                        id: curr_obj.id,
                        old_color: prev_obj.color,
                        new_color: curr_obj.color,
                    });
                }
            }
        }

        events
    }
}
