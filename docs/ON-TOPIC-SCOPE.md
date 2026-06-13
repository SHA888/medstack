# On-Topic Scope & Safety Boundary

**Status**: M0.4.1 — defines explicit content scope for moderation  
**Safety boundary**: Clinician-engineer knowledge commons; patient-facing medical advice strictly OUT

## In Scope

medstack exists to accrete **durable, peer-validated knowledge about clinical software, informatics, and data**. Answerable questions fit one of these categories:

### Clinical Software & Systems
- Architecture, design patterns, regulatory compliance for clinical software (EHR, pharmacy, imaging systems, etc.)
- Interoperability standards and integration (FHIR, HL7, DICOM, LOINC, SNOMED CT usage in software)
- Clinical decision support systems, rule engines, data pipelines
- Real-world examples: "How do we model medication reconciliation in HL7v2?" / "FHIR Bundle structure for discharge summaries — best practices?"

### Clinical Data & Informatics
- Data modeling for clinical workflows (patient timelines, lab result interpretation, longitudinal care)
- Clinical research data pipelines (cohort extraction, feature engineering, outcome definition)
- Electronic health record data quality, privacy (de-identification, HIPAA technical safeguards)
- Real-world examples: "Cohort extraction strategy for sepsis outcomes?" / "How to structure vital signs time-series for ML?"

### Engineering for Clinical Contexts
- Software engineering practices specific to regulated medical environments (V&V, change control, validation for 21 CFR Part 11)
- Testing clinical logic (edge cases in medication dosing calculations, care protocol workflows)
- Deployment, observability, incident response in clinical settings (consequences of failures are high)
- Real-world examples: "How do we validate a drug-drug interaction checker?" / "Testing strategy for a critical infusion pump integration?"

### Domain-Specific Credential Authority
- Questions where a verified credential (MD, RN, PharmD, or domain-specific engineering credential) meaningfully informs the answer
- Clinical context makes the question answerable and the answer more trustworthy
- Credential badge communicates "I have lived experience in this domain," not "trust me medically"

## Out of Scope

The following are **explicitly rejected**:

### Patient-Facing Medical Advice
- "What's my diagnosis?" — patient self-help
- "Is this medication safe for me?" — personalized medical decision-making
- "How should I manage my condition?" — clinical guidance for an individual patient
- "Should I see a doctor about this symptom?" — triage advice

**Why**: medstack is not a clinical care tool. Patient-facing medical advice requires accountability, informed consent, and real-time physician-patient relationship. Q&A cannot provide these. Broadcasting medical guidance to unknown patients is a liability and a patient-safety boundary.

### Generic Programming & Software Engineering
- "How do I write a REST API?" — generic software questions (Stack Overflow exists)
- "What's the best way to structure a database?" — decontextualized engineering questions
- "How do I learn React?" — general software learning (not clinical-context-specific)

**Why**: medstack is a niche; it exists because clinician-coders have context-bound questions that SO does not serve well. Generic programming questions dilute focus and fragment the community.

### Non-Clinical Medical Knowledge
- "How does the renin-angiotensin system work?" — pure physiology (not software/informatics)
- "What's the pathophysiology of sepsis?" — clinical science without a software/data angle
- "How should a doctor interpret a chest X-ray?" — clinical practice (not informatics)

**Why**: medstack is about *clinical software/data/informatics*, not clinical science or medical education. Those are served by other communities (medical education forums, UpToDate, etc.).

### Institutional Policies & Compliance Questions (Generic)
- "What's our hospital's IV policy?" — institutional-specific (not portable knowledge)
- "How do we handle compliance at our organization?" — generic compliance Q&A (belongs in compliance forums)

**Why**: Institutional knowledge is too specific to be durable; it changes with org policy. Generic compliance belongs elsewhere.

## Judgment Calls & Moderation

**Borderline acceptable** (approve with evidence):
- "We're building a surgical scheduling optimization — how do we model surgeon availability constraints?" ✓ (software + clinical data modeling)
- "Our lab LIS integration dropped critical values — how do we test alert logic?" ✓ (testing clinical logic)
- "How do clinicians reason about drug interactions?" ✓ (provides context for building better decision-support systems)

**Borderline unacceptable** (reject, explain boundary):
- "Is this patient's EKG abnormal?" ✗ (clinical interpretation, not software/data)
- "My hospital's EHR is slow — should I complain to IT?" ✗ (generic IT problem, not clinical-software specific)
- "How do I convince my hospital to adopt FHIR?" ✗ (organizational change management, not clinical software)

## Moderation Rules

1. **Err toward scope, not away from it**: If a question *could* be about clinical software/informatics with minor clarification, ask for details before rejecting
2. **Patient advice is automatic reject**: Any phrasing like "I have X, should I..." → reject immediately, suggest alternative forums
3. **Credential matters but isn't determinative**: A question is on-topic or not based on content, not on the asker's credentials. Credentials weight the answer, not gate the question.
4. **Durable knowledge matters**: Questions that are too specific ("our hospital's custom field") or too temporal ("what's the latest EHR update?") may be valuable but won't accrete; consider link-only if the question is truly niche.

## Appeal Path

Users who believe their question was rejected in error may:
1. Edit for clarity and resubmit
2. Explain how the question relates to clinical software/informatics in a reframing comment
3. Flag for moderator review if they believe the boundary was applied inconsistently

Moderators review appeals monthly and update this document if patterns emerge.

## Scope Review Cadence

This document is reviewed:
- **Monthly**: moderation logs analyzed for borderline cases
- **Quarterly**: this document updated if new patterns emerge or scope needs clarification
- **At v1.0**: finalized based on actual user activity and moderation experience
