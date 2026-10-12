# Assurance State Service

The current API stores validated Runs, append-only correction histories, definition-level Claim Reviews, exact-Subject Claim Health assessments and explicit model authority. Earlier Method Qualification, Applicability Decision and Claim Judgment records remain immutable readable history; they do not substitute for the new assessments. The inspection-only Web UI uses this API. Execution never creates an approved review or a supported Claim conclusion.

## Private hosted deployment

The canonical API image starts `run-ledger` on port 8081; the Web image serves port 3000. Terraform in the consuming infrastructure repository provisions PostgreSQL, workloads, Tailscale connectivity and HTTP-01 certificates. Image references must name published immutable digests; building source does not publish or deploy images.

`https://assurance.drim.dev` is reachable through Tailscale. GitHub OAuth independently authenticates people, and an explicit allowlist of stable numeric GitHub user IDs assigns project access. No GitHub organization membership, username or email grants implicit access. The UI is inspection only: neither a browser session nor its dedicated viewer credential can ingest Runs, submit reviews or select authority.

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
    "producers": {"ci": "REPLACE_WITH_A_DISTINCT_GENERATED_TOKEN"},
    "reviewers": {"reviewer": "REPLACE_WITH_ANOTHER_GENERATED_TOKEN"},
    "owners": {"owner": "REPLACE_WITH_ANOTHER_GENERATED_TOKEN"},
    "viewers": {"web": "REPLACE_WITH_ANOTHER_GENERATED_TOKEN"}
  }
}
```

These strings illustrate the schema, not deployable credentials. Generate independent random tokens of at least 32 non-whitespace bytes. Duplicate tokens across roles or projects are rejected. A project requires producer, reviewer and owner identities; viewers are optional for API-only deployments. Each write requires the exact corresponding role. Reviewer identity must equal the record's reviewer. Any valid role can inspect only its own project; viewer never authorizes a write.

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
| `GET .../runs` | Latest revision summaries, including exact Subject and executed model fingerprints. |
| `GET .../runs/{run_id}` | Immutable revision history, oldest first, with a revision cursor. |
| `GET .../subjects` | Exact recorded Subject fingerprints and descriptors. |
| `GET .../subjects/{fingerprint}/state` | Current authority's derived state for that exact recorded Subject. |
| `POST .../reviews` | Reviewer ingestion of Claim Review or Claim Health; execution never fabricates an assessment. |
| `GET .../claim-reviews` | Current definition assessments, independent of execution Subjects. |
| `GET .../reviews` | Immutable review records with current assessment and diagnostics. |
| `POST .../authority` | Owner selection with `If-None-Match: *` initially or exact `If-Match`. |
| `GET .../authority` | Currently selected complete authority. |
| `POST .../model-revisions` | Owner registration of an immutable candidate projection, without authority selection. |
| `GET .../model-revisions` | Bounded registration metadata; no projection payloads in lists. |
| `GET .../model-revisions/{revision}` | Exact projection, original registration metadata and current selection indicator. |
| `GET .../model-revisions/{revision}/claim-reviews` | Candidate definition assessments without authority selection or an execution Subject. |
| `GET .../model-revisions/{revision}/snapshot` | Consistent complete bounded authority, revision-linked reviews and latest project Run bundles. |
| `POST .../model-revisions/{revision}/reviews` | Authenticated reviewer ingestion and explicit linkage to this revision. |
| `GET .../model-revisions/{revision}/subjects/{fingerprint}/state` | This revision's exact recorded Subject assessment without changing accepted authority. |

List reads accept `limit=1..100` (default 25) and an opaque `after` cursor returned as `next`. Run revision cursors are integer revisions. Payload pages include a SQL-side 32 MiB guard; lookahead records return only cursor metadata; oversized pages fail explicitly before application materialization and require a smaller limit. Each list is an independent keyset traversal; concurrent arrivals before a cursor become visible on refresh. UI Run detail fetches one revision per page. Exact Subject state is never inferred for an unrecorded Subject. Missing authority returns 404; the UI reports an unresolved conclusion. Missing, stale, rejected and invalid reviews retain their actual core assessment and diagnostics.

State assessment reads one consistent database snapshot of current authority, all reviews and latest Run revisions. More than 5000 records or 32 MiB of review/Run payloads fails explicitly; it never substitutes truncated evidence for a complete assessment. This bound is an operational limit, not an Assurance validity rule. Larger accounts require a later bounded incremental projection. Ingestion remains limited to 16 MiB per request.

Browser-accessible `/inspection/{project}/...` routes are GET-only and allow only the listed inspection resources. They recheck the human's project access before server-side API calls; they do not proxy arbitrary paths or bearer writes.

## Candidate model revisions

The project is configured once through credentials; a change does not provision another project. Owner registration accepts the same strict authority projection as authority selection. Its content fingerprint is the revision ID. `POST /v1/projects/{project}/model-revisions` returns HTTP 201 on first registration and HTTP 200 on identical replay. Project mismatch or invalid projection returns HTTP 422. Registration metadata is append-only and retains `registration_status: candidate`, the original registering owner and timestamp; current selection is reported separately. Registration never writes `current_authorities` or the selection history.

An owner may later select that exact projection through the existing conditional `/authority` endpoint. This explicit operation retains history and requires the existing precondition; it does not automatically certify deployment, adoption or change acceptance. Selection of a projection not yet registered also creates its immutable registration. The same revision-linked reviews then remain available under selected authority. No automatic lifecycle promotion, rollout gate or archive operation is implemented.

Revision snapshot responses contain `project`, `model_revision`, `authority`, `reviews` and `bundles`. All are read in one bounded SQL snapshot. Reviews are only those explicitly linked to the revision; bundles are the latest correction heads of shared project Runs, including unjoined evidence for other definitions and Subjects. Their presence is not assignment or applicability approval. Exact dependency matching remains the core assessor's responsibility. A missing revision or Subject returns HTTP 404. Bounds include 5000 review/Run records, 32 MiB review/Run payloads and the authority projection; exceeding them fails rather than returning partial support.

Revision-scoped review submission reuses strict record validation, authenticated reviewer matching and current dependency assessment. Invalid dependencies fail with HTTP 422; protocol-valid rejected and stale assessments are retained. An identical record may be explicitly submitted for another revision when its exact dependencies permit assessment there. The record itself stays immutable; the link records authenticated submission provenance. There is no automatic cross-revision review reuse. Current `/reviews` writes and reads use the selected revision. Candidate records cannot silently affect selected authority. The additive migration links pre-existing review history to the authority selected at migration, preserving records and selection history without asserting their earlier validity or assessment context. Such links are explicitly marked `migration-selected-authority` rather than `reviewer-submission`; migration does not manufacture reviewer approval.

Snapshot and review assessment are observations at read time. A correction or another review arriving afterward may change the next assessment. No lease, freshness guarantee or review-quality guarantee is supplied by an HTTP receipt. The Web UI does not yet browse candidate revisions; authenticated API and harness clients can use these endpoints.

## Claim Review and Claim Health

Claim Review assesses the Claim definition, Cases, implementation and verification account using exact dependencies. It has no execution Subject or observations. Its accepted or rejected outcome is authored independently of Run execution. `GET .../claim-reviews` exposes the latest assessment for every Claim and shows missing and stale records explicitly.

Claim Health names one exact Subject, its current accepted Claim Review and an explicit Run/observation basis. The core validates definition dependencies, correction heads and Subject matching independently; a stale or rejected definition review cannot establish supported health. Source, artifact and deployment are distinct execution Subjects. Build and deployment provenance is inspectable information, never automatic permission to reuse observations across Subjects. Missing Cases, unresolved evidence and Challenge findings remain visible. A clean Challenge is a negative search fact, not proof of health. Current mutation integration records exact batch source input and native mutation statuses/locations; it does not materialize per-mutant Subjects or mutant-specific product Observations.

The existing append-only `assurance_reviews` JSONB table retains the native records and their fingerprints without converting historical payloads. Optional provenance and exact `challenge_basis` references are stored unchanged. Submission assesses against existing latest Run correction heads and revision-linked reviews; publish relied-on Runs before dependent assessments and retained predecessors before reassessments. Missing or changed challenge proofs remain stale dependencies, not silent success. Candidate revision links remain explicit. The UI separates definition review from exact-Subject health and labels historical review kinds. Review history exposes the original reviewer in the payload and the actual authenticated submitter and receive time separately; agent/model provenance remains part of the authored dependency artifacts rather than being inferred from the transport.

Local drafts and debugging Runs need not be uploaded. CI publication is the default accountable boundary, publishing the selected review and evidence with necessary dependencies and relevant adverse facts. A CI producer token authorizes Runs only. Transporting an unchanged locally authored review requires an authorized reviewer credential matching that record's reviewer; CI cannot relabel the reviewer or acquire review permission from producer status. Credentials must be configured explicitly. There is no new signed delegation or publisher role.

### Migration and rollback

No database migration is necessary for these record kinds: existing immutable JSONB payloads and revision links accommodate them. New review writes accept only `claim-review` and `claim-health`. Historical kinds remain parseable for inspection; they are never silently promoted into accepted Claim Reviews or supported Claim Health. Existing projects therefore show unresolved new assessments until independently authored records are published.

Back up the PostgreSQL volume before rollout. Downgrading the application after publishing new records is unsafe because an earlier parser may reject the complete snapshot. Roll back code only before new-kind ingestion, or restore a separately retained pre-rollout database snapshot into an isolated recovery deployment. Do not delete or rewrite published history to make an older binary appear compatible. This change does not publish images or deploy the service.

Build and source inspection establish compilation and schema integration, not PostgreSQL-backed behavior, OAuth authorization or live Challenge execution. Existing historical parsers remain readable; there is no automatic cross-Subject composition, recency expiration, telemetry ingestion or review-quality guarantee. Full snapshots retain the existing bounded-record and payload limits.

## Local private profile

`docker-compose.yml` retains loopback-only API and Web host bindings and an unpublished PostgreSQL port. It now starts the current authenticated service, not the isolated earlier API. Supply `ASSURANCE_POSTGRES_PASSWORD`, absolute `ASSURANCE_RUN_TOKENS_FILE` and `ASSURANCE_WEB_CONFIG_FILE` paths, `NEXTAUTH_SECRET` and a canonical HTTPS `NEXTAUTH_URL`. Compose refuses missing inputs. Use an operator-controlled HTTPS reverse proxy or tunnel to reach the loopback Web port with that origin and matching OAuth callback; plain HTTP browser authentication is deliberately unsupported.

The API binds host `127.0.0.1:8081` and Web binds `127.0.0.1:3000` by default; `ASSURANCE_API_PORT` and `ASSURANCE_WEB_PORT` override host ports. PostgreSQL data survives container recreation in `assurance-data`. API startup applies embedded migrations. Backups, restore verification, high availability and disaster recovery remain deployment responsibilities; deleting the named volume destroys history.

## Build verification and remaining gates

Product compilation and source inspection are permitted for this change. Build the API explicitly with `cargo build --locked --manifest-path services/assurance/Cargo.toml -p azimuth-assurance-server --bin run-ledger`; build the Web with `npm ci --ignore-scripts`, `npm run typecheck` and `npm run build` in `services/assurance/web`.

OAuth callbacks, authorization denial paths, PostgreSQL-backed reads and writes, image startup, TLS renewal and private network reachability still need separately authorized behavioral/live verification. No runtime result is implied by compilation. The existing `deployment/qualify.py` exercises the earlier unauthenticated profile and is not qualification evidence for this implementation.

## Preserved isolated service boundary

Earlier `azimuth-assurance-server` source, domain records and `seed-demo.sh` remain isolated historical development facilities. Its project snapshots, evidence definitions and lifecycle gates are not the current Run/review protocol, and the current CLI does not populate it. The published API image and current Web UI no longer use that boundary. Run the earlier binary explicitly only when investigating its isolated wire; do not seed the current service with `seed-demo.sh` or mistake its records for current Assurance State.
