# Azimuth Assurance Service

The current API stores validated Runs, append-only correction histories, definition-level Requirement Reviews, Requirement Status records for one exact Verification Target and main-model publications. Earlier Method Qualification, Applicability Decision and Requirement Judgment records remain immutable readable history; they do not substitute for the new assessments. The inspection-only Web UI uses this API. Execution never creates an approved review or a supported Requirement conclusion.

## UI development

For dashboard design with synthetic local data, run `npm run dev:scenarios` in
`services/assurance/web` and open `http://127.0.0.1:3100/projects/payments-demo`.
See [local UI scenarios](web/development/README.md) for profiles and boundaries.
This development-only read model does not seed PostgreSQL or validate native
assurance records. Normal API-backed operation still requires the configured
OAuth profile below.

## Private hosted deployment

The canonical API image starts `run-ledger` on port 8081; the Web image serves port 3000. Terraform in the consuming infrastructure repository provisions PostgreSQL, workloads, Tailscale connectivity and HTTP-01 certificates. Image references must name published immutable digests; building source does not publish or deploy images.

`https://assurance.drim.dev` is reachable through Tailscale. GitHub OAuth independently authenticates people, and an explicit allowlist of stable numeric GitHub user IDs assigns project access. No GitHub organization membership, username or email grants implicit access. The UI is inspection only: neither a browser session nor its dedicated viewer credential can ingest Runs, submit reviews, publish models, releases or deployments, or register model revisions.

The private HTTPS proxy routes `/api/auth/*` to Web unchanged and `/api/v1/*` and exact `/api/health` to API with `/api` stripped. Other `/api` paths return 404. Public ingress exposes only cert-manager HTTP-01 solver paths, never Web or API. DNS must distinguish public ACME ingress from the Tailscale endpoint. Forwarded origin headers come from the trusted proxy; the Web's canonical origin is fixed by configuration.

## API configuration

| Setting | Meaning |
|---|---|
| `DATABASE_URL` | PostgreSQL connection string, supplied through a Secret. |
| `ASSURANCE_RUN_ADDRESS` | Listen address; set `0.0.0.0:8081` in containers. |
| `ASSURANCE_RUN_TOKENS_FILE` | Readable mounted JSON credential file. |

The credential file is a project map:

```json
{
  "drim-dev": {
    "publishers": {"main-ci": "REPLACE_WITH_A_DISTINCT_GENERATED_TOKEN"},
    "producers": {"ci": "REPLACE_WITH_A_DISTINCT_GENERATED_TOKEN"},
    "reviewers": {"reviewer": "REPLACE_WITH_ANOTHER_GENERATED_TOKEN"},
    "owners": {"owner": "REPLACE_WITH_ANOTHER_GENERATED_TOKEN"},
    "viewers": {"web": "REPLACE_WITH_ANOTHER_GENERATED_TOKEN"}
  }
}
```

These strings illustrate the schema, not deployable credentials. Generate independent random tokens of at least 32 non-whitespace bytes. Duplicate tokens across roles or projects are rejected. A project requires producer, reviewer and owner identities; publishers (main models, releases and deployments) and viewers are optional for API-only deployments, but without a publisher nothing can be published. Each write requires the exact corresponding role. Reviewer identity must equal the record's reviewer. Any valid role can inspect only its own project; viewer never authorizes a write.

Credentials load at API startup. Restart the API after changing accepted tokens. This implementation does not provide automatic credential rotation. `/health` reports process liveness; startup connects and migrates PostgreSQL before listening.

## Web configuration

| Setting | Meaning |
|---|---|
| `ASSURANCE_API_URL` | Internal API origin, such as `http://assurance-api:8081`; no `/api` prefix. |
| `NEXTAUTH_URL` | Canonical HTTPS origin `https://assurance.drim.dev`. |
| `NEXTAUTH_SECRET` | Independent random session encryption secret, at least 32 non-whitespace bytes. |
| `ASSURANCE_WEB_CONFIG_FILE` | Readable mounted JSON file shown below. |

```json
{
  "github": {"clientId": "OAUTH_CLIENT_ID", "clientSecret": "OAUTH_CLIENT_SECRET"},
  "users": {"12345": {"projects": ["drim-dev"]}},
  "projects": {"drim-dev": {"viewerToken": "MATCH_THE_API_WEB_VIEWER_TOKEN"}}
}
```

Enroll real numeric GitHub user IDs, not the illustrative ID. Register an OAuth app with homepage `https://assurance.drim.dev` and callback `https://assurance.drim.dev/api/auth/callback/github`. GitHub authorization uses PKCE and state with `read:user` scope. Stable NextAuth 4 handles the OAuth exchange, encrypted HTTP-only Secure session cookies and sign-in/sign-out CSRF protection. GitHub access tokens are not retained in browser sessions.

