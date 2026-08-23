// AUDIT-LENSES: Ada Lovelace, Alan Turing, Donald Knuth
// INVARIANT: Generic Domain-Agnostic Perception model for ARC-AGI-3 frames. Converts visual grids into canonical ORIDs & spatial distinction properties without game-specific heuristics.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericObject {
    pub id: u32,
    pub color: u8,
    pub bbox: [u8; 4], // [min_x, min_y, max_x, max_y]
    pub centroid: [u8; 2],
    pub pixel_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalDiff {
    pub objects_appeared: Vec<u32>,
    pub objects_disappeared: Vec<u32>,
    pub objects_moved: Vec<u32>,
    pub color_changes: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalObservation {
    pub step: u64,
    pub frame_width: u8,
    pub frame_height: u8,
    pub objects: Vec<GenericObject>,
    pub diff: TemporalDiff,
}
