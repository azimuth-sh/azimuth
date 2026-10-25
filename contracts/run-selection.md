# Check execution selection

## Current execution Verification Target boundary

New Run planning accepts `source`, `artifact` and `deployment`. Historical Run descriptors retain their original kinds without conversion. Local and CI are provenance of the exact source snapshot, not separate source types. Stage scheduling cannot infer applicability or assurance transfer from matching content/model revision alone.

The existing explicit `Execution` metadata remains readable for accounts that already authored it. New package integrations use actual method/package contracts, source producer inputs and configured capabilities. No new generic `Requires` list is introduced for Verification Challenge work. Missing source capture, reviewed scope, native review record/history or tool configuration remains an explicit planning failure rather than silently omitted work.


Execution selection is separate from Check meaning and Requirement assurance. A Check may participate in several local, CI or hosted Runs. A policy selects eligible Checks by declared requirements; it is not a manually maintained Check ID inventory and is not a scheduler.

## Check requirements

A compact document-owned Check may declare:

```markdown
- Execution:
  - Verification Targets: source
  - Family: application
  - Requirements: database, container-runtime
```

`Verification Targets` lists supported Run Verification Target kinds. `Family` is one generic lower-kebab execution family, not a module, service identity or configured adapter ID. `Requirements` lists facilities that the execution environment must supply; it does not prove they work. Requirements may be omitted. Verification Target compatibility and facility requirements are structural execution conditions, not evidence applicability decisions.

A Check without execution requirements remains available for explicit bounded planning. Automatic selection reports its missing declaration rather than assigning defaults. Packages may provide authoring guidance and producer integration; core does not interpret method prose to guess eligibility.

## Policy

The selection policy is strict JSON with format `azimuth-run-selection-policy` and version `2`:

```json
{
  "format": "azimuth-run-selection-policy",
  "version": 2,
  "operation": "execute",
  "planned_at_ms": 1787300000000,
  "verification_target": {
    "kind": "source",
    "repositories": [{
      "id": "application",
      "revision": "immutable-revision",
      "content_fingerprint": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
    }]
  },
  "required_execution_context": {},
  "families": ["application"],
  "available_requirements": ["container-runtime", "database"],
  "routes": [{
    "family": "application",
    "execution-capability": "configured-runner/checks"
  }]
}
```

Operation is `execute` or `import`. The exact Verification Target and required Execution Context are ordinary Run inputs. Selected families and available facilities are explicit. Routes associate a family with one explicitly configured capability. Core never discovers providers, guesses capabilities from lexical order or routes an incompatible Check through a fallback. A module move does not change an execution family.

## Resolution

`azimuth run select --policy <file>` uses the complete model and configured adapters. It derives Check-to-Scenario selections from the account and generates the default `whole` unit when no finer finite partition is supplied. `whole` means one execution unit, not complete internal Surface coverage. A package-specific planner may supply finite units without forcing authors to enumerate them in Markdown.

Selection reports undeclared execution requirements, incompatible Verification Targets, missing facilities and unavailable routes. Checks outside selected families remain distinguishable from eligible selected work. A successful selected subset does not discharge omitted Requirement obligations. The Verification Review must still account for all relevant bindings and Check Results.

`run select` writes an `azimuth-run-selection` version 2 report containing the policy, concrete request, per-Check coverage and findings. Use `azimuth run plan --selection <report>` to consume it without manually extracting JSON. Planning recomputes the selection against current definitions and configuration and rejects stale or incomplete selections. Selection does not execute work or approve evidence. CI and operators trigger policies. Scheduling and automatic expiry are not defined here. Affected execution planning is defined below; it is separate from the evidence applicability rules in unit-inputs.md and deployment-requirement-status.md.


## Affected execution planning

`azimuth run affected --input <file> --out <file>` evaluates an explicit baseline and
candidate without invoking an agent or adapter. The core entry point is
`execution_impact::plan_affected(&Json) -> Result<Json, String>`. It is a planner,
not an execution result or permission to reuse earlier evidence.

The strict request has exactly:

```json
{
  "format": "azimuth-affected-execution-request",
  "version": 1,
  "baseline": null,
  "candidate": {
    "verification_target": {
      "kind": "source",
      "repositories": [{
        "id": "application",
        "revision": "immutable-revision",
        "content_fingerprint": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
      }]
    },
    "execution_context": {},
    "inventory_complete": true,
    "inputs": [{
      "id": "unit:application:invoice-calculation",
      "digest": "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
    }],
    "shared_inputs": [],
    "checks": [{
      "id": "invoice-total",
      "definition_fingerprint": "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
      "dependencies": ["unit:application:invoice-calculation"],
      "dependencies_complete": true
    }],
    "challenges": []
  },
  "pending": {"checks": [], "challenges": []}
}
```