The server checks project authorization before retrieving data. Viewer tokens remain in server-only modules and are never serialized to browser sessions, pages or inspection responses. API fetches reject redirects, have a 15-second deadline and bypass caches. Sessions expire after one hour; every protected request reloads the current user/project allowlist, so removing access takes effect without waiting for session expiry. `/health` returns 503 if required Web configuration cannot be loaded. Startup rejects missing or invalid configuration; builds require no deployment secrets.

Store configuration files outside the checkout and Docker build contexts. Mount Secrets read-only with permissions readable by the non-root container users. Updating OAuth credentials, session secret or API tokens requires coordinated rollout; changing the projected Web allowlist is re-read on requests, but deployment tooling should still restart workloads when changing Secret configuration.

## Inspection API

All data routes below require a project bearer credential; `/health` is process liveness only.

| Method and path | Meaning |
|---|---|
| `POST /v1/projects/{project}/runs` | Producer ingestion of validated initial Runs or immediate corrections. |
| `GET .../runs` | Latest revision summaries, including exact Verification Target and executed model fingerprints. |
| `GET .../runs/{run_id}` | Immutable revision history, oldest first, with a revision cursor. |
| `GET .../verification-targets` | Exact recorded Verification Target fingerprints and descriptors. |
| `GET .../verification-targets/{fingerprint}/state` | Default model's derived state for that exact recorded Verification Target. |
| `POST .../reviews` | Reviewer ingestion of a Requirement Review or a Requirement Status record; execution never fabricates an assessment. |
| `GET .../requirement-reviews` | Current definition assessments, independent of Verification Targets. |
| `GET .../reviews` | Immutable review records with current assessment and diagnostics. |
| `POST .../main-models` | Publisher registration of the model on main with its commit, `main_sequence` and ratification status. |
| `GET .../authority` | Complete authority of the highest `main_sequence` main-model publication, with its commit, sequence and `ratification_status`. |
| `POST .../model-revisions` | Owner registration of an immutable candidate projection. |
| `GET .../model-revisions` | Bounded registration metadata; no projection payloads in lists. |
| `GET .../model-revisions/{revision}` | Exact projection and original registration metadata. |
| `GET .../model-revisions/{revision}/requirement-reviews` | Candidate definition assessments without a Verification Target. |
| `GET .../model-revisions/{revision}/snapshot` | Consistent complete bounded authority, revision-linked reviews and latest project Run bundles. |
| `POST .../model-revisions/{revision}/reviews` | Authenticated reviewer ingestion and explicit linkage to this revision. |
| `GET .../model-revisions/{revision}/verification-targets/{fingerprint}/state` | This revision's exact recorded Verification Target assessment without changing the default model. |

List reads accept `limit=1..100` (default 25) and an opaque `after` cursor returned as `next`. Run revision cursors are integer revisions. Payload pages include a SQL-side 32 MiB guard; lookahead records return only cursor metadata; oversized pages fail explicitly before application materialization and require a smaller limit. Each list is an independent keyset traversal; concurrent arrivals before a cursor become visible on refresh. UI Run detail fetches one revision per page. Exact Verification Target state is never inferred for an unrecorded Verification Target. A project with no main-model publication returns 404 for `/authority`; the UI reports an unresolved conclusion. Missing, stale, rejected and invalid reviews retain their actual core assessment and diagnostics.

State assessment reads one consistent database snapshot of the default model's authority, all reviews and latest Run revisions. More than 5000 records or 32 MiB of review/Run payloads fails explicitly; it never substitutes truncated evidence for a complete assessment. This bound is an operational limit, not an Assurance validity rule. Larger accounts require a later bounded incremental projection. Ingestion remains limited to 16 MiB per request.

Browser-accessible `/inspection/{project}/...` routes are GET-only and allow only the listed inspection resources. They recheck the human's project access before server-side API calls; they do not proxy arbitrary paths or bearer writes.

## Candidate model revisions

The project is configured once through credentials; a change does not provision another project. Owner registration accepts the same strict authority projection as a main-model publication. Its content fingerprint is the revision ID. `POST /v1/projects/{project}/model-revisions` returns HTTP 201 on first registration and HTTP 200 on identical replay. Project mismatch or invalid projection returns HTTP 422. Registration metadata is append-only and retains `registration_status: candidate`, the original registering owner and timestamp. A candidate never becomes the default model by registration; the default model is the highest `main_sequence` main-model publication, and ratification is recorded in git. No automatic lifecycle promotion, rollout gate or archive operation is implemented.

