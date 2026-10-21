# Deployment Requirement Status

Requirement Status is the conclusion established by applicable evidence about a Requirement for an observed deployment. The assessment target is that deployment. Test Runs retain the exact source, artifact or deployment Verification Target they executed against. Build provenance and deployment reconciliation establish applicability relations; they are not product Checks and do not manufacture evidence.

## Authored applicability

Each Check can declare one strict JSON metadata line in verification.md:

```markdown
- Deployment: {"verification_target":"source","components":["api"],"contexts":[],"contribution":"both","monitor":null}
```

`verification_target` is source, artifact or deployment. `components` identifies the relevant release components, not every repository in the project. `contexts` lists exact contract fingerprints; an explicit empty array declares no configuration dependency. `contribution` is support, counterexample or both. `monitor` is null or an object with unique `id`, positive `interval_seconds` and `freshness_seconds` at least the interval; Monitors require a deployment execution Verification Target. Optional `configuration_sources` adds explicitly named release repositories to the component's complete build-input sources for source Checks (for example an independently scanned deployment repository). Exact repository revisions and content must match; unrelated extra inputs are not accepted. Optional `units` pins the complete required execution unit inventory. Optional `controls` contains `{unit,contexts}` declarations for deliberate sensitivity controls and requires `units`; only those reviewed unit/contract pairs may differ from deployed values. At least one applicable non-control unit must cover a relied-on Scenario. The policy is part of the Check fingerprint and independent Requirement Review dependencies. A mapping that converts failures into violations must itself be included in the reviewed Check implementation or inputs.

Historical Checks without this declaration remain readable. Their evidence is not automatically transferable to a deployment. A different whole-model revision alone does not invalidate evidence whose current Check and Scenario bindings, source lineage and contexts remain exact.

## Effective Execution Contexts

An application's Execution Context is the set of behavior-relevant conditions under which an application instance operates; a Run's Execution Context is the set under which the Check executed. A Context Contract defines the relevant properties and comparison rules, and the application's Context endpoint (conventionally `/internal/execution-context`) reports its effective Execution Context. The application reports facts; ASS determines whether those facts establish evidence applicability.

An owner registers a contract `{name,fields,fingerprint}`. Fields have string, boolean or integer types; the fingerprint hashes canonical content without the fingerprint field. Names describe product semantics; the envelope version is general:

```json
{"format":"azimuth-execution-context","version":2,"contexts":{"caller-policy":{"contract_revision":"sha256:…","values":{"mode":"restricted"}}}}
```

Only explicitly declared nonsecret effective values are permitted. Credentials and credential hashes must not be declared or reported. Owners review the field allowlist; structural validation rejects undeclared fields and prohibited secret-like field names, but does not classify arbitrary scalar values as secret. Adapters read actual settings; contracts do not configure applications.

A snapshot is `{captured_at_ms,instance,report}`. Every capture has `activity`: null for deployment captures and whole-Run execution captures, or the exact native execution activity ID. Execution captures bracket that Run or activity and bind the exact bundle fingerprint, Verification Target, component and process lifetime. The assessor requires captures for the activities of the actual relied-on unit attempts; an unused baseline process cannot stand in for the execution. Run actual Execution Context pins `context/<contract fingerprint>` (whole Run) or `context/<contract fingerprint>/activity/<activity ID>` to the canonical fingerprint of `{contract_revision,values}`. Production captures must be fresh, match the observed deployment and every current instance of relevant components, and lie within the observation validity window. Startup-fixed contexts only are supported; a process restart invalidates captures from its predecessor. The Kubernetes adapter uses container runtime IDs and requires the application process to be the container entry point: an in-container supervisor that restarts the application without restarting its container is unsupported. The boot-identity header must remain stable across each capture; adapters for other process models need an independently observed process-lifetime binding.

## Ingestion and assessment

Owners POST `{contract}` to `/v1/projects/{project}/context-contracts`. Authenticated producers or publishers POST `{capture}` to `/context-captures`. Client captures never contain `trusted`; ASS derives trust from authenticated Run ownership or the current deployment observation. Replays preserve original immutable bytes and do not renew timestamps.

A durable background worker assembles a bounded consistent relational snapshot and invokes the same native assessor as `azimuth assurance deployment-status --input request.json`. `GET /v1/projects/{project}/deployments/{deployment}/status` and registered-deployment model Verification Target state routes read the resulting typed relational assessment projection; they do not invoke the assessor or resolve target observations on the request path. Missing relations, contexts, reviews or fresh Monitor Check Results produce explained unresolved results. Routine Requirements retain their review exemption. Accepted Requirement Review establishes verification credibility, not an observed deployment result.

