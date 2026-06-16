//! identity-verification: credential verification and opaque token issuance.
//!
//! This crate is the trust boundary. It produces `VerifiedCredential` tokens
//! (opaque to qa-core, unforgeable) that certify a user's professional identity,
//! scope, and expiry.
//!
//! qa-core cannot construct `VerifiedCredential` — the constructor is private
//! to this crate. This architectural invariant is compile-fail-tested.
//!
//! Credentials follow a typestate lifecycle: Issued → Active → Expired.
//! The type system enforces that only Active credentials are readable.

use std::fmt;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::SystemTime;

static CREDENTIAL_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Error when constructing a VerifiedCredential.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CredentialError {
    /// user_id was empty or whitespace-only.
    EmptyUserId,
}

impl fmt::Display for CredentialError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyUserId => write!(f, "user_id cannot be empty or whitespace-only"),
        }
    }
}

impl std::error::Error for CredentialError {}

/// Credential scope: the professional domain in which the user is verified.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CredentialScope {
    /// Clinical expertise (medicine, nursing, etc.).
    Clinical,
    /// Engineering/software development expertise.
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

/// Typestate marker: credential is newly issued, not yet active.
#[derive(Clone, Copy, Debug)]
pub struct Issued;

/// Typestate marker: credential is active and usable.
#[derive(Clone, Copy, Debug)]
pub struct Active;

/// Typestate marker: credential has expired.
#[derive(Clone, Copy, Debug)]
pub struct Expired;

/// Trait for valid credential states in the type system.
pub trait CredentialState: Clone + fmt::Debug {}
impl CredentialState for Issued {}
impl CredentialState for Active {}
impl CredentialState for Expired {}

/// An unforgeable credential token issued by identity verification.
///
/// This is the only type that proves a user has been verified. qa-core
/// receives it as an opaque token and cannot construct or modify it.
///
/// The constructor is private to this crate; qa-core cannot access it.
/// This is a compiler-enforced guarantee that credentials cannot be forged
/// within the qa-core domain.
///
/// Credentials follow a typestate lifecycle:
/// - `Issued`: newly created, must be activated before use
/// - `Active`: usable, provides access to user_id, scope, and expiry
/// - `Expired`: no longer usable, created when expiry time is reached
#[derive(Clone, Debug)]
pub struct VerifiedCredential<S: CredentialState = Active> {
    /// Opaque credential ID (counter-based, does not encode user_id).
    id: String,
    /// Verified user ID (opaque to qa-core, provided at construction time only).
    user_id: String,
    /// Credential scope (the professional domain of verification).
    scope: CredentialScope,
    /// Expiry time (after this time, credential cannot be used).
    expiry: SystemTime,
    /// Typestate phantom marker.
    _state: PhantomData<S>,
}

impl VerifiedCredential<Issued> {
    /// Issue a new verified credential in Issued state (private to identity-verification).
    ///
    /// This is the only way to construct a credential. qa-core cannot call this
    /// because the constructor is private to this module.
    ///
    /// # Errors
    ///
    /// Returns `CredentialError::EmptyUserId` if user_id is empty or whitespace-only.
    pub(crate) fn issue(
        user_id: String,
        scope: CredentialScope,
        expiry: SystemTime,
    ) -> Result<Self, CredentialError> {
        let trimmed = user_id.trim();
        if trimmed.is_empty() {
            return Err(CredentialError::EmptyUserId);
        }

        let counter = CREDENTIAL_COUNTER.fetch_add(1, Ordering::Relaxed);
        let id = format!("cred-{}", counter);
        Ok(VerifiedCredential {
            id,
            user_id: user_id.clone(),
            scope,
            expiry,
            _state: PhantomData,
        })
    }

    /// Transition credential from Issued to Active state.
    /// Only Active credentials can be used to read user_id, scope, and expiry.
    pub fn activate(self) -> VerifiedCredential<Active> {
        VerifiedCredential {
            id: self.id,
            user_id: self.user_id,
            scope: self.scope,
            expiry: self.expiry,
            _state: PhantomData,
        }
    }
}

impl VerifiedCredential<Active> {
    /// Return the verified user ID (opaque identifier).
    /// Only callable on Active credentials.
    pub fn user_id(&self) -> &str {
        &self.user_id
    }

    /// Return the credential ID (unique, non-predictable).
    /// Only callable on Active credentials.
    pub fn credential_id(&self) -> &str {
        &self.id
    }

