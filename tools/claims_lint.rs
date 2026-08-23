// INVARIANT: Every quantitative claim in spec/claims.yaml must be falsable and resolve to a benchmark ID.
// KPI: 100% of claims have metric, baseline, target, gate, owner, kill_condition, and benchmark_id.

use std::fs;
use std::path::Path;
use std::process::exit;

#[derive(Debug, Default)]
struct Claim {
    id: Option<String>,
    description: Option<String>,
    metric: Option<String>,
    baseline: Option<String>,
    target: Option<String>,
    gate: Option<String>,
    owner: Option<String>,
    benchmark_id: Option<String>,
    kill_condition: Option<String>,
}

fn parse_claims_yaml(content: &str) -> Vec<Claim> {
    let mut claims = Vec::new();
    let mut current: Option<Claim> = None;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if trimmed.starts_with("- id:") {
            if let Some(c) = current.take() {
                claims.push(c);
            }
            let mut c = Claim::default();
            c.id = Some(trimmed.trim_start_matches("- id:").trim().trim_matches('"').to_string());
            current = Some(c);
        } else if let Some(ref mut c) = current {
            if let Some((key, val)) = trimmed.split_once(':') {
                let key = key.trim();
                let val = val.trim().trim_matches('"').to_string();
                match key {
                    "description" => c.description = Some(val),
                    "metric" => c.metric = Some(val),
                    "baseline" => c.baseline = Some(val),
                    "target" => c.target = Some(val),
                    "gate" => c.gate = Some(val),
                    "owner" => c.owner = Some(val),
                    "benchmark_id" => c.benchmark_id = Some(val),
                    "kill_condition" => c.kill_condition = Some(val),
                    _ => {}
                }
            }
        }
    }

    if let Some(c) = current {
        claims.push(c);
    }

    claims
}

fn main() {
    let spec_path = Path::new("spec/claims.yaml");
    println!("[CLAIMS LINT RS] Auditing claim ledger at: {:?}", spec_path);

    if !spec_path.exists() {
        eprintln!("[FAIL] Missing spec/claims.yaml file!");
        exit(1);
    }

    let content = fs::read_to_string(spec_path).expect("Failed to read spec/claims.yaml");
    let claims = parse_claims_yaml(&content);

    let mut errors = Vec::new();

    for claim in &claims {
        let cid = claim.id.as_deref().unwrap_or("UNKNOWN");

        // KPI: every quantitative claim must resolve to a benchmark id.
        if claim.metric.is_none() {
            errors.push(format!("Claim '{}' is missing field 'metric'", cid));
        }
        if claim.baseline.is_none() {
            errors.push(format!("Claim '{}' is missing field 'baseline'", cid));
        }
        if claim.target.is_none() {
            errors.push(format!("Claim '{}' is missing field 'target'", cid));
        }
        if claim.gate.is_none() {
            errors.push(format!("Claim '{}' is missing field 'gate'", cid));
        }
        if claim.owner.is_none() {
            errors.push(format!("Claim '{}' is missing field 'owner'", cid));
        }
        if claim.benchmark_id.is_none() {
            errors.push(format!("Claim '{}' is missing field 'benchmark_id'", cid));
        }
        if claim.kill_condition.is_none() {
            errors.push(format!("Claim '{}' is missing field 'kill_condition'", cid));
        }

        if cid.contains("POST-FRONTIER") && claim.benchmark_id.is_none() {
            errors.push(format!("Post-frontier claim '{}' must resolve to a valid benchmark_id!", cid));
        }
    }

    if !errors.is_empty() {
        eprintln!("[FAIL] Found {} validation errors in claim ledger:", errors.len());
        for err in &errors {
            eprintln!("  - {}", err);
        }
        exit(1);
    }

    println!(
        "[PASS] All {} claims in ledger are valid, falsable, and resolved to benchmark IDs.",
        claims.len()
    );
}
