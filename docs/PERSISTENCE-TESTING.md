# Persistence Adapter Testing

This document describes how to run the conformance test suite against both SQLite and Postgres persistence adapters.

## SQLite Conformance Tests

SQLite tests use in-memory databases and require no external setup.

```bash
cargo test --package persistence-sqlite --test conformance
```

These tests are run automatically in CI on every push.

## Postgres Conformance Tests

Postgres tests require a running Postgres database. Set the `MEDOVERFLOW_POSTGRES_TEST_URL` environment variable to point to your test database.

### Setup (local development)

1. Ensure Postgres is running locally:
   ```bash
   # On macOS with Homebrew:
   brew services start postgresql

   # On Linux (Ubuntu/Debian):
   sudo systemctl start postgresql
   ```

2. Create a test database:
   ```bash
   createdb medoverflow_test
   ```

3. Run the conformance tests:
   ```bash
   MEDOVERFLOW_POSTGRES_TEST_URL="postgres://localhost/medoverflow_test" \
     cargo test --package persistence-postgres --test conformance
   ```

### Running Both Adapters in Parallel

To run conformance tests against both SQLite and Postgres simultaneously:

```bash
# Terminal 1: SQLite tests
cargo test --package persistence-sqlite --test conformance

# Terminal 2: Postgres tests (in another terminal)
MEDOVERFLOW_POSTGRES_TEST_URL="postgres://localhost/medoverflow_test" \
  cargo test --package persistence-postgres --test conformance
```

Or as a single command with `cargo test --workspace`:

```bash
# Run all tests (both adapters)
# SQLite tests run unconditionally
# Postgres tests are skipped if MEDOVERFLOW_POSTGRES_TEST_URL is not set
cargo test --test conformance
```

### Setup (CI environment)

To enable Postgres testing in CI:

1. Add a Postgres service to the CI workflow (e.g., GitHub Actions)
2. Set `MEDOVERFLOW_POSTGRES_TEST_URL` environment variable
3. Run conformance tests

Example GitHub Actions service configuration (not yet implemented):

```yaml
services:
  postgres:
    image: postgres:15
    env:
      POSTGRES_DB: medoverflow_test
      POSTGRES_HOST_AUTH_METHOD: trust
    options: >-
      --health-cmd pg_isready
      --health-interval 10s
      --health-timeout 5s
      --health-retries 5
    ports:
      - 5432:5432
```

## Test Coverage

Both adapters are tested against the same conformance test suite, which covers:

- **Question aggregates**: round-trip persistence, tags, revisions
- **Answer aggregates**: round-trip persistence, credentials, revisions, votes
- **Edge cases**: missing records, identity preservation, vote preservation

All tests verify that data round-trips with full fidelity (no data loss or corruption).

## Troubleshooting

**Tests are being skipped**: Ensure `MEDOVERFLOW_POSTGRES_TEST_URL` is set before running Postgres tests. Without it, tests skip gracefully rather than failing.

**Postgres connection refused**: Check that Postgres is running and the connection string is correct.

**Schema conflicts**: Tests create tables using `IF NOT EXISTS`, so multiple test runs are safe. To clean up:

```bash
psql medoverflow_test -c "DROP TABLE IF EXISTS answer_votes, answer_revisions, answers, question_revisions, questions CASCADE;"
```

## DoD Verification

Task 2.1.3 is complete when:
- [ ] Single test suite runs against SQLite and Postgres in parallel
- [ ] All tests pass on both adapters
- [ ] CI runs SQLite tests on every push
- [ ] Documentation explains how to run Postgres tests locally
