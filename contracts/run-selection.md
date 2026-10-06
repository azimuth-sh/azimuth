# Check execution selection

Execution selection is separate from Check meaning and Claim assurance. A Check may participate in several local, CI or hosted Runs. A policy selects eligible Checks by declared requirements; it is not a manually maintained Check ID inventory and is not a scheduler.

## Check requirements

A compact document-owned Check may declare:

```markdown
- Execution:
  - Subjects: workspace, ci-candidate
  - Family: application
  - Requirements: database, container-runtime
```

`Subjects` lists supported Run Subject kinds. `Family` is one generic lower-kebab execution family, not a module, service identity or configured adapter ID. `Requirements` lists facilities that the execution environment must supply; it does not prove their health. Requirements may be omitted. Subject compatibility and facility requirements are structural execution conditions, not evidence applicability decisions.

A Check without execution requirements remains available for explicit bounded planning. Automatic selection reports its missing declaration rather than assigning defaults. Packages may provide authoring guidance and producer integration; core does not interpret method prose to guess eligibility.

## Policy

The selection policy is strict JSON with format `azimuth-run-selection-policy` and version `1`:

```json
{
  "format": "azimuth-run-selection-policy",
  "version": 1,
  "operation": "execute",
  "planned_at_ms": 1787300000000,
  "subject": {
    "kind": "workspace",
    "repositories": [{
      "id": "application",
      "revision": "immutable-revision",
      "content_fingerprint": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
    }]
  },
  "required_context": {},
  "families": ["application"],
  "available_requirements": ["container-runtime", "database"],
  "routes": [{
    "family": "application",
    "execution-capability": "configured-runner/checks"
  }]
}
```

Operation is `execute` or `import`. The exact Subject and required context are ordinary Run inputs. Selected families and available facilities are explicit. Routes associate a family with one explicitly configured capability. Core never discovers providers, guesses capabilities from lexical order or routes an incompatible Check through a fallback. A module move does not change an execution family.

## Resolution

`azimuth run select --policy <file>` uses the complete model and configured adapters. It derives Check-to-Case selections from the account and generates the default `whole` unit when no finer finite partition is supplied. `whole` means one execution unit, not complete internal Surface coverage. A package-specific planner may supply finite units without forcing authors to enumerate them in Markdown.

Selection reports undeclared execution requirements, incompatible Subjects, missing facilities and unavailable routes. Checks outside selected families remain distinguishable from eligible selected work. A successful selected subset does not discharge omitted Claim obligations. Independent Judgment must still account for all relevant bindings and observations.

`run select` writes an `azimuth-run-selection` report containing the policy, concrete request, per-Check coverage and findings. Use `azimuth run plan --selection <report>` to consume it without manually extracting JSON. Planning recomputes the selection against current definitions and configuration and rejects stale or incomplete selections. Selection does not execute work or approve evidence. CI and operators trigger policies. Scheduling, affected-only optimization, automatic expiry and cross-Subject evidence reuse are not defined here.
