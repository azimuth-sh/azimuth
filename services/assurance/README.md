# Azimuth assurance service

This is the open reference implementation of D40's lifecycle boundary. It keeps three kinds of
state distinct:

- repository-authored evidence definitions and semantic qualifications;
- immutable accepted-model snapshots and their claim contracts;
- immutable CI or production observations and challenge results;
- derived lifecycle-gate decisions and focused work.

The service is optional. Nothing in `azimuth check`, routine changes, repository finalization or
the accepted model requires it to be running.

## Run the complete application

From this directory:

```bash
docker compose up --build
./seed-demo.sh
```

The API listens on `http://127.0.0.1:8080`, PostgreSQL is exposed on port 5434, and the diagnostic
interface is at `http://127.0.0.1:3000`. The seed is replay-safe.

## Run during development

Start PostgreSQL by any convenient method and set `DATABASE_URL`, then run:

```bash
cargo run --manifest-path Cargo.toml -p azimuth-assurance-server
cd web
npm install
npm run dev
```

The backend defaults to `postgres://azimuth:azimuth@localhost:5432/azimuth`. The web process reads
`ASSURANCE_API_URL`, defaulting to `http://127.0.0.1:8080`.

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
boundary. Docker access is therefore required. The original lifecycle experiment stays
at `../../experiments/assurance-service` and consumes the same domain crate.

## Deliberate reference-service limits

This application has no authentication, tenant isolation, signed provenance, retention policy,
report-object storage, backup policy, rate limiting, service telemetry or availability objective.
Deploying it beyond an isolated evaluation environment requires those controls. Its purpose is to
freeze and demonstrate the provider-neutral semantic protocol before production hardening.
