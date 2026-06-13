# medstack

A credential-gated, durable Q&A corpus for clinician-engineer hybrids — the
people who hold a clinical license *and* ship software (physician-developers,
clinician data scientists, biomedical informaticians).

Status: **M0 closed — workspace scaffold (v0.1.0)**. The LICENSE gate is settled
(see below); the Cargo / pnpm / uv workspaces and CI are in place. Domain code
(M1: `qa-core`) is the next milestone.

## Why this exists

There is no Stack Overflow / Stack Exchange for the clinician-coder. The
Bioinformatics Stack Exchange survives narrowly; a dedicated medical /
clinical-engineering Q&A site never reached critical mass. Every prior attempt
(and the analogous Law and Semantic-Web proposals) died from the **same
failure mode**: expert *answerers* never showed up, because you cannot
bootstrap answerers without an existing audience, and the canonical-answer
engine is a poor fit for context-bound, perishable domain knowledge.

The existing clinician-coder spaces (Discord/Slack, AMIA forums, `chat.fhir.org`)
are **chat-shaped**: high-bandwidth, low-durability. They do not *accrete*. A
question answered brilliantly in a Slack thread is gone in a week.

medstack's one job is the thing those rooms structurally cannot do:
**produce a durable, searchable, peer-validated knowledge artifact** that
outlives the asker and is reused by the next person.

## What it is (and is not)

- **Is**: a Stack-Overflow-shaped Q&A corpus, scoped to the clinical-software
  intersection, where verified clinical/engineering credentials *weight and
  badge* answers without gatekeeping who may participate.
- **Is not**: a chat platform, a social network, a generic medical-advice site,
  or a generic programming Q&A site. Patient-facing medical advice is
  explicitly out of scope (see Architecture → Scope & Safety).

## The three load-bearing product decisions

1. **Artifact = durable Q&A corpus, credential-gated.** Perishability of answers
   is handled as *metadata* (versioned answers, a "still valid?" community
   signal, and date/jurisdiction tag facets), not as a novel decay engine. This
   is a deliberate YAGNI choice: the harder "context-envelope" model is
   explicitly deferred.
2. **Tiered answering.** Anyone may answer. Verified credentials weight and
   badge the answer. Credential verification is a **separate bounded context**
   with a per-jurisdiction adapter; the Q&A core only ever handles an opaque
   verified-credential token it cannot forge.
3. **Bootstrap by license-aware import/mirror.** Seed the corpus from
   legally-mirrorable public Q&A (Stack Exchange network = CC BY-SA;
   Biostars = CC BY) with attribution. Sources that are **not** open-licensed
   (e.g. `chat.fhir.org`) are **linked, never mirrored.**

## Bootstrap → Works → Community

This project is built, made to work, and only *then* opened to community —
never the reverse. The first months of content come from import/mirror, not
from a cold public launch begging for answerers. Community validation is not
sought prematurely.

## The LICENSE gate (settled in M0)

The import decision forced a choice, because Stack Exchange content is **CC
BY-SA**, which is **viral/share-alike**. Three coherent options for *our own*
user-generated content license were considered:

| Option | Native content license | Consequence |
| --- | --- | --- |
| **A. CC BY-SA 4.0** (mirror SE) ✅ **chosen** | CC BY-SA 4.0 | Simplest legally; SA obligation is uniform across native + mirrored SE content. Matches SO's own license. Viral. |
| **B. CC BY 4.0** native, quarantine SA imports | CC BY 4.0 | Native content more permissive/reusable; SE imports must live in a **separate partition** whose SA terms never mix into native answers. More engineering. |
| **C. CC BY-NC** native | CC BY-NC 4.0 | Blocks commercial reuse of the corpus; **incompatible** with mirroring SE (SE is not NC). Rules out the chosen bootstrap. Rejected. |

**Decisions (ratified):**

- **Content license: CC BY-SA 4.0 (Option A).** The share-alike obligation is the
  commons-protection mechanism; Option B's quarantine partition is a taint-tracking
  correctness liability. Full rationale in [`LICENSE-CONTENT.md`](LICENSE-CONTENT.md).
- **Code license: AGPL-3.0-only.** Network-copyleft prevents a closed fork capturing
  the community's contributions — the same commons logic at the software layer. See
  [`LICENSE`](LICENSE).

## Repository layout

- `README.md` — this file
- `ARCHITECTURE.md` — bounded contexts, ports/adapters, data model, scope & safety
- `TODO.md` / `Plans.md` — atomic task/subtask backlog, SemVer-milestoned
- `LICENSE`, `LICENSE-CONTENT.md` — code (AGPL-3.0-only) and content (CC BY-SA 4.0) licenses
- `docs/` — attribution-rendering contract, on-topic scope, badge semantics
- `Cargo.toml` + `crates/` — Rust workspace: `qa-core`, `identity-verification`, `ingestion`, `search`
- `pnpm-workspace.yaml` + `web/` — TypeScript web client
- `ingestion/pyproject.toml` — Python (uv) import/ETL project
- `deny.toml` — cargo-deny policy (licenses, advisories, bans)
- `.github/workflows/ci.yml` — fmt, clippy, cargo-deny, the architecture coupling test, tsc/eslint, ruff/mypy

## Stack

Rust core (library-first, binary-last), TypeScript (pnpm) web client,
Python (uv) only for the import/ETL tooling where existing dump-parsing
libraries make it cheaper. All dependencies latest stable. Constrained-machine
friendly. Detail in `ARCHITECTURE.md`.
