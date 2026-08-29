---
name: azimuth-apply
description: Implement an approved Azimuth change, update current facets, run honest engineering checks and leave the change ready for finalization. Use after approval or when resuming an active change; do not archive it.
---

# Apply a change

Implement the approved target without treating proposal prose as source truth.

## Workflow

1. Read every change artifact, affected current model package, the format contracts under `contracts/` and repository guidance. Run `azimuth change status <id>`.
2. If `work-packages.md` exists, validate and coordinate it. Otherwise follow dependency order in the change's `plan.md`.
3. Implement observable behavior and mechanisms within the approved boundary. Record necessary departures instead of silently changing the proposal.
4. Apply accepted intent deltas to package `spec.md`. Distil only mechanisms that now exist into current `design.md`.
5. Keep every current framework Claim routine. Do not add a package `verification.md`, Check, Evidence Binding, Qualification or Claim Judgment merely to describe ordinary tests.
6. Add Realizes only where required and where the production site genuinely establishes part of a named case-level Claim. An unmarked native test remains ordinary engineering work.
7. Build and run focused tests while iterating, then affected component and composed suites. Emit every relevant language manifest and run `azimuth validate` over their union.
8. When the change alters an enumerated surface, run its real enumerator and validate the negative path with a temporary representative untagged member. Remove the temporary member afterward.
9. Run `azimuth report traceability` when relationships change and inspect version 2 export when a public graph shape changes.
10. For Run-bundle work, verify every relevant correction set with `azimuth run verify --bundle <file>...`, then inspect the same set with `azimuth run inspect --bundle <file>...`. Follow the strict [Run bundle format](../../../contracts/run-bundle.md); protocol validity is not current model acceptance or Assurance State.
11. For adapter work, verify configured descriptions, create Check-only, Challenge-only and mixed launch plans as applicable from the complete model, and exercise the relevant execute or import path. Test candidate dispositions, required-form coverage, lanes, scope, launch inputs, content and descriptor drift, process bounds, response validation, valid adverse facts, scheduled omission diagnostics and failure without output. Follow the strict [adapter](../../../contracts/adapter.md) and [launch-plan](../../../contracts/run-launch-plan.md) formats.
12. Complete plan and work-package statuses, write `outcome.md`, and leave the proposal at `implemented` until acceptance is genuinely established.

## Alpha-2 boundaries

- Azimuth is an evidence control plane; repository declarations own intent and reviewed meaning.
- Current non-routine verification follows Check → Evidence Binding → Qualification. One current Claim Judgment binds the total applicable composition for each non-routine case Claim.
- The owning `verification.md` authors Checks, Evidence Bindings, Qualifications, Claim Judgments, Challengers and Challenge Plans. Project `azimuth/standards/verification.md` authors Decision Policies and the one current Challenge Schedule.
- Source may implement a Check but never owns its Claim relationship, form or context.
- Challengers and Challenge Plans search for objections; a clean challenge is not product evidence.
- Evidence Bindings and Claim Judgments use Decision Policies with required open forms. The project Challenge Schedule assigns every required or declared form to one `gate | scheduled` lane without changing decision fingerprints.
- Provider-neutral Run bundle version 1 and local `azimuth run verify`/`inspect` are current.
- `azimuth adapter verify`, complete-model `azimuth run plan`, and bounded `run execute` and `run import` transport are current. Configuration defaults to strict `azimuth/adapters.json`.
- Planning accepts Check-only, Challenge-only or mixed strict requests and has no partial-model or `--only` path. It preserves `selected | missing-decision | stale-decision | rejected-decision | invalid-decision | inapplicable | unresolved-relation` candidates and resolves Checks, current accepted Qualifications and Claim Judgments, required forms, schedule lanes, semantic scope and accountable launch inputs from the complete model.
- Each Challenge Plan request names an explicit configured capability, finite work units and a nonzero target cap. Core never auto-selects a capability, trusts provider selectors or widens an unresolved relation to a path, glob or suite.
- A clean Challenge Result is only a negative search fact. An allowed incomplete scheduled omission carries one exact `challenge-selection` diagnostic and no fabricated result; `deferred` is not a result.
- Invoke only staged configured content without a shell or ambient environment. Require supported fresh process-group isolation before spawn and one bounded core exchange whose deadline covers request writing, concurrent bounded-stream draining and core's own wait.
- Signal the process group on every terminal path and clean members and inherited pipes while they retain group membership. Authorized descendants may escape with `setsid`, `setpgid` or equivalent; core does not guarantee their termination. This is not non-escapable descendant containment, a sandbox, daemon supervision or hostile-code isolation.
- Validate request identity, description, launch, provenance, actual selection and the complete bundle before atomic output. A valid adverse or incomplete fact exits zero; no nonzero exit may publish an output bundle.
- Marker-derived mechanisms retain the two-argument annotation. Extractors derive one ecosystem-semantic qualified `site`, path-free typed binding and companion Artifact, and fail closed on ambiguity, unsupported identity or non-normal/outside-root locators. They never use a file path to disambiguate semantic identity.
- `model.extract` is a declared capability class, not a current Run or extraction command.
- Ledger ingest, authorization, retention and Assurance State remain deferred. Current planning defines no cache-validity, cross-Subject reuse or historical applicability inference.
- Adapters are bounded short-lived processes, not daemons, webhook hosts or long-running supervisors.
- The alpha 1 service wire remains isolated, and no Assurance Service export command exists.
- Use `azimuth validate`, `azimuth report traceability` and `azimuth export` for the current model.
- Do not archive; `azimuth-archive` owns the acceptance boundary.
