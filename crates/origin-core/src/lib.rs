// ORIGIN-Ω ZERO — Core Epistemic Kernel Types and State Logic

pub mod causal_status;
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
pub use distinction::Distinction;
pub use evidence::{EvidenceRecord, SupportKind};
pub use object::{Canonical, Claim, Evidence, Obligation, Operator};
pub use obligation::{ObligationKind, TypedObligation};
pub use opcode::OpCode;
pub use orid::{ObjectKind, ORID};
pub use quotient::RelevantSet;
pub use state::{Budget, State, StateTxn};
pub use status::{EpistemicError, Status};
