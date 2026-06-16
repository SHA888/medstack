# LICENSE-CONTENT.md

## The Commons Pairing

MedOverflow uses a two-part licensing model: **AGPL-3.0 (software) + CC BY-SA 4.0 (corpus)**.

Both licenses serve the same purpose: **protect the commons from closed capture**. This document explains the choice, what it permits, what it forbids, and how it applies to mirrored content.

---

## Content License: CC BY-SA 4.0

### The Decision

The native content corpus (user-generated questions, answers, votes, and edits) is licensed under **Creative Commons Attribution-ShareAlike 4.0 International (CC BY-SA 4.0)**.

### Why CC BY-SA, not CC BY?

We considered CC BY 4.0 (permissive, no share-alike requirement) with a separate partition for Stack Exchange content. That design had two fatal flaws:

1. **Taint-tracking is a correctness liability.** Edits, refinements, and answer-builds-on-answer would create a constantly-shifting partition boundary. Tracking which parts of a refined answer came from a SA-licensed source vs. which are native becomes a mutation-tracking problem, not a contained engineering task. The only way to implement it safely is to forbid cross-partition edits — which neuters the corpus's defining feature: answers building on answers.

2. **Contributor mental model.** Stack Exchange users expect CC BY-SA. Matching that license means their mental model of "my Q&A is CC BY-SA" carries over with zero friction. A quarantine scheme confuses that signal.

**CC BY-SA 4.0 is the coherent choice.** It applies uniformly to all content, native and mirrored, with one set of rules.

### What CC BY-SA means

**Attribution (BY):** Anyone who uses, adapts, or builds on content from MedOverflow must:
- Credit the original author(s) and source (MedOverflow)
- Provide a link to the license or a copy of it
- Indicate if the content was modified
- Make this attribution visible and non-strippable

**Share-Alike (SA):** Any derivative work (modified content, compilations, adaptations) must be licensed under CC BY-SA 4.0 (or a compatible license).

### What this permits and forbids

| Use case | Permitted? | Notes |
|----------|-----------|-------|
| Read and learn | ✅ | No restrictions. Attribution not required just for reading. |
| Quote with attribution | ✅ | Quote any excerpt, attribute source and author. |
| Embed in CC BY-SA project | ✅ | Your project must also be CC BY-SA. |
| Embed in open-source project (GPL/MIT/Apache) | ⚠️ | Compatibility is complex. GPL-licensed code can include CC BY-SA content, but the interplay of two copyleft licenses (GPL on software, CC BY-SA on content) can create interpretation disputes. MIT/Apache permissive code cannot satisfy CC BY-SA's share-alike requirement. Consult a lawyer familiar with GPL-content-licensing hybrids. |
| Embed in permissive/proprietary software | ❌ | Proprietary code cannot satisfy SA requirement. Forbidden. |
| Translate and republish | ✅ | Must be CC BY-SA 4.0. Credit original author. |
| Create a derivative Q&A site | ✅ | Must be CC BY-SA 4.0. Attribute all sources. |
| Commercial use | ✅ | CC BY-SA permits commercial reuse. Must keep SA intact. |

**The hard foreseeable:** Option A explicitly forecloses proprietary embedding of corpus snippets. If you ever wanted MedOverflow's corpus trivially embeddable into closed clinical software, this license blocks that. We're choosing to foreclose that path on purpose — that's exactly the capture the commons is designed to resist.

---

## Code License: AGPL-3.0

### The Decision

The MedOverflow software (qa-core, identity-verification, ingestion, search, web client) is licensed under **GNU Affero General Public License v3.0 (AGPL-3.0)**.

### Why AGPL, not MIT/Apache?

AGPL-3.0 includes Section 13: the **network-copyleft provision**. It says: if you modify and *run* MedOverflow as a network service (not just internally, but exposed as an API or hosted instance), you must release your modifications under AGPL-3.0 to anyone using that service.

**Why this matters for a community knowledge commons:**

A permissive license (MIT/Apache) would let someone fork MedOverflow, modify it, and run a closed competing instance against the community's corpus (mirrored from Stack Exchange, Biostars, etc.). The software improvements stay proprietary. The corpus stays open, but the engine that serves it is closed.

AGPL-3.0 closes that moat: anyone running a modified MedOverflow as a network service must release their source code. Improvements flow back to the community.

This is the software-side analogue of the content-side CC BY-SA. Both protect the commons from closed capture.

### What AGPL-3.0 permits and forbids

| Use case | Permitted? | Notes |
|----------|-----------|-------|
| Internal use (no network exposure) | ✅ | Use MedOverflow internally. No disclosure required. |
| Modify and use internally | ✅ | Modify the code, use it in-house. No disclosure required. |
| Modify and distribute as source | ✅ | Distribute modified source under AGPL-3.0. |
| Host as a network service (unmodified) | ✅ | Run MedOverflow as-is. No disclosure needed. |
| Host as a network service (modified) | ⚠️ | **Disclosure required.** You must release your modifications to users. |
| Modify and offer as a proprietary SaaS | ❌ | Proprietary hosting of modified MedOverflow violates AGPL-3.0. |
| Combine with proprietary code (link) | ⚠️ | **"Linked" includes both source linking AND network service usage.** If you embed MedOverflow in your binary (source linking), proprietary code must be AGPL-compatible. If you deploy modified MedOverflow as a network service (API, web app), you must disclose modifications to users (AGPL §13). Both are distinct triggering conditions. Consult a lawyer. |

