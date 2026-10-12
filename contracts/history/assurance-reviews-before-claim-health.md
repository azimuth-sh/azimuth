# Compact independent assurance reviews

These records review the compact Claim-first account. They remain separate from authored Checks and inline Evidence Bindings. Binding identity is the project-global Check–Case pair; there is no authored binding ID. Decision Policies and challenge scheduling for this lane remain deferred.

## Authority projection

`azimuth assurance authority` exports the full unselected model's project, model fingerprint, Checks, Cases, bindings and Claims, plus an authority fingerprint. A Claim carries its criticality, exact Case dependencies, selected mechanisms and their implementation fingerprints, binding dependencies and structural gaps. Check fingerprints include method declarations and supporting source and package inputs; Claim fingerprints include intent, design, verification and realization support. No local module path becomes an entity identity.

The owner registers immutable candidate projections inside the existing project, separately from explicitly selecting accepted authority. A candidate projection is not accepted intent merely because it is exported, registered or assessed. Revision-scoped review submissions and snapshots identify the exact assessed projection; local workflow records cannot replace authenticated review ingestion. The projection's `fingerprint` is the Run protocol canonical fingerprint of the complete projection excluding that field. Its `model_fingerprint` matches the complete model identity used by Run planning.

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

## Independent review input preparation

`azimuth assurance review-input --project <id> --request <file> [--review <file>...] [--bundle <file>...] [model options] --out <file>` prepares an `azimuth-review-input` version 1 packet. A strict request identifies exactly one method (`kind`, `check`), applicability target (`kind`, `check`, `case`, `subject_fingerprint`) or Claim (`kind`, `claim`, `subject_fingerprint`). Routine Claims do not require independent Claim Judgment.

The packet pins current authority and target dependencies, authored definitions, exact matching Subject descriptors, current relevant Observations and their active Run bundles, existing review assessments and visible gaps. Its fingerprint covers the complete packet. It is preparation, not a review record: it creates no reviewer, reviewed-at time, rationale, decision or conclusion. Unknown targets and malformed inputs fail; absent evidence produces explicit gaps.

Record freshness follows relevant Check, Case, binding, mechanism, Claim and execution dependencies. Unrelated changes to the full authority projection do not independently stale a review. Changing only a Case contribution does not requalify an unchanged method; applicability still pins that binding and must be reviewed against its current meaning. A stale or invalid applicability dependency propagates into its dependent Judgment even when that Judgment previously concluded unresolved.

## Markdown review bundles

Markdown is an authoring representation of the three native review kinds. It does not replace their dependency, currentness or authenticated submission rules. A change normally keeps Method Qualifications and Applicability Decisions in `reviews/check-assessments.md` and Claim Judgments in `reviews/claim-judgments.md`. These paths are conventions; the CLI takes explicit inputs. Authoring reviews remain a separate contract.

A file starts with one level-one title. Each level-two heading is exactly `Method qualification: <id>`, `Applicability decision: <id>` or `Claim judgment: <id>`. IDs are unique across the supplied change bundles. Metadata precedes prose; duplicate or unknown fields fail with the file and record line. Scalar values may be enclosed in backticks. Required common fields:

- `Input`: portable path relative to the explicit change root, with no traversal or symlink escape. It resolves to the retained `azimuth-review-input` packet.
- `Input fingerprint`: the packet's native canonical fingerprint, not a digest of its serialization.
- `Reviewer`: the actual accountable reviewer identity.
- `Reviewed at`: actual UTC time in `YYYY-MM-DDTHH:MM:SSZ`, compiled to native Unix milliseconds. Placeholders fail.
- Optional `Agent` and `Model`: names of the agent/harness and exact model used. These are reviewer-supplied provenance, not authenticated identities. Human reviews may omit both.
- Optional `Reassesses`: a prior change-local record ID supplied in the same compilation. The preceding record must have the same logical target and an earlier review time. Exact Subject and definition fingerprints can differ after a correction; both remain recorded independently.

Kind-specific metadata:

| Heading | Required scalar fields | Required list |
|---|---|---|
| Method qualification | `Check`, `Decision` (`accepted` or `rejected`) | none |
| Applicability decision | `Check`, `Case`, `Decision` (`accepted` or `rejected`) | `Limitations` |
| Claim judgment | `Claim`, `Conclusion` (`supported`, `violated` or `unresolved`) | `Residual risks` |

Lists use indented `  - value` entries. An empty list has its field caption and no entries. Each record requires `### Rationale` with nonempty prose. Optional `### Findings` is retained in the native rationale under that caption. Findings have no automated semantic interpretation. Target IDs and kind must match the exact input packet.

