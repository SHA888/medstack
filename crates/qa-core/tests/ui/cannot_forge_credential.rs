// This fixture documents the FIRST line of defense: qa-core has no dependency on
// identity-verification (enforced by tests/architecture.rs), so it cannot even
// name `VerifiedCredential`. The import below fails to resolve (E0432), which is
// exactly the boundary this fixture asserts: the credential type is unreachable
// from qa-core, therefore unforgeable here.
//
// The SECOND line of defense — that `issue` is private even to code that *can*
// link the crate — is tested separately in
// identity-verification/tests/ui/cannot_call_issue.rs (E0624), because that
// requires the dependency this fixture proves qa-core does not have.

use identity_verification::VerifiedCredential;

fn main() {
    // Unreachable: the import above does not resolve from qa-core.
    let _: Option<VerifiedCredential> = None;
}
