# Assurance State Service

The current API stores validated Runs, append-only correction and independent review histories, explicit model authority and exact-Subject Assurance State. The inspection-only Web UI uses this API. Execution never creates an approved review or a supported Claim conclusion.

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
| `POST .../reviews` | Reviewer ingestion; independent records are never fabricated by execution. |
| `GET .../reviews` | Immutable review records with current assessment and diagnostics. |
| `POST .../authority` | Owner selection with `If-None-Match: *` initially or exact `If-Match`. |
| `GET .../authority` | Currently selected complete authority. |

List reads accept `limit=1..100` (default 25) and an opaque `after` cursor returned as `next`. Run revision cursors are integer revisions. Payload pages include a SQL-side 32 MiB guard; lookahead records return only cursor metadata; oversized pages fail explicitly before application materialization and require a smaller limit. Each list is an independent keyset traversal; concurrent arrivals before a cursor become visible on refresh. UI Run detail fetches one revision per page. Exact Subject state is never inferred for an unrecorded Subject. Missing authority returns 404; the UI reports an unresolved conclusion. Missing, stale, rejected and invalid reviews retain their actual core assessment and diagnostics.

State assessment reads one consistent database snapshot of current authority, all reviews and latest Run revisions. More than 5000 records or 32 MiB of review/Run payloads fails explicitly; it never substitutes truncated evidence for a complete assessment. This bound is an operational limit, not an Assurance validity rule. Larger accounts require a later bounded incremental projection. Ingestion remains limited to 16 MiB per request.

Browser-accessible `/inspection/{project}/...` routes are GET-only and allow only the listed inspection resources. They recheck the human's project access before server-side API calls; they do not proxy arbitrary paths or bearer writes.

## Local private profile

`docker-compose.yml` retains loopback-only API and Web host bindings and an unpublished PostgreSQL port. It now starts the current authenticated service, not the isolated earlier API. Supply `ASSURANCE_POSTGRES_PASSWORD`, absolute `ASSURANCE_RUN_TOKENS_FILE` and `ASSURANCE_WEB_CONFIG_FILE` paths, `NEXTAUTH_SECRET` and a canonical HTTPS `NEXTAUTH_URL`. Compose refuses missing inputs. Use an operator-controlled HTTPS reverse proxy or tunnel to reach the loopback Web port with that origin and matching OAuth callback; plain HTTP browser authentication is deliberately unsupported.

The API binds host `127.0.0.1:8081` and Web binds `127.0.0.1:3000` by default; `ASSURANCE_API_PORT` and `ASSURANCE_WEB_PORT` override host ports. PostgreSQL data survives container recreation in `assurance-data`. API startup applies embedded migrations. Backups, restore verification, high availability and disaster recovery remain deployment responsibilities; deleting the named volume destroys history.

## Build verification and remaining gates

Product compilation and source inspection are permitted for this change. Build the API explicitly with `cargo build --locked --manifest-path services/assurance/Cargo.toml -p azimuth-assurance-server --bin run-ledger`; build the Web with `npm ci --ignore-scripts`, `npm run typecheck` and `npm run build` in `services/assurance/web`.

OAuth callbacks, authorization denial paths, PostgreSQL-backed reads and writes, image startup, TLS renewal and private network reachability still need separately authorized behavioral/live verification. No runtime result is implied by compilation. The existing `deployment/qualify.py` exercises the earlier unauthenticated profile and is not qualification evidence for this implementation.

## Preserved isolated service boundary

Earlier `azimuth-assurance-server` source, domain records and `seed-demo.sh` remain isolated historical development facilities. Its project snapshots, evidence definitions and lifecycle gates are not the current Run/review protocol, and the current CLI does not populate it. The published API image and current Web UI no longer use that boundary. Run the earlier binary explicitly only when investigating its isolated wire; do not seed the current service with `seed-demo.sh` or mistake its records for current Assurance State.