```markdown
# Check assessments

## Method qualification: routing-method-01

- Check: `registered-routing-policy`
- Input: `.azimuth/review-inputs/routing-method-01.json`
- Input fingerprint: `<actual packet fingerprint>`
- Reviewer: `<actual reviewer identity>`
- Reviewed at: `<actual UTC timestamp>`
- Agent: `<agent name, if applicable>`
- Model: `<model identifier, if applicable>`
- Decision: rejected

### Rationale

The oracle reads expected policy from the registration metadata it compares,
so the method cannot detect incorrect declarations.

### Findings

Maintain expected policy independently of production registrations.
```

The placeholders illustrate the layout and are not valid records. Tooling generates the target/input metadata; reviewers supply the real identity, time and assessment.

### Commands

```text
azimuth assurance prepare-candidate --project <id> [model options] --out <receipt>
azimuth assurance snapshot --project <id> [--review <native-record-or-bundle>...]
  [--bundle <run-bundle>...] [model options] --out <snapshot>
azimuth assurance review-input --project <id> --request <request>
  [--review <native-record-or-bundle>...] [--bundle <run-bundle>...]
  [model options] --out <packet>
azimuth assurance review-draft --input <packet> --input-ref <change-relative packet>
  --id <record-id> --out <Markdown draft>
azimuth assurance review-compile --project <id> --change-root <directory>
  --input <change-relative Markdown>... [--review <native-record-or-bundle>...]
  [--bundle <run-bundle>...] [model options] --out <compiled bundle>
azimuth assurance review-export --input <compiled bundle> --id <record-id>
  --out <native record>
```

`--input` is repeatable for compilation, allowing the two review files to share a reassessment history. `--review` accepts existing strict native records or compiled bundles. The compiler validates input packet fingerprints and copies exact dependencies, qualification fingerprints and Observation basis into existing native records. It assesses currentness against the explicitly loaded complete candidate, supplied reviews and Run history; it does not trust a Markdown outcome as an assurance conclusion. Historical stale or superseded records remain present with their assessment; native currentness rules govern use. Unavailable packets, incomplete required dependencies or invalid native records prevent compilation. No command runs an agent or supplies an assessment.

A compiled `azimuth-review-bundle` version 1 contains `project`, `authority_fingerprint`, `records` and a canonical `fingerprint`. Each entry contains its local `id`, `input: {ref, fingerprint}`, `provenance: {agent, model}` (explicit null when omitted), `reassesses` (local ID or null), `record` (the existing strict native JSON), and `assessment: {status, latest, diagnostics}`. This container is a publication artifact. Its reassessment relationship and agent/model metadata are explicitly separate from native record fields; native ingestion schemas remain unchanged. Retain the bundle alongside individually exported records so those relationships are available to publication tooling. Existing ASS native-record ingestion retains the rationale and digest-bearing source artifact; structured bundle provenance is not yet an ASS query field.

Each native record includes a source Artifact with the local record ID, locator `<change-relative Markdown>#<record-id>` and SHA-256 of its canonical record block. That block runs from its level-two heading to the next level-two heading (or EOF), normalizes line endings to LF, removes trailing whitespace at the block end and ends in one newline. Appending a review therefore leaves previous native record fingerprints unchanged. Internal edits alter the digest. Native fingerprints cover the source digest, so Agent/Model changes also change the record fingerprint. The compiler never overwrites prior submissions in ASS; submitted Markdown records must remain immutable. CI retains the Markdown, input packets and compiled bundle as publication artifacts.

A preparation receipt is exactly `{format: "azimuth-candidate-preparation", version: 1, project, model_revision}`, where `model_revision` is the complete authority projection fingerprint. `snapshot` produces exactly `{project, model_revision, authority, reviews, bundles}` for local workflow assessment. Neither is an ASS registration receipt or evidence of authenticated transport. Preparation and compilation neither select accepted authority nor deploy software. Publication uses existing owner, producer and reviewer endpoints in dependency order and preserves reviewer authority; one coordinator token cannot impersonate all reviewers.

Review-input packets retain the complete explicitly supplied Run history in `evidence_bundles`, including correction predecessors. The selected `observations` and `subjects` remain target-relevant. This permits packet replay and protocol validation without discarding the ancestors needed to interpret a corrected head. For the current authority, compilation regenerates the packet from its retained reviews and Run history and requires an exact match; historical authority packets remain digest-pinned and their native records are assessed against the current candidate.
