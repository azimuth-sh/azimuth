# Change: Rename Claim-first vocabulary to Requirement, Scenario and Verification Target

Status: proposed

Obligations, stable identifiers, evidence semantics and assessment rules are unchanged. This change renames domain terms, authored syntax, serialized names, schemas, routes and labels; the framework's own accepted model is migrated mechanically with every Claim and Case identifier preserved. The single intent delta (`specs/framework/release-artifacts.md`) moves the values the first-alpha release contract pins to the new source version and migration line without changing the obligation.

Dima explicitly authorized implementation, a fresh Assurance State Service (ASS) database without data migration (all ASS data deleted), deletion of the unused legacy gate API, a new drim.dev release, redeployment of every component this requires, and all merges. Commits, pushes and the ASS reset are authorized; release acceptance and change archival are not.

## Problem

The domain vocabulary does not describe what the framework does for readers. "Claim", "Case", "Subject", "Check Observation", "Verification Context", "Challenger" and "Claim Status" are used across authored specifications, contracts, the CLI, the Run protocol, ASS storage, collectors and the UI. Several are ambiguous with unrelated meanings (an OIDC subject, a test case, a runtime observation).

The target words are also currently rejected by the product: the installation validator flags `## Requirement:` and `### Scenario:` as retired syntax, and the intent-delta reference and the `azimuth-propose` skill forbid them. The rename reverses that earlier decision and must remove the rejection deliberately.

## Outcome

One vocabulary is used consistently in documentation, authored syntax, contracts, code, serialized names, ASS schema and API, collectors, the UI and the drim.dev integration:

> Modules organize Requirements. Scenarios describe expected behavior. Checks produce Results. ASS assesses the evidence and shows Requirement Status for a deployment.

Mapping (full matrix in `design.md`): Claim → Requirement; specification Case → Scenario; Check Observation → Check Result; assurance Subject → Verification Target; Verification Context → Execution Context; Challenger → Verification Challenge; Claim assurance account → Requirement assurance record; Claim Review → Requirement Review; Claim Status and remaining Claim Health → Requirement Status. Decided with Dima: the remaining exact-target health record kind becomes a Requirement Status record qualified by its Verification Target, and the Challenge Schedule, Plan, Selection and Result names are unchanged.

Terms that stay: Module, Check, Run, Evidence, Monitor, Release, Deployment, Environment, Source Snapshot, Build Artifact, Context Contract, Context endpoint, Collector, Reconciler, Publisher, Assessor, `verification.md`, and Observation for runtime facts.

## Scope

In scope, in dependency order:

- the Azimuth core, parser, fingerprints, CLI, embedded skills and references, contracts, glossary and the framework's own model;
- language annotations, test harnesses and extractors;
- ASS domain, server, relational schema, API, web UI, development data and the lifecycle rehearsal;
- the Collector and Reconciler;
- release tooling and publication workflows;
- the drim.dev, drim.dev GitOps and drim.dev infrastructure repositories, including the app's Execution Context endpoint (`/internal/execution-context`, decided with Dima);
- a scoped reset of ASS-owned data, fresh evidence, a new drim.dev release and its deployment.

Excluded: any change to assessment rules, review policy, applicability, freshness, materialization or invalidation behavior; re-enabling excluded challenges; compatibility aliases, fallback parsers, dual-write paths or legacy endpoints; migration of existing ASS records; relabeling or re-fingerprinting of historical immutable evidence; release acceptance; change archival; drim.dev business data, the shared PostgreSQL cluster, credentials and unrelated infrastructure. Archived changes, history files and retained release records stay untouched and are never republished as current evidence.

## Affected claims

No Claim proposition or identifier changes. Stable Claim and Case identifiers are preserved even where they contain old words (for example `verification-context-is-observed-and-bounded`); renaming an identifier is a remove-and-add decision that this change does not make. Each such identifier is recorded as a justified exception in the final terminology audit. The framework's accepted model files change syntax and vocabulary and therefore their digests; prior reviews and baselines do not carry over.

## Completion conditions

- `design.md` terminology matrix, contract and identity strategy and reset boundary are approved before implementation.
- Every active component uses the matrix vocabulary; the final terminology audit lists only justified historical, identifier and unrelated occurrences.
- `azimuth validate` is clean; frozen vectors are regenerated from the renamed contracts and the Python protocol mirror is byte-identical to Rust.
- A fresh empty ASS database initializes; business data, credentials and unrelated infrastructure are unchanged.
- Fresh Runs and Check Results are produced by actual execution with the renamed tooling and published; a new drim.dev release is created and actually deployed, and the reconciled deployment, Execution Context captures, Requirement Reviews and computed Requirement Status are visible in production ASS using the new terms.
- Requirement Review and Requirement Status semantics and materialized-assessment behavior and performance are unchanged (API timings reported separately from browser timings).
- Release acceptance stays pending and the change is not archived by this work.
