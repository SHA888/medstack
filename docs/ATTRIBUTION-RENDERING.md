# Attribution Rendering Contract

**Status**: M0.2.2 — defines non-strippable attribution for all mirrored content
**License choice**: CC BY-SA 4.0 (Option A selected; no quarantine partition)

## Contract

Every content record sourced from an external corpus (Stack Exchange, Biostars, link records) **must** render the following fields inline and in that order:

1. **Source** — origin corpus (e.g., "Stack Exchange", "Biostars", "FHIR Zulip")
2. **Author** — original author name or handle
3. **License** — the source's license (e.g., "CC BY-SA 4.0", "CC BY 4.0", "Link only")
4. **Date** — when the content was originally posted
5. **Link** — URL to the original post (for external verification and direct access)

### Non-strippability

The rendering path **must not have a code path that omits any of these five fields**. This is enforced at compile time:

- Each render function (HTML, JSON, plaintext) includes all five as required parameters, not optional
- Type system prevents conditional rendering (no `Option<>` for attribution fields)
- Unit tests assert presence; CI fails if any field is missing in output

### Rendering format (example)

```html
<div class="attribution">
  <p>
    <strong>Source:</strong> Stack Exchange (Clinical Informatics) |
    <strong>Author:</strong> Jane Doe |
    <strong>License:</strong> CC BY-SA 4.0 |
    <strong>Date:</strong> 2023-05-15 |
    <strong>Original:</strong> <a href="https://example.com/q/123">View on Stack Exchange</a>
  </p>
</div>
```

### Native vs. Mirrored

- **Native** (user-generated Q&A on medstack): License::Native(CcBySa4); no external attribution required
- **Mirrored** (Stack Exchange, Biostars): License::CcBySa4 or License::CcBy4; all five fields mandatory
- **Link-only** (FHIR Zulip): License::LinkOnly; source + author + link mandatory; body never copied

### Per-source matrix

| Source | License | Required fields | Body copied? | Notes |
|--------|---------|-----------------|--------------|-------|
| Stack Exchange | CC BY-SA 4.0 | Source, Author, License, Date, Link | ✓ Yes | Viral SA applies to derivative works |
| Biostars | CC BY 4.0 | Source, Author, License, Date, Link | ✓ Yes | Attribution required; no SA |
| FHIR Zulip | Link only | Source, Author, Link | ✗ No | Not open-licensed; reference only |
| medstack native | CC BY-SA 4.0 | (none; no external source) | N/A | User-generated; SA applies to subsequent use |

## CI enforcement

- **Compilation test**: Render functions with optional attribution fields fail to compile
- **Unit test**: Render output parsed; asserts presence of all five fields
- **Integration test**: Every mirrored item displayed in search/detail views includes full attribution
- **Lint rule** (if custom): AST scan flags render calls missing any attribution parameter

## Rationale

CC BY-SA 4.0 (Option A) was chosen because:
1. Stack Exchange content is CC BY-SA 4.0 (viral/share-alike)
2. Biostars is CC BY 4.0 (compatible; less restrictive)
3. Uniform license across native + mirrored content avoids quarantine overhead
4. SA requires attribution + derivative-work tracking; this contract makes that visible, not buried in metadata

Attribution non-strippability is a patient-safety and legal boundary: the next user must be able to verify the source and understand the license obligation they inherit (esp. SA virality).
