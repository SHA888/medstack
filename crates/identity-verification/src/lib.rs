//! identity-verification: credential verification and opaque token issuance.
//!
//! This crate is the trust boundary. It produces `VerifiedCredential` tokens
//! (opaque to qa-core, unforgeable) that certify a user's professional identity,
//! scope, and expiry.
//!
//! qa-core cannot construct `VerifiedCredential` — the constructor is private
//! to this crate. This architectural invariant is compile-fail-tested in
//! `tests/ui/cannot_forge_credential.rs` (an external crate that tries to call
//! the private `issue` constructor and fails to compile).
//!
//! Credentials follow a typestate lifecycle: Issued → Active → Expired. The
//! typestate governs the *lifecycle* (you must `activate` before reading, and
//! you can only reach `Expired` once the clock has actually passed expiry).
//! Wall-clock validity itself cannot be a compile-time state — time passes
//! after activation — so it is enforced at the transition boundaries
//! (`activate` refuses already-expired credentials, `expire` refuses
//! still-valid ones) and re-checked at use (`authority_weight` returns 0.0
//! once `as_of` is past expiry).

use std::fmt;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::SystemTime;

#[allow(dead_code)]
static CREDENTIAL_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Error when constructing or transitioning a VerifiedCredential.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CredentialError {
    /// user_id was empty or whitespace-only.
    EmptyUserId,
    /// Attempted to activate a credential whose expiry is already in the past.
    /// An expired credential must never become Active.
    AlreadyExpired,
    /// Attempted to move a still-valid credential into the Expired state.
    /// `Expired` truthfully means "past expiry"; revoking a valid credential is
    /// a separate concept (deferred — see crate docs) and is not `expire`.
    NotYetExpired,
}

impl fmt::Display for CredentialError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyUserId => write!(f, "user_id cannot be empty or whitespace-only"),
            Self::AlreadyExpired => {
                write!(f, "cannot activate a credential whose expiry has passed")
            }
            Self::NotYetExpired => {
                write!(f, "cannot expire a credential that is still valid")
            }
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

/// Base authority weight contributed by a credential's scope, before freshness decay.
///
/// MedOverflow gates *software / informatics* questions, and the credential badge
/// means "engineering/informatics authority on the software question, NOT clinical
/// endorsement" (see badge semantics, task 0.4.2). Weights therefore reflect
/// relevance to that engineering axis: Engineering full, Research high, Clinical
/// somewhat lower.
///
/// These values are **PROVISIONAL** and expected to be calibrated with product
/// input. The exhaustive `match` (no catch-all) forces a deliberate choice for
/// every scope, so adding a scope variant is a compile error here rather than a
/// silent default.
fn scope_base_weight(scope: CredentialScope) -> f64 {
    match scope {
        CredentialScope::Engineering => 1.0,
        CredentialScope::Research => 0.90,
        CredentialScope::Clinical => 0.80,
    }
}

/// Typestate marker: credential is newly issued, not yet active.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Issued;

/// Typestate marker: credential is active and usable.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Active;

/// Typestate marker: credential has expired.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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
/// - `Expired`: reached only via `expire`, which requires the clock to be past
///   expiry — so the `Expired` state truthfully means "past expiry"
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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

impl<S: CredentialState> VerifiedCredential<S> {
    /// Helper to transition between typestate markers by reconstructing the credential.
    ///
    /// Explicitly copies all credential fields (id, user_id, scope, expiry) to ensure they
    /// are preserved across state changes. Rust's exhaustiveness checking requires all fields
    /// in the struct literal, so adding a field to VerifiedCredential will cause a compile error
    /// here—preventing silent data loss if transition() is not updated.
    fn transition<T: CredentialState>(self) -> VerifiedCredential<T> {
        VerifiedCredential {
            id: self.id,
            user_id: self.user_id,
            scope: self.scope,
            expiry: self.expiry,
            _state: PhantomData,
        }
    }
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
    #[allow(dead_code)]
    pub(crate) fn issue(
        user_id: String,
        scope: CredentialScope,
        expiry: SystemTime,
    ) -> Result<Self, CredentialError> {
        let trimmed = user_id.trim();
        if trimmed.is_empty() {
            return Err(CredentialError::EmptyUserId);
        }

        // SeqCst ensures a total ordering of credential issuance across all threads.
        // This is necessary at a security boundary: audit trails and authority weight
        // calculations must be able to rely on consistent ordering of credentials.
        let counter = CREDENTIAL_COUNTER.fetch_add(1, Ordering::SeqCst);
        let id = format!("cred-{}", counter);
        Ok(VerifiedCredential {
            id,
            user_id: trimmed.to_string(),
            scope,
            expiry,
            _state: PhantomData,
        })
    }

