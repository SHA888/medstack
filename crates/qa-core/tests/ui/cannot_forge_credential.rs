// This file intentionally tries to import VerifiedCredential from qa-core.
// It should fail to compile because qa-core has no dependency on identity-verification.
// This architectural constraint prevents qa-core from forging credentials.

use identity_verification::VerifiedCredential;

fn main() {
    let cred = VerifiedCredential::issue("user-123".to_string());
    let _ = cred;
}
