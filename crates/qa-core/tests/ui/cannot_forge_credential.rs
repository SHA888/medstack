// This file intentionally tries to import VerifiedCredential from identity-verification.
// It should fail to compile because qa-core has no dependency on identity-verification.
// This architectural constraint (verified by the architecture test in ci.yml) prevents
// qa-core from forging credentials. The compile-fail test documents this boundary.

use identity_verification::VerifiedCredential;

fn main() {
    let cred = VerifiedCredential::issue("user-123".to_string()).ok();
    let _ = cred;
}
