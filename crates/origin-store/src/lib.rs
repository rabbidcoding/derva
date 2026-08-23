#![forbid(unsafe_code)]

// ORIGIN-Ω ZERO Subsystem: origin-store
// Content-addressed immutable persistence layer.

pub mod commit;
pub mod object_store;
pub mod wal;

pub use commit::{CommitDag, CommitNode};
pub use object_store::{ObjectStore, StoreError};
pub use origin_core::{Canonical, ORID};
pub use wal::{PendingTxn, WalError, WalStatus, WriteAheadLog};

pub fn crate_name() -> &'static str {
    "origin-store"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_boundary() {
        assert_eq!(crate_name(), "origin-store");
    }
}