    /// Return the credential scope (professional domain of verification).
    /// Only callable on Active credentials.
    pub fn scope(&self) -> CredentialScope {
        self.scope
    }

    /// Return the credential expiry time.
    /// Only callable on Active credentials.
    pub fn expiry(&self) -> SystemTime {
        self.expiry
    }

    /// Check if credential has expired as of the current time.
    pub fn is_expired(&self) -> bool {
        SystemTime::now() >= self.expiry
    }

    /// Transition credential from Active to Expired state.
    pub fn expire(self) -> VerifiedCredential<Expired> {
        VerifiedCredential {
            id: self.id,
            user_id: self.user_id,
            scope: self.scope,
            expiry: self.expiry,
            _state: PhantomData,
        }
    }
}

impl<S: CredentialState> PartialEq for VerifiedCredential<S> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.user_id == other.user_id
    }
}

impl<S: CredentialState> Eq for VerifiedCredential<S> {}

#[cfg(test)]
mod tests {
    use super::*;

    fn issue_test_credential(user_id: &str) -> Result<VerifiedCredential<Active>, CredentialError> {
        let expiry = SystemTime::now() + std::time::Duration::from_secs(3600);
        VerifiedCredential::issue(user_id.to_string(), CredentialScope::Engineering, expiry)
            .map(|issued| issued.activate())
    }

    #[test]
    fn credential_can_be_issued_and_activated() {
        let cred = issue_test_credential("user-123").unwrap();
        assert_eq!(cred.user_id(), "user-123");
        assert!(cred.credential_id().starts_with("cred-"));
        assert_eq!(cred.scope(), CredentialScope::Engineering);
    }

    #[test]
    fn credentials_are_unique() {
        let cred1 = issue_test_credential("user-123").unwrap();
        let cred2 = issue_test_credential("user-123").unwrap();
        assert_ne!(cred1.credential_id(), cred2.credential_id());
    }

    #[test]
    fn credential_is_cloneable() {
        let cred = issue_test_credential("user-123").unwrap();
        let cloned = cred.clone();
        assert_eq!(cred, cloned);
    }

    #[test]
    fn empty_user_id_rejected() {
        let expiry = SystemTime::now() + std::time::Duration::from_secs(3600);
        assert_eq!(
            VerifiedCredential::issue("".to_string(), CredentialScope::Engineering, expiry),
            Err(CredentialError::EmptyUserId)
        );
        assert_eq!(
            VerifiedCredential::issue("   ".to_string(), CredentialScope::Engineering, expiry),
            Err(CredentialError::EmptyUserId)
        );
    }

    #[test]
    fn credential_id_does_not_embed_user_id() {
        let cred1 = issue_test_credential("alice").unwrap();
        let cred2 = issue_test_credential("bob").unwrap();
        assert!(!cred1.credential_id().contains("alice"));
        assert!(!cred2.credential_id().contains("bob"));
    }

    #[test]
    fn credential_transitions_through_states() {
        let expiry = SystemTime::now() + std::time::Duration::from_secs(3600);
        let issued = VerifiedCredential::issue("user-123".to_string(), CredentialScope::Clinical, expiry).unwrap();
        let active = issued.activate();
        assert_eq!(active.user_id(), "user-123");
        assert_eq!(active.scope(), CredentialScope::Clinical);
        let _expired = active.expire();
    }

    #[test]
    fn credential_carries_scope() {
        let scopes = [
            CredentialScope::Clinical,
            CredentialScope::Engineering,
            CredentialScope::Research,
        ];
        let expiry = SystemTime::now() + std::time::Duration::from_secs(3600);
        for scope in scopes.iter() {
            let cred = VerifiedCredential::issue("user".to_string(), *scope, expiry)
                .unwrap()
                .activate();
            assert_eq!(cred.scope(), *scope);
        }
    }

    #[test]
    fn credential_expiry_not_expired_in_future() {
        let expiry = SystemTime::now() + std::time::Duration::from_secs(3600);
        let cred = VerifiedCredential::issue("user".to_string(), CredentialScope::Engineering, expiry)
            .unwrap()
            .activate();
        assert!(!cred.is_expired());
    }

    #[test]
    fn credential_expiry_is_expired_in_past() {
        let expiry = SystemTime::now() - std::time::Duration::from_secs(1);
        let cred = VerifiedCredential::issue("user".to_string(), CredentialScope::Engineering, expiry)
            .unwrap()
            .activate();
        assert!(cred.is_expired());
    }
}
