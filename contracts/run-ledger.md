# Run ledger service

The `run-ledger` executable is a separate current Run-protocol service boundary. It uses the core Run parser and verifier directly. It does not translate the earlier Assurance Service observation wire or derive approved judgments from execution.

## Configuration

`DATABASE_URL` names its PostgreSQL database. `ASSURANCE_RUN_TOKENS_FILE` names an externally supplied project map of separately generated producer, reviewer and owner credentials, as specified below. No default credentials are accepted. `ASSURANCE_RUN_ADDRESS` defaults to `127.0.0.1:8081`. Hosted delivery supplies a Secret reference and TLS/private connectivity through the deployment environment.

Every project endpoint requires exactly one `Authorization: Bearer <credential>` field valid for that project. Credentials are kept out of records and responses. HTTP 401 rejects missing or wrong credentials, including access to another project. Health is public. This service permission authorizes record submission; it does not approve the submitted method or Claim.

## Endpoints

| Endpoint | Meaning |
| --- | --- |
| `POST /v1/projects/{project}/runs` | Submit raw strict `azimuth-run-bundle` JSON. |
| `GET /v1/projects/{project}/runs/{run_id}` | Retrieve the immutable correction history. |
| `GET /v1/projects/{project}/subjects/{subject_fingerprint}/state` | Inspect the latest revision of every Run for that exact Subject. |

Submission validates syntax, all bundle fingerprints, provenance, selection and outcome reduction before persistence. A revision-zero Run starts a history. Subsequent submission must be the immediate next revision and pass core correction-set validation, including predecessor and anchor consistency. Project/Run history updates serialize in one database transaction. An identical replay returns HTTP 200; first ingestion returns HTTP 201. Conflicting or out-of-order history returns HTTP 409; invalid protocol content returns HTTP 422. Bodies are bounded to 16 MiB.

Run bundles are append-only. No automatic expiry, deletion, cross-Subject reuse or newest-run-wins verdict is defined. Correction history remains retrievable; subject inspection uses the latest valid revision of each distinct Run. Different model and Check fingerprints remain visible instead of being silently combined.

## Authority of the state

State joins execution facts with the explicitly selected current authority and independent review records, as specified below. Protocol validation alone does not approve a method, applicability or Claim.

The earlier service executable and database records remain isolated. They are not a compatibility bridge into current Run state.


## Independent review and current authority

The ledger uses the compact independent-review contract in `contracts/assurance-reviews.md`. Executions, Method Qualifications, Applicability Decisions and Claim Judgments remain distinct records. A satisfied Observation alone cannot establish a supported Claim.

Credentials are loaded from the private `ASSURANCE_RUN_TOKENS_FILE` as a project map. Each project supplies nonempty `producers`, `reviewers` and `owners` maps from identity to token. Duplicate JSON object keys in the credential file fail startup. Tokens contain at least 32 non-whitespace bytes and are distinct across all projects and roles. Producers submit Runs, reviewers submit review records, and owners select the current authority. Any role can read its own project's records. The authenticated reviewer identity must equal the review record's `reviewer` field. Identity and role assignment are operator-controlled; credentials do not prove human independence or review quality.

`POST /v1/projects/{project}/authority` validates and retains an immutable full-model authority projection. The route's project must match the projection. Initial selection requires `If-None-Match: *`; replacement requires `If-Match` containing the current authority fingerprint. Owner selection history is retained. Supply exactly one conditional header; duplicate or conflicting conditions fail with HTTP 409. Repeating the current selection with its matching `If-Match`, or retrying the same initial selection with `If-None-Match: *`, returns HTTP 200 without adding a selection event. Missing or stale preconditions fail with HTTP 409. Selection is an explicit owner action: ingestion of a new Run or review does not select a model or approve intent. Candidate authority must be kept in a separate project instance until accepted; registering a projection is not Azimuth change acceptance.

`GET /v1/projects/{project}/authority` returns the selected authority. `POST /v1/projects/{project}/reviews` validates record structure, fingerprints and authenticated reviewer identity, and assesses its dependencies against the selected authority, stored reviews and correction heads. Invalid dependencies fail; stale or rejected records remain auditable. Identical content is an idempotent replay. `GET /v1/projects/{project}/reviews` returns immutable records.

`GET /v1/projects/{project}/subjects/{subject-fingerprint}/state` joins the selected authority, review records and latest valid Run corrections for the exact Subject. It exposes missing, rejected, stale, invalid or accepted review assessments and Claim conclusions without generating reviews. Complete current credible evidence plus an independent supported Claim Judgment is required for supported state. Missing structural support and omitted evidence remain gaps; ambiguous review records remain unresolved. Routine Claims report when review is not required without inventing a positive Judgment.

Run history and review history are not deleted when authority changes. No evidence is silently reused across Subjects. Decision Policies and challenge scheduling for this compact review lane remain deferred.