Newer Monitor executions supersede older ones, including incomplete or inconclusive attempts. Equal-time conflicting Check Results are unresolved. Correction heads are resolved before temporal selection. Applicable declared counterexamples remain visible even when review is missing. A failed test defaults to inconclusive; only an explicitly reviewed mapping can classify it as a violation.

## Public test integrations

Azimuth.Testing.Xunit and @azimuth-sh/playwright capture actual test context and emit versioned per-attempt reports. The azimuth_testing adapter reconciles exact native TRX/Playwright identities against project-authored mappings and constructs native Run bundles through azimuth_run. Context-bearing partitions have explicit launch boundaries; missing tests, retries, skips and capture failures cannot be converted to invented successful coverage.

Publishing is a separate operation. It verifies native bundles and uploads exact bytes with scoped credentials. CI may build all tools from one exact Azimuth Git commit, restore a local NuGet package feed and install a local npm tarball; registry publication is not required. The release records the selected tooling and contract identities.

## Storage rollout

ASS storage version 6 uses the Requirement, Scenario, Verification Target and Check Result vocabulary and initializes only a fresh, empty database; any other schema signature is refused rather than upgraded. Rolling it out replaces ASS-owned data: stop producers, reset the ASS database, start the service, confirm fresh initialization and republish evidence from source. Rolling an image back alone is unsupported; recovery is re-initialization and republication, not an automatic destructive rollback.

## Runtime target evidence

The [target observation protocol](target-observation.md) separates measurement identity from deployment attribution. A collector POSTs measured image/process identities and stable Check definitions or locally allowlisted context reports. It does not query ASS for a Verification Target, deployment selection or contract. ASS retains raw facts relationally, correlates them with independently reconciled historical intervals and materializes validated native evidence only after an unambiguous match. Original observation fingerprints and both reconciliation records remain accountable. Unmatched records are visible and pending; they do not establish support and cannot be assigned by arrival time. Existing exact-target Run and capture protocols retain their separate meaning.


## Materialized assessment reads

`GET /v1/projects/{project}/deployments/{deployment}/status/summary` returns the assessment header and compact `requirements` rows (`requirement`, `status`) for overview/list use, omitting detailed evidence. The full status endpoint retains the native result fields and detailed Requirement evidence while adding service freshness metadata. Both endpoints expose `assessment` with `state` (`ready`, `pending`, or `expired`), `generation`, `published_generation`, `evaluated_at_ms` and `valid_until_ms`. Only explicit `ready` authorizes presentation of the result as current. Missing metadata, pending refresh and elapsed validity do not imply a current Satisfied status. Presentation labels the internal `supported | violated | unresolved` values Satisfied / Not satisfied / Unresolved and labels Check Result outcomes Passed / Failed / Inconclusive; a passing Check or accepted Review alone never implies a satisfied Requirement. No page read renews evidence validity.

Input facts and reconciliation durably enqueue invalidation in the committing transaction. Workers follow Check/Scenario/Requirement and context dependencies, coalesce work, and use the native selected-Requirement evaluator with the complete validation closure. Changing the observed deployment or review basis may affect all its Requirements. Native evidence applicability and reduction semantics remain authoritative; projection tables are rebuildable, not authored decisions. The worker publishes affected Scenario facts, Requirement results and the deployment assessment header atomically, only if its input generation is still current. Failures leave work durable and the projection pending.

Time passage can invalidate support without new evidence. The worker schedules freshness boundaries, and the read path independently refuses to call an expired projection current. Historical evidence remains immutable. Publication prepares release/deployment relationships but never creates evidence for an unobserved deployment or accepts a release.

Definition-based Requirement Review applicability is materialized independently per model revision. `/requirement-reviews` and `/model-revisions/{revision}/requirement-reviews` read typed projected decisions and reference original immutable review records. Their `assessment` metadata distinguishes `ready` from `pending`, with no time expiry for definition-only applicability. Changed review associations and relevant retained Verification Challenge Run correction/target relationships invalidate the dependent revision. Unrelated ordinary Monitor Check Results do not invalidate definition review applicability. A pending review projection is not a missing or rejected human review.

Derived deployment Requirement graphs have bounded retention: publishing a replacement projection removes superseded computed graphs in the same transaction. Original evidence, review records, raw observations and reconciliation history are never pruned by projection maintenance. Existing repeatable-read readers retain a coherent prior snapshot through database MVCC. Projection generations are not immutable evidence identities and cannot authorize release acceptance.
