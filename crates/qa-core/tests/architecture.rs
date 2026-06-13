//! Architecture invariant test (task 0.3.5).
//!
//! qa-core is the hexagon: it may depend on nothing in this workspace. Only
//! ports (traits) point outward; nothing in qa-core may import from
//! identity-verification, ingestion, or search.
//!
//! This is an integration test (lives in `tests/`), so cargo sets the current
//! working directory to the package root (`crates/qa-core/`) when it runs.
//! We read this crate's own `Cargo.toml` and assert that none of the forbidden
//! sibling crates appear as dependencies.
//!
//! Run with: `cargo test -p qa-core --test architecture`

/// Crates qa-core must never depend on.
const FORBIDDEN_DEPS: [&str; 3] = ["identity-verification", "ingestion", "search"];

/// Extract dependency names from the `[dependencies]`, `[dev-dependencies]`,
/// and `[build-dependencies]` sections of a Cargo.toml string.
///
/// Minimal parser sufficient for stub manifests: collects the key on the left
/// of `key = ...` and `key.workspace = true` style lines, plus `[dependencies.foo]`
/// table headers, within any dependency section.
fn dependency_names(manifest: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut in_dep_section = false;

    for raw in manifest.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line.starts_with('[') {
            // A `[dependencies.foo]` style header names a dependency directly.
            if let Some(inner) = line.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                let inner = inner.trim();
                let dep_table = inner
                    .strip_prefix("dependencies.")
                    .or_else(|| inner.strip_prefix("dev-dependencies."))
                    .or_else(|| inner.strip_prefix("build-dependencies."));
                if let Some(name) = dep_table {
                    names.push(name.trim().to_string());
                    in_dep_section = false;
                    continue;
                }
                in_dep_section = matches!(
                    inner,
                    "dependencies" | "dev-dependencies" | "build-dependencies"
                );
            }
            continue;
        }

        if in_dep_section {
            // `serde = "1"` or `qa-core.workspace = true` → key is before `=` or `.`.
            let key = line
                .split('=')
                .next()
                .unwrap_or("")
                .split('.')
                .next()
                .unwrap_or("")
                .trim()
                .trim_matches('"');
            if !key.is_empty() {
                names.push(key.to_string());
            }
        }
    }

    names
}

#[test]
fn qa_core_has_zero_outward_deps() {
    let manifest = std::fs::read_to_string("Cargo.toml")
        .expect("qa-core/Cargo.toml must exist (test CWD is the package root)");

    let deps = dependency_names(&manifest);

    for forbidden in FORBIDDEN_DEPS {
        assert!(
            !deps.iter().any(|d| d == forbidden),
            "qa-core must not depend on `{forbidden}` (found in qa-core/Cargo.toml). \
             Only ports (traits) point outward; nothing may import from \
             identity-verification, ingestion, or search."
        );
    }
}

#[test]
fn dependency_parser_detects_forbidden_deps() {
    // Guard the parser itself: a manifest that DOES list a forbidden crate must
    // be detected, otherwise the invariant test above could pass vacuously.
    let bad = "\
[package]
name = \"qa-core\"

[dependencies]
serde = \"1\"
ingestion = { path = \"../ingestion\" }
";
    let deps = dependency_names(bad);
    assert!(deps.iter().any(|d| d == "ingestion"));
    assert!(deps.iter().any(|d| d == "serde"));

    // Table-header form is detected too.
    let bad_table = "\
[dependencies.search]
path = \"../search\"
";
    assert!(dependency_names(bad_table).iter().any(|d| d == "search"));
}
