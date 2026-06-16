//! Port definitions: interfaces to external systems.
//!
//! Ports are the boundaries of the qa-core hexagon. They define what services
//! qa-core needs from external systems (identity-verification, search, persistence)
//! without depending directly on those systems. Adapters implement these ports.
//!
//! This enforces the architectural rule: only ports point outward; the domain logic
//! itself has zero outward dependencies.

use crate::domain::id::UserId;

/// Port for credential verification.
///
/// This port defines how qa-core requests verification of user credentials.
/// The identity-verification crate implements this port, returning opaque
/// VerifiedCredential tokens that qa-core cannot forge.
///
/// Implementations must:
/// - Return `Some(credential)` if the user has been verified
/// - Return `None` if the user is not verified or does not exist
/// - Never return an invalid or expired credential (type system enforces this)
pub trait CredentialPort {
    /// Verify a user by ID, returning an opaque credential token if verified.
    ///
    /// # Arguments
    /// - `user_id`: The user to verify
    ///
    /// # Returns
    /// - `Some(credential)`: User is verified and active
    /// - `None`: User is not verified, does not exist, or is inactive
    ///
    /// # Implementation notes
    /// Implementers (identity-verification crate) are responsible for:
    /// - Checking user identity against external authority systems
    /// - Managing credential expiry and lifecycle
    /// - Issuing unforgeable tokens that qa-core cannot construct
    fn verify_credential(&self, user_id: UserId) -> Option<()>;
    // TODO: When identity-verification is available in M2, replace Option<()>
    // with Option<VerifiedCredential<Active>>. Keeping as () for now to avoid
    // circular dependencies during M1 (library-only phase).
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mock credential port for testing qa-core logic.
    struct MockCredentialPort {
        verified_users: std::collections::HashSet<u64>,
    }

    impl MockCredentialPort {
        fn new(verified_users: Vec<u64>) -> Self {
            MockCredentialPort {
                verified_users: verified_users.into_iter().collect(),
            }
        }
    }

    impl CredentialPort for MockCredentialPort {
        fn verify_credential(&self, user_id: UserId) -> Option<()> {
            if self.verified_users.contains(&user_id.inner()) {
                Some(())
            } else {
                None
            }
        }
    }

    #[test]
    fn mock_port_returns_none_for_unverified() {
        let port = MockCredentialPort::new(vec![1, 2, 3]);
        assert_eq!(port.verify_credential(UserId::new(1)), Some(()));
        assert_eq!(port.verify_credential(UserId::new(999)), None);
    }

    #[test]
    fn mock_port_verifies_users() {
        let port = MockCredentialPort::new(vec![42, 100]);
        assert_eq!(port.verify_credential(UserId::new(42)), Some(()));
        assert_eq!(port.verify_credential(UserId::new(100)), Some(()));
        assert_eq!(port.verify_credential(UserId::new(50)), None);
    }
}
