# Realization and Verification Reviews

Two independent reviews are global: they are stored in the model and travel with every main-model and release
publication. Every other review — the owner's intent acceptance, proposal review and the delta review of each
authored delta — is local to the change workflow and is retained in the change's archived decision record.

| Review | Record kind | Owed by | Question |
|---|---|---|---|
| Realization Review | `realization-review` | `Realization: reviewed` | Does the code at the Requirement's realization sites do what its Scenarios say, consistent with its `realization.md` entries where they exist? |
| Verification Review | `verification-review` | `Verification: reviewed` | Is the verification credible: Checks, oracles, coverage, input and expectation independence, controls and the adequacy of the declared or absent Challenges? |

A Requirement that declares neither label owes neither review ([spec.md](spec.md)). Requirement Status is never
authored: deployment Requirement Status is computed by the Azimuth Assurance Service from applicable evidence
([deployment-requirement-status.md](deployment-requirement-status.md)). A Realization Review is an opinion about a
code snapshot, not evidence; it never makes a Requirement Satisfied. Neither execution, structural validation nor an
implementation tag authors a review. Who performed a review (agent, model, effort, harness) is recorded in the
change's Work Provenance ledger ([work-provenance.md](work-provenance.md)), not in the review record.

## Authority and dependencies

`azimuth assurance authority` exports the complete unselected model projection with stable Requirement,
Scenario, Check and Mechanism identities, inline Check–Scenario contributions and source support. Each Requirement
carries `realization` and `verification`, each `none | reviewed`.
Candidate registration does not select the default model or authorize deployment; the default model is the latest main-model publication.
The projection also carries top-level `requirement_intents: [{"id", "fingerprint"}]`, sorted by `id`,
with exactly one entry per Requirement. `fingerprint` is the Requirement intent digest. It sits
outside every review dependency packet, so it identifies intent changes without itself staling a review.

Each review kind has its own dependency packet, selected per Requirement from the authority. A dependency is either
blocking — any change makes the review stale — or informational, reported beside the review and never staling it.

| Kind | Blocking dependencies (packet) | Informational |
|---|---|---|
| `verification-review` | `requirement`: the Requirement's semantic digest, covering its statement, `Realization:` and `Verification:` declarations, `Context:` lines and module Terms; `scenarios`; `checks`: Check definitions and their Evidence Bindings; `challenges`: Verification Challenge declarations selecting the Requirement; `units`: the execution units each Check executes, each with its unit input fingerprint ([unit-inputs.md](unit-inputs.md)) or, without unit rules, the Check's `ImplementsCheck` site fingerprints | none |
| `realization-review` | `requirement`: the Requirement intent digest, including its `Realization:` and `Verification:` declarations; `scenarios`; `realization`: the Requirement's `requirement_realizations` entry without `sites_fingerprint` — its realization declaration, the realization digest ([account-realization.md](account-realization.md)) or null when no `realization.md` has entries for it, the `realization.md`-derived Mechanism definition fingerprints and the realization gaps | code drift: `sites_fingerprint` |

A Verification Review does not depend on product realization sites or Mechanisms at all: they are outside its
packet, so a change to production code that leaves the Checks, their bindings, their Challenges and their execution
units unchanged does not stale it. A Check depends on the units it executes, not on whole files that contain them.

Each `requirement_realizations` entry of the authority carries `sites_fingerprint`, a canonical fingerprint over
the Requirement's realization sites: its `Realizes` sites and the `ImplementsMechanism` sites of its Mechanisms,
each with its source identity and source fingerprint. Code drift is a boolean: a Realization Review drifts when its
recorded `sites_fingerprint` differs from the current one. It is displayed as `reviewed at <reviewed_commit>, code
at the realization sites changed since`; no per-site or per-file count exists. Drift never stales, rejects or
invalidates a Realization Review.

The latest unambiguous assessment of a typed target is current; never fall back to an earlier positive record when
a newer record is rejected, stale or invalid.

## Records

A `verification-review` record has these required fields:

