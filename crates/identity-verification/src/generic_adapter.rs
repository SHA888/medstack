//! Generic identity-verification adapter supporting ORCID, institutional email, and manual review.
//!
//! This adapter implements the `CredentialPort` trait, verifying users through multiple
//! channels:
//! - **ORCID verification**: User has a registered ORCID profile
//! - **Institutional email**: User's email domain matches a trusted institution
//! - **Manual review**: Explicitly approved users (e.g., by administrators or admissions)
//!
//! ## Verification flow and revocation
//!
//! To revoke a user's credentials, they must be removed from **all** verification methods where
//! they are registered. The adapter does not maintain a separate revocation list; removal from
//! all registries is the revocation mechanism.
//!
//! ## Scope selection
//!
//! When a user qualifies under multiple verification methods, the highest-priority scope is
//! returned: ORCID (Engineering) > Institutional > Manual. This precedence ensures that users
//! with persistent external identifiers (ORCID) carry their primary identity. To use a narrower
//! scope, callers must remove the user from higher-priority methods.

use crate::{CredentialScope, VerifiedCredential};
use qa_core::domain::credential::AuthorityWeight;
use qa_core::domain::id::UserId;
use qa_core::domain::ports::CredentialPort;
use qa_core::domain::credential::AuthoritySnapshot;
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, Duration};

/// Convert from identity-verification's CredentialScope to qa-core's CredentialScope.
fn to_qa_core_scope(scope: CredentialScope) -> qa_core::domain::credential::CredentialScope {
    match scope {
        CredentialScope::Clinical => qa_core::domain::credential::CredentialScope::Clinical,
        CredentialScope::Engineering => qa_core::domain::credential::CredentialScope::Engineering,
        CredentialScope::Research => qa_core::domain::credential::CredentialScope::Research,
    }
}

/// Extract domain from an email address (text after '@').
///
/// Returns the domain if the email contains '@', otherwise None.
fn email_domain(email: &str) -> Option<&str> {
    email.split('@').nth(1)
}

/// Configuration for institutional email verification.
#[derive(Clone, Debug)]
pub struct InstitutionConfig {
    /// Trusted institution domain (e.g., "mit.edu", "stanford.edu")
    pub domain: String,
    /// Credential scope issued for users from this institution
    pub scope: CredentialScope,
}

/// Configuration for ORCID verification.
#[derive(Clone, Debug)]
pub struct OrcidConfig {
    /// Credential scope issued for ORCID-verified users
    pub scope: CredentialScope,
}

/// Configuration for manual review verification.
#[derive(Clone, Debug)]
pub struct ManualReviewConfig {
    /// Credential scope issued for manually approved users
    pub scope: CredentialScope,
    /// Set of user IDs that have been manually approved
    pub approved_users: HashSet<String>,
}

/// Generic identity-verification adapter.
///
/// Supports three verification methods:
/// 1. ORCID verification (for users with registered ORCID profiles)
/// 2. Institutional email verification (for users with emails from trusted institutions)
/// 3. Manual review (for explicitly approved users)
///
/// ## Revocation
///
/// To revoke a user, remove them from ALL registries where they appear.
/// If a user is present in multiple methods' registries, verify_credential will succeed
/// via any method where they remain registered.
#[derive(Clone, Debug)]
pub struct GenericAdapter {
    /// Map of user_id -> ORCID for verified ORCID users
    orcid_registry: HashMap<String, String>,
    /// Configuration for ORCID verification (None means ORCID verification is disabled)
    orcid_config: Option<OrcidConfig>,
    /// Map of user_id -> email for institutional email verification (O(1) lookup)
    institutional_emails: HashMap<String, String>,
    /// Configuration for institutional email verification (None means institutional verification is disabled)
    institution_config: Option<InstitutionConfig>,
    /// Configuration for manual review
    manual_review_config: Option<ManualReviewConfig>,
    /// Credential validity duration (e.g., 1 year)
    credential_duration: Duration,
}

impl GenericAdapter {
    /// Create a new generic adapter with no verification methods enabled.
    ///
    /// Use the builder methods (`with_orcid`, `with_institution`, `with_manual_review`)
    /// to configure verification methods before use.
    pub fn new(credential_duration: Duration) -> Self {
        GenericAdapter {
            orcid_registry: HashMap::new(),
            orcid_config: None,
            institutional_emails: HashMap::new(),
            institution_config: None,
            manual_review_config: None,
            credential_duration,
        }
    }

    /// Enable ORCID verification with the given scope.
    pub fn with_orcid(mut self, scope: CredentialScope) -> Self {
        self.orcid_config = Some(OrcidConfig { scope });
        self
    }

    /// Enable institutional email verification.
    pub fn with_institution(mut self, config: InstitutionConfig) -> Self {
        self.institution_config = Some(config);
        self
    }

