#![no_main]
// AUDIT-LENSES: Ken Thompson, Donald Knuth, Dennis Ritchie
// INVARIANT: Fuzz target for OIR text parser and type checker; 0 crashes allowed.

use libfuzzer_sys::fuzz_target;
use origin_oir::ir::OirModule;
use origin_oir::typecheck::TypeChecker;
use origin_oir::verify::OirVerifier;

fuzz_target!(|data: &[u8]| {
    if let Ok(text) = std::str::from_utf8(data) {
        if let Ok(module) = OirModule::parse_text(text) {
            let _ = TypeChecker::check_module(&module);
            let _ = OirVerifier::verify(&module);
        }
    }
});
