//! Credential authority, modeled inside qa-core's own vocabulary.
//!
//! qa-core must not depend on identity-verification (the architecture test
//! enforces this), so it cannot name `VerifiedCredential`. But an answer still
//! needs to record the *authority it was authored under*. This module gives
//! qa-core a domain-owned snapshot for exactly that.
//!
//! Why a snapshot and not a live credential: authority weight decays with time
//! and a credential can later expire or be revoked. Recording the scope and
//! weight **as of authoring time** keeps the honest historical signal on the
//! answer ("this was written by a verified Engineering credential weighing
//! 0.94"), independent of the credential's later fate. The identity-verification
//! adapter maps its `VerifiedCredential<Active>` into an `AuthoritySnapshot` at
//! the port boundary (M2).

use std::fmt;

/// The professional domain a credential was verified in, as qa-core sees it.
///
/// This is qa-core's own vocabulary, deliberately separate from
/// `identity_verification::CredentialScope`. The verification adapter maps
/// between the two; qa-core never imports the verification crate. Exhaustive,
/// no catch-all.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum CredentialScope {
    /// Clinical expertise (medicine, nursing, etc.).
    Clinical,
    /// Engineering / software development expertise.
    Engineering,
    /// Research expertise (academic, data science, etc.).
    Research,
}

impl fmt::Display for CredentialScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Clinical => write!(f, "Clinical"),
            Self::Engineering => write!(f, "Engineering"),
            Self::Research => write!(f, "Research"),
        }
    }
}

/// A validated authority weight in the closed range `[0.0, 1.0]`.
///
/// Parse-Don't-Validate: an out-of-range or non-finite weight is unrepresentable
/// — `new` is the only constructor and it rejects anything outside `[0.0, 1.0]`
/// (including NaN/infinity).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct AuthorityWeight(f64);

/// Error when constructing an [`AuthorityWeight`].
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AuthorityWeightError {
    /// The value was not finite or fell outside the `[0.0, 1.0]` range.
    OutOfRange,
}

impl fmt::Display for AuthorityWeightError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfRange => write!(f, "authority weight must be a finite value in [0.0, 1.0]"),
        }
    }
}

impl std::error::Error for AuthorityWeightError {}

impl AuthorityWeight {
    /// Construct an authority weight, rejecting non-finite or out-of-range values.
    ///
    /// # Errors
    ///
    /// Returns `AuthorityWeightError::OutOfRange` if `value` is NaN, infinite, or
    /// outside `[0.0, 1.0]`.
    pub fn new(value: f64) -> Result<Self, AuthorityWeightError> {
        if value.is_finite() && (0.0..=1.0).contains(&value) {
            Ok(AuthorityWeight(value))
        } else {
            Err(AuthorityWeightError::OutOfRange)
        }
    }

    /// The weight as an `f64` in `[0.0, 1.0]`.
    pub fn value(self) -> f64 {
        self.0
    }
}

/// A snapshot of the authority an answer was authored under, captured at
/// authoring time.
///
/// Carrying both `scope` and `weight` (rather than a presence bit) means the
/// answer records *which* professional domain vouched for it and *how strongly*,
/// frozen at the moment of authoring.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct AuthoritySnapshot {
    scope: CredentialScope,
    weight: AuthorityWeight,
}

impl AuthoritySnapshot {
    /// Capture an authority snapshot from a scope and a validated weight.
    pub fn new(scope: CredentialScope, weight: AuthorityWeight) -> Self {
        AuthoritySnapshot { scope, weight }
    }

    /// The professional domain the author was verified in.
    pub fn scope(&self) -> CredentialScope {
        self.scope
    }

    /// The authority weight captured at authoring time.
    pub fn weight(&self) -> AuthorityWeight {
        self.weight
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weight_accepts_in_range() {
        assert_eq!(AuthorityWeight::new(0.0).unwrap().value(), 0.0);
        assert_eq!(AuthorityWeight::new(1.0).unwrap().value(), 1.0);
        assert_eq!(AuthorityWeight::new(0.5).unwrap().value(), 0.5);
    }

    #[test]
    fn weight_rejects_out_of_range() {
        assert_eq!(
            AuthorityWeight::new(-0.1).unwrap_err(),
            AuthorityWeightError::OutOfRange
        );
        assert_eq!(
            AuthorityWeight::new(1.1).unwrap_err(),
            AuthorityWeightError::OutOfRange
        );
    }

    #[test]
    fn weight_rejects_non_finite() {
        assert_eq!(
            AuthorityWeight::new(f64::NAN).unwrap_err(),
            AuthorityWeightError::OutOfRange
        );
        assert_eq!(
            AuthorityWeight::new(f64::INFINITY).unwrap_err(),
            AuthorityWeightError::OutOfRange
        );
    }

    #[test]
    fn snapshot_carries_scope_and_weight() {
        let snap = AuthoritySnapshot::new(
            CredentialScope::Engineering,
            AuthorityWeight::new(0.9).unwrap(),
        );
        assert_eq!(snap.scope(), CredentialScope::Engineering);
        assert_eq!(snap.weight().value(), 0.9);
    }

    #[test]
    fn scope_display() {
        assert_eq!(format!("{}", CredentialScope::Clinical), "Clinical");
        assert_eq!(format!("{}", CredentialScope::Engineering), "Engineering");
        assert_eq!(format!("{}", CredentialScope::Research), "Research");
    }
}
