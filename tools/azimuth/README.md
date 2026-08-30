# azimuth

The dependency-free Rust core for Azimuth's evidence control plane. It derives repository-owned
Claims, their normative Cases and assurance graph, validates that graph, reports traceability,
exports version 4 JSON and plans and hosts bounded adapter exchanges for provider-neutral Run
bundles.

Install a checkout with `cargo install --path tools/azimuth`.

## Current commands

```text
azimuth validate
azimuth validate --only 'billing/**'
azimuth report traceability
azimuth report traceability --out traceability.json
azimuth export --out model.json
azimuth init --agents codex,claude|none [--adopt-alias]
azimuth reference list
azimuth reference show <id> [--format text|json]
azimuth agent add|remove <codex|claude>
azimuth component add <id> --manifest <file>
azimuth component remove <id>
azimuth update [--check|--dry-run]
azimuth migrate plan --out <file>
azimuth migrate apply --plan <file>

azimuth adapter verify [--config <file>]
azimuth run plan --request <file> [--model <dir>] [--standards <file>] \
  [--workspace <file>] [--manifest <file>...] [--config <file>] [--out <file>]
azimuth run execute --plan <file> [--predecessor <bundle>...] \
  [--config <file>] [--out <file>]
azimuth run import --plan <file> --input <id>=<file>... \
  [--predecessor <bundle>...] [--config <file>] [--out <file>]
azimuth run verify --bundle run.json
azimuth run inspect --bundle run.json
azimuth run inspect --bundle run.json --format json --out inspection.json

azimuth explore create <id> --title <text>
azimuth explore list
azimuth explore show <id>
azimuth explore archive <id> --date YYYY-MM-DD [--explorations <dir>]

azimuth change create <id> --title <text>
azimuth change list
azimuth change show <id>
azimuth change status <id>
azimuth change work-packages <id>
azimuth change brief <id> --package <package-id>
azimuth change check azimuth/changes/<id>
azimuth change finalize azimuth/changes/<id>
azimuth change archive azimuth/changes/<id> --date YYYY-MM-DD

azimuth project check --project project.json --workset workset.json
azimuth project export --project project.json --workset workset.json
azimuth project locate --reference azimuth/project-reference.json
azimuth project finalize --project project.json --workset workset.json --out snapshot.json
azimuth project accept-change --project project.json --before active.json \
  --after archived.json --change <id> --date YYYY-MM-DD --out snapshot.json
```

`azimuth validate` is the sole top-level deterministic validation command. It accepts only explicit
options. Exit code `0` means clean, `1` means Findings were reported and `2` means the account could
not be derived. `azimuth report traceability` is a pure projection over selected Cases with
inherited parent context; it creates no authored authority or execution fact. `azimuth export`
writes model version 4.

`azimuth explore archive` requires an active lower-kebab id, a real Gregorian date and exactly one `Status: approved` field in the exploration preamble. It moves the complete package to `archive/YYYY-MM-DD-<id>/` without rewriting content and rejects an occupied destination.

`azimuth init` requires an explicit integration choice and writes the tracked `azimuth/installation.json` ownership account. The consumer skills, templates, authoring references and migration edges come from the release-owned embedded resource cohort, not this repository's contributor `.agents/skills/`. `azimuth reference` exposes version-matched parser guidance without installing duplicate reference files.

`azimuth agent` changes selected managed integrations. `azimuth component` validates and records an already installed exact-release annotation or emitter component without editing its native manifest. `azimuth update` compares and synchronizes the complete managed resource cohort offline; modified managed files, alias drift and component drift fail before the update. `azimuth migrate` separately plans historical account transitions and applies only a fingerprint-current automatic plan. It does not weaken normal validation.

Model input defaults to `azimuth/model`. Decision Policies and the current Challenge Schedule default to `azimuth/standards/verification.md`, the workspace defaults beside the model root, and `--manifest` is repeatable. Selection operates on declared ids, not paths.

The former top-level validator alias and positional selector are absent. Claim Judgments are repository declarations rather than a separate command. There is no Assurance Service export. Nested change and project commands retain their bounded lifecycle meanings.

`azimuth run verify` validates one or more revisions of the strict [`azimuth-run-bundle` version 1](../../contracts/run-bundle.md) protocol. Protocol-consistent violations, challenge findings and partial Runs exit `0`; internal protocol Findings exit `1`; and malformed JSON, schema errors or usage exit `2`. `azimuth run inspect` emits a deterministic text or JSON account, including protocol Findings on exit `1`, and labels repository authority and Assurance State unresolved.

Adapter configuration defaults to strict [`azimuth/adapters.json`](../../contracts/adapter.md). `azimuth adapter verify` stages each configured executable and resource, performs the version 1 description handshake and fails closed on content, identity or capability drift. Core never searches `PATH`, invokes a shell or inherits the ambient environment.

