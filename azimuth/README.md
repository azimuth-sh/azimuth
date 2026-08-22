# Azimuth artifacts

This directory contains the repository-owned part of the Azimuth evidence control plane. Physical colocation makes one domain area readable without collapsing intent, mechanism, implementation linkage and reviewed evidentiary meaning.

```text
azimuth/
├── workspace.json        # local areas, derived surfaces and area obligations
├── model/<spec-id>/
│   ├── spec.md            # required intent
│   ├── design.md          # current mechanisms when required
│   └── verification.md    # Checks, decisions, Challengers and Challenge Plans
├── standards/
│   └── verification.md   # Decision Policies and the current Challenge Schedule
├── changes/
│   ├── <active-change>/
│   └── archive/
└── explorations/
    ├── <active-exploration>/
    └── archive/
```

The leaf directory is a model package, not a fixed-file template. `spec.md` is its anchor. Sibling files are discovered only by the exact names above and remain absent when their facet has nothing to say. Every current framework Claim is routine, so the canonical model has no current `verification.md` files. Routine Claims cannot receive Checks, bindings, Qualifications or Claim Judgments.

Every file declares the spec id it belongs to. Declared identity is authoritative; package paths are a navigation convention. Moving a package changes source locations without changing semantic identities.

Requirement Claims state normative propositions and own criticality. Case-level Claims refine observable conditions and retain stable `<spec>#<case>` identities. Mechanisms and implementation sites may relate to several Claims; directory proximity never creates a semantic edge.

For a non-routine Claim, `verification.md` declares first-class Checks, sparse Check-to-Claim Evidence Bindings, one Qualification per binding, one total-composition Claim Judgment per case Claim, Challengers and Challenge Plans. Project standards declare Decision Policies and one Challenge Schedule. Source uses `ImplementsCheck(<project-global-check-id>)`; extractors emit implementation identity only. Unmarked native tests remain ordinary engineering tests.

[`contracts/run-bundle.md`](../contracts/run-bundle.md) defines the strict standalone `azimuth-run-bundle` version 1 exchange. One bundle revision freezes an exact Subject, semantic plan, actual selection, physical activities, ordered attempts, terminal Observations and Challenge Results, provenance and canonical fingerprints. `azimuth run verify` validates that protocol and a correction set; `azimuth run inspect` presents its deterministic account without a service.

Run bundles are exchange inputs, not accepted model-package facets. The strict [`contracts/adapter.md`](../contracts/adapter.md) contract configures short-lived provider processes in `azimuth/adapters.json`; [`contracts/run-launch-plan.md`](../contracts/run-launch-plan.md) binds a reusable provider-neutral semantic Plan to exact configured capability routes. Core plans Checks and Challenges from the complete unselected model, stages configured content and import inputs from the streams it hashes, and validates the returned bundle before atomic publication.

On supported hosts, core starts each adapter in a fresh process group. One configured deadline bounds request writing, response and diagnostic reads and core's own wait; core signals remaining group members on every terminal path. An adapter descendant that deliberately calls `setsid`, `setpgid` or an equivalent can leave that group. It cannot extend core's wait beyond the deadline, but Azimuth does not guarantee its termination. This is not non-escapable descendant containment, daemon supervision, hostile-code isolation or a filesystem or network sandbox.

An adapter-returned protocol-valid `timed-out` Run fact exits zero only when its complete response arrives inside the host deadline. A host-enforced deadline exits one and publishes no bundle.

The implemented journey is `azimuth adapter verify`, `azimuth run plan`, then either `azimuth run execute` or `azimuth run import`. Planning accepts Check-only, Challenge-only and mixed requests. It resolves exact current qualified Qualification or current accepted Claim Judgment targets from the complete model, requires the requested Plan union to cover every policy-required form, and freezes schedule lanes, semantic scope and accountable source inputs into explicit capability routes. No selector widens to a path, glob or whole suite.

A protocol-valid Challenge Result is `clean | findings | inconclusive`; clean is only a negative search fact. Every planned Challenge omitted from a partial, cancelled or timed-out Run has one scoped diagnostic and no fabricated Result; scheduled omission is allowed deferral, while gate omission records execution failure. `model.extract` is a declared capability rather than a current operation. Durable ingestion, authorization, retention and Subject-specific Assurance State remain ledger work. Current planning defines no cache, cadence, historical-applicability or cross-Subject reuse semantics. Adapters are bounded short-lived processes, not daemons, webhooks or long-running services. The existing service stays isolated on its alpha 1 v1 wire until the Run-ledger replacement and receives no compatibility bridge.

`workspace.json` uses the same area-and-mount vocabulary as federation without adding a repository field. It binds independently derived surfaces to enumerators and may require non-routine Claims to have realizations in named areas. Areas derive from source paths; markers do not repeat them.

A marker-derived mechanism implementation has exactly the raw fields `spec`, `mechanism`, `site`, `binding`, `file`, `lang` and `source_fingerprint`, plus one exact companion Artifact. The emitter derives a compiler- or runtime-semantic qualified `site` and raw `<address-kind>:<site>` binding. Assembly atomically rewrites that binding and companion id to `<area>|<address-kind>|<site>`. The file remains an accountable locator and never disambiguates semantic identity. Local and federated assembly apply the same rule.

Format contracts live in the repository's top-level `contracts/`. Proposed states and immutable history live in `changes/`. Neither is scanned as the accepted current model.

An exploration is project-level research and decision shaping above individual changes. Its required anchor is `exploration.md`; optional research and change maps appear only when warranted. A downstream proposal points to the exploration and decision ids it carries.

## Federated placement

In a multi-repository project, each repository may own one or more model roots with the same package contract. A project catalog assigns every model source one intent authority. Two model sources cannot own the same spec.

Source is grouped into stable areas with named mounts. Complete assembly and exact revision receipts are described by `contracts/workspace.md` and `tools/azimuth/README.md`. Each product checkout carries a small `azimuth/project-reference.json`; `azimuth project locate` resolves the singular catalog and reports that repository's areas and model sources.

Repository accounts enumerate tracked active and archived change directories. Complete assembly rejects duplicate change ids. Project-aware acceptance compares complete accepted-active and tested-archive worksets; a local archive cannot substitute for that account.
