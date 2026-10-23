# Requirement Review

Requirement Review assesses definitions. Requirement Status is never authored: deployment Requirement
Status is computed by the Azimuth Assurance Service from applicable evidence.
Neither execution, structural validation nor an implementation tag authors an independent review.
Check Review, Method Qualification, Applicability Decision and Requirement Judgment are not mandatory
steps in the current compact account. Their earlier records remain historical facts with their
original meanings; see [the claim-health contract](https://github.com/azimuth-sh/azimuth/blob/main/contracts/history/assurance-reviews-before-claim-health.md).
The authored `requirement-status` record kind, review `provenance` and the review `dependencies`
packet are removed without a compatibility reader; [the previous contract](https://github.com/azimuth-sh/azimuth/blob/main/contracts/history/assurance-reviews-before-work-provenance.md)
records their former shape. Who performed a review (agent, model, effort, harness) is recorded in the
change's Work Provenance ledger ([work-provenance.md](https://github.com/azimuth-sh/azimuth/blob/main/contracts/work-provenance.md)), not in the review record.

## Authority and dependencies

`azimuth assurance authority` exports the complete unselected model projection with stable Requirement,
Scenario, Check and Mechanism identities, inline Check–Scenario contributions and source support.
Candidate registration does not select the default model or authorize deployment; the default model is the latest main-model publication.
The projection also carries top-level `requirement_intents: [{"id", "fingerprint"}]`, sorted by `id`,
with exactly one entry per Requirement. `fingerprint` is the Requirement intent digest. It sits
outside every review dependency packet, so it identifies intent changes without ever staling a
Requirement Review.
Review currentness follows relevant definitions and implementation/input fingerprints, rather
than unrelated changes to the whole model. Changing a relevant dependency requires reassessment.
The latest unambiguous assessment of a typed target is current; never fall back to an earlier
positive record when a newer record is rejected, stale or invalid.

## Requirement Review

Native kind: `requirement-review`. The independent reviewer examines Requirement/Scenario completeness, design,
realizations, Check implementations, input and expectation independence, oracles, coverage,
Verification Challenge definitions and resolved source scopes, assumptions and limitations.

A record has exactly these fields, all required:

```json
{
  "kind": "requirement-review",
  "reviewer": "…",
  "reviewed_at_ms": 0,
  "rationale": "…",
  "artifacts": [{"id": "…", "digest": "sha256:…", "locator": "…"}],
  "requirement": "…",
  "requirement_fingerprint": "sha256:…",
  "dependencies_fingerprint": "sha256:…",
  "decision": "accepted",
  "reassesses": null,
  "challenge_basis": [],
  "fingerprint": "sha256:…"
}
```

`decision` is `accepted | rejected`. The record requires no execution Verification Target or Check
Result basis. `reviewer` is the accountable reviewer, `reviewed_at_ms` the actual review time,
`rationale` substantive, `artifacts` nonempty with unique ids, and `fingerprint` the canonical
fingerprint of every other field.
Acceptance expresses credibility of the reviewed account, not an observed Requirement Status.
Acceptance requires both an adequate verification approach and an implementation capable of
fulfilling its declared obligations, including required measurements, independent controls and
native reporting. Known material implementation gaps block acceptance even when documented.
Justified scope boundaries remain reviewable, but cannot omit a required proposition without
adequate alternative coverage. Execution may remain pending; unavailable execution environments
alone do not establish an implementation defect. The shipped `azimuth-review-requirement` skill guides
this semantic assessment; record compilation does not decide these criteria.

`dependencies_fingerprint` is the canonical SHA-256 of the exact review dependency packet
`{checks, scenarios, requirements, bindings}` selected for this Requirement from the authority
(`requirement_dependencies`). The packet's definitions include realization sites and fingerprints,
mechanisms, Check implementations, Verification Challenge declarations and implementation/configuration
support. The record stores only the fingerprint; the packet itself is in the review input and is
recoverable from the stored authority of the reviewed model revision. Source paths are locators, not
entity identities. A review is current only while both `requirement_fingerprint` and
`dependencies_fingerprint` equal the current authority's; a matching reviewer label or unchanged
prose alone cannot establish currentness.

## Reassessment

`reassesses` is null or the canonical fingerprint of an earlier retained Requirement Review. The
predecessor must have the same typed target (the same Requirement) and a strictly earlier
`reviewed_at_ms`; otherwise the successor is invalid. A predecessor missing from the retained history
leaves the successor `missing`, not an implicit approval.

## Challenge proof dependencies

Every Requirement Review carries `challenge_basis`, an array (possibly empty) of exact references:

```json
[
  {
    "run": "sha256:<run-id>",
    "bundle_fingerprint": "sha256:<bundle-fingerprint>",
    "result_fingerprint": "sha256:<challenge-result-fingerprint>",
    "target_fingerprint": "sha256:<challenged-assessment-fingerprint>"
  }
]
```

Each value is a canonical SHA-256 fingerprint. Exact reference tuples must be unique; authored
array order has no separate semantic requirement. Preparation emits them deterministically from
current Run correction heads and recorded execution order. A missing or corrected Run head,
changed result or changed target makes the dependent assessment stale. Prior assessment targets
may supply explicit objection context; their presence never converts historical conclusions into
positive current assurance.

A matching adverse Challenge Result attacking the assessment's own exact fingerprint leaves it
unresolved pending reassessment. An independently authored successor can assess the prior-target
objection in its rationale and retain these exact proof dependencies. A clean result remains a
negative search fact. These references do not give Requirement Review an execution Verification Target or product
Check Result basis: the Verification Challenge Run retains its execution Verification Target separately.

## Authored records and preparation

Keep aggregate files such as `reviews/requirement-reviews.md` in the change. Names are conventions,
not identities. Headings use `## Requirement review: <record-id>`. Metadata pins an `Input` packet,
`Input fingerprint`, `Requirement`, `Reviewer`, actual UTC `Reviewed at` and `Decision`; rationale is
substantive Markdown. Optional `Reassesses` names an earlier record ID in the aggregate bundle;
compilation resolves that ID to its native fingerprint. Compilation rejects a `- Agent:` or `- Model:`
field and a `## Requirement status:` heading with a `<file>:<line>:` error: agent and model provenance
belongs to the Work Provenance ledger, and Requirement Status is computed rather than authored.

The compiled `azimuth-review-bundle` (version 2) holds `{format, version, project,
authority_fingerprint, records, fingerprint}`. Each `records` entry is exactly
`{id, input: {ref, fingerprint}, reassesses, record, assessment: {status, latest, diagnostics}}`, where
entry `reassesses` is the Markdown record ID or null and `record` is the native record above.

`azimuth assurance review-input` retains the selected assessments and their transitive reassessment predecessors in deterministic fingerprint order. This retained history reproduces review states during compilation; a genuinely absent predecessor remains an explicit gap. Unrelated review histories are not added.

`azimuth assurance review-input` accepts only `{"kind": "requirement-review", "requirement": "<id>"}`
and prepares exact definitions, the full dependency packet, existing assessments and visible gaps.
Its `verification_targets`, `check_results` and `evidence_bundles` arrays are always empty for a
definition review. Packets may also include optional
`challenge_bundles`: scoped native Run proofs for the selected assessment objections, not a
dump of all project execution history. Compilation validates those bundles and pins the
corresponding `challenge_basis` references. It creates no reviewer, time,
decision or positive conclusion. `azimuth assurance review-compile` compiles aggregate Markdown
against those exact packets and the current account. Placeholders and changed inputs fail.
Generated packets and execution reports belong in ignored change `.azimuth/`. The compiled bundle
is printed (or written with `--out`) and is not committed. `review-compile` also seals each compiled
native review record into the change's `reviews/` directory beside the Markdown, as the committed JSON
the ratification gate reads: a Markdown file holding one review seals to `reviews/<stem>.json`; one
holding several seals each to `reviews/<stem>.<review-id>.json`. The record is created at review time
against the reviewed model by the reviewer's own step, and the author commits Markdown and sealed JSON
together. Recompiling an identical record leaves the file untouched; a different record at the same name
fails unless `--replace-sealed` is passed, and two reviews never seal to one file.

## Challenging an assessment

A machine-executed Verification Challenge targets an exact Requirement Review selected at planning
time; `requirement-review` is the only assessment target kind. Its execution Verification Target is separate from the assessment target. The current
Stryker integration pins the exact source input for an isolated mutation batch and retains native
mutation statuses and locations. It does not materialize exact per-mutant Verification Targets or mutant-specific
product Check Results. Such captures would require distinct Verification Targets before those Check Results
could be used as product evidence; a batch result is not evidence of the Requirement Status of the unmodified deployment.
Results are `clean | findings | inconclusive`. Clean only means the bounded search found no
objection. Findings require disposition and correction/reassessment where relevant, and cannot
be silently ignored when relying on the attacked assessment. No mutation score or clean search
establishes a supported Requirement Status.

## Publication and service authority

Local drafts, correction attempts and exploratory Runs remain local by default. CI publishes the
current package relied on for delivery, including exact Verification Targets, unchanged local reviewer
records, relevant adverse/unresolved evidence and provenance. Subsequent hosted evidence is published
after authorized deployment, and the service computes deployment Requirement Status from it.
CI transport does not change the author of a review. Submission still requires an authorized
reviewer credential matching the recorded reviewer; producer permission alone cannot submit one.
Publish relied-on Runs before their dependent review records, and predecessors before
reassessments, so the service can validate exact proof dependencies. Submission assesses against
the revision-linked review history and existing latest Run correction heads; the record is retained
unchanged and immutable.
The service preserves immutable histories and exposes currentness and dependency gaps against
the default model's authority. Source, artifact and deployed evidence cannot be joined solely because
their model revision matches. Any cross-target reliance requires an explicit applicability and
provenance contract; unsupported composition remains a visible gap.

## Service inspection and migration

The service stores reviews with explicit model-revision links without rewriting them; its storage is
described in [run-ledger.md](https://github.com/azimuth-sh/azimuth/blob/main/contracts/run-ledger.md). Review submissions accept Requirement Review records only.
Definition assessments are exposed independently of Verification Targets through selected and candidate
`requirement-reviews` resources. Exact-target state for a source or artifact Verification Target
exposes Review currentness and the observed Check Results as unassessed facts with status
`unresolved`; only a deployment's Requirement Status is computed. Authenticated submitter and receive
time are transport facts; who performed a review is Work Provenance. Missing, stale, rejected and
ambiguous records cannot infer a supported Requirement Status.

There is no authority selection, release acceptance or reassessment from execution. Core's
historical parsers for Method Qualification, Applicability Decision and Requirement Judgment retain
their original meanings for inspection; they do not convert old qualifications or judgments into new
reviews. The removal of `requirement-status`, review `provenance` and the `dependencies` packet has no
compatibility reader: the Assurance Service database is reset and current reviews are re-encoded.

## Deployment Requirement Status

The standard change-development revision 10 requires independent Requirement Reviews and machine execution facts before publication; there is no authored Requirement Status record. AAS computes deployment Requirement Status after deployment reconciliation, using reviewed definitions, exact Run Verification Targets, release provenance and compatible effective-context captures. Who performed each review is recorded in the change's Work Provenance ledger; see `azimuth reference show work-provenance`.
