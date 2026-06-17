// This fixture links identity-verification as an external crate and attempts to
// call VerifiedCredential::issue with the CORRECT arity. It must fail to compile
// because `issue` is `pub(crate)` — only this crate may construct a credential.
// This is the actual credential-forgery guard (E0624: associated function is
// private), distinct from the qa-core fixture which fails at the import (no dep).

use identity_verification::{CredentialScope, VerifiedCredential};
use std::time::SystemTime;

fn main() {
    let _forged = VerifiedCredential::issue(
        "forged-user".to_string(),
        CredentialScope::Engineering,
        SystemTime::now(),
    );
}
