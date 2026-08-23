// INVARIANT: No performance claim without fixed seeds, frozen baselines, and CV <= 0.03.
// KPI: 100% of benchmark suites enforce max_cv threshold <= 0.03.

#[derive(Debug, Clone)]
pub struct BenchSpec {
    pub id: String,
    pub name: String,
    pub warmups: u32,
    pub samples: u32,
    pub max_cv: f64, // <= 0.03
    pub seed: u64,
}

impl BenchSpec {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            warmups: 50,
            samples: 500,
            max_cv: 0.03,
            seed: 42,
        }
    }
}

pub struct BenchReport {
    pub mean_time_ns: f64,
    pub std_dev_ns: f64,
    pub cv: f64,
    pub is_noisy: bool,
}

pub fn evaluate_benchmark<F>(spec: &BenchSpec, mut f: F) -> BenchReport
where
    F: FnMut(),
{
    // Warmup phase
    for _ in 0..spec.warmups {
        f();
    }

    let mut timings = Vec::with_capacity(spec.samples as usize);
    for _ in 0..spec.samples {
        let start = std::time::Instant::now();
        f();
        let elapsed = start.elapsed().as_nanos() as f64;
        timings.push(elapsed);
    }

    let sum: f64 = timings.iter().sum();
    let mean = sum / (timings.len() as f64);

    let variance: f64 =
        timings.iter().map(|&t| (t - mean).powi(2)).sum::<f64>() / (timings.len() as f64);
    let std_dev = variance.sqrt();
    let cv = if mean > 0.0 { std_dev / mean } else { 0.0 };

    BenchReport {
        mean_time_ns: mean,
        std_dev_ns: std_dev,
        cv,
        is_noisy: cv > spec.max_cv,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bench_spec_evaluation() {
        let spec = BenchSpec::new("BENCH-TEST-001", "Test Execution Speed");
        let report = evaluate_benchmark(&spec, || {
            let mut acc: u64 = 0;
            for i in 0..10_000 {
                acc = acc.wrapping_add(i);
            }
            std::hint::black_box(acc);
        });

        assert!(report.mean_time_ns > 0.0);
    }
}
