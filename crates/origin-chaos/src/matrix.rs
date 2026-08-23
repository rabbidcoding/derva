// AUDIT-LENSES: Ken Thompson, Dennis Ritchie, Elon Musk
// INVARIANT: Configuration parser for crash injection matrix.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignConfig {
    pub name: String,
    pub target_crash_points: u64,
    pub seed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjectionTargetsConfig {
    pub wal_header_append: bool,
    pub wal_payload_append: bool,
    pub wal_fsync: bool,
    pub object_store_write: bool,
    pub commit_dag_publication: bool,
    pub root_pointer_update: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureModesConfig {
    pub kill_immediate: f64,
    pub write_truncation: f64,
    pub byte_corruption: f64,
    pub fsync_loss: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrashMatrixConfig {
    pub campaign: CampaignConfig,
    pub injection_targets: InjectionTargetsConfig,
    pub failure_modes: FailureModesConfig,
}

impl CrashMatrixConfig {
    pub fn from_toml_str(s: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(s)
    }

    pub fn default_campaign() -> Self {
        Self {
            campaign: CampaignConfig {
                name: "release_crash_recovery_campaign".into(),
                target_crash_points: 1_000_000,
                seed: 42,
            },
            injection_targets: InjectionTargetsConfig {
                wal_header_append: true,
                wal_payload_append: true,
                wal_fsync: true,
                object_store_write: true,
                commit_dag_publication: true,
                root_pointer_update: true,
            },
            failure_modes: FailureModesConfig {
                kill_immediate: 0.40,
                write_truncation: 0.30,
                byte_corruption: 0.20,
                fsync_loss: 0.10,
            },
        }
    }
}
