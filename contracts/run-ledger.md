# Run ledger service

The `run-ledger` executable is a separate current Run-protocol service boundary. It uses the core Run parser and verifier directly. It does not translate the earlier Assurance Service `/observations` wire or derive approved judgments from execution.

## Configuration

`DATABASE_URL` names its PostgreSQL database. `ASSURANCE_RUN_TOKENS_FILE` names an externally supplied project map of separately generated producer, reviewer and owner credentials, as specified below. No default credentials are accepted. `ASSURANCE_RUN_ADDRESS` defaults to `127.0.0.1:8081`. Hosted delivery supplies a Secret reference and TLS/private connectivity through the deployment environment.

Every project endpoint requires exactly one `Authorization: Bearer <credential>` field valid for that project. Credentials are kept out of records and responses. HTTP 401 rejects missing or wrong credentials, including access to another project. Health is public. This service permission authorizes record submission; it does not approve the submitted method or Requirement.

## Endpoints

| Endpoint | Meaning |
| --- | --- |
| `POST /v1/projects/{project}/runs` | Submit raw strict `azimuth-run-bundle` JSON. |
| `GET /v1/projects/{project}/runs/{run_id}` | Retrieve the immutable correction history. |
| `GET /v1/projects/{project}/verification-targets/{verification_target}/state` | Inspect the latest revision of every Run for that exact Verification Target fingerprint. |

Submission validates syntax, all bundle fingerprints, provenance, selection and outcome reduction before persistence. A revision-zero Run starts a history. Subsequent submission must be the immediate next revision and pass core correction-set validation, including predecessor and anchor consistency. Project/Run history updates serialize in one database transaction. An identical replay returns HTTP 200; first ingestion returns HTTP 201. Conflicting or out-of-order history returns HTTP 409; invalid protocol content returns HTTP 422. Bodies are bounded to 16 MiB.

Run bundles are append-only. No automatic expiry, deletion, cross-target reuse or newest-run-wins verdict is defined. Correction history remains retrievable; Verification Target inspection uses the latest valid revision of each distinct Run. Different model and Check fingerprints remain visible instead of being silently combined.

## Authority of the state

State joins execution facts with the default model's authority and independent review records, as specified below. Protocol validation alone does not approve a method, applicability or Requirement.

The earlier service executable and database records remain isolated. They are not a compatibility bridge into current Run state.


## Independent review and current authority

The ledger uses the compact independent-review contract in `contracts/assurance-reviews.md`. Executions, Method Qualifications, Applicability Decisions and Requirement Judgments remain distinct records. A satisfied Check Result alone cannot establish a supported Requirement.

Credentials are loaded from the private `ASSURANCE_RUN_TOKENS_FILE` as a project map. Each project supplies nonempty `producers`, `reviewers` and `owners` maps from identity to token. Duplicate JSON object keys in the credential file fail startup. Tokens contain at least 32 non-whitespace bytes and are distinct across all projects and roles. Producers submit Runs, reviewers submit review records, publishers publish main models, releases and deployments, and owners register candidate model revisions. Any role can read its own project's records. The authenticated reviewer identity must equal the review record's `reviewer` field. Identity and role assignment are operator-controlled; credentials do not prove human independence or review quality.

There is no accepted-authority pointer, owner selection or release acceptance. The default model of a project is the latest main-model publication (see "Main-model publication"); `GET /v1/projects/{project}/authority` returns that publication's authority with its `main_sequence` and `ratification_status`, or HTTP 404 when none exists. `POST /v1/projects/{project}/reviews` validates record structure, fingerprints and authenticated reviewer identity, and assesses its dependencies against the default model, stored reviews and correction heads. Invalid dependencies fail; stale or rejected records remain auditable. Identical content is an idempotent replay. `GET /v1/projects/{project}/reviews` returns immutable records.

`GET /v1/projects/{project}/verification-targets/{verification_target}/state` joins the default model's authority, review records and latest valid Run corrections for the exact Verification Target. It exposes missing, rejected, stale, invalid or accepted review assessments and Requirement conclusions without generating reviews. Complete current credible evidence plus an independent supported Requirement Judgment is required for supported state. Missing structural support and omitted evidence remain gaps; ambiguous review records remain unresolved. Routine Requirements report when review is not required without inventing a positive Judgment.

Run history and review history are not deleted when authority changes. No evidence is silently reused across Verification Targets. Decision Policies and challenge scheduling for this compact review lane remain deferred.

## Candidate model revisions

Owner `POST /v1/projects/{project}/model-revisions` accepts a strict full authority projection and registers its fingerprint as an immutable revision. It returns HTTP 201 initially and HTTP 200 for an identical replay, without selecting it as the default model. No project or credential set is created. Owners, reviewers and producers are configured during onboarding.