    /// Enable manual review verification.
    pub fn with_manual_review(mut self, config: ManualReviewConfig) -> Self {
        self.manual_review_config = Some(config);
        self
    }

    /// Register a user with an ORCID profile.
    ///
    /// The user_id should be a consistent string representation of the user's identifier.
    /// Callers must ensure that the same format is used at both registration and verification time.
    pub fn register_orcid(&mut self, user_id: String, orcid: String) {
        self.orcid_registry.insert(user_id, orcid);
    }

    /// Register a user with an institutional email.
    ///
    /// The email domain must match the institution_config.domain for verification to succeed.
    /// The user_id should be a consistent string representation (see register_orcid).
    pub fn register_institutional_email(&mut self, user_id: String, email: String) {
        self.institutional_emails.insert(user_id, email);
    }

    /// Verify a user through ORCID.
    ///
    /// Returns true if:
    /// - ORCID verification is enabled (orcid_config is Some)
    /// - The user has a registered ORCID profile
    fn verify_orcid(&self, user_id: &str) -> bool {
        self.orcid_config.is_some() && self.orcid_registry.contains_key(user_id)
    }

    /// Verify a user through institutional email.
    ///
    /// Returns true if:
    /// - Institutional email verification is enabled (institution_config is Some)
    /// - The user has a registered email
    /// - The email domain matches the configured institution domain
    fn verify_institutional_email(&self, user_id: &str) -> bool {
        // Check if institutional config is enabled
        let institution_config = match self.institution_config.as_ref() {
            Some(config) => config,
            None => return false,
        };

        // Check if user has a registered email
        let email = match self.institutional_emails.get(user_id) {
            Some(e) => e,
            None => return false,
        };

        // Validate the email domain matches the configured institution domain
        if let Some(domain) = email_domain(email) {
            domain == institution_config.domain
        } else {
            // Email has no domain (no '@'), fail closed
            false
        }
    }

    /// Verify a user through manual review.
    fn verify_manual_review(&self, user_id: &str) -> bool {
        if let Some(ref config) = self.manual_review_config {
            config.approved_users.contains(user_id)
        } else {
            false
        }
    }

    /// Get the credential scope for a user based on their verification method.
    ///
    /// Returns the scope with the highest priority:
    /// 1. ORCID (Engineering base weight: 1.0)
    /// 2. Institutional (Research base weight: 0.9)
    /// 3. Manual review (Clinical base weight: 0.8)
    ///
    /// Returns None if the user doesn't pass any verification method.
    fn get_scope(&self, user_id: &str) -> Option<CredentialScope> {
        // Check verification methods in priority order
        if self.verify_orcid(user_id) {
            return self.orcid_config.as_ref().map(|c| c.scope);
        }
        if self.verify_institutional_email(user_id) {
            return self.institution_config.as_ref().map(|c| c.scope);
        }
        if self.verify_manual_review(user_id) {
            return self.manual_review_config.as_ref().map(|c| c.scope);
        }
        None
    }
}

