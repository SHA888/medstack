# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project status

**medstack** is a credential-gated, durable Q&A corpus for clinician-engineers (people holding both a clinical license and shipping software). Currently in **pre-code scaffold (v0.0.0)**.

**✅ M0 LICENSE DECISION CLOSED:**
Decisions made and committed:
1. **Content license**: CC BY-SA 4.0 (Option A chosen). Rationale: SA virality is the commons-protection mechanism; Option B's quarantine is a taint-tracking correctness liability.
2. **Code license**: AGPL-3.0 (chosen). Rationale: Network-copyleft prevents closed forks; same logic as CC BY-SA protects the commons at the software layer.
3. Full rationale documented in `LICENSE` and `LICENSE-CONTENT.md`

See README.md §"The unresolved gate: LICENSE" for the historical tradeoff matrix.

## Architecture summary (Hexagonal, 4 bounded contexts)

The design deliberately separates concerns to allow independent failure, scaling, and reasoning. Read ARCHITECTURE.md for full details; key points:

### **qa-core**: the hexagon (question, answer, vote, tag logic)
- No knowledge of credential verification, ingestion, or search
- Dependency rule (CI-enforced): **only ports (traits) point outward; nothing imports from identity-verification, ingestion, or search**
- Every value object is "Parse-Don't-Validate": illegal states are unrepresentable
  - Empty question body → parse fails, never enters domain
  - Unknown license on content → exhaustive `match`, no catch-all
  - Forged credential weight → impossible at type level (private constructor)

### **identity-verification**: the trust boundary
- Produces `VerifiedCredential` tokens (opaque to qa-core, unforgeable)
- Per-jurisdiction adapters (Indonesia STR/KKI, US NPI, generic ORCID+email+manual)
- Scope + expiry as first-class; authority weight = pure function of (scope, freshness)

### **ingestion**: license-aware mirroring (bootstrap Q&A corpus)
- License carried as **data on every record** (`CcBySa4`, `CcBy4`, `Native`, `LinkOnly`)
- Stack Exchange (CC BY-SA), Biostars (CC BY) mirrored with attribution; FHIR Zulip link-only (no copy)
- All content under uniform CC BY-SA 4.0 (Option A chosen); no quarantine needed
- Attribution rendering non-optional and non-strippable (test-enforced): source, author, license, date, link

### **search**: read-side projection (full-text + tag facets)
- No writes to qa-core; independent scaling
- Surfaces staleness via date/jurisdiction facets

## Key design principles

1. **Parse-Don't-Validate**: Inbound payloads (question, answer, imported row, credential) parsed at boundary into types that cannot represent illegal state. Unknown enum value → failure. Empty string where `NonEmpty` required → compile error.

2. **License as data**: Every content record carries its source license (not a comment). Structural enforcement: rendering and reuse paths respect partition boundaries if Option B chosen.

3. **Credential opacity**: `VerifiedCredential` unforgeable by qa-core (private constructor in verification crate). Compile-fail test (`trybuild`) asserts qa-core cannot construct one.

4. **Perishability handled as metadata**: Versioned answers, date/jurisdiction tag facets, lightweight "still valid?" community signal. Explicit *deferral* of automated decay/context-envelope models (not deleted, documented).

5. **Principles need CI**: Coupling rules, SemVer, formatting, type-safety, attribution non-strippability all enforced by CI checks. A convention without a red build is a wish.

## Stack (when M0 closes)

- **Rust**: `qa-core`, `identity-verification`, `search` indexing glue
  - Workspace per domain boundary, library-first, binary-last
  - `cargo install cargo-semver-checks`
  - Compiler-enforced newtypes for IDs, exhaustive enums, `Send`/`Sync` where serving

- **TypeScript (pnpm)**: web client
  - pnpm workspaces mirror domain boundaries
  - Discriminated unions + branded types + `readonly`/`as const`
  - Zod/Valibot at I/O boundaries

- **Python (uv)**: ingestion/ETL only
  - Dump parsers for SE, Biostars
  - Parse-Don't-Validate at boundaries (Pydantic/msgspec)

- **Constrained-machine friendly**: single-binary SQLite deployment for self-hosting; Postgres for hosted

## Development commands (M0+)

Once workspaces exist (task 0.3), the following will be standard:

**Rust:**
```bash
cargo check
cargo test
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings  # `--` separates lint flags from cargo args
cargo semver-checks  # SemVer compliance before tags (needs a published baseline)
cargo deny check     # Dependency audit; reads deny.toml at workspace root
```

**TypeScript:**
```bash
cd web
pnpm install
pnpm exec tsc --noEmit   # Type check
pnpm exec eslint .       # Lint (no --fix in CI: a gate checks, it does not mutate)
pnpm test
```

