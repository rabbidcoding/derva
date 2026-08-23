#![forbid(unsafe_code)]

// AUDIT-LENSES: Ken Thompson, Donald Knuth, John Carmack
// INVARIANT: Artifact Guard + Dependency Invalidation preventing stale compilation execution and domain boundary bypass.
// KPI: Stale artifact execution = 0 over 1M mutation scenarios; Guard false-negative = 0; Guard overhead < 5% p50.

use origin_core::ORID;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtifactError {
    StaleDependencyRoot {
        expected: ORID,
        actual: ORID,
    },
    SchemaHashMismatch {
        expected_schema_hash: u64,
        actual_schema_hash: u64,
    },
    DomainGuardRejected,
}

impl std::fmt::Display for ArtifactError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArtifactError::StaleDependencyRoot { expected, actual } => write!(
                f,
                "Stale artifact: dependency root mismatch expected {:?} got {:?}",
                expected, actual
            ),
            ArtifactError::SchemaHashMismatch {
                expected_schema_hash,
                actual_schema_hash,
            } => write!(
                f,
                "Schema hash mismatch: expected {:x} got {:x}",
                expected_schema_hash, actual_schema_hash
            ),
            ArtifactError::DomainGuardRejected => write!(f, "Domain guard rejected input values"),
        }
    }
}

impl std::error::Error for ArtifactError {}

#[derive(Debug, Clone)]
pub struct DomainGuard {
    pub min_value: f64,
    pub max_value: f64,
}

impl DomainGuard {
    #[inline(always)]
    pub fn accepts(&self, input: f64) -> bool {
        input >= self.min_value && input <= self.max_value
    }
}

#[derive(Debug, Clone)]
pub struct CompiledArtifact {
    pub artifact_id: String,
    pub dep_root: ORID,
    pub schema_hash: u64,
    pub guard: DomainGuard,
}

impl CompiledArtifact {
    /// Validates artifact freshness and domain guard acceptance prior to execution
    #[inline]
    pub fn validate_and_acquire(
        &self,
        current_dep_root: ORID,
        current_schema_hash: u64,
        input: f64,
    ) -> Result<(), ArtifactError> {
        // 1. Dependency root staleness check
        if self.dep_root != current_dep_root {
            return Err(ArtifactError::StaleDependencyRoot {
                expected: current_dep_root,
                actual: self.dep_root,
            });
        }

        // 2. Schema hash integrity check
        if self.schema_hash != current_schema_hash {
            return Err(ArtifactError::SchemaHashMismatch {
                expected_schema_hash: current_schema_hash,
                actual_schema_hash: self.schema_hash,
            });
        }

        // 3. Domain guard check
        if !self.guard.accepts(input) {
            return Err(ArtifactError::DomainGuardRejected);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use origin_core::{orid::ObjectKind, ORID};
    use std::time::Instant;

    #[test]
    fn test_stale_artifact_execution_zero_over_1m_mutation_scenarios() {
        let base_dep = ORID::compute(ObjectKind::Claim, b"dep_base");
        let base_schema = 0xdeadbeef12345678u64;

        let artifact = CompiledArtifact {
            artifact_id: "art_001".into(),
            dep_root: base_dep,
            schema_hash: base_schema,
            guard: DomainGuard {
                min_value: -100.0,
                max_value: 100.0,
            },
        };

        let mutations = 1_000_000;
        let mut stale_accepted = 0;

        for i in 0..mutations {
            let mut dep = base_dep;
            let mut schema = base_schema;

            if i % 2 == 0 {
                // Mutate dep root
                dep = ORID::compute(ObjectKind::Claim, format!("dep_mut_{}", i).as_bytes());
            } else {
                // Mutate schema hash
                schema = base_schema ^ (i as u64 + 1);
            }

            if artifact.validate_and_acquire(dep, schema, 0.0).is_ok() {
                stale_accepted += 1;
            }
        }

        println!(
            "[ARTIFACT FUZZ] Evaluated {} mutations | Stale executions accepted: {}",
            mutations, stale_accepted
        );

        assert_eq!(
            stale_accepted, 0,
            "Stale artifact execution MUST be strictly 0"
        );
    }

    #[test]
    fn test_guard_false_negative_zero() {
        let dep = ORID::compute(ObjectKind::Claim, b"dep");
        let artifact = CompiledArtifact {
            artifact_id: "art_guard".into(),
            dep_root: dep,
            schema_hash: 0x1234,
            guard: DomainGuard {
                min_value: 0.0,
                max_value: 10.0,
            },
        };

        // Out of bounds inputs
        let out_of_bounds = vec![-0.001, 10.001, -100.0, 50.0];
        for input in out_of_bounds {
            let res = artifact.validate_and_acquire(dep, 0x1234, input);
            assert_eq!(
                res,
                Err(ArtifactError::DomainGuardRejected),
                "Out-of-bounds input {} MUST be rejected by guard",
                input
            );
        }
    }

    #[test]
    fn test_guard_overhead_less_than_5_percent_p50() {
        let dep = ORID::compute(ObjectKind::Claim, b"dep");
        let artifact = CompiledArtifact {
            artifact_id: "art_bench".into(),
            dep_root: dep,
            schema_hash: 0x1234,
            guard: DomainGuard {
                min_value: -100.0,
                max_value: 100.0,
            },
        };

        let iterations = 100_000;

        // Baseline execution representing fast path operation (simulated 200ns fast-path tick)
        let start_base = Instant::now();
        let mut sum_base = 0.0f64;
        for i in 0..iterations {
            for k in 0..50 {
                sum_base += (i as f64 + k as f64) * 1.0001;
            }
        }
        let dur_base = start_base.elapsed();

        // Guarded execution (guard check performed once per fast-path tick)
        let start_guard = Instant::now();
        let mut sum_guard = 0.0f64;
        for i in 0..iterations {
            if artifact.validate_and_acquire(dep, 0x1234, 5.0).is_ok() {
                for k in 0..50 {
                    sum_guard += (i as f64 + k as f64) * 1.0001;
                }
            }
        }
        let dur_guard = start_guard.elapsed();

        assert_eq!(sum_base, sum_guard);

        let overhead_percent =
            ((dur_guard.as_nanos() as f64 - dur_base.as_nanos() as f64) / dur_base.as_nanos() as f64) * 100.0;

        println!(
            "[GUARD BENCHMARK] Base: {:?} | Guarded: {:?} | Overhead: {:.2}%",
            dur_base, dur_guard, overhead_percent
        );

        assert!(
            overhead_percent < 5.0,
            "Guard overhead {:.2}% MUST be < 5.0%",
            overhead_percent
        );
    }
}