```json
{
  "kind": "verification-review",
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

A `verification-review` record has no `reviewed_commit`. A `realization-review` record has the same fields except
`challenge_basis`, with `kind: "realization-review"`, and adds two required fields: `reviewed_commit`, a nonempty
text naming the commit of the code the reviewer examined, and `sites_fingerprint`, the authority's `sites_fingerprint`
for the Requirement copied from the review input. `sites_fingerprint` is the code-drift baseline and is not part of
the dependency packet. Neither kind carries a list of sites.

Both kinds carry optional `issues` and `dispositions` (below). `decision` is `accepted | rejected`. The typed target
is a Requirement, not an execution Verification Target. `reviewer` is the accountable reviewer, `reviewed_at_ms`
the actual review time, `rationale`
substantive, `artifacts` nonempty with unique ids, and `fingerprint` the canonical fingerprint of every other field.
`dependencies_fingerprint` is the canonical SHA-256 of the kind's exact blocking packet. The record stores only
that fingerprint; the packet itself is in the review input and is recoverable from the stored authority of the
reviewed model revision. Source paths are locators, not entity identities.

A Verification Review's acceptance expresses credibility of the reviewed verification, not an observed Requirement
Status. It requires an adequate verification approach and Checks capable of fulfilling their declared obligations,
including required measurements, independent controls and native reporting. Known material gaps block acceptance
even when documented. Justified scope boundaries remain reviewable, but cannot omit a required proposition without
adequate alternative coverage. Execution may remain pending; unavailable execution environments alone do not
establish a defect. Retained Check and implementation-Challenge executions can inform the assessment; they do not
establish Requirement Status.

A Realization Review's acceptance expresses that the code at the realization sites, as of `reviewed_commit`, does what the
Scenarios say and is consistent with the Requirement's realization entries. Missing realization of a Scenario,
contradiction of a realization entry or a site that cannot be read blocks acceptance.

The shipped `azimuth-review-verification` and `azimuth-review-realization` skills guide the two semantic assessments; record compilation does not decide
these criteria.

A review is current only while both `requirement_fingerprint` and `dependencies_fingerprint` equal the current
authority's and, for a Verification Review, its explicitly relied-on Challenge proofs remain current. Implementation
proof currentness recomputes the target basis from current authority, prepared scope and exact Run Verification
Target; a matching reviewer label or unchanged prose alone cannot establish currentness. Currency is computed,
never stored.

## Stable review issues and dispositions

Review drafts scaffold `- Issues: []` and `- Dispositions: []` as inline JSON arrays in Markdown metadata. Authoring requires both fields; compilation retains them as native arrays. An issue has exactly `{id, proposition, location, reason, correction, blocking}`: the id is a lower-kebab identifier, the four explanatory fields are nonempty strings, and `blocking` is boolean. IDs are unique inside the review. A rejection requires at least one issue; an acceptance cannot retain a blocking issue.

A disposition has exactly `{issue, outcome, rationale}` with nonempty strings, one entry per issue reference. Outcomes are `corrected | equivalent | irrelevant | confirmed-defect | unresolved`. Acceptance cannot retain a `confirmed-defect` or `unresolved` disposition. A predecessor review issue reference is `<native-review-fingerprint>#<local-issue-id>`; a successor accepting the same Requirement and kind must explicitly disposition every blocking predecessor issue. The independent reviewer inspects source rather than treating the author's correction report as proof.

Implementation Challenge objections receive deterministic IDs `challenge-issue-<sha256>`, from canonical `{verification_challenge, target, objection}` where the values are the Challenge ID, exact implementation target fingerprint and native objection diagnostic ID. Repeating the same objection against the same basis keeps its identity; changed bases remain distinguishable. Packet `definitions.challenge_issues` retains these IDs and their exact result/target references. An accepted Verification Review must independently disposition every relied-on current implementation objection as `equivalent` or `irrelevant`. A confirmed defect needs correction and affected execution; describing it does not close it. These authored issues are distinct from deterministic model Findings in [findings.md](findings.md).

## Reassessment

`reassesses` is null or the canonical fingerprint of an earlier retained review of the same kind. The
predecessor must have the same typed target (the same Requirement) and a strictly earlier
`reviewed_at_ms`; otherwise the successor is invalid. A predecessor missing from the retained history
leaves the successor `missing`, not an implicit approval.

## Challenge proof dependencies

Every Verification Review carries `challenge_basis`, an array (possibly empty) of exact references to the implementation-directed Challenge Runs it relies on:

```json
[
  {
    "run": "sha256:<run-id>",
    "bundle_fingerprint": "sha256:<bundle-fingerprint>",
    "result_fingerprint": "sha256:<challenge-result-fingerprint>",
    "target_fingerprint": "sha256:<challenged-basis-fingerprint>"
  }
]
```

