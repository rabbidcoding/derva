#![forbid(unsafe_code)]

// AUDIT-LENSES: Steve Jobs, Niklaus Wirth, John Carmack
// INVARIANT: Stable-Region Detector determining JIT/AOT compilation eligibility based on domain guards, hit counts, unresolved obligations, and churn.
// KPI: 0 artifacts compiled with unresolved critical obligations; False-stable rate < 0.1% on churn benchmark; Configurable and versioned hit threshold.

#[derive(Debug, Clone, PartialEq)]
pub struct StabilityConfig {
    pub version: u32,
    pub min_hit_threshold: u64,
    pub max_churn_threshold: f64,
    pub require_domain_guard_stable: bool,
}

impl Default for StabilityConfig {
    fn default() -> Self {
        Self {
            version: 1,
            min_hit_threshold: 100,
            max_churn_threshold: 0.05, // 5% max churn rate
            require_domain_guard_stable: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionMetrics {
    pub region_id: String,
    pub hit_count: u64,
    pub churn_rate: f64,
    pub domain_guard_stable: bool,
    pub unresolved_critical_obligations: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EligibilityResult {
    Eligible {
        region_id: String,
        confidence_score: f64,
        config_version: u32,
    },
    Ineligible {
        reason: &'static str,
        unresolved_critical_count: u64,
    },
}

pub struct StableRegionDetector {
    pub config: StabilityConfig,
}

impl StableRegionDetector {
    pub fn new(config: StabilityConfig) -> Self {
        Self { config }
    }

    /// Evaluates if an OIR region is stable and eligible for compilation
    pub fn evaluate_region(&self, metrics: &RegionMetrics) -> EligibilityResult {
        // 1. Mandatory Invariant: 0 artifacts compiled with unresolved critical obligations
        if metrics.unresolved_critical_obligations > 0 {
            return EligibilityResult::Ineligible {
                reason: "Unresolved critical obligations present",
                unresolved_critical_count: metrics.unresolved_critical_obligations,
            };
        }

        // 2. Check hit count threshold
        if metrics.hit_count < self.config.min_hit_threshold {
            return EligibilityResult::Ineligible {
                reason: "Hit count below minimum threshold",
                unresolved_critical_count: 0,
            };
        }

        // 3. Check churn rate threshold
        if metrics.churn_rate > self.config.max_churn_threshold {
            return EligibilityResult::Ineligible {
                reason: "Churn rate exceeds maximum threshold",
                unresolved_critical_count: 0,
            };
        }

        // 4. Check domain guard stability
        if self.config.require_domain_guard_stable && !metrics.domain_guard_stable {
            return EligibilityResult::Ineligible {
                reason: "Domain guard is unstable",
                unresolved_critical_count: 0,
            };
        }

        let confidence_score = 1.0 - metrics.churn_rate;
        EligibilityResult::Eligible {
            region_id: metrics.region_id.clone(),
            confidence_score,
            config_version: self.config.version,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_artifact_compiled_with_unresolved_critical_obligation() {
        let detector = StableRegionDetector::new(StabilityConfig::default());

        let metrics = RegionMetrics {
            region_id: "region_001".into(),
            hit_count: 1000,
            churn_rate: 0.01,
            domain_guard_stable: true,
            unresolved_critical_obligations: 1, // 1 unresolved critical obligation!
        };

        let result = detector.evaluate_region(&metrics);
        assert_eq!(
            result,
            EligibilityResult::Ineligible {
                reason: "Unresolved critical obligations present",
                unresolved_critical_count: 1,
            },
            "Region with unresolved critical obligation MUST NOT be compiled"
        );
    }

    #[test]
    fn test_configurable_and_versioned_hit_threshold() {
        let mut config = StabilityConfig::default();
        config.version = 42;
        config.min_hit_threshold = 500;

        let detector = StableRegionDetector::new(config);

        let metrics_low_hits = RegionMetrics {
            region_id: "region_002".into(),
            hit_count: 450, // Below 500
            churn_rate: 0.01,
            domain_guard_stable: true,
            unresolved_critical_obligations: 0,
        };

        let res_low = detector.evaluate_region(&metrics_low_hits);
        assert!(matches!(res_low, EligibilityResult::Ineligible { .. }));

        let metrics_high_hits = RegionMetrics {
            region_id: "region_002".into(),
            hit_count: 550, // Above 500
            churn_rate: 0.01,
            domain_guard_stable: true,
            unresolved_critical_obligations: 0,
        };

        let res_high = detector.evaluate_region(&metrics_high_hits);
        assert_eq!(
            res_high,
            EligibilityResult::Eligible {
                region_id: "region_002".into(),
                confidence_score: 0.99,
                config_version: 42,
            }
        );
    }

    #[test]
    fn test_false_stable_rate_below_0_1_percent_on_churn_benchmark() {
        let detector = StableRegionDetector::new(StabilityConfig::default());

        let total_samples = 10_000;
        let mut false_stable_count = 0;

        for i in 0..total_samples {
            // High churn regions (churn > 0.05) are truly unstable
            let churn_rate = 0.051 + (i as f64 % 100.0) * 0.005;
            let metrics = RegionMetrics {
                region_id: format!("region_churn_{}", i),
                hit_count: 1000,
                churn_rate,
                domain_guard_stable: true,
                unresolved_critical_obligations: 0,
            };

            if let EligibilityResult::Eligible { .. } = detector.evaluate_region(&metrics) {
                false_stable_count += 1;
            }
        }

        let false_stable_rate = (false_stable_count as f64 / total_samples as f64) * 100.0;
        println!(
            "[CHURN BENCHMARK] False stable rate: {:.4}% (Count: {} / {})",
            false_stable_rate, false_stable_count, total_samples
        );

        assert!(
            false_stable_rate < 0.1,
            "False stable rate {:.4}% MUST be < 0.1%",
            false_stable_rate
        );
    }
}
