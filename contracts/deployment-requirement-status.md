# Deployment Requirement Status

Requirement Status is the conclusion established by applicable evidence about a Requirement for an observed deployment. The assessment target is that deployment. Test Runs retain the exact source, artifact or deployment Verification Target they executed against. Build provenance and deployment reconciliation establish applicability relations; they are not product Checks and do not manufacture evidence.

## Authored applicability

Each Check can declare one strict JSON metadata line in verification.md:

```markdown
- Deployment: {"verification_target":"deployment","components":["web-api"],"contexts":[{"component":"web-api","leaves":["features.labs_enabled","auth.challenge_scheme"]}],"contribution":"both","monitor":null}
```

`verification_target` is source, artifact or deployment. `components` identifies the relevant release components, not every repository in the project. `contexts` lists the context leaves the Check's result depends on, as `{component, leaves}` entries; each `component` must be a registered component and listed in `components`, and every leaf must exist in that component's execution-context declaration (`check-unknown-leaf`). An explicit empty array declares no configuration dependency. Values are never authored in the Check. `contribution` is support, counterexample or both. `monitor` is null or an object with unique `id`, positive `interval_seconds` and `freshness_seconds` at least the interval; Monitors require a deployment execution Verification Target. Optional `configuration_sources` adds explicitly named release repositories to the component's complete build-input sources for source Checks (for example an independently scanned deployment repository). Exact repository revisions and content must match; unrelated extra inputs are not accepted. Optional `units` pins the complete required execution unit inventory. Optional `controls` contains `{unit,contexts}` declarations, with `contexts` in the same `{component, leaves}` form and a subset of the Check's dependencies, for deliberate sensitivity controls and requires `units`; only those reviewed unit/leaf pairs may differ from deployed values. At least one applicable non-control unit must cover a relied-on Scenario. The dependency list is part of the policy, which is part of the Check fingerprint and independent Requirement Review dependencies. A mapping that converts failures into violations must itself be included in the reviewed Check implementation or inputs.

Historical Checks without this declaration remain readable. Their evidence is not automatically transferable to a deployment. A different whole-model revision alone does not invalidate evidence whose current Check and Scenario bindings, source lineage and contexts remain exact.

## Effective Execution Contexts

An application's Execution Context is the set of behavior-relevant conditions under which an application instance operates; a Run's Execution Context is the set under which the Check executed. A component's execution-context declaration defines the relevant properties, and the component's Context endpoint (conventionally `/internal/execution-context`) reports its effective Execution Context. The component reports facts; the Azimuth Assurance Service determines whether those facts establish evidence applicability.

### Component declaration

Each component that reports a context has one declaration file at the path its `context` field names in the project catalog (conventionally `azimuth/execution-contexts/<component-id>.json`):

```json
{
  "format": "azimuth-component-context",
  "version": 1,
  "component": "web-api",
  "leaves": {
    "auth.authenticate_scheme": {"type": "string"},
    "features.labs_enabled": {"type": "boolean", "description": "Labs routes are served"}
  }
}
```

`leaves` is a flat map from a dotted leaf name to `{type, description?}`; `type` is `string | boolean | integer`. Dots are naming only and core never parses them as hierarchy. A leaf name containing `password`, `secret`, `credential`, `token`, `privatekey` or `private_key` is rejected, as is a declaration that names a component other than the one registered. Descriptions are documentation and do not participate in any fingerprint. Declarations that are not runtime contexts (test-stand profiles, fixture contexts) remain ordinary context contracts and are not component declarations.

Core derives from each declaration the context contract it registers, unchanged in shape: `{name: <component-id>, fields: {leaf: type}, fingerprint}`, the fingerprint hashing canonical `{name, fields}`. Registration, validation and capture matching are the unchanged contract routes below.

### Served report

Each component serves one report on its own internal endpoint, on a listener that is not public and reports effective values including fail-closed defaults:

```json
{"format":"azimuth-execution-context","version":2,"contexts":{"web-api":{"contract_revision":"sha256:…","values":{"auth.authenticate_scheme":"Bearer","features.labs_enabled":false}}}}
```

The envelope stays at version 2. It holds exactly one context, named by the component id, with dotted leaf names as the keys of `values`. The report is `Cache-Control: no-store` and carries the boot-identity header. A component's declared leaves must equal its served leaves; a leaf declared and not reported (`context-leaf-unreported`) or reported and not declared (`context-leaf-undeclared`) fails the component's own conformance check, and the service rejects a capture whose keys differ from the registered contract. The service matches each context by name to a registered contract and does not require the context name to equal the capture's `component`; the collector alone enforces that the report holds exactly one context named by the component id.

Only explicitly declared nonsecret effective values are permitted. Credentials and credential hashes must not be declared or reported. Owners review the leaf allowlist; structural validation rejects undeclared leaves and prohibited secret-like names, but does not classify arbitrary scalar values as secret. Adapters read actual settings; declarations do not configure applications.

