// AUDIT-LENSES: Steve Jobs, Linus Torvalds, Ken Thompson, Donald Knuth
// INVARIANT: Authoritative Rust binary listening for JSONL request stream from Python agent.

use derva_arc3_adapter::bridge::{ArcBridgeEngine, StepRequest};
use std::io::{self, BufRead, Write};

fn main() {
    let mut engine = ArcBridgeEngine::new();
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        if let Ok(text) = line {
            if text.trim().is_empty() {
                continue;
            }
            match serde_json::from_str::<StepRequest>(&text) {
                Ok(req) => {
                    let resp = engine.process_step(req);
                    if let Ok(json_out) = serde_json::to_string(&resp) {
                        println!("{}", json_out);
                        let _ = io::stdout().flush();
                    }
                }
                Err(err) => {
                    eprintln!("[DERVA Rust IPC Error] Failed to parse StepRequest: {} | JSON: {}", err, text);
                    let _ = io::stderr().flush();
                }
            }
        }
    }

}

