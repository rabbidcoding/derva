// AUDIT-LENSES: Steve Jobs, Donald Knuth, Guido van Rossum, Bill Gates
// INVARIANT: Command line entrypoint for ORIGIN-Ω ZERO CLI & Epistemic Debugger.

use origin_cli::debugger::{DebuggerResponse, EpistemicDebugger};
use std::env;
use std::time::Instant;

fn print_usage() {
    println!("ORIGIN-Ω ZERO — Epistemic Debugger CLI");
    println!("Usage:");
    println!("  origin why <ORID> [--json]");
    println!("  origin why-not <ORID> [--json]");
    println!("  origin evidence <ORID> [--json]");
    println!("  origin history <ORID> [--json]");
    println!("  origin obligations [--json]");
    println!("  origin causal <ORID> [--json]");
    println!("  origin replay <COMMIT_ROOT> [--verify] [--json]");
    println!("  origin profile [--json]");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        return;
    }

    let cmd = args[1].as_str();
    let is_json = args.iter().any(|arg| arg == "--json");
    let is_verify = args.iter().any(|arg| arg == "--verify");
    let start = Instant::now();

    match cmd {
        "why" => {
            let orid_arg = args.get(2).map(|s| s.as_str()).unwrap_or("orid:claim:default");
            let explanation = EpistemicDebugger::explain_why(orid_arg);
            let elapsed = start.elapsed().as_micros();

            if is_json {
                let resp = DebuggerResponse::new("why", Some(orid_arg.to_string()), elapsed, explanation);
                println!("{}", serde_json::to_string_pretty(&resp).unwrap());
            } else {
                println!("=== WHY EXPLANATION for ORID [{}] ===", orid_arg);
                println!("Status: {}", explanation.status);
                println!("Verified: {}", explanation.is_verified);
                println!("Root Evidence Items: {}", explanation.root_evidence_count);
                println!("Epistemic Score: {:.4}", explanation.epistemic_score);
                println!("Verification Chain:");
                for step in &explanation.verification_chain {
                    println!("  -> {}", step);
                }
            }
        }
        "why-not" => {
            let orid_arg = args.get(2).map(|s| s.as_str()).unwrap_or("orid:claim:default");
            let explanation = EpistemicDebugger::explain_why_not(orid_arg);
            let elapsed = start.elapsed().as_micros();

            if is_json {
                let resp = DebuggerResponse::new("why-not", Some(orid_arg.to_string()), elapsed, explanation);
                println!("{}", serde_json::to_string_pretty(&resp).unwrap());
            } else {
                println!("=== WHY-NOT EXPLANATION for ORID [{}] ===", orid_arg);
                println!("Current Status: {}", explanation.current_status);
                println!("Missing Requirements:");
                for req in &explanation.missing_requirements {
                    println!("  - {}", req);
                }
            }
        }
        "replay" => {
            let root_arg = args.get(2).map(|s| s.as_str()).unwrap_or("orid:commit:root");
            let res = EpistemicDebugger::replay_commit(root_arg, is_verify);
            let elapsed = start.elapsed().as_micros();

            if is_json {
                let resp = DebuggerResponse::new("replay", Some(root_arg.to_string()), elapsed, res);
                println!("{}", serde_json::to_string_pretty(&resp).unwrap());
            } else {
                println!("=== REPLAY EXECUTION for Commit Root [{}] ===", root_arg);
                println!("Steps Replayed: {}", res.steps_replayed);
                println!("Verified Parity: {}", res.verified_parity);
                println!("Final State Hash: {}", res.final_state_hash);
            }
        }
        "profile" => {
            let report = EpistemicDebugger::profile_summary();
            let elapsed = start.elapsed().as_micros();

            if is_json {
                let resp = DebuggerResponse::new("profile", None, elapsed, report);
                println!("{}", serde_json::to_string_pretty(&resp).unwrap());
            } else {
                println!("=== FAST RUNTIME PROFILE SUMMARY ===");
                println!("Fast Path Hit Rate: {:.1}%", report.fast_hit_rate_pct);
                println!("Scalar Fallback Rate: {:.1}%", report.scalar_fallback_rate_pct);
                println!("SIMD Speedup Factor: {:.2}x", report.simd_speedup_factor);
                println!("Total Micro-Ops Executed: {}", report.total_microops_executed);
            }
        }
        _ => {
            let orid_arg = args.get(2).map(|s| s.as_str()).unwrap_or("orid:claim:default");
            let explanation = EpistemicDebugger::explain_why(orid_arg);
            let elapsed = start.elapsed().as_micros();
            let resp = DebuggerResponse::new(cmd, Some(orid_arg.to_string()), elapsed, explanation);
            println!("{}", serde_json::to_string_pretty(&resp).unwrap());
        }
    }
}
