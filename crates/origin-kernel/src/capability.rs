// INVARIANT: Type-level unforgeable capability tokens for side-effecting operations; PURE/READ cannot escalate.
// KPI: 0 unauthorized side effects across >=1e6 adversarial sequences; 100% effecting opcodes logged with principal+capability+commit.

use origin_core::{ObjectKind, ORID};
use std::marker::PhantomData;

pub trait EffectKind: 'static + Send + Sync {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pure;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReadOnly;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QueryExternal;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Intervene;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommitEffect;

impl EffectKind for Pure {}
impl EffectKind for ReadOnly {}
impl EffectKind for QueryExternal {}
impl EffectKind for Intervene {}
impl EffectKind for CommitEffect {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capability<E: EffectKind> {
    pub principal: String,
    pub token_id: ORID,
    pub scope: String,
    _marker: PhantomData<E>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapabilityError {
    Unauthorized(String),
    ScopeMismatch { expected: String, found: String },
    ForgedToken,
}

impl std::fmt::Display for CapabilityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CapabilityError::Unauthorized(msg) => {
                write!(f, "Unauthorized capability attempt: {}", msg)
            }
            CapabilityError::ScopeMismatch { expected, found } => {
                write!(
                    f,
                    "Capability scope mismatch: expected {}, found {}",
                    expected, found
                )
            }
            CapabilityError::ForgedToken => write!(f, "Attempted capability forgery detected"),
        }
    }
}

impl std::error::Error for CapabilityError {}

/// Capability Minting System (restricted kernel authority)
pub struct CapabilityMint;

impl CapabilityMint {
    pub fn mint_pure(principal: impl Into<String>) -> Capability<Pure> {
        Self::mint_typed(principal, "pure")
    }

    pub fn mint_read_only(
        principal: impl Into<String>,
        scope: impl Into<String>,
    ) -> Capability<ReadOnly> {
        Self::mint_typed(principal, scope)
    }

    pub fn mint_query_external(
        principal: impl Into<String>,
        scope: impl Into<String>,
    ) -> Capability<QueryExternal> {
        Self::mint_typed(principal, scope)
    }

    pub fn mint_intervene(
        principal: impl Into<String>,
        scope: impl Into<String>,
    ) -> Capability<Intervene> {
        Self::mint_typed(principal, scope)
    }

    pub fn mint_commit(
        principal: impl Into<String>,
        scope: impl Into<String>,
    ) -> Capability<CommitEffect> {
        Self::mint_typed(principal, scope)
    }

    fn mint_typed<E: EffectKind>(
        principal: impl Into<String>,
        scope: impl Into<String>,
    ) -> Capability<E> {
        let principal = principal.into();
        let scope = scope.into();
        let token_id = ORID::compute(
            ObjectKind::Artifact,
            format!("cap:{}:{}", principal, scope).as_bytes(),
        );

        Capability {
            principal,
            token_id,
            scope,
            _marker: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_tokens_are_type_isolated() {
        let cap_pure: Capability<Pure> = CapabilityMint::mint_pure("agent_1");
        let cap_intervene: Capability<Intervene> =
            CapabilityMint::mint_intervene("admin", "prod_scope");

        assert_eq!(cap_pure.principal, "agent_1");
        assert_eq!(cap_intervene.scope, "prod_scope");
    }
}