    /// Transition credential from Issued to Active state.
    ///
    /// `as_of` is the moment the activation is evaluated against (normally the
    /// current time, injected so this is testable and deterministic).
    ///
    /// # Errors
    ///
    /// Returns `CredentialError::AlreadyExpired` if `as_of` is at or past the
    /// credential's expiry. An already-expired credential must never become
    /// Active — this is what ties the typestate to the clock at the boundary.
    pub fn activate(
        self,
        as_of: SystemTime,
    ) -> Result<VerifiedCredential<Active>, CredentialError> {
        if as_of >= self.expiry {
            return Err(CredentialError::AlreadyExpired);
        }
        Ok(self.transition())
    }
}

impl VerifiedCredential<Active> {
    /// Return the verified user ID (opaque identifier).
    /// Only callable on Active credentials.
    pub fn user_id(&self) -> &str {
        &self.user_id
    }

    /// Return the credential ID.
    ///
    /// This is a **unique, opaque, monotonic** identifier (`cred-<n>` from a
    /// process-wide counter). It does not encode the user_id.
    ///
    /// Note: it is *not* unpredictable. Because issuance is a monotonic counter,
    /// IDs are enumerable, so `credential_id` must NOT be used as an unguessable
    /// security token or capability. Making it unpredictable (e.g. a random UUID)
    /// is deferred: it would require a CSPRNG dependency, and the monotonic
    /// ordering is currently relied on for audit trails. Only callable on Active
    /// credentials.
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

    /// Check whether the credential has expired as of the given time.
    ///
    /// `as_of` is injected (rather than read from the clock internally) so this
    /// is pure and testable. An Active credential can be expired-by-clock: the
    /// typestate cannot freeze wall-clock time, so callers re-check validity at
    /// use via this method or `authority_weight`.
    pub fn is_expired(&self, as_of: SystemTime) -> bool {
        as_of >= self.expiry
    }

    /// Transition credential from Active to Expired state.
    ///
    /// `as_of` is the moment the transition is evaluated against. `expire`
    /// succeeds only when the credential is actually past expiry, so the
    /// `Expired` typestate cannot be used to relabel a still-valid credential.
    ///
    /// # Errors
    ///
    /// Returns `CredentialError::NotYetExpired` if `as_of` is before expiry.
    pub fn expire(self, as_of: SystemTime) -> Result<VerifiedCredential<Expired>, CredentialError> {
        if as_of < self.expiry {
            return Err(CredentialError::NotYetExpired);
        }
        Ok(self.transition())
    }

