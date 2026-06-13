# Credential Badge Semantics & UI Copy

**Status**: M0.4.2 — clarifies what credential badges mean to users  
**Safety boundary**: badges communicate domain expertise, never clinical endorsement

## The Problem

A credential badge next to a medical Q&A answer can be misread as "this person is endorsing this as safe/correct medical advice for you." It is not. A clinician's software expertise does not make them your doctor, and a medstack answer is never a substitute for clinical consultation.

This document specifies the UI copy, badge placement, and language that prevents this misreading.

## Badge Design

### Visual Indicator

- **Position**: Answer header, next to author name (not prominent)
- **Icon**: Stethoscope + terminal (or medical + code symbol), subdued color (gray, not green)
- **Text label**: "Verified clinician-engineer" or credential-specific (e.g., "MD" + credential metadata)
- **Hover tooltip**: See "Tooltip Text" below

### Credential Scopes

Credentials carry a **scope** that describes which domains they cover:

| Scope | Badge text | Meaning |
|-------|-----------|---------|
| **Clinical** | "Verified: Clinical License" | MD, DO, RN, DDS, DVM, etc. — licensed clinical practice |
| **Engineering** | "Verified: Credential" | ME, CS degree + relevant experience; demonstrated software expertise |
| **Research** | "Verified: Researcher" | PhD, research institution affiliation; domain expertise in clinical research |

### Tooltip Text (on hover)

```
This answer is from someone with verified [scope] credentials. 
This badge means they have relevant domain expertise, NOT that they 
are providing medical advice. Always consult your doctor for medical decisions.
```

### Badge Metadata (API payload)

When an answer is returned with a credential, the JSON includes:

```json
{
  "answer_id": "...",
  "body": "...",
  "author_id": "...",
  "credential": {
    "scope": "Clinical",        // Clinical | Engineering | Research
    "issued": "2024-06-01",     // credential issue date
    "expires": "2027-06-01",    // credential expiry
    "authority_weight": 0.85,   // pure function of (scope, freshness)
    "jurisdiction": "US-KA"     // optional jurisdiction (e.g., state medical license)
  }
}
```

The **front-end client is responsible** for rendering this metadata with the safety copy (not the API).

## Critical UI Copy

The following copy **must appear** on every answer detail page where a user might make a medical decision:

### Disclaimer Banner (if any answer on the page has a clinical credential)

```
⚠️ medstack answers are peer-validated Q&A about clinical software and data.
They are NOT clinical advice. Always consult your doctor for medical decisions.
```

**Placement**: Below the question, above the first answer  
**Style**: Light warning color (yellow background), readable but not alarming  
**Dismissible**: Yes (remembers dismissal for session)

### Per-Answer Safety Copy (if answer has Clinical credential)

```
This answer is from a verified clinician with [scope] credentials.
Credential badges indicate domain expertise relevant to software/informatics,
not medical advice for your personal health. Consult your doctor.
```

**Placement**: Just above the badge, small italic text  
**Style**: Subdued color (gray), no warning styling (not alarmist)  
**Conditional**: Only shown if credential scope includes "Clinical"

## What Badges Do NOT Mean

Explicitly state in help/FAQ:

- ❌ "A badge means this answer is medically safe for me" — no
- ❌ "A badge means this person is my doctor" — no
- ❌ "A badge means this answer is fact-checked by a clinician" — no (it's peer-validated like SO, just from someone with domain credentials)
- ❌ "I should follow this answer's medical advice because of the badge" — no

## What Badges DO Mean

- ✓ "This person has a verified clinical/engineering/research credential relevant to the domain"
- ✓ "This person has lived experience building/designing clinical software"
- ✓ "This answer comes from someone who understands clinical workflows and constraints"
- ✓ "Authority weight on this answer reflects their domain expertise, not medical authority over you"

## Copy for Different User Journeys

### First-time user landing on a clinical Q&A

```
Welcome to medstack, a Q&A site for clinician-engineers.

What we answer: How do you build clinical software? How do you integrate EHRs? 
How do you model clinical data? Verified credentials show domain expertise.

What we don't: Medical advice for your health. For that, see your doctor.
```

### User reading an answer with a clinical credential

```
💡 "Verified: MD + Software Engineer" means this person is a doctor 
AND a software engineer. On THIS question about EHR data modeling, 
that expertise matters. It does NOT mean they are advising you medically.
```

### User about to vote/reply

```
Remember: medstack is about clinical software/informatics, not patient advice. 
Your answer should help other engineers/informaticians build better systems, 
not guide patients on medical decisions.
```

## Scope Constraints

**Engineering credential on a clinical-advice-sounding question** → flag for moderation

Example: A software engineer with an "Engineering" credential answers a question that sounds like patient advice. The badge should not appear; the question should be closed as off-topic.

```
Q: "I'm a software engineer, but my mom has diabetes. Should she take metformin?"
❌ No "Verified" badge, regardless of asker's credentials
❌ Question closed: off-topic (patient medical advice)
```

**Clinical credential on a medical-theory question** → badge OK, but with extra caution

Example: A doctor answers a physiology question. The badge communicates their clinical knowledge, but the disclaimer still applies.

```
Q: "How do we model the renin-angiotensin system in a decision-support rule engine?"
✓ Clinical badge OK (their medical knowledge is relevant to the software design question)
✓ Disclaimer still required (this is not a patient-advice forum)
```

## Content Policy & Moderation

Any answer that **reads as medical advice to a patient** must be:
1. **Flagged** during moderation
2. **Either edited** to focus on the software/informatics angle, or **closed**
3. **Reported** if the author used their credential to lend false authority to patient advice

Examples of problems:

| Answer | Problem | Action |
|--------|---------|--------|
| "Metformin is first-line for type 2 diabetes; your mom should ask her doctor about it" | Reads as medical advice; badge used to lend authority | Close: off-topic patient advice |
| "FHIR profiles for diabetes management should model glucose targets per ADA guidelines; here's how we implemented it" | Legitimate software design; clinical knowledge is relevant | Keep; badge OK; disclaimer present |
| "Your symptoms sound like X; you should see a doctor" | Direct patient triage; misuse of badge | Close; consider mod action |

## Testing & Enforcement

- **Unit test**: Badge tooltip text present and includes "NOT medical advice" language
- **Integration test**: Disclaimer banner renders on answer detail if any credential present
- **Visual regression**: Badge styling, placement, and copy remain consistent
- **Moderation audit**: Monthly review of flagged answers; pattern analysis for scope drift

## Updates to This Document

As the community matures:
- Quarterly review of user feedback on badge clarity
- A/B test alternative disclaimer copy if user confusion detected
- Update scope definitions if new credential types emerge (e.g., pharmacist, PT)

This document is binding for all UI copy and moderation decisions until superseded by a formal update.