**Python:**
```bash
cd ingestion
uv sync
uv run ruff check          # add --fix locally only; CI runs check-only
uv run mypy .
uv run pytest
```

**Architecture test (enforces coupling rule):**
```bash
cargo test --package qa-core --test architecture
# Integration test (tests/architecture.rs); asserts qa-core has zero outward deps.
# NOT --lib: the test lives in tests/, so --lib would match nothing and pass vacuously.
```

## Before writing any code

- [ ] M0.1: Content license (A/B) and code license (AGPL/Apache/MIT) decided and ratified
- [ ] M0.2: `LICENSE` and `LICENSE-CONTENT.md` files written
- [ ] M0.3: Workspace stubs (Cargo, pnpm, uv) created
- [ ] M0.3: CI configured with all lints, `cargo-semver-checks`, `cargo-deny`
- [ ] M0.4: On-topic scope document drafted (patient-advice OUT, clinical-software IN)
- [ ] M0.5: Architecture test harness in place asserting no outward deps from qa-core

## How the design resists unwanted states

**Forged credentials**: `qa-core` cannot construct `VerifiedCredential` (private constructor in `identity-verification` crate). The compiler prevents any code from building if it tries. Test: compile-fail test via `trybuild`.

**Invalid licenses**: `License` enum has no catch-all; Rust exhaustiveness checking forces handling of every variant. Unknown license in import → parse error. Test: assert unknown license fails to parse.

**Credential leakage**: `qa-core` receives only opaque token, never raw credential data. Type system prevents reading fields that don't exist. Audit: grep qa-core for `VerifiedCredential::` — should only call `authority_weight()` and similar public methods.

**Attribution non-strippability**: Rendering code always includes source, author, license, date, and link (no conditional paths). Type-enforced at render sites.

**Attribution stripping**: Every mirrored record carries source, author, license as non-optional fields. Rendering code is forbidden to silently drop them (test asserts all rendered items emit source+author+license).

**Coupling violation**: `cargo-deny` + architecture test asserts `qa-core` has zero outward dependencies. CI fails if violated.

## Deferred designs (documented, not deleted)

The following are real alternatives, parked intentionally. Do not remove them; update their rationale if circumstances change:

- **Automated perishability decay / context-validity envelopes** — deferred because "versioned answers + date/jurisdiction facets + still-valid signal" is cheaper. Revisit if data shows staleness is a real problem.
- **Provenance graphs** — deferred until the corpus is large enough to justify.
- **Advanced jurisdiction adapters** (NPI, STR) — only stubs in M2; full implementation only if pilot shows demand.

## Common mistakes to avoid

1. **License decision is closed.** CC BY-SA 4.0 (content) + AGPL-3.0 (code) were chosen. Do not add quarantine logic—it was explicitly rejected. See LICENSE-CONTENT.md for rationale.

2. **Don't make `VerifiedCredential` forgeable by qa-core.** The constructor must stay private to `identity-verification`. If qa-core needs to create one, the design has a hole.

3. **Don't catch-all on license enum.** Use exhaustive pattern matching:
   ```rust
   match content.license {
       License::CcBySa4 => ...,
       License::CcBy4 => ...,
       License::Native(variant) => ...,
       License::LinkOnly => ...,
   }
   ```
   If you want a fallback, that's a design question, not a code question.

4. **Don't strip attribution from mirrored content.** The test suite will catch it, but it's a patient-safety and legal boundary.

5. **Don't add qa-core dependencies on `identity-verification` or `ingestion`.** The architecture test will fail CI. If you need to, re-examine whether the concern genuinely belongs in qa-core.

## Commit and documentation conventions

**Do not include `Co-Authored-By:` trailers in commit messages.** This applies to all assistant-generated commits, including those produced by Claude Code or any other AI tool. Commit attribution stays with the human author. Boilerplate trailers add noise to the history without conveying meaningful authorship and have been retroactively stripped from past commits.

**English-only requirement:**
- All Plans.md content must be in English (headers, table columns, task descriptions, status markers).
- No Japanese characters in Plans.md status markers (use `cc:done` instead of `cc:完了`, `cc:wip` instead of `cc:WIP`, etc).
- All harness output and documentation must be in English.
- This applies strictly to tracked files; commit to this constraint when editing Plans.md.

## References

- **README.md**: Three load-bearing product decisions, bootstrap strategy, LICENSE gate
- **ARCHITECTURE.md**: Bounded contexts, ports/adapters, domain framing, perishability model, scope & safety boundary
- **TODO.md**: Atomic backlog by SemVer milestone; M0 must close before M1 starts
