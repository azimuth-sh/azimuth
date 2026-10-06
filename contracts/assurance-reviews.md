# Compact independent assurance reviews

These records review the compact Claim-first account. They remain separate from authored Checks and inline Evidence Bindings. Binding identity is the project-global Check–Case pair; there is no authored binding ID. Decision Policies and challenge scheduling for this lane remain deferred.

## Authority projection

`azimuth assurance authority` exports the full unselected model's project, model fingerprint, Checks, Cases, bindings and Claims, plus an authority fingerprint. A Claim carries its criticality, exact Case dependencies, selected mechanisms and their implementation fingerprints, binding dependencies and structural gaps. Check fingerprints include method declarations and supporting source and package inputs; Claim fingerprints include intent, design, verification and realization support. No local module path becomes an entity identity.

The owner explicitly selects a projection in the ledger. A candidate projection is not accepted intent merely because it is exported or selected in a development ledger. The projection's `fingerprint` is the Run protocol canonical fingerprint of the complete projection excluding that field. Its `model_fingerprint` matches the complete model identity used by Run planning.

## Review records

Every record has exactly these common fields:

| Field | Meaning |
| --- | --- |
| `kind` | `method-qualification`, `applicability-decision` or `claim-judgment`. |
| `reviewer` | Independent reviewer identity; ledger authentication must match it. |
| `reviewed_at_ms` | Nonnegative safe-integer Unix milliseconds. |
| `rationale` | Nonempty explanation of the actual assessment. |
| `artifacts` | Nonempty array of `{id, digest, locator}` review artifacts. Digests are SHA-256 fingerprints; locators are not identities. |
| `fingerprint` | Run protocol canonical fingerprint of this exact record excluding `fingerprint`. |

Unknown fields, duplicate keys, duplicate dependencies, invalid identities, unsafe timestamps and mismatched fingerprints fail. Core validates accountable references but does not inspect an artifact's review quality.

### Method Qualification

Additional fields: `check`, `check_fingerprint`, `decision` (`accepted` or `rejected`). The record assesses whether the pinned Check method can credibly establish its stated propositions under its declared limits. It does not establish a Case from a particular execution.

### Applicability Decision

Additional fields: `check`, `case`, `check_fingerprint`, `case_fingerprint`, `binding_fingerprint`, `subject_fingerprint`, `method_qualification_fingerprint`, `basis`, `decision` and `limitations`.

`basis` is a nonempty array of exact `{run, bundle_fingerprint, observation}` references. `decision` is `accepted` or `rejected`; `limitations` is a string array. The record assesses whether those Observations contribute the authored binding's proposition for this particular Case and exact Subject. It pins the qualified method and all contributing definitions. A qualified HTTP denial method does not automatically establish network isolation or a persisted-effect proposition.

### Claim Judgment

Additional fields: `claim`, `claim_fingerprint`, `subject_fingerprint`, `cases`, `mechanisms`, `bindings`, `conclusion` and `residual_risks`.

`cases` and `mechanisms` contain `{id, fingerprint}` dependencies. `bindings` contains `{check, case, fingerprint, applicability_fingerprint}`; the applicability reference is explicitly null when missing. `conclusion` is `supported`, `violated` or `unresolved`; `residual_risks` is a string array. The independent reviewer accounts for all current Claim dependencies and gaps. Execution success does not create this record.

## Assessment and state

Assessment distinguishes `accepted`, `rejected`, `stale`, `invalid` and `missing`. Unknown or changed definitions, different Subjects, missing observations, obsolete correction heads and newer relevant Observations cannot silently preserve earlier positive decisions. The latest review time for a typed target selects the current record; ambiguous records at that time are invalid, without falling back to an earlier accepted record.

A supported Claim requires complete current dependencies, accepted current qualifications and applicability decisions, satisfied applicable Observations, no structural gaps, and an independent current supported Judgment. A violated Claim requires an independent violated Judgment with credible applicable counterevidence; a failed Check alone is an adverse execution fact. Incomplete or conflicting support remains unresolved. Routine Claims report when independent review is not required without manufacturing a supported Judgment.

The ledger retains immutable histories and computes exact Subject-specific state against the owner's selected authority. Evidence from other Subjects or changed model support remains visible and unjoined. See [run-ledger.md](run-ledger.md) for persistence and permissions.