### Captures and Run pins

A snapshot is `{captured_at_ms,instance,report}`. Every capture has `activity`: null for deployment captures and whole-Run execution captures, or the exact native execution activity ID. Execution captures bracket that Run or activity and bind the exact bundle fingerprint, Verification Target, component and process lifetime. The assessor requires captures for the activities of the actual relied-on unit attempts; an unused baseline process cannot stand in for the execution. The Run's actual Execution Context pins one entry per leaf the Check depends on: `context/<component>/<leaf>` for the whole Run or `context/<component>/<leaf>/activity/<activity ID>`, valued with `sha256:` of the canonical JSON scalar observed. Component ids contain no `/` or `.` and leaf names no `/`, so the key parses unambiguously. Production captures must be fresh, match the observed deployment and every current instance of relevant components, and lie within the observation validity window. Startup-fixed contexts only are supported; a process restart invalidates captures from its predecessor. The Kubernetes adapter uses container runtime IDs and requires the application process to be the container entry point: an in-container supervisor that restarts the application without restarting its container is unsupported. The boot-identity header must remain stable across each capture; adapters for other process models need an independently observed process-lifetime binding.

### Leaf dependency semantics: unchanged since the Run

A Check's evidence applies to a deployment only if the deployment's current capture holds, for every listed `{component, leaf}`, the same value the Run pinned. A deployment where any listed leaf differs, or whose capture lacks the leaf, gets no support from that Run; if the leaf later returns to the Run's value the evidence applies again. Controls follow the same comparison except that a reviewed control's unit/leaf pair is required to differ. Leaf values the Check does not list never affect applicability. The dependency list belongs to the Check fingerprint.

## Context conditions and not-applicable

Requirements and Scenarios may carry `Context:` conditions ([spec.md](spec.md)). The assessor evaluates every condition of a Requirement or Scenario against the deployment's current context capture of the named component.

- A condition that evaluates false, or whose leaf the capture lacks, makes the Requirement (Requirement-level lines) or that Scenario (Scenario-level lines) `not-applicable` for this deployment. Not-applicable is not a violation, is not a Check Result, and counts neither toward Satisfied nor toward Not satisfied. Evidence is neither required nor read for it.
- If every Scenario of a Requirement is not applicable, the Requirement is `not-applicable`. A Requirement with at least one applicable Scenario is assessed over the applicable Scenarios only.
- If the deployment has no current capture for a named component, or the capture is stale or inconsistent with the observed instances, the condition is unknown and the Requirement or Scenario stays `unresolved`, with a reason that names the component.
- Presentation labels the value "Not applicable here" and shows the condition lines. A not-applicable Requirement is neither satisfied nor failing: it neither disqualifies a rollback candidate nor counts toward its `non_routine_requirements`.
- The condition lines are part of the text under review, so a changed condition invalidates the Requirement Review like any text change.

## Unit-level reuse

A source Check whose every unit is declared hermetic by the project partition rule applies to a release or deployment when each unit's input fingerprint equals the release manifest's, per [unit-inputs.md](unit-inputs.md). The assessor otherwise requires the existing whole-source equality. Deployment-level and artifact-level Checks never use reuse. Each evidence record carries `reused` and `executed_for_release`.

## Ingestion and assessment

Owners POST `{contract}` to `/v1/projects/{project}/context-contracts`. Authenticated producers or publishers POST `{capture}` to `/context-captures`. Client captures never contain `trusted`; ASS derives trust from authenticated Run ownership or the current deployment observation. Replays preserve original immutable bytes and do not renew timestamps.

A durable background worker assembles a bounded consistent relational snapshot and invokes the same native assessor as `azimuth assurance deployment-status --input request.json`. `GET /v1/projects/{project}/deployments/{deployment}/status` and registered-deployment model Verification Target state routes read the resulting typed relational assessment projection; they do not invoke the assessor or resolve target observations on the request path. Missing relations, contexts, reviews or fresh Monitor Check Results produce explained unresolved results. Routine Requirements retain their review exemption. Accepted Requirement Review establishes verification credibility, not an observed deployment result.

Newer Monitor executions supersede older ones, including incomplete or inconclusive attempts. Equal-time conflicting Check Results are unresolved. Correction heads are resolved before temporal selection. Applicable declared counterexamples remain visible even when review is missing. A failed test defaults to inconclusive; only an explicitly reviewed mapping can classify it as a violation.

## Public test integrations

Azimuth.Testing.Xunit and @azimuth-sh/playwright capture actual test context and emit versioned per-attempt reports. The azimuth_testing adapter reconciles exact native TRX/Playwright identities against project-authored mappings and constructs native Run bundles through azimuth_run. Context-bearing partitions have explicit launch boundaries; missing tests, retries, skips and capture failures cannot be converted to invented successful coverage.

Publishing is a separate operation. It verifies native bundles and uploads exact bytes with scoped credentials. CI may build all tools from one exact Azimuth Git commit, restore a local NuGet package feed and install a local npm tarball; registry publication is not required. The release records the selected tooling and contract identities.

