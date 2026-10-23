# Requirement Review and Requirement Status

Requirement Review assesses definitions. A Requirement Status record assesses observed behavior for one exact
Verification Target.
Neither execution, structural validation nor an implementation tag authors an independent review.
Check Review, Method Qualification, Applicability Decision and Requirement Judgment are not mandatory
steps in the current compact account. Their earlier records remain historical facts with their
original meanings; see [the previous contract](assurance-reviews-before-claim-health.md).

## Authority and dependencies

`azimuth assurance authority` exports the complete unselected model projection with stable Requirement,
Scenario, Check and Mechanism identities, inline Check–Scenario contributions and source support.
Candidate registration does not select the default model or authorize deployment; the default model is the latest main-model publication.
Review currentness follows relevant definitions and implementation/input fingerprints, rather
than unrelated changes to the whole model. Changing a relevant dependency requires reassessment.
The latest unambiguous assessment of a typed target is current; never fall back to an earlier
positive record when a newer record is rejected, stale or invalid.

## Requirement Review

Native kind: `requirement-review`. The independent reviewer examines Requirement/Scenario completeness, design,
realizations, Check implementations, input and expectation independence, oracles, coverage,
Verification Challenge definitions and resolved source scopes, assumptions and limitations.

A record pins `requirement`, `requirement_fingerprint` and exact `dependencies`, with
`decision: accepted | rejected`. It requires no execution Verification Target or Check Result basis.
Common record metadata retains the accountable `reviewer`, actual `reviewed_at_ms`, substantive
`rationale`, supporting `artifacts` and canonical `fingerprint`.
Acceptance expresses credibility of the reviewed account, not an observed Requirement Status.
Acceptance requires both an adequate verification approach and an implementation capable of
fulfilling its declared obligations, including required measurements, independent controls and
native reporting. Known material implementation gaps block acceptance even when documented.
Justified scope boundaries remain reviewable, but cannot omit a required proposition without
adequate alternative coverage. Execution may remain pending; unavailable execution environments
alone do not establish an implementation defect. The shipped `azimuth-review-requirement` skill guides
this semantic assessment; record compilation does not decide these criteria.

`dependencies` is the exact packet `{checks, scenarios, requirements, bindings}` selected for this Requirement.
Its definitions include realization sites and fingerprints, mechanisms, Check implementations,
Verification Challenge declarations and implementation/configuration support. Source paths are locators, not
entity identities. Review currentness checks these dependencies; a matching reviewer label or
unchanged prose alone cannot establish currentness.

## Requirement Status record

Native kind: `requirement-status`: a Requirement Status record for one Verification Target. The
assessor examines Check Results against the current reviewed Requirement and its Scenario
contributions for the exact Verification Target and recorded examination scope. Current
execution Verification Target kinds are `source`, `artifact` and `deployment`; local/CI origin is provenance.
Historical variants retain their original meanings and do not create automatic reuse.

A record pins `requirement`, `requirement_fingerprint`, exact `dependencies`, `verification_target_fingerprint`,
`requirement_review_fingerprint` and `basis` references to `{run, bundle_fingerprint, check_result}`.
It retains `conclusion: supported | violated | unresolved` and `residual_risks`, plus the common
review metadata. A supported conclusion requires a current accepted Requirement Review, complete applicable
Scenario support, credible controls and no unresolved structural or evidence gaps. A failed Check
alone is an adverse Check Result, not an independently assessed violated Requirement.
Missing measurements, broken controls, incomplete examination and conflicting evidence remain
unresolved. A definition review is not repeated merely because unchanged Checks execute again.

## Provenance and reassessment

Native records may carry optional `provenance` with exactly:

```json
{"agent": null, "model": null, "reassesses": null}
```

`agent` and `model` are null or nonempty labels supplied by the authoring harness. `reassesses`
is null or the canonical fingerprint of an earlier retained assessment. The predecessor must
have the same typed target; a Requirement Status record therefore also requires the same exact Verification Target. The
reassessment must have a strictly later `reviewed_at_ms`. Missing predecessors remain visible
missing dependencies, not an implicit approval. Labels retain provenance but do not prove
independence, authenticate the reviewer or grant publication permission.

## Challenge proof dependencies

New Requirement Review and Requirement Status records may carry optional `challenge_basis`, an array of
exact references:

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

Keep aggregate files such as `reviews/requirement-reviews.md` and `reviews/requirement-status.md` in the change.
Names are conventions, not identities. Headings use `## Requirement review: <record-id>` or
`## Requirement status: <record-id>`. Metadata pins an `Input` packet, `Input fingerprint`, `Requirement`,
`Reviewer`, actual UTC `Reviewed at`, and `Decision` or `Conclusion`; rationale is substantive
Markdown. A Requirement Status record also retains residual risks. Optional `Agent`, `Model` and `Reassesses` retain
provenance and correction relationships. Markdown `Reassesses` names an earlier record ID in
the aggregate bundle; compilation resolves that ID to its native fingerprint. Supplied labels alone confer no independence or permission.

`azimuth assurance review-input` retains the selected assessments and their transitive reassessment predecessors in deterministic fingerprint order. This retained history reproduces review states during compilation; a genuinely absent predecessor remains an explicit gap. Unrelated review histories are not added.

`azimuth assurance review-input` prepares exact definitions, dependencies, existing assessments,
visible gaps and, for Requirement Status only, matching Verification Targets/Check Results. Packets may also include optional
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

A machine-executed Verification Challenge targets an exact Requirement Review or Requirement Status assessment selected
at planning time. Its execution Verification Target is separate from the assessment target. The current
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
records, relevant adverse/unresolved evidence and provenance. Subsequent hosted evidence and
Requirement Status assessments are published after authorized deployment.
CI transport does not change the author of a review. Submission still requires an authorized
reviewer credential matching the recorded reviewer; producer permission alone cannot submit one.
Publish relied-on Runs before their dependent review records, and predecessors before
reassessments, so the service can validate exact proof dependencies. Submission assesses against
the revision-linked review history and existing latest Run correction heads; native optional
fields are retained unchanged in the immutable record.
The service preserves immutable histories and exposes currentness and dependency gaps against
the default model's authority. Source, artifact and deployed evidence cannot be joined solely because
their model revision matches. Any cross-target reliance requires an explicit applicability and
provenance contract; unsupported composition remains a visible gap.

## Service inspection and migration

The service preserves native JSONB records and explicit model-revision links without rewriting
older reviews. New review submissions accept Requirement Review and Requirement Status records only. Definition
assessments are exposed independently of Verification Targets through selected and candidate
`requirement-reviews` resources; exact-target state exposes Review currentness and Requirement Status separately.
Authenticated submitter and receive time are transport facts, distinct from authored review
provenance. Missing, stale, rejected and ambiguous records cannot infer a supported Requirement Status.

There is no authority selection, release acceptance or reassessment from execution.
Historical parsers retain their original meanings for inspection; they do not convert old
qualifications or judgments into new reviews. After new-kind ingestion, application rollback
requires a parser capable of those records or an isolated pre-rollout database recovery; deleting
or rewriting immutable history is not a migration strategy.

## Candidate relocation

New candidate authority fingerprints exclude machine-local model/workspace document paths, declaration line numbers and validation findings. Repository-relative implementation locators, qualified source identities, source fingerprints, definitions and workspace mount configuration remain in the model identity. Moving a local candidate folder does not create a new authority; changing its definitions or source support does. Historical authority records retain their original fingerprints and are not rewritten.
