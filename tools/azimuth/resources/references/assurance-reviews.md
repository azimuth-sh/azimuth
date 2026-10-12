# Claim Review and Claim Health

Claim Review assesses definitions. Claim Health assesses observed behavior for an exact Subject.
Neither execution, structural validation nor an implementation tag authors an independent review.
Check Review, Method Qualification, Applicability Decision and Claim Judgment are not mandatory
steps in the current compact account. Their earlier records remain historical facts with their
original meanings; see [the previous contract](https://github.com/azimuth-sh/azimuth/blob/main/contracts/history/assurance-reviews-before-claim-health.md).

## Authority and dependencies

`azimuth assurance authority` exports the complete unselected model projection with stable Claim,
Case, Check and Mechanism identities, inline Check–Case contributions and source support.
Candidate registration does not select accepted authority or authorize deployment.
Review currentness follows relevant definitions and implementation/input fingerprints, rather
than unrelated changes to the whole model. Changing a relevant dependency requires reassessment.
The latest unambiguous assessment of a typed target is current; never fall back to an earlier
positive record when a newer record is rejected, stale or invalid.

## Claim Review

Native kind: `claim-review`. The independent reviewer examines Claim/Case completeness, design,
realizations, Check implementations, input and expectation independence, oracles, coverage,
Challenger definitions and resolved source scopes, assumptions and limitations.

A record pins `claim`, `claim_fingerprint` and exact `dependencies`, with
`decision: accepted | rejected`. It requires no execution Subject or Observation basis.
Common record metadata retains the accountable `reviewer`, actual `reviewed_at_ms`, substantive
`rationale`, supporting `artifacts` and canonical `fingerprint`.
Acceptance expresses credibility of the reviewed account, not observed product health.

`dependencies` is the exact packet `{checks, cases, claims, bindings}` selected for this Claim.
Its definitions include realization sites and fingerprints, mechanisms, Check implementations,
Challenger declarations and implementation/configuration support. Source paths are locators, not
entity identities. Review currentness checks these dependencies; a matching reviewer label or
unchanged prose alone cannot establish currentness.

## Claim Health

Native kind: `claim-health`. The assessor examines Observations against the current reviewed
Claim and its Case contributions for the exact Subject and recorded examination scope. Current
execution Subject kinds are `source`, `artifact` and `deployment`; local/CI origin is provenance.
Historical variants retain their original meanings and do not create automatic reuse.

A record pins `claim`, `claim_fingerprint`, exact `dependencies`, `subject_fingerprint`,
`claim_review_fingerprint` and `basis` references to `{run, bundle_fingerprint, observation}`.
It retains `conclusion: supported | violated | unresolved` and `residual_risks`, plus the common
review metadata. Supported health requires a current accepted Claim Review, complete applicable
Case support, credible controls and no unresolved structural or evidence gaps. A failed Check
alone is an adverse observation, not an independently assessed violated Claim.
Missing measurements, broken controls, incomplete examination and conflicting evidence remain
unresolved. A definition review is not repeated merely because unchanged Checks execute again.

## Provenance and reassessment

Native records may carry optional `provenance` with exactly:

```json
{"agent": null, "model": null, "reassesses": null}
```

`agent` and `model` are null or nonempty labels supplied by the authoring harness. `reassesses`
is null or the canonical fingerprint of an earlier retained assessment. The predecessor must
have the same typed target; Claim Health therefore also requires the same exact Subject. The
reassessment must have a strictly later `reviewed_at_ms`. Missing predecessors remain visible
missing dependencies, not an implicit approval. Labels retain provenance but do not prove
independence, authenticate the reviewer or grant publication permission.

## Challenge proof dependencies

New Claim Review and Claim Health records may carry optional `challenge_basis`, an array of
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
negative search fact. These references do not give Claim Review an execution Subject or product
Observation basis: the Challenger Run retains its execution Subject separately.

## Authored records and preparation

Keep aggregate files such as `reviews/claim-reviews.md` and `reviews/claim-health.md` in the change.
Names are conventions, not identities. Headings use `## Claim review: <record-id>` or
`## Claim health: <record-id>`. Metadata pins an `Input` packet, `Input fingerprint`, `Claim`,
`Reviewer`, actual UTC `Reviewed at`, and `Decision` or `Conclusion`; rationale is substantive
Markdown. Health also retains residual risks. Optional `Agent`, `Model` and `Reassesses` retain
provenance and correction relationships. Markdown `Reassesses` names an earlier record ID in
the aggregate bundle; compilation resolves that ID to its native fingerprint. Supplied labels alone confer no independence or permission.

`azimuth assurance review-input` prepares exact definitions, dependencies, existing assessments,
visible gaps and, for health only, matching Subjects/Observations. Packets may also include optional
`challenge_bundles`: scoped native Run proofs for the selected assessment objections, not a
dump of all project execution history. Compilation validates those bundles and pins the
corresponding `challenge_basis` references. It creates no reviewer, time,
decision or positive conclusion. `azimuth assurance review-compile` compiles aggregate Markdown
against those exact packets and the current account. Placeholders and changed inputs fail.
Generated packets, compiled bundles and execution reports belong in ignored change `.azimuth/`.

## Challenging an assessment

A machine-executed Challenger targets an exact Claim Review or Claim Health assessment selected
at planning time. Its execution Subject is separate from the assessment target. The current
Stryker integration pins the exact source input for an isolated mutation batch and retains native
mutation statuses and locations. It does not materialize exact per-mutant Subjects or mutant-specific
product Observations. Such captures would require distinct Subjects before those observations
could be used as product evidence; a batch result is not evidence of unmodified deployment health.
Results are `clean | findings | inconclusive`. Clean only means the bounded search found no
objection. Findings require disposition and correction/reassessment where relevant, and cannot
be silently ignored when relying on the attacked assessment. No mutation score or clean search
establishes positive health.

## Publication and service authority

Local drafts, correction attempts and exploratory Runs remain local by default. CI publishes the
current package relied on for delivery, including exact Subjects, unchanged local reviewer
records, relevant adverse/unresolved evidence and provenance. Subsequent hosted evidence and
health assessments are published after authorized deployment.
CI transport does not change the author of a review. Submission still requires an authorized
reviewer credential matching the recorded reviewer; producer permission alone cannot submit one.
Publish relied-on Runs before their dependent review records, and predecessors before
reassessments, so the service can validate exact proof dependencies. Submission assesses against
the revision-linked review history and existing latest Run correction heads; native optional
fields are retained unchanged in the immutable record.
The service preserves immutable histories and exposes currentness and dependency gaps against
explicit model authority. Source, artifact and deployed evidence cannot be joined solely because
their model revision matches. Any cross-Subject reliance requires an explicit applicability and
provenance contract; unsupported composition remains a visible gap.

## Service inspection and migration

The service preserves native JSONB records and explicit model-revision links without rewriting
older reviews. New review submissions accept Claim Review and Claim Health only. Definition
assessments are exposed independently of Subjects through selected and candidate
`claim-reviews` resources; exact-Subject state exposes Review currentness and Health separately.
Authenticated submitter and receive time are transport facts, distinct from authored review
provenance. Missing, stale, rejected and ambiguous records cannot infer supported health.

There is no automatic authority selection, deployment acceptance or reassessment from execution.
Historical parsers retain their original meanings for inspection; they do not convert old
qualifications or judgments into new reviews. After new-kind ingestion, application rollback
requires a parser capable of those records or an isolated pre-rollout database recovery; deleting
or rewriting immutable history is not a migration strategy.
