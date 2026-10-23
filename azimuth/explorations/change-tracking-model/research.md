# Research: Change tracking model

Sourced detail for `exploration.md`. Facts were read on 2026-10-23 from `main` after the web changes merged (`c32ee7a`). Nothing here is a decision.

## Sources read

| Area | Where |
|---|---|
| Review basis and currentness | `contracts/assurance-reviews.md` |
| Markers and extractor-derived site fingerprint | `contracts/markers.md`, `contracts/manifest.md` |
| Check, Challenge, policy fingerprints | `tools/azimuth/src/fingerprint.rs` |
| Requirement digest and judgment preimage | `tools/azimuth/src/model.rs` (`requirement_digest`, `requirement_judgment_preimage`) |
| Run Verification Target and bundle fingerprints | `contracts/run-bundle.md`, `tools/azimuth/src/run.rs` |
| Unit inputs and reuse | `contracts/unit-inputs.md` |
| Release, observation, publication | `contracts/release-publication.md`, `contracts/deployment-observation.md`, `contracts/deployment-requirement-status.md` |
| Extractors | `tools/extractors/{python,rego,typescript,dotnet,helm}` |

Not inspected in detail: what feeds the design, verification and support digests in a Requirement's review fingerprint, and the TypeScript Check-implementation hashing beyond the declaration-text helper.

## Extractor hashing today

| Extractor | Marker | Hashed |
|---|---|---|
| Python | Realizes | source text of the enclosing function or class (`ast.get_source_segment`) |
| Python | ImplementsCheck | bytes of the whole module |
| Rego | ImplementsCheck | text of the whole file |
| TypeScript | declaration | `getText` of the declaration node, comments inside included |
| .NET | method site | raw source lines from the first to the last sequence point of the method |
| .NET | Check support | path and bytes of every source file declaring the type, nested types included |
| Helm | Realizes | Helm version, `[release, namespace, chart]`, raw source template bytes, default `values.yaml` bytes, every `--values` file, and `JSON.stringify` of the rendered resource |

The Helm site identity is `helm:<chart>:<release>:<apiVersion>:<kind>:<namespace>:<name>`, so the identity is already per rendered resource; only the hashed content is broader than that.

## Experiment: Drim deployment repository, before and after one release

Method: export the Drim deployment repository at `f8b148a` and `ca1ed5f`, build the repository's Helm emitter and run it the way the release pipeline does (no `--values`), render the chart, and run the Rego emitter. Local Helm was v3.14.3 while the pipeline pins v3.17.3; both runs used the same Helm, so the comparison is valid.

| Site | Rendered document | Fingerprint |
|---|---|---|
| NetworkPolicy `web-api-ingress` | identical | unchanged |
| Service `web-api` (template `web-api.yaml`) | identical (same document hash before and after) | changed |
| Rego Check `hosted-resources-declare-private-api-isolation` | n/a | changed (64 lines added) |
| Rego Check `credential-delivery-contains-only-secret-references` | n/a | unchanged |

The Service fingerprint changed only because the raw template file changed: a one-line condition feeding a feature-flag environment variable on the Deployment in the same file. Only two sites carry a `Realizes` marker in that chart; two new templates added in the same release carry none.

Consumer observation (Drim): an earlier edit to a Python file that carries Check implementations changed the authority and made critical reviews stale, which led to routing a pipeline fix through a different file. This is consistent with whole-module hashing of Check implementations.

## Table 1: what is tracked and hashed today

| Use case | Thing tracked | What is hashed, and at what granularity |
|---|---|---|
| Review currentness | Requirement | `id`, terms, predicate, domain, `over`, conditions, and each Scenario's id and digest. A Scenario digest includes the parent Requirement's statement. |
| Review currentness | Design, verification, support accounts | Three digests included in the review fingerprint (inputs not inspected). |
| Review currentness | Realizes sites | Qualified source identity and source fingerprint, per the extractor table. |
| Review currentness | Mechanisms | Canonical digest of the Mechanism record, with site fingerprints. |
| Review currentness | Check | `id`, method text, terminal, and each implementation's identity and source fingerprint. |
| Review currentness | Check-Scenario bindings | Fingerprint over binding, Scenario digest, Method Qualification fingerprint, policy fingerprint. |
| Review currentness | Verification Challenge | `id`, form, `searches_for`, targets, requirements, mechanisms, checks, required scope. |
| Review currentness | Decision policy, schedule | Canonical hash of each. |
| Review currentness | Candidate authority | Canonical hash of the model projection, excluding local paths, line numbers and findings. |
| Evidence applicability | Verification Target | `source`: repository revision plus producer-asserted content fingerprint. `artifact`: artifact digests. `deployment`: deployment fingerprint plus artifact digests. |
| Evidence applicability | Run | Plan, per-Check, selection, Check Result and bundle fingerprints; `corrects` chain; adapter, configuration and launch fingerprints. |
| Evidence applicability | Test units | `sha256(rule_fingerprint, sorted (path, content digest) of the unit inputs)`. Inputs are every checkout file minus the private paths of other units. |
| Evidence applicability | Execution context | Pinned leaf values compared by equality with the deployment capture; context contract hashes `{name, fields}`. |
| Release and deployment | Release | Canonical hash of sources, builds, artifacts, components, deployments, `model_revision`, required reviews, unit-input manifest and, when present, `integration`. |
| Release and deployment | Deployment observation | Identity over release, observed configuration, applied revision and Argo state; desired component entries carry image and canonical template hash. |
| Release and deployment | Main model publication | `model_revision`, `publication_revision`, both source commits, recorded ratification. |
| Reviews and status | Review and status records | Canonical record fingerprint; pinned `requirement_fingerprint`, dependencies, Verification Target fingerprint, basis references, predecessor fingerprint. |

