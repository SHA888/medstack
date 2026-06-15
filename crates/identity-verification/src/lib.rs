//! identity-verification: credential verification and opaque token issuance.
//!
//! This crate is the trust boundary. It produces `VerifiedCredential` tokens
//! (opaque to qa-core, unforgeable) that certify a user's professional identity
//! and scope.
//!
//! qa-core cannot construct `VerifiedCredential` — the constructor is private
//! to this crate. This architectural invariant is compile-fail-tested.

use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

static CREDENTIAL_COUNTER: AtomicU64 = AtomicU64::new(0);

/// An unforgeable credential token issued by identity verification.
///
/// This is the only type that proves a user has been verified. qa-core
/// receives it as an opaque token and cannot construct or modify it.
///
/// The constructor is private to this crate; qa-core cannot access it.
/// This is a compiler-enforced guarantee that credentials cannot be forged
/// within the qa-core domain.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct VerifiedCredential {
    /// Unique identifier for this credential instance.
    id: String,
    /// Verified user ID (opaque to qa-core).
    user_id: String,
}

impl VerifiedCredential {
    /// Issue a new verified credential (private to identity-verification).
    ///
    /// This is the only way to construct a credential. qa-core cannot call this
    /// because the constructor is private to this module.
    #[allow(dead_code)]
    fn new(user_id: String) -> Self {
        let counter = CREDENTIAL_COUNTER.fetch_add(1, Ordering::SeqCst);
        let id = format!("cred-{}-{}", user_id, counter);
        VerifiedCredential { id, user_id }
    }

    /// Public issue method (visible to rest of identity-verification crate).
    #[allow(dead_code)]
    pub(crate) fn issue(user_id: String) -> Self {
        Self::new(user_id)
    }

    /// Return the verified user ID (opaque identifier).
    pub fn user_id(&self) -> &str {
        &self.user_id
    }

    /// Return the credential ID.
    pub fn credential_id(&self) -> &str {
        &self.id
    }
}

impl fmt::Display for VerifiedCredential {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "VerifiedCredential({})", self.user_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credential_can_be_issued() {
        let cred = VerifiedCredential::issue("user-123".to_string());
        assert_eq!(cred.user_id(), "user-123");
        assert!(cred.credential_id().starts_with("cred-user-123-"));
    }

    #[test]
    fn credentials_are_unique() {
        let cred1 = VerifiedCredential::issue("user-123".to_string());
        let cred2 = VerifiedCredential::issue("user-123".to_string());
        assert_ne!(cred1.credential_id(), cred2.credential_id());
    }

    #[test]
    fn credential_is_cloneable() {
        let cred = VerifiedCredential::issue("user-123".to_string());
        let cloned = cred.clone();
        assert_eq!(cred, cloned);
    }

    #[test]
    fn credential_display() {
        let cred = VerifiedCredential::issue("user-123".to_string());
        assert!(format!("{}", cred).contains("user-123"));
    }
}