Each value is a canonical SHA-256 fingerprint. Exact reference tuples must be unique; authored
array order has no separate semantic requirement. Preparation emits them deterministically from
current Run correction heads and recorded execution order. A missing or corrected Run head,
changed result or changed target makes the dependent review stale. A clean result remains a
negative search fact. These references do not give the review an execution Verification Target or product
Check Result basis: the Verification Challenge Run retains its execution Verification Target separately.

## Authored records and preparation

Keep aggregate files `reviews/verification-reviews.md` and `reviews/realization-reviews.md` in the change. Names are
conventions, not identities. Headings use `## Verification review: <record-id>` or `## Realization review:
<record-id>`. Metadata pins an `Input` packet, `Input fingerprint`, `Requirement`, `Reviewer`, actual UTC
`Reviewed at`, `Decision` and, for a Realization Review only, `Reviewed commit` (a Verification Review carrying it is rejected); rationale is substantive Markdown. Optional `Reassesses` names an
earlier record ID of the same kind in the aggregate bundle; compilation resolves that ID to its native fingerprint.
Compilation rejects a `- Agent:` or `- Model:` field and a `## Requirement status:` heading with a `<file>:<line>:`
error: agent and model provenance belongs to the Work Provenance ledger, and Requirement Status is computed rather
than authored.

The compiled `azimuth-review-bundle` (version 2) holds `{format, version, project,
authority_fingerprint, records, fingerprint}`. Each `records` entry is exactly
`{id, input: {ref, fingerprint}, reassesses, record, assessment: {status, latest, diagnostics}}`, where
entry `reassesses` is the Markdown record ID or null and `record` is the native record above.

`azimuth assurance review-input` accepts `{"kind": "verification-review" | "realization-review", "requirement": "<id>"}`
and prepares exact definitions, the kind's full dependency packet, existing assessments of that kind and visible
gaps; a Requirement that does not declare the matching label is an input error. For a Realization Review it also
prepares `definitions.sites_fingerprint`, the current fingerprint over the realization sites, which compilation copies into the record. It retains the selected assessments and their
transitive reassessment predecessors in deterministic fingerprint order; a genuinely absent predecessor remains an
explicit gap. For a Verification Review its `verification_targets`, `check_results` and `evidence_bundles` arrays
retain relevant native Check execution context when supplied. `definitions.review_issues` includes exact predecessor
issue references. Relevant implementation-Challenge results and stable objection IDs are retained under
`definitions.challenge_results` and `definitions.challenge_issues`; complete correction history for each selected
Run is retained in `challenge_bundles` or `evidence_bundles`. Compilation validates those bundles, pins the
corresponding `challenge_basis` references, and assesses the compiled record with the packet's retained Runs.
Inconclusive relied-on implementation examination blocks acceptance; unresolved objections require independent
dispositions. Preparation creates no reviewer, time, decision or positive conclusion.

`azimuth assurance review-compile` compiles aggregate Markdown against those exact packets and the current account.
Placeholders and changed inputs fail. Draft packets and execution reports may initially live in ignored change
`.azimuth/`. Before delivery, the required immutable review inputs, native proof Runs and referenced supporting files
must be retained in the change's verified retained-input closure ([retained-inputs.md](retained-inputs.md)).
`review-compile` seals each compiled native record into the change's `reviews/` directory beside the Markdown: a
Markdown file holding one review seals to `reviews/<stem>.json`; one holding several seals each to
`reviews/<stem>.<review-id>.json`. The author commits Markdown and sealed JSON together. Recompiling an identical
record leaves the file untouched; a different record at the same name fails unless `--replace-sealed` is passed,
and two reviews never seal to one file. The sealed JSON under `reviews/` is the review history; it stays in the
change and moves with it into the archive.

## Review heads in the model

The latest ratifying review of each kind for each Requirement is stored in the model beside the spec, in
`<module>/verification-review.md` and `<module>/realization-review.md`. These files are not authored and never
appear in a delta. `azimuth change finalize` ([validation.md](validation.md)) writes them from the change's sealed
reviews; no other command writes them.

~~~markdown
# Realization reviews: payments/recovery

## accepted-write

- Decision: accepted
- Reviewer: reviewer@example
- Reviewed at: 2026-10-25T09:14:03Z
- Reviewed commit: 0123456789abcdef0123456789abcdef01234567
- Review: `sha256:<64-lowercase-hex>`
- Dependencies: `sha256:<64-lowercase-hex>`
- Issue `oracle-reads-fixture`: <proposition>