Observed properties: unit inputs are a static path partition, so a change to any non-private file changes every unit's fingerprint; hermeticity is the rule author's claim. Extractor fingerprints record no scheme or normalizer version.

## Table 2: candidate tracking (not decided)

| Use case | Thing | Candidate tracking and hash | Stales a review | Change from today |
|---|---|---|---|---|
| Review currentness | Requirement, Scenarios, design, verification | Keep; hash normalized content; show a reviewed-versus-current diff | Yes | scheme version, delta view |
| Review currentness | Check implementation | Normalized hash of the Check's own site plus declared test-side dependencies | Yes | function-level for Python; method plus declared helpers for .NET; Rego stays whole-file |
| Review currentness | Realizes sites | Tag stays as navigation index; fingerprint leaves the review basis | No | production change triggers re-execution only |
| Review currentness | Mechanisms | Track the declaration; site change is an advisory notice | Declaration only | site fingerprints stop gating |
| Review currentness | Helm resources | Canonical rendered resource per identity; marker as manifest annotation | No | drop raw template, default values and Helm version from the hash |
| Review currentness | Bindings, Challenge, policy | Keep | Yes | none |
| Review currentness | Candidate authority | Keep as publication identity; per-Requirement dependency sets decide staleness | No | none |
| Evidence applicability | Verification Target, Run, units, context | Keep; pin the Check fingerprint in Runs explicitly | n/a | optional coverage-derived unit inputs later |
| Release and deployment | Release, observation, publication | Keep | n/a | none |
| New | Scheme version | Every fingerprint carries a scheme and normalizer version | n/a | new |
| New | Reviewed-basis snapshot | Retain the reviewed text or a Git reference | n/a | new |
| New | Executed-sites record | Runs record production sites executed | No | new, optional |
| New | Map completeness lint | Flag relevant untagged files and rendered resources | n/a | new |

## Migration shape

An explicit carry-forward, as the owner proposed:

1. Freeze the current main-model publication. It pins both repositories' commits, so the content can be reproduced.
2. Validate every review against the publication's authority under the current scheme. Reviews already stale stay stale.
3. Compute the new scheme at the same commits and derive the new authority.
4. Append immutable carry-forward records (review, old and new fingerprint, snapshot, actor, time). Map Check fingerprints pinned by Runs in the same way, only for Runs whose Check fingerprint equals the current old-scheme value.
5. Publish the new authority at the same commits; the assessor accepts a review that is current directly or through a valid carry-forward chain.
6. Stop emitting the old scheme later.

Soundness rule: carrying a review is a relabelling of identical content, valid only when the new scheme covers the same content or less. Widened coverage needs review of the added part. A carry-forward is an authenticated publisher or CI attestation, not a Requirement Review.

Cost estimate (loose, not grounded in the assessor code): small contract text, medium service ingestion and storage, a delicate assessor change that deserves independent review, a medium offline tool whose hardest part is detecting widened coverage, small UI. Hands-on effort is likely hours per piece; wall-clock time is dominated by the change workflow, independent review, CI and rollout.

## Agent context for a failed Requirement (future feature)

Navigation edges available today: Requirement to Scenarios to Checks; Check to its implementation site; Realizes and Mechanism sites; design Mechanisms and Elements. Not available: what a Check actually executed, completeness of the tag map, and a diff between the last satisfied and the failing release. Releases already pin exact commits, so a path-restricted diff is derivable. Test results already carry the defining file of each executed test, which points an agent at test files.

A brief could contain: Requirement statement and failing Scenarios; failing Check, its implementation with line ranges and Run diagnostics; Realizes and Mechanism sites; design excerpts; the path-restricted diff since the last satisfied release; and a caveat that the map may be incomplete.
