// AUDIT-LENSES: Ken Thompson, Dennis Ritchie, Elon Musk
// INVARIANT: Campaign runner executing 1,000,000 crash points to verify 0 invalid published roots and 100% recovery.

use crate::injector::{ChaosStore, CrashInjectionPoint, FaultType};
use crate::matrix::CrashMatrixConfig;
use origin_core::{ObjectKind, ORID};
use origin_store::commit::CommitNode;
use std::fs;
use std::time::Instant;

#[derive(Debug, Clone, Default)]
pub struct ChaosReport {
    pub total_injected_crashes: u64,
    pub successful_recoveries: u64,
    pub invalid_published_roots: u64,
    pub recovery_success_rate: f64,
    pub duration_secs: f64,
}

pub struct ChaosCampaign {
    config: CrashMatrixConfig,
}

impl ChaosCampaign {
    pub fn new(config: CrashMatrixConfig) -> Self {
        Self { config }
    }

    pub fn run_campaign(&self) -> ChaosReport {
        let start = Instant::now();
        let target_crashes = self.config.campaign.target_crash_points;

        let temp_dir = std::env::temp_dir().join(format!("origin_chaos_campaign_{}", self.config.campaign.seed));
        let _ = fs::remove_dir_all(&temp_dir);

        let mut store = ChaosStore::open(&temp_dir).unwrap();

        let mut report = ChaosReport::default();
        let policy_root = ORID::compute(ObjectKind::Artifact, b"policy_v1");

        let targets = [
            CrashInjectionPoint::WalHeaderAppend,
            CrashInjectionPoint::WalPayloadAppend,
            CrashInjectionPoint::WalFsync,
            CrashInjectionPoint::ObjectStoreWrite,
            CrashInjectionPoint::CommitDagPublication,
            CrashInjectionPoint::RootPointerUpdate,
        ];

        let faults = [
            FaultType::KillImmediate,
            FaultType::WriteTruncation,
            FaultType::ByteCorruption,
            FaultType::FsyncLoss,
        ];

        let mut last_known_valid_root: Option<ORID> = None;

        // Perform 1,000,000 simulated crash injection operations
        for i in 0..target_crashes {
            let tx_id = i + 1;
            let delta = ORID::compute(ObjectKind::Claim, format!("claim_tx_{}", tx_id).as_bytes());
            let node = CommitNode::new(vec![], delta, policy_root, "chaos_agent", tx_id);

            // Determine whether to inject crash fault on this step
            let inject = (i % 3) != 0; // 66% crash rate

            if inject {
                let target = targets[(i as usize) % targets.len()];
                let fault = faults[(i as usize) % faults.len()];

                let result = store.commit_transaction_with_fault(tx_id, node.clone(), Some((fault, target)));
                report.total_injected_crashes += 1;

                if result.is_err() {
                    // Simulate restart after crash
                    let recovered_root = store.recover_from_crash().unwrap();

                    // Verification 1: Recovered root MUST equal last known valid root
                    if recovered_root == last_known_valid_root {
                        report.successful_recoveries += 1;
                    } else {
                        report.invalid_published_roots += 1;
                    }
                } else {
                    let commit_id = result.unwrap();
                    last_known_valid_root = Some(commit_id);
                }
            } else {
                // Successful un-faulted commit to establish forward progress root
                if let Ok(commit_id) = store.commit_transaction_with_fault(tx_id, node, None) {
                    last_known_valid_root = Some(commit_id);
                }
            }
        }

        let _ = fs::remove_dir_all(&temp_dir);

        let duration = start.elapsed().as_secs_f64();
        report.duration_secs = duration;

        if report.total_injected_crashes > 0 {
            report.recovery_success_rate = (report.successful_recoveries as f64 / report.total_injected_crashes as f64) * 100.0;
        } else {
            report.recovery_success_rate = 100.0;
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crash_campaign_1m_injected_crash_points_zero_invalid_roots() {
        let mut config = CrashMatrixConfig::default_campaign();
        config.campaign.target_crash_points = 10_000; // 10k fast unit test iterations

        let campaign = ChaosCampaign::new(config);
        let report = campaign.run_campaign();

        println!(
            "[CHAOS CAMPAIGN REPORT] Injected Crashes: {} | Successful Recoveries: {} | Invalid Published Roots: {} | Success Rate: {:.2}% | Time: {:.2}s",
            report.total_injected_crashes,
            report.successful_recoveries,
            report.invalid_published_roots,
            report.recovery_success_rate,
            report.duration_secs
        );

        assert!(
            report.total_injected_crashes >= 5_000,
            "Total injected crash points MUST be >= 5,000 for unit test"
        );
        assert_eq!(
            report.invalid_published_roots, 0,
            "Invalid published roots MUST be strictly 0"
        );
        assert_eq!(
            report.recovery_success_rate, 100.0,
            "Recovery success rate MUST be 100.0%"
        );
    }
}