`baseline` is null or a snapshot with exactly the same fields as `candidate`.
`verification_target` is a validated native Run Verification Target; the report
retains both descriptors and their independently computed fingerprints.
`execution_context` is an object of unique identity keys to string values.
Input and definition digests use lowercase `sha256:<64 hex>` notation.
Input identities are unique; Check and Challenge ids are unique within their own
kind. Dependency, shared-input and pending lists contain unique nonempty identities
without whitespace. Unknown/missing fields, duplicate fields or identities, wrong
types and malformed digests fail. Array declaration order does not determine
selection priority; report inventories and explanations are sorted.

`inventory_complete` is an explicit declaration of whether the snapshot contains
all eligible Check and Challenge definitions for the caller's execution scope.
The pure planner cannot prove an arbitrary supplied inventory complete. Normal
local/CI integrations derive snapshots from the complete native model with
`execution_impact::snapshot_from_model(model, project, verification_target,
execution_context, shared_inputs)`, not manually maintained Check-id lists. This
builder derives Requirement, Check, binding, Challenge declaration and producer
input identities and fingerprints. It retains exact repository content inputs on
all source-target actions as a conservative fallback for source not proven covered
by finer dependency relationships. Caller-supplied shared inputs are additional
build/configuration/manifest inputs and are not an alternative evidence cache.

`execution_impact::snapshot_from_model_with_units` takes those same arguments plus
`&unit_inputs::UnitInputs` computed by the existing project partition machinery.
For a single-repository source target, a Check covered by a nonempty set of wholly
hermetic units uses their canonical input fingerprints instead of the whole-source
fallback. Unit fingerprints already include shared inputs, overlaps/widening and
the complete partition rule. Other Checks and all model-derived Challenges retain
whole-source fallback. Multi-repository unit scope is rejected rather than guessed.
These builders do not add AST/call-graph inference. A finer Challenge dependency
manifest may be supplied to the pure planner only by an independently established
complete producer; sparse source markers alone do not establish completeness.

### Selection and missing inputs

The planner compares additions, removals and digest changes across the union of
baseline/candidate inputs. An action is selected when it is new, its definition or
dependency membership changed, any current or previous dependency changed, or its
id remains in `pending`. `pending` carries unresolved obligations from earlier
iterations; callers remove an obligation only after its required outcome and
currentness have actually been established. Comparing just the last edit without
carrying outstanding obligations is not a complete correction plan.

The following widen execution to all declared candidate actions, with explicit
reasons: absent/incomplete baseline, changed execution context, changed target kind,
changed non-source exact target, changed shared-input declarations, changed shared
inputs, changed inputs with no declared consumer in either snapshot, empty or
incomplete dependency inventories, and unavailable dependency/shared inputs.
Changed source target identity alone is not a cache hit or proof of evidence
applicability; the compared declared inputs determine selection while the report
retains the actual target difference. A target's source content fallback normally
selects affected actions even when no finer source relation exists.

A missing candidate dependency/shared input, incomplete candidate action inventory,
or pending id absent from the candidate produces `status: "incomplete"` and explicit
diagnostics. Missing pending work is never silently discharged by deleting its
definition. An explicitly complete empty inventory is allowed; missing/inferred
inventories do not establish that no work is required.

### Report

The report has exactly `format: "azimuth-affected-execution"`, `version: 1`,
`status: "complete" | "incomplete"`, `request_fingerprint`,
`baseline_verification_target`, `baseline_verification_target_fingerprint`,
`verification_target`, `verification_target_fingerprint`, `changed_inputs`,
`widening`, `checks`, `challenges`, `selected`, `removed`, `diagnostics` and
`fingerprint`. Absent baseline descriptor/fingerprint values are null.

Each Check/Challenge row is exactly `{id, status, definition_fingerprint, reasons,
changed_dependencies}`. Row status is `selected | unaffected | missing`; a missing
row has a null definition fingerprint. `selected` and `removed` each contain the
sorted `checks` and `challenges` id arrays. Removed definitions are reported even
when no pending obligation exists. `widening`, `diagnostics`, `changed_inputs`,
row reasons and changed dependencies are sorted unique string arrays.
`request_fingerprint` pins the canonical submitted request; `fingerprint` pins the
canonical report excluding itself. No timestamp or environment discovery affects
the selection.

`complete` means the planner can account for its declared candidate scope; it never
means the selected work executed or passed. `unaffected` means declared inputs are
unchanged; it does not relabel a historical Run for the candidate or establish
cross-target applicability. The execution integrator still resolves configured
capabilities, exact scenarios and native units through ordinary Run planning.
Required full-suite policy can always broaden this selection. CI reports
incomplete planning or execution failures as diagnostics; it does not create work
packages, assignments or correction requests.
