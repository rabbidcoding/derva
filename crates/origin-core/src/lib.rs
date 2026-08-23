#![forbid(unsafe_code)]

// ORIGIN-Ω ZERO — Core Epistemic Kernel Types and State Logic
// INVARIANT: origin-core is pure data + semantics; zero I/O, zero async runtime.

pub mod causal_status;
pub mod codec;
pub mod distinction;
pub mod evidence;
pub mod object;
pub mod obligation;
pub mod opcode;
pub mod orid;
pub mod quotient;
pub mod state;
pub mod status;

pub use causal_status::{CausalError, CausalStatus};
pub use codec::{
    decode_bytes_bounded, decode_exact, decode_str_bounded, decode_varint, encode_bytes_bounded,
    encode_str_bounded, encode_varint, CodecError, DEFAULT_MAX_BOUND,
};
pub use distinction::Distinction;
pub use evidence::{EvidenceRecord, SupportKind};
pub use object::{Canonical, Claim, Evidence, Obligation, Operator};
pub use obligation::{ObligationKind, TypedObligation};
pub use opcode::OpCode;
pub use orid::{ObjectKind, ORID};
pub use quotient::RelevantSet;
pub use state::{Budget, State, StateTxn};
pub use status::{EpistemicError, Status};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_origin_core_zero_unsafe_pure_semantics() {
        let s = State::new();
        assert_eq!(s.schema_version, 0);
    }
}
