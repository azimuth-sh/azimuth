# Azimuth assurance service

This directory preserves the isolated service boundary introduced in alpha 1. Alpha.5 advances its project-snapshot payload to version 2 so each Claim contract carries one opaque `caseStatement` instead of parser-shaped Gherkin steps. Its evidence definitions, qualifications, observations and gate decisions remain runnable for service development and private dogfood while the Run-ledger replacement is pending.

This protocol is not a projection of the alpha 5 core model. Its observation resources are legacy service records, not current Run Observations or current Azimuth execution authority. The current CLI does not populate this wire, and neither `azimuth validate` nor repository finalization requires a running service.

## Private single-team deployment

The supplied Compose profile is for private dogfood on one host. Generate a URL-safe deployment credential and keep it in the process environment:

```bash
export ASSURANCE_POSTGRES_PASSWORD="$(openssl rand -hex 32)"
docker compose up --detach --build
./seed-demo.sh
```

The API listens on `http://127.0.0.1:8080` and the diagnostic interface listens on `http://127.0.0.1:3000`. PostgreSQL has no host-published port. `ASSURANCE_API_PORT` and `ASSURANCE_WEB_PORT` may select different loopback ports. The database password has no default; Compose refuses to resolve until the deployment supplies it.

Loopback is the repository-owned containment boundary. Remote access must cross an operator-controlled trusted reverse proxy, SSH tunnel or VPN. This profile does not add application authentication, authorization or tenant isolation, and it is not ready for direct internet exposure. TLS termination, rate limiting, abuse controls and proxy configuration remain operator responsibilities.

The named `assurance-data` volume retains PostgreSQL data when the application containers stop or are recreated. API startup applies embedded SQLx migrations before serving. Downgrades are unsupported. This first schema has no prior version to upgrade from; every later schema-changing release must demonstrate that an old image's accepted history remains readable after the new image applies its forward migration.

A named volume is not a recovery system. Backups, restore tests, host-volume loss, corruption, high availability and disaster recovery are outside this private profile.

Stop the applications without deleting their data:

```bash
docker compose stop
```

Delete the stack only when its ledger history is no longer required. `docker compose down --volumes` deletes the named database volume and is therefore destructive.

## Run during development

Start PostgreSQL by any convenient method, set `DATABASE_URL`, then run:

```bash
cargo run --manifest-path Cargo.toml -p azimuth-assurance-server
cd web
npm install
npm run dev
```

The backend requires `DATABASE_URL`. The web process reads `ASSURANCE_API_URL`, defaulting to `http://127.0.0.1:8080`.

## Protocol surface

The endpoints below describe only the isolated legacy service API with version 2 project snapshots. Their resource names do not declare current alpha 5 Checks, Evidence Bindings, Method Qualifications, Applicability Decisions, Runs or Observations.

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

Client-supplied record ids are immutable. An identical replay returns 200 and does not add a row; different content under the same id returns 409. A model snapshot and every claim-contract fingerprint are recomputed at ingestion. Definitions use structured claim references and are accepted only when that exact contract exists in at least one registered snapshot. Definition identity is logical: a changed semantic fingerprint appends a version and makes a qualification over the prior version stale.

There is no current CLI command that exports an alpha 5 model into this protocol. For a local service-only walkthrough, `./seed-demo.sh` invokes the domain crate's synthetic snapshot example and posts the resulting version 2 payload directly. That seed is a legacy protocol fixture; it does not establish a mapping from current repository declarations or execution facts.

Within the isolated wire, the service stores supplied snapshots and does not derive routes or area membership. A contract may remain qualified across snapshots when its version 2 semantics are unchanged, but a later snapshot and revision still require their own legacy observation record. Version 1 snapshot payloads are not accepted or migrated; alpha releases carry no compatibility promise for this isolated protocol.

Times are unsigned Unix seconds. Gate evaluation uses the request's `at` value for observation and challenge applicability, so tests and lifecycle controllers do not sleep. `evaluatedAt` records when the service preserved the decision.

## Verification

```bash
cargo test --manifest-path Cargo.toml --all-targets
cd web
npm run typecheck
npm run build
```

The Rust component suite starts real PostgreSQL with Testcontainers and drives the public HTTP boundary. Docker access is therefore required. The private-deployment qualifier additionally checks resolved Compose containment, ledger survival across the documented lifecycle and every catalog-selected image/platform build. The original lifecycle experiment stays at `../../experiments/assurance-service` and consumes the same domain crate.

## Deliberate reference-service limits

This application has no authentication, tenant isolation, signed provenance, retention policy, report-object storage, backup policy, rate limiting, service telemetry or availability objective. Its private profile exists to exercise the isolated legacy service behind an explicit operator-owned network boundary, not to imply production hardening or implement the future provider-neutral Run protocol.