`azimuth run plan` loads the complete unselected model and accepts Check-only, Challenge-only or mixed strict requests. It creates a provider-neutral semantic Plan, then freezes Subject, operation, one configured adapter and one explicit capability route per selection in a separate [launch plan](../../contracts/run-launch-plan.md). Both request arrays are required and their union is non-empty. There is no partial-model or `--only` path.

Check requests name an exact capability, finite Cases and finite units. Challenge requests name an
authored Plan, exact capability, finite units and nonzero candidate cap. Planning resolves all
twelve Method Qualification, Applicability Decision and Claim Judgment selector forms and
preserves `selected | missing-decision | stale-decision | rejected-decision | invalid-decision |
inapplicable | unresolved-relation`. Every candidate counts before cross-Plan deduplication. Any
adverse candidate, cap overflow, context mismatch or conflicting route fails planning.

The fixed requested Plan union must supply a runnable selection for every form required by each selected decision's Policy. Core validates the exact operation class, current Challenger form and one-adapter boundary; it never chooses or adds a capability. Generated Challenges carry a stable target-derived id, `gate | scheduled` lane and exact semantic scope. Challenge routes project every source-backed scope item to one accountable locator input. Scope affects Plan identity; locator projection affects launch identity.

`azimuth run execute` invokes an execute launch. `azimuth run import` invokes an import launch and requires one or more exact `<id>=<file>` inputs. Executable, resource and import bytes are staged and hashed from the same opened streams. Both operations independently bound output bytes. On a supported host, core starts the adapter in a fresh process group before adapter code runs. One deadline bounds core request writing, response and diagnostic reads and process wait; core signals remaining group members on every terminal path. A host without the required primitive fails before spawn with exit `1`.

An authorized descendant can deliberately use `setsid`, `setpgid` or an equivalent to leave the group. It cannot extend core's wait beyond the deadline, but core does not guarantee its termination. This is not non-escapable descendant containment, daemon supervision, hostile-code isolation or a filesystem or network sandbox.

Core validates the complete response and publishes only by atomic replacement. Repeatable predecessors must form one exact correction chain; a response is revision zero or exactly the next revision with the terminal correction anchor.

A valid violated Observation, Challenge finding, partial or cancelled Run, or adapter-returned protocol-valid `timed-out` Run fact exits `0`. A host-enforced process deadline is a transport timeout and exits `1`, as does a semantic, identity, content, other transport or bundle mismatch. The returned `timed-out` fact is valid only when its complete bundle arrives inside the host deadline; the host timeout publishes no bundle. CLI and schema failures exit `2`. Neither nonzero class leaves the requested output file.

## Model

The intent graph has one assurance centre. A parent Claim states the independently governed
normative proposition and owns criticality, production realization, total Claim Judgment and future
Assurance State. Its Cases are normative constituents with identities
`<spec>#<claim>/<case>`—addressable for evidence, Runs, Observations and impact, but not governed
independently.

All current framework Claims are routine. They owe no realization, Check, Evidence Binding or
decision. Ordinary tests still protect the implementation, but they are outside the Azimuth
evidence graph.

For a non-routine Claim, `verification.md` owns:

- a Check with one atomic terminal proposition;
- sparse Evidence Bindings from that Check to individual Cases;
- shared Method Qualifications for exact Check method compositions;
- exactly one Applicability Decision for each binding;
- exactly one total-composition Claim Judgment for each standard or critical parent Claim;
- Challengers that name open objection forms; and
- Challenge Plans with semantic selectors over the graph.

Evidence Bindings and Claim Judgments name project Decision Policies. One current Challenge Schedule assigns every required or declared form exactly once to `gate | scheduled`.

One Check may bind to several Cases and one Case may receive several Checks. Source only declares
`ImplementsCheck(<project-global-check-id>)`. Workspace or federation assembly attaches semantic
source identity. Evidence meaning never comes from the source marker.

Method Qualification fingerprints compose the exact Check, form, common context and policy.
Applicability Decision fingerprints compose the Case digest, binding-specific proposition and
context, current Method Qualification and policy. Claim Judgment fingerprints bind the exact total
parent composition, including every Case and evidence edge. Challenge selection traverses stable
decision, Case, Claim, realization, mechanism, Check and binding relations. Paths, line numbers and
globs are not semantic selectors, and zero selection never widens to a suite.

## Implementation map

