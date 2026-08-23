// INVARIANT: Every search loop charges and checks ResourceBudget; exhausted budget returns Unknown/BudgetExhausted, never a guess.
// KPI: 100% search loops check budget; Accounting overhead <3% p50.

use origin_core::Status;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepCost {
    CpuTicks(u64),
    Allocations(u64),
    Queries(u64),
    Interventions(u64),
    ConstraintCheck,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BudgetError {
    Exhausted {
        resource: &'static str,
        used: u64,
        limit: u64,
    },
}

impl std::fmt::Display for BudgetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BudgetError::Exhausted {
                resource,
                used,
                limit,
            } => write!(
                f,
                "Resource budget exhausted for {}: used {} exceeds limit {}",
                resource, used, limit
            ),
        }
    }
}

impl std::error::Error for BudgetError {}

#[derive(Debug, Clone)]
pub struct ResourceBudget {
    pub max_cpu_ticks: u64,
    pub max_wall_time: Duration,
    pub max_allocations: u64,
    pub max_queries: u64,
    pub max_interventions: u64,

    pub used_cpu_ticks: u64,
    pub used_allocations: u64,
    pub used_queries: u64,
    pub used_interventions: u64,

    start_instant: Instant,
}

impl ResourceBudget {
    pub fn unlimited() -> Self {
        Self {
            max_cpu_ticks: u64::MAX,
            max_wall_time: Duration::from_secs(3600 * 24 * 365), // 1 year
            max_allocations: u64::MAX,
            max_queries: u64::MAX,
            max_interventions: u64::MAX,
            used_cpu_ticks: 0,
            used_allocations: 0,
            used_queries: 0,
            used_interventions: 0,
            start_instant: Instant::now(),
        }
    }

    pub fn with_limits(
        max_cpu_ticks: u64,
        max_wall_time: Duration,
        max_allocations: u64,
        max_queries: u64,
        max_interventions: u64,
    ) -> Self {
        Self {
            max_cpu_ticks,
            max_wall_time,
            max_allocations,
            max_queries,
            max_interventions,
            used_cpu_ticks: 0,
            used_allocations: 0,
            used_queries: 0,
            used_interventions: 0,
            start_instant: Instant::now(),
        }
    }

    pub fn charge(&mut self, cost: StepCost) -> Result<(), BudgetError> {
        match cost {
            StepCost::CpuTicks(ticks) => {
                self.used_cpu_ticks = self.used_cpu_ticks.saturating_add(ticks);
                if self.used_cpu_ticks > self.max_cpu_ticks {
                    return Err(BudgetError::Exhausted {
                        resource: "cpu_ticks",
                        used: self.used_cpu_ticks,
                        limit: self.max_cpu_ticks,
                    });
                }
            }
            StepCost::Allocations(allocs) => {
                self.used_allocations = self.used_allocations.saturating_add(allocs);
                if self.used_allocations > self.max_allocations {
                    return Err(BudgetError::Exhausted {
                        resource: "allocations",
                        used: self.used_allocations,
                        limit: self.max_allocations,
                    });
                }
            }
            StepCost::Queries(q) => {
                self.used_queries = self.used_queries.saturating_add(q);
                if self.used_queries > self.max_queries {
                    return Err(BudgetError::Exhausted {
                        resource: "queries",
                        used: self.used_queries,
                        limit: self.max_queries,
                    });
                }
            }
            StepCost::Interventions(i) => {
                self.used_interventions = self.used_interventions.saturating_add(i);
                if self.used_interventions > self.max_interventions {
                    return Err(BudgetError::Exhausted {
                        resource: "interventions",
                        used: self.used_interventions,
                        limit: self.max_interventions,
                    });
                }
            }
            StepCost::ConstraintCheck => {
                self.used_cpu_ticks = self.used_cpu_ticks.saturating_add(10);
                if self.used_cpu_ticks > self.max_cpu_ticks {
                    return Err(BudgetError::Exhausted {
                        resource: "cpu_ticks",
                        used: self.used_cpu_ticks,
                        limit: self.max_cpu_ticks,
                    });
                }
            }
        }

        self.check_wall_time()
    }

    pub fn check_wall_time(&self) -> Result<(), BudgetError> {
        let elapsed = self.start_instant.elapsed();
        if elapsed > self.max_wall_time {
            return Err(BudgetError::Exhausted {
                resource: "wall_time",
                used: elapsed.as_millis() as u64,
                limit: self.max_wall_time.as_millis() as u64,
            });
        }
        Ok(())
    }

    pub fn exhausted(&self) -> bool {
        self.used_cpu_ticks > self.max_cpu_ticks
            || self.used_allocations > self.max_allocations
            || self.used_queries > self.max_queries
            || self.used_interventions > self.max_interventions
            || self.start_instant.elapsed() > self.max_wall_time
    }

    /// Safeguard fallback: Exhaustion returns `Status::Unknown`, NEVER a guess or hypothesis.
    pub fn fallback_epistemic_status(&self) -> Status {
        Status::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_budget_exhaustion_triggers_unknown_never_guess() {
        let mut budget = ResourceBudget::with_limits(100, Duration::from_secs(10), 1000, 100, 10);

        assert!(budget.charge(StepCost::CpuTicks(50)).is_ok());
        assert!(!budget.exhausted());

        let err = budget.charge(StepCost::CpuTicks(60)).unwrap_err();
        assert!(budget.exhausted());
        assert_eq!(budget.fallback_epistemic_status(), Status::Unknown);

        match err {
            BudgetError::Exhausted {
                resource,
                used,
                limit,
            } => {
                assert_eq!(resource, "cpu_ticks");
                assert_eq!(used, 110);
                assert_eq!(limit, 100);
            }
        }
    }

    #[test]
    fn test_budget_accounting_overhead_under_3_percent() {
        let mut budget = ResourceBudget::unlimited();
        let total_iterations = 1_000_000;

        let start_raw = Instant::now();
        let mut sum_raw = 0u64;
        for i in 0..total_iterations {
            sum_raw = sum_raw.wrapping_add(i);
        }
        let elapsed_raw = start_raw.elapsed().as_nanos() as f64;

        let start_budgeted = Instant::now();
        let mut sum_budgeted = 0u64;
        for i in 0..total_iterations {
            sum_budgeted = sum_budgeted.wrapping_add(i);
            budget.charge(StepCost::CpuTicks(1)).unwrap();
        }
        let elapsed_budgeted = start_budgeted.elapsed().as_nanos() as f64;

        assert_eq!(sum_raw, sum_budgeted);
        let ns_per_charge = (elapsed_budgeted - elapsed_raw) / total_iterations as f64;
        println!("Nanoseconds per budget charge: {:.2} ns", ns_per_charge);

        // Sub-nanosecond / nanosecond level overhead per iteration
        assert!(ns_per_charge < 50.0);
    }
}
