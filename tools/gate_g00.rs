// INVARIANT: Gate G00 closes Phase P00 only if 100% of P00 requirements (T001-T009) are verified green.
// KPI: 10/10 checks PASS; 0 bypasses allowed.

use std::fs;
use std::path::Path;
use std::process::{Command, exit};

fn run_check(name: &str, program: &str, args: &[&str]) -> bool {
    println!("[G00 AUDIT] Running check: {}...", name);
    let output = Command::new(program)
        .args(args)
        .output();

    match output {
        Ok(res) => {
            if res.status.success() {
                println!("  -> PASS: {}", name);
                true
            } else {
                println!("  -> FAIL: {}", name);
                println!("     stdout: {}", String::from_utf8_lossy(&res.stdout));
                println!("     stderr: {}", String::from_utf8_lossy(&res.stderr));
                false
            }
        }
        Err(err) => {
            println!("  -> FAIL: {} (Failed to execute: {})", name, err);
            false
        }
    }
}

fn main() {
    println!("========== G00 TRUTH + REPOSITORY GATE AUDIT (RUST HARNESS) ==========");

    let checks = vec![
        ("zero-training", "python3", vec!["tools/zero_train_guard.py"]),
        ("claims-ledger", "python3", vec!["tools/claims_lint.py"]),
        ("toolchain-manifest", "python3", vec!["tools/toolchain_manifest.py"]),
        ("cargo-check", "cargo", vec!["check", "--workspace"]),
        ("cargo-clippy", "cargo", vec!["clippy", "--workspace", "--", "-D", "warnings"]),
        ("cargo-fmt", "cargo", vec!["fmt", "--", "--check"]),
        ("cargo-test", "cargo", vec!["test", "--workspace"]),
        ("attestation-verifier", "./tools/verify_attestation.sh", vec!["Cargo.toml"]),
        ("bench-suite-check", "cargo", vec!["test", "-p", "origin-bench"]),
        ("governance-contract", "python3", vec!["-c", "from pathlib import Path; assert Path('.github/CODEOWNERS').exists() and Path('.github/ruleset.production.json').exists()"]),
    ];

    let mut results = Vec::new();
    for (name, program, args) in checks {
        let ok = run_check(name, program, &args);
        results.push((name, ok));
    }

    let failed: Vec<&str> = results.iter().filter(|(_, ok)| !*ok).map(|(n, _)| *n).collect();

    let status_str = if failed.is_empty() { "PASS" } else { "FAIL" };
    let mut report_md = format!(
        "# G00 Gate Execution Report\n\n> **Status:** {}\n\n## Verification Checklist\n\n",
        status_str
    );

    for (name, ok) in &results {
        let mark = if *ok { "✅ PASS" } else { "❌ FAIL" };
        report_md.push_str(&format!("- [{}] {}\n", mark, name));
    }

    let report_path = Path::new("reports/gates/G00.md");
    if let Some(parent) = report_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(report_path, report_md).expect("Failed to write reports/gates/G00.md");

    println!("\n[G00 GATE REPORT] Written to reports/gates/G00.md");

    if !failed.is_empty() {
        eprintln!("[G00 GATE FAIL] {} checks failed: {:?}", failed.len(), failed);
        exit(1);
    } else {
        println!("[G00 GATE PASS] 10/10 checks green. Phase P00 closed successfully.");
        exit(0);
    }
}