- `spec.rs` parses strict parent Claims and normative Cases.
- `design.rs` parses current mechanisms and structural bindings.
- `verification.rs` parses verification declarations, Decision Policies and the schedule.
- `manifest.rs` reads strict linkage collections.
- `workspace.rs` derives areas, surfaces and realization obligations.
- `validation.rs` reports categorized Findings through one exhaustive registry.
- `traceability.rs` derives traceability, Challenge resolutions and decision-impact edges.
- `run.rs` parses, fingerprints, reduces, verifies and inspects standalone Run bundles.
- `adapter.rs` parses strict configuration and derives adapter and capability identities.
- `run_plan.rs` resolves complete-model Checks and Challenges and builds semantic and launch plans.
- `adapter_host.rs` stages content, hosts bounded processes and validates returned bundles.
- `model.rs` owns the graph and export version 4.
- `change.rs` handles change projection, finalization and archive gates.
- `federation.rs` assembles revision-bound repository accounts.
- `workflow.rs` scaffolds changes and validates path-isolated work packages.
- `resources.rs` embeds the release-owned consumer resource cohort and reference registry.
- `installation.rs` owns installation integrity, explicit components, offline update and account migration.

The strict manifest collections are `realizes`, `check_implementations`, `mechanism_implementations`, `class_members`, `enumerations` and `artifacts`. Source fingerprints have the exact lexical form `sha256:<64-lowercase-hex>`. Removed alpha-era collections are rejected; there is no compatibility reader.

A raw marker-derived mechanism implementation has exactly `spec`, `mechanism`, `site`, `binding`, `file`, `lang` and `source_fingerprint`. Its binding is `<address-kind>:<site>` and it has one exact companion Artifact with matching id, kind and file. The emitter derives an ecosystem-semantic qualified site; file paths cannot disambiguate it. Assembly resolves the file's area and atomically rewrites both binding and companion id to `<area>|<address-kind>|<site>`. The paired companion is marker-only, optional Artifact properties survive, and the assembled id is not expanded again. Local and federated assembly apply identical rules.

The ecosystem account is closed: .NET uses namespace/type/method/metadata signature; Java and Kotlin use binary class/method/JVM descriptor; TypeScript and JavaScript use package, compiler module, receiver, symbol and canonical overloads; Go uses import path, receiver, function and typed signature with positional generics; Python uses the one root-relative module and `__qualname__`; Rust uses one conventional Cargo target, reachable module and normalized declared signature whose type-path spelling remains semantic; and C++ accepts only an external-linkage, non-module, non-template, unconstrained program-global declaration and uses its qualified name and canonical function type. Ambiguous or unsupported identities fail rather than incorporating a path.

## Run and adapter execution plane

Azimuth implements one immutable provider-neutral bundle revision for a bounded Run over one exact
Subject and semantic plan. The bundle records actual selection, physical activities, ordered
attempts, one terminal Observation per selected Check-to-Case projection and one Challenge Result
per selected Challenger target. Canonical fingerprints and full-replacement corrections make the
standalone account deterministic without making it current repository acceptance.

The semantic Plan is provider-neutral. A separate launch plan binds it to one configured adapter and exact capability routes so provider substitution changes launch identity and the derived Run id. Adapters expose a closed semantic capability dictionary with open configured addresses: `model.extract`, `check.execute`, `check.import`, `challenge.execute` and `challenge.import`.

Current planning resolves exact current positive Method Qualifications, Applicability Decisions
and Claim Judgments from the complete model. Every Challenge selection contains its lane, target
and semantic scope; every Challenge route repeats the exact form and explicit capability and
carries accountable inputs for source-backed scope. A Check request does not require either
decision, because executing a Check and judging evidentiary applicability remain separate meanings.

Challenge Results are exactly `clean | findings | inconclusive`. Clean records only a negative
search fact and creates no evidence or decision. Every planned Challenge omitted from a partial,
cancelled or timed-out Run has one selection-scoped execution diagnostic and no fabricated Result.
Scheduled omission is allowed deferral; gate omission remains an honest execution failure. Added
or substituted targets, context, units or scope are mismatches. Method findings fan through their
dependent applicability edges; applicability findings stay local to the exact binding.

`model.extract` execution is not implemented. Adapters are bounded short-lived processes; there is no long-running adapter, daemon, service bridge or webhook boundary.

The current Run bundle version 1 requires adapter provenance and rejects the unpublished earlier shape without a compatibility reader.

The optional Assurance Service remains isolated on its alpha 1 v1 wire until a Run-ledger change replaces it. Core does not ingest Run bundles or service execution records, and the service is not model authority. Authorization, durable storage, retention and Subject-specific Assurance State remain ledger responsibilities. Current planning defines no cache validity, cadence, cross-Subject reuse or historical applicability. `azimuth run ingest` is unknown.

## Federation

Federated projects use a versioned project catalog plus a complete workset. Repository manifests carry typed area/address source identity, exact revision, owned model-source digests and producer identity. Complete assembly rejects missing inputs, ownership conflicts, revision skew and duplicate change authority. A local assembly is explicitly incomplete and cannot be finalized.

## Tests

Run `cargo test --manifest-path tools/azimuth/Cargo.toml`. Fixtures are synthetic and independent of consumer repositories. `experiments/challenge-planning/check.sh` exercises all twelve selectors, mixed planning, explicit route inputs, mutation, fault and broad-analysis outcomes, scheduled omission, import provenance and selection mismatch through public commands.