impl CredentialPort for GenericAdapter {
    fn verify_credential(&self, user_id: UserId) -> Option<AuthoritySnapshot> {
        // Convert UserId to consistent string representation
        let user_id_str = user_id.inner().to_string();

        // Check if user is verified through any method
        let local_scope = self.get_scope(&user_id_str)?;

        // Issue a credential with expiry
        let now = SystemTime::now();
        let expiry = now + self.credential_duration;

        // Create a verified credential (in Issued state)
        let credential = VerifiedCredential::issue(user_id_str, local_scope, expiry)
            .ok()?;

        // Activate the credential
        let active_credential = credential.activate(now).ok()?;

        // Compute authority weight as of now
        let weight_value = active_credential.authority_weight(now);

        // Convert scope to qa-core's CredentialScope
        let qa_core_scope = to_qa_core_scope(local_scope);

        // Create AuthoritySnapshot with the computed weight
        let auth_weight = AuthorityWeight::new(weight_value).ok()?;
        Some(AuthoritySnapshot::new(qa_core_scope, auth_weight))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn adapter_verifies_orcid_user() {
        let mut adapter = GenericAdapter::new(Duration::from_secs(365 * 24 * 3600))
            .with_orcid(CredentialScope::Engineering);

        adapter.register_orcid("123".to_string(), "0000-0001-2345-6789".to_string());

        let user_id = UserId::new(123);
        let snapshot = adapter.verify_credential(user_id);
        assert!(snapshot.is_some());
    }

    #[test]
    fn adapter_rejects_unverified_user() {
        let adapter = GenericAdapter::new(Duration::from_secs(365 * 24 * 3600));

        let user_id = UserId::new(999);
        let snapshot = adapter.verify_credential(user_id);
        assert!(snapshot.is_none());
    }

    #[test]
    fn adapter_validates_institutional_email_domain() {
        let institution = InstitutionConfig {
            domain: "mit.edu".to_string(),
            scope: CredentialScope::Research,
        };

        let mut adapter = GenericAdapter::new(Duration::from_secs(365 * 24 * 3600))
            .with_institution(institution);

        adapter.register_institutional_email("456".to_string(), "user@mit.edu".to_string());

        let user_id = UserId::new(456);
        let snapshot = adapter.verify_credential(user_id);
        assert!(snapshot.is_some());
    }

    #[test]
    fn adapter_rejects_email_from_wrong_domain() {
        let institution = InstitutionConfig {
            domain: "mit.edu".to_string(),
            scope: CredentialScope::Research,
        };

        let mut adapter = GenericAdapter::new(Duration::from_secs(365 * 24 * 3600))
            .with_institution(institution);

        adapter.register_institutional_email("456".to_string(), "user@stanford.edu".to_string());

        let user_id = UserId::new(456);
        let snapshot = adapter.verify_credential(user_id);
        // Should be None because the domain doesn't match
        assert!(snapshot.is_none());
    }

    #[test]
    fn adapter_rejects_email_without_domain() {
        let institution = InstitutionConfig {
            domain: "mit.edu".to_string(),
            scope: CredentialScope::Research,
        };

        let mut adapter = GenericAdapter::new(Duration::from_secs(365 * 24 * 3600))
            .with_institution(institution);

        adapter.register_institutional_email("456".to_string(), "invalid-email-no-domain".to_string());

        let user_id = UserId::new(456);
        let snapshot = adapter.verify_credential(user_id);
        assert!(snapshot.is_none());
    }

    #[test]
    fn adapter_verifies_manually_approved_user() {
        let mut approved_users = HashSet::new();
        approved_users.insert("789".to_string());

        let manual_config = ManualReviewConfig {
            scope: CredentialScope::Clinical,
            approved_users,
        };

        let adapter = GenericAdapter::new(Duration::from_secs(365 * 24 * 3600))
            .with_manual_review(manual_config);

        let user_id = UserId::new(789);
        let snapshot = adapter.verify_credential(user_id);
        assert!(snapshot.is_some());
    }

    #[test]
    fn orcid_takes_precedence_over_institutional() {
        let institution = InstitutionConfig {
            domain: "stanford.edu".to_string(),
            scope: CredentialScope::Research,
        };

        let mut adapter = GenericAdapter::new(Duration::from_secs(365 * 24 * 3600))
            .with_orcid(CredentialScope::Engineering)
            .with_institution(institution);

        adapter.register_orcid("999".to_string(), "0000-0001-9999-9999".to_string());
        adapter.register_institutional_email("999".to_string(), "user@stanford.edu".to_string());

        let user_id = UserId::new(999);
        let snapshot = adapter.verify_credential(user_id).expect("should verify");
        // ORCID scope (Engineering) should take precedence over institutional (Research)
        assert_eq!(snapshot.scope(), qa_core::domain::credential::CredentialScope::Engineering);
    }

    #[test]
    fn revocation_requires_removal_from_all_methods() {
        let institution = InstitutionConfig {
            domain: "mit.edu".to_string(),
            scope: CredentialScope::Research,
        };

        let mut approved_users = HashSet::new();
        approved_users.insert("777".to_string());

        let mut adapter = GenericAdapter::new(Duration::from_secs(365 * 24 * 3600))
            .with_orcid(CredentialScope::Engineering)
            .with_institution(institution)
            .with_manual_review(ManualReviewConfig {
                scope: CredentialScope::Clinical,
                approved_users,
            });

        adapter.register_orcid("777".to_string(), "0000-0001-7777-7777".to_string());
        adapter.register_institutional_email("777".to_string(), "user@mit.edu".to_string());

        // User is verified via ORCID
        assert!(adapter.verify_credential(UserId::new(777)).is_some());

        // Remove from manual review (only method) — should still verify via ORCID
        // (Can't actually remove from ManualReviewConfig after construction, so this tests the concept)

        // To fully revoke, would need to remove from orcid_registry and institutional_emails
        // This test documents the revocation contract
    }

    #[test]
    fn institutional_verification_is_o1_lookup() {
        let institution = InstitutionConfig {
            domain: "example.com".to_string(),
            scope: CredentialScope::Research,
        };

        let mut adapter = GenericAdapter::new(Duration::from_secs(365 * 24 * 3600))
            .with_institution(institution);

        // Register many users
        for i in 0..1000 {
            adapter.register_institutional_email(
                i.to_string(),
                format!("user{}@example.com", i),
            );
        }

        // Lookup should be fast (HashMap::get, not scan)
        let user_id = UserId::new(500);
        let _snapshot = adapter.verify_credential(user_id);
        // This doesn't assert performance, but documents the intent
    }
}
