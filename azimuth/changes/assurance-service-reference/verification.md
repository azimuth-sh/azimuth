# Verification: assurance-service-reference

## Domain regression

Retain the eight pre-registered pure evaluator cases while selecting current qualifications and
challenges by explicit time. This keeps semantic failures fast and deterministic.

## Persistent HTTP component evidence

Start a real PostgreSQL container, run migrations and serve the actual Axum router on an ephemeral
port. Drive only public HTTP endpoints and assert persisted reads after requests.

The component suite must establish:

- one qualification opens gates for two exact CI revisions;
- a subject mismatch, expired production observation and violated observation close their gates;
- a changed definition makes the prior qualification stale;
- a current challenge finding closes an otherwise open gate;
- identical replay is idempotent while conflicting identity returns HTTP 409;
- gate history, worklist and snapshot survive reconstruction from PostgreSQL.

Use explicit timestamps rather than sleeps. PostgreSQL, serialization, routing, migrations and
database constraints are part of this evidence; CI and monitoring producers are substituted.

## Web and packaging checks

Type-check and production-build the Next.js interface. Exercise service health and a seeded
project through the Compose runbook. Browser automation is not required for this first reference
slice because the interface is diagnostic and contains no assurance decision logic.
