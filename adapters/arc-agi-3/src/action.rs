// AUDIT-LENSES: Steve Jobs, John Carmack, Niklaus Wirth
// INVARIANT: Dynamic Action Space representation for ARC-AGI-3 environment. No hardcoded action IDs.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArcActionSpace {
    pub actions: Vec<ArcAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ArcAction {
    Simple {
        id: u8,
    },
    Spatial {
        id: u8,
        x_min: u8,
        x_max: u8,
        y_min: u8,
        y_max: u8,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectedAction {
    pub action_id: u8,
    pub x: Option<u8>,
    pub y: Option<u8>,
    pub hypothesis_id: String,
}