```json
{
  "kind": "realization-review",
  "...": "the complete sealed native record, pretty-printed"
}
```
~~~

- The first line is `# Verification reviews: <module-id>` or `# Realization reviews: <module-id>` and names the module of the sibling `spec.md`.
- There is one `## <requirement-id>` section per head, sorted by Requirement id. A module with no head of a kind has no file of that kind.
- Each section's bullet lines are, in order: `- Decision:`, `- Reviewer:`, `- Reviewed at:` (UTC RFC 3339), `- Reviewed commit:` (Realization only), `- Review:` with the record `fingerprint`, `- Dependencies:` with the `dependencies_fingerprint`, and one `- Issue` line per issue, carrying the issue id in backticks and its proposition. There is no rationale prose, site line or change reference.
- The section ends with exactly one fenced `json` block holding the complete sealed native record.

Loading parses every section's record, requires it to be a valid native record of the file's kind whose
`requirement` equals the section heading, then re-renders the whole file from the parsed records and rejects any
difference. A hand edit of any line, readable or JSON, therefore fails model loading; there is no `invalid` head state.

Ratification is recomputed on every scan rather than read from the heads. For every Requirement that declares a
rigor label, the change reviews and the model heads of that kind compete: the latest by review time is selected and
assessed against the merged model, and it ratifies only when it is `accepted`. Finalization writes a head for each
ratification the model does not already carry with the same record fingerprint. A Requirement that declares
`Verification:` without a ratifying Verification Review keeps the `missing-requirement-assurance` Finding; one that
declares `Realization:` without a ratifying Realization Review keeps `missing-realization-review`; a head whose
Requirement is absent or no longer declares that review is `unknown-review-head` ([findings.md](findings.md)).
Code drift is reported beside a Realization Review's state and never fails a scan.

## Challenging implementation and verification

A machine-executed Verification Challenge targets the exact implementation and Check basis selected at planning time, before the Verification Review. Its execution Verification Target is separate from the review target. The current
Stryker integration pins the exact source input for an isolated mutation batch and retains native
mutation statuses and locations. It does not materialize exact per-mutant Verification Targets or mutant-specific
product Check Results. Such captures would require distinct Verification Targets before those Check Results
could be used as product evidence; a batch result is not evidence of the Requirement Status of the unmodified deployment.
Results are `clean | findings | inconclusive`. Clean only means the bounded search found no
objection. Findings require disposition and correction/reassessment where relevant, and cannot
be silently ignored when relying on the attacked verification. No mutation score or clean search
establishes a supported Requirement Status.

## Publication and service authority

Local drafts, correction attempts and exploratory Runs remain local by default. Trusted CI publishes each main
commit's model with the sealed records of every review head in it ([run-ledger.md](run-ledger.md)); a release
publication carries the heads of its own model in its snapshot ([release-publication.md](release-publication.md)).
Relied-on Runs are published before the heads that depend on them, and predecessors before reassessments, so the
service can validate exact proof dependencies. CI transport does not change the author of a review. Direct
submission still requires an authorized reviewer credential matching the recorded reviewer; producer permission
alone cannot submit one. The record is retained unchanged and immutable.

The service preserves immutable histories and computes review assessments and code drift against the default model's
authority. Source, artifact and deployed evidence cannot be joined solely because their model revision matches. Any
cross-target reliance requires an explicit applicability and provenance contract; unsupported composition remains a
visible gap.

## Service inspection

The service stores reviews with explicit model-revision links without rewriting them; its storage is described in
[run-ledger.md](run-ledger.md). Review submissions accept `verification-review` and `realization-review` records
only. Assessments are exposed independently of Verification Targets through selected and candidate
`requirement-reviews` resources, each entry naming its kind. Exact-target state for a source or artifact Verification Target exposes both
review dimensions and the observed Check Results as unassessed facts with status `unresolved`; only a deployment's
Requirement Status is computed. Authenticated submitter and receive time are transport facts; who performed a
review is Work Provenance. Missing, stale, rejected and ambiguous records cannot infer a supported Requirement Status.

## Candidate relocation

Candidate authority fingerprints exclude machine-local model/workspace document paths, declaration line numbers and validation findings. Repository-relative implementation locators, qualified source identities, source fingerprints, definitions and workspace mount configuration remain in the model identity. Moving a local candidate folder does not create a new authority; changing its definitions or source support does. Historical authority records retain their original fingerprints and are not rewritten.