## Storage rollout

The Azimuth Assurance Service storage is at version 8. It initializes an empty database; an existing assurance schema of another version or signature is refused rather than upgraded, and there is no upgrade branch. Rolling out a version replaces service-owned data: dump the database offline, stop producers, reset the database, start the service, confirm fresh initialization and republish from git and CI; observations and captures rebuild themselves. Run and deployment history, measured first-supported timestamps and the last stored rollback status are lost by a reset. Rolling an image back is another reset, never an automatic destructive rollback. Version 8 removes the accepted-authority pointer, authority selection history and release acceptance, and the release `baseline_model_revision`; it adds main-model publications, the release `integration` and `unit_inputs` documents, per-Requirement `supported_since_ms`, per-Check run relations and the `not-applicable` Scenario status. A real upgrade path is built only when an external deployment exists.

## Runtime target evidence

The [target observation protocol](target-observation.md) separates measurement identity from deployment attribution. A collector POSTs measured image/process identities and stable Check definitions or locally allowlisted context reports. It does not query ASS for a Verification Target, deployment selection or contract. ASS retains raw facts relationally, correlates them with independently reconciled historical intervals and materializes validated native evidence only after an unambiguous match. Original observation fingerprints and both reconciliation records remain accountable. Unmatched records are visible and pending; they do not establish support and cannot be assigned by arrival time. Existing exact-target Run and capture protocols retain their separate meaning.


## Materialized assessment reads

`GET /v1/projects/{project}/deployments/{deployment}/status/summary` returns the assessment header and compact `requirements` rows (`requirement`, `status`) for overview/list use, omitting detailed evidence. The full status endpoint retains the native result fields and detailed Requirement evidence while adding service freshness metadata. Both endpoints expose `assessment` with `state` (`ready`, `pending`, or `expired`), `generation`, `published_generation`, `evaluated_at_ms` and `valid_until_ms`. Only explicit `ready` authorizes presentation of the result as current. Missing metadata, pending refresh and elapsed validity do not imply a current Satisfied status. Presentation labels the internal `supported | violated | unresolved` values Satisfied / Not satisfied / Unresolved and labels Check Result outcomes Passed / Failed / Inconclusive; a passing Check or accepted Review alone never implies a satisfied Requirement. No page read renews evidence validity.

Input facts and reconciliation durably enqueue invalidation in the committing transaction. Workers follow Check/Scenario/Requirement and context dependencies, coalesce work, and use the native selected-Requirement evaluator with the complete validation closure. Changing the observed deployment or review basis may affect all its Requirements. Native evidence applicability and reduction semantics remain authoritative; projection tables are rebuildable, not authored decisions. The worker publishes affected Scenario facts, Requirement results and the deployment assessment header atomically, only if its input generation is still current. Failures leave work durable and the projection pending.

Time passage can invalidate support without new evidence. The worker schedules freshness boundaries, and the read path independently refuses to call an expired projection current. Historical evidence remains immutable. Publication prepares release/deployment relationships but never creates evidence for an unobserved deployment or accepts a release.

Definition-based Requirement Review applicability is materialized independently per model revision. `/requirement-reviews` and `/model-revisions/{revision}/requirement-reviews` read typed projected decisions and reference original immutable review records. Their `assessment` metadata distinguishes `ready` from `pending`, with no time expiry for definition-only applicability. Changed review associations and relevant retained Verification Challenge Run correction/target relationships invalidate the dependent revision. Unrelated ordinary Monitor Check Results do not invalidate definition review applicability. A pending review projection is not a missing or rejected human review.

Derived deployment Requirement graphs have bounded retention: publishing a replacement projection removes superseded computed graphs in the same transaction. Original evidence, review records, raw observations and reconciliation history are never pruned by projection maintenance. Existing repeatable-read readers retain a coherent prior snapshot through database MVCC. Projection generations are not immutable evidence identities and authorize nothing.

## Checks run, Runs relied on, and first-supported time

Both `releases/{release}/summary` and the deployment `status/summary` add, per Requirement, `checks`: one entry per Check with `check_id`, `scenarios`, `outcome` (`supported | counterexample | inconclusive | skipped | missing`) and `runs`, the Runs relied on, each `{run_fingerprint, run_id, executed_for_release, reused, subject: {repository, commit}, units: [{id, input_fingerprint, matched}], executed_at_ms}`. `skipped` is a Check whose selected tests were skipped or not executed ([run-bundle.md](run-bundle.md)); it contributes no support. A not-applicable Requirement or Scenario lists no Checks.

Per (deployment, Requirement) the assessment also records `supported_since_ms`, the start of the current continuous Satisfied streak. It is null unless the Requirement is currently Satisfied and resets whenever the status leaves Satisfied, including to not-applicable. It is measured from the service's own assessments, not backfilled from Run times.