Revision snapshot responses contain `project`, `model_revision`, `authority`, `reviews` and `bundles`. All are read in one bounded SQL snapshot. Reviews are only those explicitly linked to the revision; bundles are the latest correction heads of shared project Runs, including unjoined evidence for other definitions and Verification Targets. Their presence is not assignment or applicability approval. Exact dependency matching remains the core assessor's responsibility. A missing revision or Verification Target returns HTTP 404. Bounds include 5000 review/Run records, 32 MiB review/Run payloads and the authority projection; exceeding them fails rather than returning partial support.

Revision-scoped review submission reuses strict record validation, authenticated reviewer matching and current dependency assessment. Invalid dependencies fail with HTTP 422; protocol-valid rejected and stale assessments are retained. An identical record may be explicitly submitted for another revision when its exact dependencies permit assessment there. The record itself stays immutable; the link records authenticated submission provenance. There is no automatic cross-revision review reuse. Current `/reviews` writes and reads use the default model's revision. Candidate records cannot silently affect the default model's assessments.

Snapshot and review assessment are observations at read time. A correction or another review arriving afterward may change the next assessment. No lease, freshness guarantee or review-quality guarantee is supplied by an HTTP receipt. The Web model workspace browses main and candidate revisions with explicit model/system context. Authenticated API and harness clients use the same revision endpoints.

## Requirement Review and Requirement Status

Requirement Review assesses the Requirement definition, Scenarios, implementation and verification account using exact dependencies. It has no Verification Target or Check Results. Its accepted or rejected outcome is authored independently of Run execution. `GET .../requirement-reviews` exposes the latest assessment for every Requirement and shows missing and stale records explicitly.

A Requirement Status record names one exact Verification Target, its current accepted Requirement Review and an explicit Run/Check Result basis. The core validates definition dependencies, correction heads and Verification Target matching independently; a stale or rejected definition review cannot establish a supported Requirement Status. Source, artifact and deployment are distinct Verification Targets. Build and deployment provenance is inspectable information, never automatic permission to reuse Check Results across Verification Targets. Missing Scenarios, unresolved evidence and Challenge findings remain visible. A clean Challenge is a negative search fact, not proof that a Requirement is satisfied. Current mutation integration records exact batch source input and native mutation statuses/locations; it does not materialize per-mutant Verification Targets or mutant-specific product Check Results.

Explicit relational tables retain native Review records and their fingerprints. Named typed columns and ordered child tables preserve optional fields, explicit nulls, provenance and dependency references; JSON is reconstructed only for API and native-core boundaries, not persisted. Optional provenance and exact `challenge_basis` references are stored unchanged. Submission assesses against existing latest Run correction heads and revision-linked reviews; publish relied-on Runs before dependent assessments and retained predecessors before reassessments. Missing or changed challenge proofs remain stale dependencies, not silent success. Candidate revision links remain explicit. The UI separates definition review from exact-Verification-Target Requirement Status and labels historical review kinds. Review history exposes the original reviewer in the payload and the actual authenticated submitter and receive time separately; agent/model provenance remains part of the authored dependency artifacts rather than being inferred from the transport.

Local drafts and debugging Runs need not be uploaded. CI publication is the default accountable boundary, publishing the selected review and evidence with necessary dependencies and relevant adverse facts. A CI producer token authorizes Runs only. Transporting an unchanged locally authored review requires an authorized reviewer credential matching that record's reviewer; CI cannot relabel the reviewer or acquire review permission from producer status. Credentials must be configured explicitly. There is no new signed delegation or publisher role.

### Migration and rollback

The relational storage generation starts in a fresh database using `server/relational_migrations` and explicit schema initialization at storage schema version 8. Only fresh initialization is supported: startup rejects any recorded storage schema whose version or signature differs from the running binary. There is no in-place upgrade from earlier storage schema versions. Retain the original database and take an offline dump; reset the database, republish main's model, reviews and releases from git and CI, and let the reconciler and collector rebuild deployments and captures. This is a reset and republication, not a historical-data migration. Records omitted from the republication remain only in the retained dump. New review writes accept only `requirement-review` and `requirement-status`. Historical kinds remain parseable for inspection; they are never silently promoted into accepted Requirement Reviews or supported Requirement Status. Existing projects therefore show unresolved new assessments until independently authored records are published.

Back up the PostgreSQL volume before rollout. Downgrading the application after publishing new records is unsafe because an earlier parser may reject the complete snapshot. Roll back code only before new-kind ingestion, or restore a separately retained pre-rollout database snapshot into an isolated recovery deployment. Do not delete or rewrite published history to make an older binary appear compatible. Image publication and deployment are separate operator actions; neither authorizes a release.

