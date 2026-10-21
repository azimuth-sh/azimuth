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

Selection reports undeclared execution requirements, incompatible Verification Targets, missing facilities and unavailable routes. Checks outside selected families remain distinguishable from eligible selected work. A successful selected subset does not discharge omitted Requirement obligations. Independent Judgment must still account for all relevant bindings and Check Results.

`run select` writes an `azimuth-run-selection` version 2 report containing the policy, concrete request, per-Check coverage and findings. Use `azimuth run plan --selection <report>` to consume it without manually extracting JSON. Planning recomputes the selection against current definitions and configuration and rejects stale or incomplete selections. Selection does not execute work or approve evidence. CI and operators trigger policies. Scheduling, affected-only optimization, automatic expiry and cross-target evidence reuse are not defined here.
