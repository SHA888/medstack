//! Generic identity-verification adapter supporting ORCID, institutional email, and manual review.
//!
//! This adapter implements the `CredentialPort` trait, verifying users through multiple
//! channels:
//! - **ORCID verification**: User has a registered ORCID profile
//! - **Institutional email**: User's email is from a trusted institution (institution domain)
//! - **Manual review**: Explicitly approved users (e.g., by administrators or admissions)
//!
//! Each verification method is independent; a user must pass at least one method to be verified.

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
#[derive(Clone, Debug)]
pub struct GenericAdapter {
    /// Map of user_id -> ORCID for verified ORCID users
    orcid_registry: HashMap<String, String>,
    /// Configuration for ORCID verification
    orcid_config: Option<OrcidConfig>,
    /// Map of email -> user_id for institutional email verification
    institutional_emails: HashMap<String, String>,
    /// Configuration for institutional email verification
    institution_config: Option<InstitutionConfig>,
    /// Configuration for manual review
    manual_review_config: Option<ManualReviewConfig>,
    /// Credential validity duration (e.g., 1 year)
    credential_duration: Duration,
}

impl GenericAdapter {
    /// Create a new generic adapter with default settings.
    ///
    /// By default, the adapter has no verification methods enabled. Use the builder
    /// methods to configure verification methods.
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
    /// This allows the adapter to recognize the user as verified via ORCID.
    pub fn register_orcid(&mut self, user_id: String, orcid: String) {
        self.orcid_registry.insert(user_id, orcid);
    }

    /// Register a user with an institutional email.
    ///
    /// This allows the adapter to recognize the user as verified via institutional email.
    pub fn register_institutional_email(&mut self, user_id: String, email: String) {
        self.institutional_emails.insert(email, user_id);
    }

    /// Verify a user through ORCID.
    ///
    /// Returns true if:
    /// - ORCID verification is enabled
    /// - The user has a registered ORCID profile
    fn verify_orcid(&self, user_id: &str) -> bool {
        self.orcid_config.is_some() && self.orcid_registry.contains_key(user_id)
    }

    /// Verify a user through institutional email.
    ///
    /// For now, this is a stub. In a full implementation, it would:
    /// - Extract the domain from the user_id (if it's an email)
    /// - Check if the domain matches the institutional domain
    /// - Or check a registered email list
    fn verify_institutional_email(&self, user_id: &str) -> bool {
        if self.institution_config.is_none() {
            return false;
        }

        // Check if the user_id is registered as an institutional email
        self.institutional_emails.values().any(|uid| uid == user_id)
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
    /// Returns the scope if the user is verified, or None if not verified.
    fn get_scope(&self, user_id: &str) -> Option<CredentialScope> {
        // Check verification methods in order (ORCID > Institutional > Manual)
        if self.verify_orcid(user_id) {
            self.orcid_config.as_ref().map(|c| c.scope)
        } else if self.verify_institutional_email(user_id) {
            self.institution_config.as_ref().map(|c| c.scope)
        } else if self.verify_manual_review(user_id) {
            self.manual_review_config.as_ref().map(|c| c.scope)
        } else {
            None
        }
    }
}

impl CredentialPort for GenericAdapter {
    fn verify_credential(&self, user_id: UserId) -> Option<AuthoritySnapshot> {
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
    fn adapter_verifies_institutional_email_user() {
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
}
