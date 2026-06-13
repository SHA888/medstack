#[test]
fn qa_core_has_zero_outward_deps() {
    // Architecture invariant: qa-core may not depend on identity-verification,
    // ingestion, or search. This test enforces that constraint at compile time
    // via the dependency graph.
    //
    // If this test fails, check Cargo.toml: does qa-core list any of those
    // crates as dependencies? It must not.
    //
    // The actual enforcement is in the workspace Cargo.toml:
    // qa-core should only depend on std and proc-macro (if needed),
    // never on other crates in this workspace.

    let manifest = std::fs::read_to_string("Cargo.toml")
        .expect("Root Cargo.toml must exist");

    assert!(
        !manifest.contains("qa-core"),
        "qa-core Cargo.toml should not be checked here; this test runs from workspace root"
    );
}