**The network-copyleft trigger:** If you modify MedOverflow and run it as a network service (REST API, web app, etc.), users of that service can demand the source code. This includes:
- Running a modified MedOverflow instance
- Deploying MedOverflow behind a proxy or gateway
- Offering MedOverflow-based services to end users
- Hosting a derivative clinical Q&A system

---

## The Coherent Pair: CC BY-SA 4.0 + AGPL-3.0

Both licenses apply the same logic to different layers:

| Layer | License | Protection |
|-------|---------|-----------|
| **Content** (corpus) | CC BY-SA 4.0 | Downstream users cannot embed the corpus into closed/proprietary products without SA obligations. Improvements to the knowledge artifact stay open. |
| **Software** (engine) | AGPL-3.0 | Downstream operators cannot run closed forks of MedOverflow as network services without releasing modifications. Improvements to the software stay open. |

Together, they form a **commons-protection pairing** that's well-tested and widely understood:
- **Wikipedia** uses this shape: GPL (MediaWiki engine) + CC BY-SA (content)
- **OpenStreetMap** uses this shape: ODBL (content) + ODbL-compatible code
- **Free/open clinical data initiatives** increasingly use this pairing

---

## Per-Source License Matrix

When MedOverflow mirrors or imports external content, the source license travels with it.

| Source | License | Mirroring | Attribution | Notes |
|--------|---------|-----------|-------------|-------|
| **Stack Exchange** (SO, SU, SF, etc.) | CC BY-SA 4.0 | Full mirror (Q&A body) | Required: author, link, license | Legally compatible with MedOverflow CC BY-SA 4.0. Edits preserve SA. |
| **Biostars** | CC BY 4.0 | Full mirror (Q&A body) | Required: author, link, license | CC BY is compatible with CC BY-SA 4.0 (SA obligation applies to MedOverflow corpus as a whole). |
| **FHIR Zulip** | Proprietary (Zulip ToS) | Link-only (no body copy) | Required: title, URL, source attribution | Cannot mirror the body due to ToS. Store only metadata + link. Attribution block always present. |
| **Native MedOverflow content** | CC BY-SA 4.0 | N/A (created here) | Required: author, timestamp, license | All user-generated content defaults to CC BY-SA 4.0. |

---

## Attribution Requirements (Non-Strippable)

Every question, answer, and piece of mirrored content must render:

1. **Author**: The person who wrote it
2. **Source**: Where it came from (Stack Exchange, Biostars, MedOverflow-native, etc.)
3. **License**: The applicable license (CC BY-SA 4.0, CC BY 4.0, Link-only, etc.)
4. **Date**: When it was posted/last edited
5. **Link** (for external sources): URL to the original

These **must be structural and non-strippable**. A user cannot remove attribution when downloading, exporting, or copying. If code allows removing attribution, that's a bug.

---

## FAQ

### Q: Can I use MedOverflow in a closed clinical tool?

**A:** No, not without a separate commercial license negotiation. The CC BY-SA 4.0 corpus forbids embedding in proprietary products without SA obligations, which proprietary code cannot satisfy. If this is a use case you need, contact the maintainer.

### Q: Can I fork MedOverflow and run it privately?

**A:** Yes, AGPL-3.0 permits internal use. You can modify and run MedOverflow in-house without releasing changes. As soon as you expose it as a network service, disclosure becomes required.

### Q: Can I run MedOverflow on my hospital's internal network?

**A:** Yes. If it's strictly internal (no external users, no network exposure beyond the hospital), AGPL-3.0 doesn't require disclosure. Once you connect it to the internet or allow external users, you must release modifications.

### Q: What if I want to embed a single answer in my proprietary software?

**A:** You cannot, under CC BY-SA 4.0, without making your software CC BY-SA-compatible (essentially, open-sourcing it). You can cite it, link to it, quote it with attribution — but embedding the full text into proprietary code violates the SA obligation.

### Q: Can I sell access to MedOverflow?

**A:** You can charge for a MedOverflow service (AGPL-3.0 permits commercial use). But if you modify the software, you must release modifications under AGPL-3.0. If you modify the content corpus, you must release those modifications under CC BY-SA 4.0.

### Q: What if I translate MedOverflow into another language?

**A:** Translations are derivative works. AGPL-3.0 (software) and CC BY-SA 4.0 (content) both permit translation. You must release the translation under the same license, with attribution to the original author.

### Q: Can I use MedOverflow's content for machine learning training?

**A:** CC BY-SA 4.0 permits this use. You must attribute the source corpus and the individual authors. If you publish a model trained on MedOverflow, you must disclose the training data source. If your model itself becomes a derivative work of the corpus (e.g., a Q&A generation model), ensure your licensing reflects SA obligations.

---

## Version History

- **2026-06-13**: Initial decision. CC BY-SA 4.0 (content) + AGPL-3.0 (code). Rationale: commons protection via network-copyleft and content-level SA.
