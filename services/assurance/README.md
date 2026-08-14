# Azimuth assurance service

This is the open reference implementation of D40's lifecycle boundary. It keeps repository
evidence, accepted-model snapshots, execution observations and derived gate decisions distinct.
The service is optional: neither `azimuth check` nor repository finalization requires a running
ledger.

## Private single-team deployment

The supplied Compose profile is for private dogfood on one host. Generate a URL-safe deployment
credential and keep it in the process environment:

```bash
export ASSURANCE_POSTGRES_PASSWORD="$(openssl rand -hex 32)"
docker compose up --detach --build
./seed-demo.sh
```

The API listens on `http://127.0.0.1:8080` and the diagnostic interface listens on
`http://127.0.0.1:3000`. PostgreSQL has no host-published port. `ASSURANCE_API_PORT` and
`ASSURANCE_WEB_PORT` may select different loopback ports. The database password has no default;
Compose refuses to resolve until the deployment supplies it.

Loopback is the repository-owned containment boundary. Remote access must cross an
operator-controlled trusted reverse proxy, SSH tunnel or VPN. This profile does not add
application authentication, authorization or tenant isolation, and it is not ready for direct
internet exposure. TLS termination, rate limiting, abuse controls and proxy configuration remain
operator responsibilities.

The named `assurance-data` volume retains PostgreSQL data when the application containers stop or
are recreated. API startup applies embedded SQLx migrations before serving. Downgrades are
unsupported. This first schema has no prior version to upgrade from; every later schema-changing
release must demonstrate that an old image's accepted history remains readable after the new image
applies its forward migration.

A named volume is not a recovery system. Backups, restore tests, host-volume loss, corruption,
high availability and disaster recovery are outside this private profile.

Stop the applications without deleting their data:

```bash
docker compose stop
```

Delete the stack only when its ledger history is no longer required. `docker compose down
--volumes` deletes the named database volume and is therefore destructive.

## Run during development

Start PostgreSQL by any convenient method, set `DATABASE_URL`, then run:

```bash
cargo run --manifest-path Cargo.toml -p azimuth-assurance-server
cd web
npm install
npm run dev
```

The backend requires `DATABASE_URL`. The web process reads `ASSURANCE_API_URL`, defaulting to
`http://127.0.0.1:8080`.

## Protocol surface

Every resource is scoped below `/v1/projects/{projectId}`:

| Method and path | Purpose |
|---|---|
| `POST /v1/projects` | Register one assurance account. |
| `GET /v1/projects` | Discover accounts for the diagnostic interface. |
| `POST/GET .../model-snapshots` | Register and read accepted-model claim contracts. |
| `POST/GET .../definitions` | Version and read stable evidence definitions. |
| `POST/GET .../qualifications` | Append and read semantic qualifications. |
| `POST/GET .../observations` | Append and read execution observations. |
| `POST/GET .../challenges` | Append and read challenge streams. |
| `POST .../gates/evaluate` | Derive and preserve a decision for an exact target. |
| `GET .../gate-decisions` | Read immutable decision history, newest first. |
| `GET .../gates` | Read the latest decision for every evaluated target. |
| `GET .../work-items` | Read focused work from currently closed targets. |
| `GET .../snapshot` | Export the portable project account and decision history. |

Client-supplied record ids are immutable. An identical replay returns 200 and does not add a row;
different content under the same id returns 409. A model snapshot and every claim-contract
fingerprint are recomputed at ingestion. Definitions use structured claim references and are
accepted only when that exact contract exists in at least one registered snapshot. Definition
identity is logical: a changed semantic fingerprint appends a version and makes a qualification
over the prior version stale.

Generate a repository-authored snapshot only after the accepted model is hole-free:

```bash
azimuth assurance export --project <id> --out assurance-snapshot.json \
  --manifest <each-current-manifest>
curl --header 'content-type: application/json' --data @assurance-snapshot.json \
  http://127.0.0.1:8080/v1/projects/<id>/model-snapshots
```

The CLI remains responsible for parsing specs and workspaces, running enumerators and checking
realization completeness. The service stores that result; it does not derive routes or area
membership. A contract may remain qualified across model snapshots when its semantics are
unchanged, but the new exact snapshot and revision still require their own observation.

Times are unsigned Unix seconds. Gate evaluation uses the request's `at` value for observation and
challenge applicability, so tests and lifecycle controllers do not sleep. `evaluatedAt` records
when the service preserved the decision.

## Verification

```bash
cargo test --manifest-path Cargo.toml --all-targets
cd web
npm run typecheck
npm run build
```

The Rust component suite starts real PostgreSQL with Testcontainers and drives the public HTTP
boundary. Docker access is therefore required. The private-deployment qualifier additionally
checks resolved Compose containment, ledger survival across the documented lifecycle and every
catalog-selected image/platform build. The original lifecycle experiment stays at
`../../experiments/assurance-service` and consumes the same domain crate.

## Deliberate reference-service limits

This application has no authentication, tenant isolation, signed provenance, retention policy,
report-object storage, backup policy, rate limiting, service telemetry or availability objective.
Its private profile exists to dogfood the provider-neutral semantic protocol behind an explicit
operator-owned network boundary, not to imply production hardening.