Build and source inspection establish compilation and schema integration, not PostgreSQL-backed behavior, OAuth authorization or live Challenge execution. Existing historical parsers remain readable; there is no automatic cross-Verification-Target reuse, recency expiration, telemetry ingestion or review-quality guarantee. Full snapshots retain the existing bounded-record and payload limits.

## Local private profile

`docker-compose.yml` retains loopback-only API and Web host bindings and an unpublished PostgreSQL port. It starts the current authenticated service. Supply `ASSURANCE_POSTGRES_PASSWORD`, absolute `ASSURANCE_RUN_TOKENS_FILE` and `ASSURANCE_WEB_CONFIG_FILE` paths, `NEXTAUTH_SECRET` and a canonical HTTPS `NEXTAUTH_URL`. Compose refuses missing inputs. Use an operator-controlled HTTPS reverse proxy or tunnel to reach the loopback Web port with that origin and matching OAuth callback; plain HTTP browser authentication is deliberately unsupported.

The API binds host `127.0.0.1:8081` and Web binds `127.0.0.1:3000` by default; `ASSURANCE_API_PORT` and `ASSURANCE_WEB_PORT` override host ports. PostgreSQL data survives container recreation in `assurance-data`. API startup applies embedded migrations. Backups, restore verification, high availability and disaster recovery remain deployment responsibilities; deleting the named volume destroys history.

## Build verification and remaining gates

Product compilation and source inspection are permitted for this change. Build the API explicitly with `cargo build --locked --manifest-path services/assurance/Cargo.toml -p azimuth-assurance-server --bin run-ledger`; build the Web with `npm ci --ignore-scripts`, `npm run typecheck` and `npm run build` in `services/assurance/web`.

OAuth callbacks, authorization denial paths, PostgreSQL-backed reads and writes, image startup, TLS renewal and private network reachability still need separately authorized behavioral/live verification. No runtime result is implied by compilation. `deployment/qualify.py --lifecycle` rehearses the current authenticated API through Compose with ephemeral credentials from `deployment/runtime.py`; it does not exercise GitHub OAuth, TLS or private network reachability.

The `domain` crate is not part of the API image or the run ledger. It is retained only for `experiments/assurance-service`.

## Production workspace APIs

Model navigation now reads `/model-revisions/{revision}/catalog`, independent of
release lookup. Owner POST registers a complete native catalog against that exact
revision. Publisher release ingestion also attaches its catalog atomically;
conflicting immutable catalogs reject the publication. Existing history is not
rewritten. For a revision registered without a release, export authority and
catalog with the same complete model inputs, register the authority, then POST the
catalog with an owner credential.

Production overview follows the explicit current selection at
`/environments/production/deployment`. Publisher POST accepts:

```json
{
  "deployment_id": "production-release-42",
  "expected_revision": 0,
  "reason": "Rollout completed; designate this recorded deployment as current."
}
```

Use the selection revision returned by GET for replacements. First selection uses
0. Publish the deployment record first; it must belong to the same environment.
A stale predecessor returns 409, preventing a late rollout from overwriting a
more recent selection. The deployment controller must obtain the expected revision
before its rollout and publish this designation only after actual rollout. A
rollback explicitly selects its recorded target. Designation neither authorizes a
release nor proves deployment readiness or Requirement Status. Tokens remain in the
calling publisher/controller; the browser has no write capability.

Exact-system Requirement Status responses provide structured explanations, coverage and
accepted versus unassessed evidence references. The native assessor produces these
fields; the UI does not decide applicability, inspect English diagnostics to
classify failures or infer freshness from timestamps. See the
[Run ledger contract](../../contracts/run-ledger.md) for schemas and authority
boundaries.

Build verification for this update uses Rust compilation and Web type checking.
No tests were added or run. PostgreSQL migration execution, concurrent selection
writes and authenticated endpoint behavior have not been exercised against a live
database in this work; compilation does not establish those runtime properties.
No production registration, selection or deployment is performed by these source
changes.

## Deployment observations

The Web config parser still accepts server-only `ownerTokens: {project: token}` per user, but no Web action uses it: the UI is inspection only and there is no release acceptance.

Deploy API and Web together. The production dashboard labels last observed release and explicitly warns when no fresh complete observation matches its selected deployment. Enable the separate reconciler only after image, credential, configured workload and network prerequisites are ready. No existing deployment is selected by migration.

## Migration and rollback boundary

Downgrades are unsupported. Back up the persistent PostgreSQL data before deployment and retain the exact image/configuration identities. A rollback must use application/schema-compatible images; reverting data or schema requires a separately reviewed recovery procedure or a later schema-changing release. Container stop/start and recreation must preserve the existing database volume and its immutable Run/model/review history.