    /// Compute authority weight as a pure function of (scope, freshness).
    ///
    /// `as_of` is injected, making this a **pure, deterministic** function: the
    /// same `(scope, expiry, as_of)` always produces the same output, with no
    /// hidden clock reads. It combines:
    /// - **Scope base weight**: per-scope, via `scope_base_weight` (provisional).
    /// - **Freshness decay**: linear in time-to-expiry over a one-year window
    ///   (1+ year out → multiplier 1.0; at expiry → 0.0).
    ///
    /// Returns 0.0 once `as_of` is at or past expiry. This is reachable for an
    /// Active credential because time passes after activation.
    pub fn authority_weight(&self, as_of: SystemTime) -> f64 {
        const FRESHNESS_WINDOW_SECS: u64 = 365 * 24 * 3600; // 1 year

        let time_to_expiry = match self.expiry.duration_since(as_of) {
            Ok(duration) => duration.as_secs_f64(),
            Err(_) => return 0.0, // expired as of `as_of`
        };

        let freshness_window = FRESHNESS_WINDOW_SECS as f64;
        let freshness_multiplier = (time_to_expiry / freshness_window).clamp(0.0, 1.0);

        scope_base_weight(self.scope) * freshness_multiplier
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    /// Issue and activate a test credential valid for one hour from `now`.
    fn issue_test_credential(user_id: &str) -> Result<VerifiedCredential<Active>, CredentialError> {
        let now = SystemTime::now();
        let expiry = now + Duration::from_secs(3600);
        VerifiedCredential::issue(user_id.to_string(), CredentialScope::Engineering, expiry)?
            .activate(now)
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
        let expiry = SystemTime::now() + Duration::from_secs(3600);
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
        let now = SystemTime::now();
        let expiry = now + Duration::from_secs(3600);
        let issued =
            VerifiedCredential::issue("user-123".to_string(), CredentialScope::Clinical, expiry)
                .unwrap();
        let active = issued.activate(now).unwrap();
        assert_eq!(active.user_id(), "user-123");
        assert_eq!(active.scope(), CredentialScope::Clinical);
        // Can only expire once the clock is past expiry.
        let _expired = active.expire(expiry + Duration::from_secs(1)).unwrap();
    }

    #[test]
    fn activate_rejects_already_expired_credential() {
        let now = SystemTime::now();
        let expiry = now - Duration::from_secs(1); // already in the past
        let issued =
            VerifiedCredential::issue("user".to_string(), CredentialScope::Engineering, expiry)
                .unwrap();
        assert_eq!(issued.activate(now), Err(CredentialError::AlreadyExpired));
    }

    #[test]
    fn expire_rejects_still_valid_credential() {
        let now = SystemTime::now();
        let expiry = now + Duration::from_secs(3600);
        let cred =
            VerifiedCredential::issue("user".to_string(), CredentialScope::Engineering, expiry)
                .unwrap()
                .activate(now)
                .unwrap();
        assert_eq!(cred.expire(now), Err(CredentialError::NotYetExpired));
    }

    #[test]
    fn expire_succeeds_once_past_expiry() {
        let now = SystemTime::now();
        let expiry = now + Duration::from_secs(60);
        let cred =
            VerifiedCredential::issue("user".to_string(), CredentialScope::Engineering, expiry)
                .unwrap()
                .activate(now)
                .unwrap();
        assert!(cred.expire(expiry + Duration::from_secs(1)).is_ok());
    }

    #[test]
    fn credential_carries_scope() {
        let scopes = [
            CredentialScope::Clinical,
            CredentialScope::Engineering,
            CredentialScope::Research,
        ];
        let now = SystemTime::now();
        let expiry = now + Duration::from_secs(3600);
        for scope in scopes.iter() {
            let cred = VerifiedCredential::issue("user".to_string(), *scope, expiry)
                .unwrap()
                .activate(now)
                .unwrap();
            assert_eq!(cred.scope(), *scope);
        }
    }

    #[test]
    fn credential_expiry_not_expired_in_future() {
        let now = SystemTime::now();
        let expiry = now + Duration::from_secs(3600);
        let cred =
            VerifiedCredential::issue("user".to_string(), CredentialScope::Engineering, expiry)
                .unwrap()
                .activate(now)
                .unwrap();
        assert!(!cred.is_expired(now));
    }

    #[test]
    fn credential_is_expired_after_expiry_passes() {
        // Activate while valid, then evaluate is_expired at a time past expiry.
        let now = SystemTime::now();
        let expiry = now + Duration::from_secs(60);
        let cred =
            VerifiedCredential::issue("user".to_string(), CredentialScope::Engineering, expiry)
                .unwrap()
                .activate(now)
                .unwrap();
        assert!(cred.is_expired(expiry + Duration::from_secs(1)));
    }

    #[test]
    fn authority_weight_full_for_year_away() {
        let now = SystemTime::now();
        let expiry = now + Duration::from_secs(365 * 24 * 3600);
        let cred =
            VerifiedCredential::issue("user".to_string(), CredentialScope::Engineering, expiry)
                .unwrap()
                .activate(now)
                .unwrap();
        // Engineering base 1.0 * freshness 1.0 = 1.0 exactly (pure function).
        assert!((cred.authority_weight(now) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn authority_weight_decays_with_freshness() {
        // Engineering credential expiring in 6 months (half the freshness window).
        let now = SystemTime::now();
        let expiry = now + Duration::from_secs(6 * 30 * 24 * 3600);
        let cred =
            VerifiedCredential::issue("user".to_string(), CredentialScope::Engineering, expiry)
                .unwrap()
                .activate(now)
                .unwrap();
        let weight = cred.authority_weight(now);
        assert!(weight > 0.4 && weight < 0.6, "half-window should be ~0.5");
    }

    #[test]
    fn authority_weight_approaches_zero_near_expiry() {
        let now = SystemTime::now();
        let expiry = now + Duration::from_secs(24 * 3600); // 1 day
        let cred =
            VerifiedCredential::issue("user".to_string(), CredentialScope::Engineering, expiry)
                .unwrap()
                .activate(now)
                .unwrap();
        // 1/365 of the window ≈ 0.0027.
        assert!(cred.authority_weight(now) < 0.01);
    }

    #[test]
    fn authority_weight_is_zero_at_and_past_expiry() {
        let now = SystemTime::now();
        let expiry = now + Duration::from_secs(60);
        let cred =
            VerifiedCredential::issue("user".to_string(), CredentialScope::Engineering, expiry)
                .unwrap()
                .activate(now)
                .unwrap();
        assert_eq!(cred.authority_weight(expiry), 0.0);
        assert_eq!(cred.authority_weight(expiry + Duration::from_secs(1)), 0.0);
    }

    #[test]
    fn authority_weight_is_deterministic() {
        // Pure function: identical inputs (including the same as_of) are EXACTLY
        // equal — no floating-point tolerance needed now that the clock is injected.
        let now = SystemTime::now();
        let expiry = now + Duration::from_secs(30 * 24 * 3600);
        let cred =
            VerifiedCredential::issue("user".to_string(), CredentialScope::Engineering, expiry)
                .unwrap()
                .activate(now)
                .unwrap();
        assert_eq!(cred.authority_weight(now), cred.authority_weight(now));
    }

    #[test]
    fn authority_weight_varies_by_scope() {
        // Same expiry and same as_of: scope is now a real input, so the three
        // scopes produce distinct weights (Engineering > Research > Clinical).
        let now = SystemTime::now();
        let expiry = now + Duration::from_secs(90 * 24 * 3600);
        let mk = |scope| {
            VerifiedCredential::issue("user".to_string(), scope, expiry)
                .unwrap()
                .activate(now)
                .unwrap()
        };
        let clinical = mk(CredentialScope::Clinical).authority_weight(now);
        let engineering = mk(CredentialScope::Engineering).authority_weight(now);
        let research = mk(CredentialScope::Research).authority_weight(now);

        assert!(engineering > research, "engineering must outweigh research");
        assert!(research > clinical, "research must outweigh clinical");
        // Ratios reflect the provisional scope base weights (1.0 / 0.9 / 0.8).
        assert!((research / engineering - 0.90).abs() < 1e-9);
        assert!((clinical / engineering - 0.80).abs() < 1e-9);
    }

    #[test]
    fn authority_weight_clamped_to_one_for_far_future() {
        let now = SystemTime::now();
        let expiry = now + Duration::from_secs(5 * 365 * 24 * 3600); // 5 years
        let cred =
            VerifiedCredential::issue("user".to_string(), CredentialScope::Engineering, expiry)
                .unwrap()
                .activate(now)
                .unwrap();
        // Clamped: Engineering base 1.0 * clamp(5.0, .., 1.0) = 1.0.
        assert!((cred.authority_weight(now) - 1.0).abs() < 1e-9);
    }
}
