# Deployment Requirement Status

The assessment target is an observed deployment. Test Runs retain the exact source, artifact or deployment Verification Target they executed against. Build provenance and deployment reconciliation establish applicability relations; they are not product Checks and do not manufacture evidence.

## Authored applicability

Each Check can declare one strict JSON metadata line in verification.md:

```markdown
- Deployment: {"verification_target":"source","components":["api"],"contexts":[],"contribution":"both","monitor":null}
```

`verification_target` is source, artifact or deployment. `components` identifies the relevant release components, not every repository in the project. `contexts` lists exact contract fingerprints; an explicit empty array declares no configuration dependency. `contribution` is support, counterexample or both. `monitor` is null or an object with unique `id`, positive `interval_seconds` and `freshness_seconds` at least the interval; Monitors require a deployment execution Verification Target. Optional `configuration_sources` adds explicitly named release repositories to the component's complete build-input sources for source Checks (for example an independently scanned deployment repository). Exact repository revisions and content must match; unrelated extra inputs are not accepted. Optional `units` pins the complete required execution unit inventory. Optional `controls` contains `{unit,contexts}` declarations for deliberate sensitivity controls and requires `units`; only those reviewed unit/contract pairs may differ from deployed values. At least one applicable non-control unit must cover a relied-on Scenario. The policy is part of the Check fingerprint and independent Requirement Review dependencies. A mapping that converts failures into violations must itself be included in the reviewed Check implementation or inputs.

Historical Checks without this declaration remain readable. Their evidence is not automatically transferable to a deployment. A different whole-model revision alone does not invalidate evidence whose current Check and Scenario bindings, source lineage and contexts remain exact.

## Effective contexts

An owner registers a contract `{name,fields,fingerprint}`. Fields have string, boolean or integer types; the fingerprint hashes canonical content without the fingerprint field. Names describe product semantics; the envelope version is general:

```json
{"format":"azimuth-execution-context","version":2,"contexts":{"caller-policy":{"contract_revision":"sha256:…","values":{"mode":"restricted"}}}}
```

Only explicitly declared nonsecret effective values are permitted. Credentials and credential hashes must not be declared or reported. Owners review the field allowlist; structural validation rejects undeclared fields and prohibited secret-like field names, but does not classify arbitrary scalar values as secret. Adapters read actual settings; contracts do not configure applications.

A snapshot is `{captured_at_ms,instance,report}`. Every capture has `activity`: null for deployment captures and whole-Run execution captures, or the exact native execution activity ID. Execution captures bracket that Run or activity and bind the exact bundle fingerprint, Verification Target, component and process lifetime. The assessor requires captures for the activities of the actual relied-on unit attempts; an unused baseline process cannot stand in for the execution. The Run's actual Execution Context pins one entry per leaf the Check depends on: `context/<component>/<leaf>` for the whole Run or `context/<component>/<leaf>/activity/<activity ID>`, valued with `sha256:` of the canonical JSON scalar observed. Component ids contain no `/` or `.` and leaf names no `/`, so the key parses unambiguously. Production captures must be fresh, match the observed deployment and every current instance of relevant components, and lie within the observation validity window. Startup-fixed contexts only are supported; a process restart invalidates captures from its predecessor. The Kubernetes adapter uses container runtime IDs and requires the application process to be the container entry point: an in-container supervisor that restarts the application without restarting its container is unsupported. The boot-identity header must remain stable across each capture; adapters for other process models need an independently observed process-lifetime binding.

## Ingestion and assessment

Owners POST `{contract}` to `/v1/projects/{project}/context-contracts`. Authenticated producers or publishers POST `{capture}` to `/context-captures`. Client captures never contain `trusted`; AAS derives trust from authenticated Run ownership or the current deployment observation. Replays preserve original immutable bytes and do not renew timestamps.

`GET /v1/projects/{project}/deployments/{deployment}/status` assembles a bounded consistent relational snapshot and invokes the same native assessor as `azimuth assurance deployment-status --input request.json`. Current model Verification Target state routes delegate registered deployments to that assessor. Missing relations, contexts, reviews or fresh Monitor Check Results produce explained unresolved results. Routine Requirements retain their review exemption. Accepted Requirement Review establishes verification credibility, not an observed deployment result.

Newer Monitor executions supersede older ones, including incomplete or inconclusive attempts. Equal-time conflicting Check Results are unresolved. Correction heads are resolved before temporal selection. Applicable declared counterexamples remain visible even when review is missing. A failed test defaults to inconclusive; only an explicitly reviewed mapping can classify it as a violation.

## Public test integrations

Azimuth.Testing.Xunit and @azimuth-sh/playwright capture actual test context and emit versioned per-attempt reports. The azimuth_testing adapter reconciles exact native TRX/Playwright identities against project-authored mappings and constructs native Run bundles through azimuth_run. Context-bearing partitions have explicit launch boundaries; missing tests, retries, skips and capture failures cannot be converted to invented successful coverage.

Publishing is a separate operation. It verifies native bundles and uploads exact bytes with scoped credentials. CI may build all tools from one exact Azimuth Git commit, restore a local NuGet package feed and install a local npm tarball; registry publication is not required. The release records the selected tooling and contract identities.

## Storage rollout

AAS storage version 8 uses the Requirement, Scenario, Verification Target and Check Result vocabulary and initializes an empty database; an existing assurance schema of another version or signature is refused rather than upgraded. Rolling it out replaces AAS-owned data: stop producers, reset the AAS database, start the service, confirm fresh initialization and republish evidence from source. Rolling an image back alone is unsupported; recovery is re-initialization and republication, not an automatic destructive rollback.