- `GET .../model-revisions` lists paginated registration metadata.
- `GET .../model-revisions/{revision}` returns the projection, registration metadata and whether it is the default model.
- `GET .../model-revisions/{revision}/snapshot` returns `{project, model_revision, authority, reviews, bundles}` in one bounded database snapshot. Reviews are explicitly linked to this revision; bundles are current project Run heads whose semantic applicability remains the core assessor's responsibility.
- Reviewer `POST .../model-revisions/{revision}/reviews` validates the unchanged strict review record and credential-backed reviewer identity against that revision.
- `GET .../model-revisions/{revision}/verification-targets/{verification_target}/state` assesses the exact recorded Verification Target without changing the default model.

Snapshots fail above 5000 review/Run records or 32 MiB including authority bytes. They never infer support from partial evidence. Absent revision or Verification Target returns HTTP 404. A revision becomes the default model only by main-model publication; candidate registration never selects anything.

Review payloads remain immutable; explicit revision links preserve submission provenance. There is no automatic cross-revision reuse. Current review reads follow the default model; historical candidate reviews remain accessible through their revision snapshots.


## Explicit Check Run model association

An owner may POST `{"run_fingerprint":"sha256:...","reason":"..."}` to `/v1/projects/{project}/model-revisions/{revision}/run-models`. The representative Run and revision must already exist in the same project. The service derives the native Run model fingerprint from the stored Run, requires a nonempty Check-only plan whose Check IDs and fingerprints exist in that authority, and retains the owner, representative, rationale and time. Identical model associations are idempotent; the original attribution is immutable.

This explicitly joins existing and subsequent Runs with that exact native model fingerprint to revision snapshots. It does not infer a relationship from Check names, change the accepted revision, establish source applicability, merge Verification Targets, or manufacture a Requirement Status conclusion. Native dependency/currentness assessment and exact-target independent Requirement Status still apply. Producers and collectors cannot create this association. Challenge histories remain explicitly associated by publication.

## Revision catalogs

A model revision owns an optional immutable navigation catalog independently of any
release. `GET /v1/projects/{project}/model-revisions/{revision}/catalog` returns
`{project, model_revision, fingerprint, catalog, submitted_by, provenance_basis,
submitted_at}` or HTTP 404 when no catalog has been registered. Absence is not a
reason to invent a hierarchy from IDs or choose an arbitrary release's catalog.

An owner can POST the raw catalog exported by `azimuth assurance catalog` to the
same route. Export it from the same complete model inputs used for the authority.
Validation reuses the native release catalog contract: module parents must exist,
cycles and duplicate membership are rejected, and Requirements, Scenarios and Checks must
exactly cover the revision's authority identities. Registration returns 201; an
identical replay returns 200 without rewriting original provenance. Different
content for the same revision returns 409. Catalog identity is a canonical content
fingerprint; attaching a catalog does not change the authority fingerprint, select
that authority, approve a review or establish Requirement Status.

Release publication registers its optional catalog under the release's exact model
revision in the same transaction. A conflict rejects the whole publication. An
identical publication replay can register its catalog if it has not yet been
attached; release history is not rewritten. Existing stored releases are not
arbitrarily selected or backfilled during migration. Their catalog can be attached
explicitly by the owner or through verified publication replay.

## Current deployment selection

A recorded deployment and the deployment currently designated for an environment
are separate facts. No timestamp, deployment ID order or release publication implicitly makes a deployment current.

- `GET .../deployments/{deployment_id}` returns one immutable deployment record and
  its release fingerprint, including when it lies outside the first list page.
- `GET .../environments` lists current selection metadata, with ordinary keyset
  pagination by environment.
- `GET .../environments/{environment}/deployment` returns the latest explicit
  selection, its deployment, release fingerprint, selection revision, selecting
  publisher, time and rationale. No selection returns 404.
- Publisher POST to the same environment route accepts exactly
  `{deployment_id, expected_revision, reason}`. Use revision 0 for the first
  selection, otherwise the revision read from GET. The deployment must already
  exist in the same project and name this exact environment. A bounded, nonblank
  reason is required. Environment and deployment IDs follow the native ID syntax.

Selection serializes per project/environment. A stale expected revision returns
409. An exact immediate retry by the same publisher, with the same deployment,
reason and predecessor revision, returns the existing selection without another
history event. Every other successful selection appends a monotonically numbered
history row. Rollback means explicitly selecting another recorded deployment;
records and earlier selections are retained. Publisher authorization expresses
operational designation, not model approval. Viewer and producer
credentials cannot select deployments; deployment selection itself remains an automation operation.

The environment API does not independently observe the cluster. A deployment
controller or authorized operator must publish the actual deployment record and
explicitly designate it after rollout. Selection is not evidence of readiness,
reachability or correctness; those remain Check and Requirement Status obligations.

## Structured Requirement Status explanations

Each Requirement returned by exact-target state inspection includes `explanation`.
The existing `status`, `review`, `requirement_status`, diagnostics and structural-gap semantics
remain unchanged. The explanation is derived inside the native assessor from the
same complete authority, review set and active Run correction heads:

- `summary` and `status`: current accepted Requirement Status rationale, or an explanation of
  why no current assessment establishes a conclusion.
