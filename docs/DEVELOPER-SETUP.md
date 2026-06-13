# Developer Setup

## Prerequisites

Ensure you have installed:
- **Rust**: `rustc`, `cargo` (latest stable)
- **Node.js**: v20+ (for TypeScript/pnpm)
- **Python**: 3.11+ (for ingestion tooling)
- **uv**: Rust-based Python package manager (`pip install uv`)
- **pnpm**: Node package manager (`npm install -g pnpm`)

## Pre-commit Hooks

This project uses the [pre-commit framework](https://pre-commit.com/) to enforce code quality before commits.

### Installation (one-time)

```bash
pip install pre-commit
pre-commit install
```

This creates Git hooks that run automatically on every `git commit`.

### What runs

- **Rust**: `cargo fmt --check`, `cargo clippy -D warnings`, `cargo deny check licenses`
- **Python** (ingestion/): `ruff format`, `ruff check`
- **TypeScript** (web/): `prettier` on JSON, YAML, Markdown
- **General**: Trailing whitespace, file endings, YAML/TOML/JSON validation

### Run manually

```bash
# Check all files
pre-commit run --all-files

# Check only staged files
pre-commit run
```

## Development Commands

**Rust:**
```bash
cargo check                      # Verify compilation
cargo test                       # Run all tests
cargo fmt --all                  # Format code
cargo clippy --all-targets --all-features -- -D warnings
cargo deny check                 # Audit dependencies
```

**TypeScript:**
```bash
cd web
pnpm install
pnpm exec tsc --noEmit          # Type check
pnpm exec eslint .              # Lint
pnpm test                         # Run tests (if configured)
```

**Python (ingestion):**
```bash
cd ingestion
uv sync
uv run ruff check --fix         # Lint + fix
uv run mypy .                   # Type check
uv run pytest                   # Run tests (if configured)
```

## Architecture Test

The coupling invariant (qa-core has zero outward dependencies) is enforced by:

```bash
cargo test --package qa-core --test architecture
```

This runs on every commit (via pre-commit) and in CI.

## Commit Message Format

Commits follow the [Conventional Commits](https://www.conventionalcommits.org/) format:

```
type(scope): short summary

Optional detailed explanation. Keep to ~72 chars per line.

Breaking changes documented here.
```

**Types**: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `chore`

**Scope**: Feature area (e.g., `M1.1.1`, `ci`, `docs`)

**No trailers**: Do not include `Co-Authored-By:` trailers.

### Examples

```
feat(M1.1.1): Create QuestionId, AnswerId, UserId newtypes
fix(ci): Correct clippy flag syntax in CI workflow
docs: Add developer setup guide
```

## Troubleshooting

### Pre-commit hooks not running

Verify installation:
```bash
git config core.hooksPath
```

Should show `.git/hooks`. Re-install if missing:
```bash
pre-commit install
```

### "command not found: pre-commit"

Install globally or in a virtual environment:
```bash
pip install pre-commit
# or: pipx install pre-commit
```

### Poetry/uv version conflicts

If `uv` is not found, ensure it's in PATH:
```bash
pip install uv
which uv
```

Or use the absolute path in `.pre-commit-config.yaml`.

## CI/CD

- **Local**: `pre-commit run --all-files` before pushing
- **Remote**: `.github/workflows/ci.yml` runs the same checks on every push
- **Release**: SemVer compliance checked via `cargo-semver-checks` before tagging

## Further Reading

- CLAUDE.md — architecture, design principles, development principles
- ARCHITECTURE.md — bounded contexts, ports/adapters, data flow
- Plans.md — implementation roadmap by milestone