- `reasons`: typed `code`, `category` and `message`, with optional Check/Scenario IDs,
  diagnostic details or structural-gap details. Codes include
  `requirement-review-{assessment}`, `requirement-status-{assessment}`, `structural-gaps`,
  `current-check-result-missing` and `requirement-status-assessed-{conclusion}`. Consumers do not
  classify states by matching diagnostic English.
- `review` and `requirement_status`: their assessment status, exact selected record fingerprint,
  asserted reviewer, review time, authored rationale and diagnostics. No record or
  ambiguous latest history exposes null record fields, not fabricated authorship.
- `coverage`: one entry per declared current Check–Scenario binding, with exact matching
  `check_results` and status `missing`, `unassessed`, `satisfied`, `violated` or
  `inconclusive`. Only current Check fingerprints, the exact selected Verification Target and
  active correction heads participate.
- `evidence`: `supporting`, `violations` and `inconclusive` contain Check Results
  explicitly referenced by the basis of the current accepted Requirement Status assessment.
  Other matching Check Results remain `unassessed`. Every Check Result retains Run,
  bundle and Check Result fingerprints, Check, Scenario, outcome, `observed_at_ms`,
  `selected_in_basis` and `accepted_basis`.
- `residual_risks`: authored residual risks from the selected Requirement Status record, if any.

A violated Check Result can remain unassessed when no current accepted Requirement Status record
covers it; this does not convert the Requirement's conclusion automatically. A stale
assessment's formerly selected Check Results are not presented as accepted support.
Missing coverage is inspectable information, not a newly authored assessment.
There is no automatic expiry, alert interpretation, cross-system evidence transfer
or Requirement Status transition event stream. Such policies remain separate framework work.

Deployments of an environment are listed with ordinary keyset pagination by deployment id and, separately, as history (see "Deployment history"). Version 8 storage replaces earlier migration history; see [deployment-requirement-status.md](deployment-requirement-status.md).

## Reconciled deployment observations

See `contracts/deployment-observation.md` for exact CAS, replay, freshness, runtime component and deployment Verification Target rules. The latest observation may invalidate confidence in an earlier current selection without deleting history. There is no acceptance: no automatic or owner action accepts a release, applies source or model changes, archives a Change or mutates a workload, and rollback is performed by the deployment system and only observed.

## Main-model publication

`POST /v1/projects/{project}/main-models`, role Publisher, body bounded to 16 MiB, publishes the model of one commit on main:

```json
{"format": "azimuth-main-model-publication", "version": 1,
 "commit": "<40-hex>", "main_sequence": 1234,
 "authority": {},
 "reviews": [],
 "catalog": {},
 "ratification": {"status": "ratified", "reason": "...",
   "pull_request": {"number": 54, "approvals": [{"login": "...", "submitted_at_ms": 0}]},
   "verified_at_ms": 0}}
```

`authority` is the native full-model authority projection, `reviews` the sealed native Requirement Review records committed in git, `catalog` an optional navigation catalog as in release publication, and `ratification.status` is `ratified | unratified`. `main_sequence` is the first-parent commit count of `commit` on main, computed and attested by CI.

The service validates the authority, registers its model revision as for candidate revisions, stores each review with provenance basis `publisher-transported-assertion` and re-assesses every review against the posted authority, recording the result. It stores a publication keyed by `(project, commit)`, unique also by `(project, main_sequence)`. First publication returns HTTP 201 and an identical replay HTTP 200. HTTP 400 reports an invalid payload, authority or review failure; HTTP 403 a wrong role; HTTP 409 the same commit with different content or a different commit with an already used `main_sequence`. Every publication is stored. The latest model is the one with the highest `main_sequence`, never arrival order, and the default model never regresses to a lower sequence. `ratification` is a publisher assertion: the service stores it as submitted and cannot call the hosting API.

## Delivery gap, history and rollback candidates

All routes below are read-only and accept any role.

- `GET .../environments/{environment}/delivery-gap` returns `{environment, deployment_id, release_fingerprint, release_model_revision, main_model_revision, main_sequence, requirements: [{id, difference, criticality, release_status}]}`. `difference` is `added | changed | removed`, computed between the model of the environment's deployed release and the latest main model by Requirement text and criticality fingerprints; `release_status` is the deployed release's Status for the Requirement, or null when it is added. A deployment is never assessed against main's model; the gap report is the only place main's model meets an environment.
- `GET .../environments/{environment}/deployment-history?limit=&page_token=` returns `{deployments: [{deployment_id, release_fingerprint, label, first_observed_ms, last_observed_ms, rollout_id, current}], next_page_token}`, newest first with token-based pagination. `current` marks the latest complete observed deployment. It replaces the capped, id-ordered deployment list for history display.
- `GET .../environments/{environment}/rollback-candidates?limit=` returns earlier deployments of the environment whose last stored per-deployment status summary had every non-routine Requirement Satisfied, each with the summary and `last_status_at_ms`. The summary is the one stored by the latest assessment of that deployment; no history table exists, so a candidate shows a status that may be old and is never presented as current. Not-applicable Requirements do not satisfy the criterion. Azimuth performs no rollback.
