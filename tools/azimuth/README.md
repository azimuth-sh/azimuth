# azimuth

The dependency-free Rust core for Azimuth's evidence control plane. It derives repository-owned
Requirements, their normative Scenarios and assurance graph, validates that graph, reports traceability,
exports version 5 JSON and plans and hosts bounded adapter exchanges for provider-neutral Run
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
azimuth run plan --request <file> [--model <dir>] \
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
not be derived. `azimuth report traceability` is a pure projection over selected Scenarios with
inherited parent context; it creates no authored authority or execution fact. `azimuth export`
writes model version 4.

`azimuth explore archive` requires an active lower-kebab id, a real Gregorian date and exactly one `Status: approved` field in the exploration preamble. It moves the complete package to `archive/YYYY-MM-DD-<id>/` without rewriting content and rejects an occupied destination.

`azimuth init` requires an explicit integration choice and writes the tracked `azimuth/installation.json` ownership account. The consumer skills, templates, authoring references and migration edges come from the release-owned embedded resource cohort, not this repository's contributor `.agents/skills/`. `azimuth reference` exposes version-matched parser guidance without installing duplicate reference files.

`azimuth agent` changes selected managed integrations. `azimuth component` validates and records an already installed exact-release annotation or emitter component without editing its native manifest. `azimuth update` compares and synchronizes the complete managed resource cohort offline; modified managed files, alias drift and component drift fail before the update. `azimuth migrate` separately plans historical account transitions and applies only a fingerprint-current automatic plan. It does not weaken normal validation.

Model input defaults to `azimuth/model`, the workspace defaults beside the model root, and `--manifest` is repeatable. Selection operates on declared ids, not paths.

There is no top-level validator alias or positional selector, and no Assurance Service export. Nested change and project commands retain their bounded lifecycle meanings.

`azimuth run verify` validates one or more revisions of the strict [`azimuth-run-bundle` version 2](../../contracts/run-bundle.md) protocol. Protocol-consistent violations, challenge findings and partial Runs exit `0`; internal protocol Findings exit `1`; and malformed JSON, schema errors or usage exit `2`. `azimuth run inspect` emits a deterministic text or JSON account, including protocol Findings on exit `1`, and labels repository authority and Assurance State unresolved.

Adapter configuration defaults to strict [`azimuth/adapters.json`](../../contracts/adapter.md). `azimuth adapter verify` stages each configured executable and resource, performs the version 1 description handshake and fails closed on content, identity or capability drift. Core never searches `PATH`, invokes a shell or inherits the ambient environment.

`azimuth run plan` loads the complete unselected model and accepts Check-only, Challenge-only or mixed strict requests. It creates a provider-neutral semantic Plan, then freezes Verification Target, operation, one configured adapter and one explicit capability route per selection in a separate [launch plan](../../contracts/run-launch-plan.md). Both request arrays are required and their union is non-empty. There is no partial-model or `--only` path.

Check requests name an exact capability, finite Scenarios and finite units. Challenge requests name a
declared Verification Challenge, exact capability, finite units and nonzero candidate cap. Any cap
overflow, context mismatch or conflicting route fails planning.

Core validates the exact operation class, current Verification Challenge form and one-adapter boundary; it never chooses or adds a capability. Every Challenge selection carries the `gate` lane and exact semantic scope. Challenge routes project every source-backed scope item to one accountable locator input. Scope affects Plan identity; locator projection affects launch identity.

`azimuth run execute` invokes an execute launch. `azimuth run import` invokes an import launch and requires one or more exact `<id>=<file>` inputs. Executable, resource and import bytes are staged and hashed from the same opened streams. Both operations independently bound output bytes. On a supported host, core starts the adapter in a fresh process group before adapter code runs. One deadline bounds core request writing, response and diagnostic reads and process wait; core signals remaining group members on every terminal path. A host without the required primitive fails before spawn with exit `1`.

An authorized descendant can deliberately use `setsid`, `setpgid` or an equivalent to leave the group. It cannot extend core's wait beyond the deadline, but core does not guarantee its termination. This is not non-escapable descendant containment, daemon supervision, hostile-code isolation or a filesystem or network sandbox.

Core validates the complete response and publishes only by atomic replacement. Repeatable predecessors must form one exact correction chain; a response is revision zero or exactly the next revision with the terminal correction anchor.

A valid violated Check Result, Challenge finding, partial or cancelled Run, or adapter-returned protocol-valid `timed-out` Run fact exits `0`. A host-enforced process deadline is a transport timeout and exits `1`, as does a semantic, identity, content, other transport or bundle mismatch. The returned `timed-out` fact is valid only when its complete bundle arrives inside the host deadline; the host timeout publishes no bundle. CLI and schema failures exit `2`. Neither nonzero class leaves the requested output file.

## Model

The intent graph has one assurance centre. A parent Requirement states the independently governed
normative proposition and owns its review declarations, production realization, its Verification and
Realization Reviews and future Assurance State. Its Scenarios are normative constituents with identities
`<spec>#<requirement>/<scenario>`—addressable for evidence, Runs, Check Results and impact, but not governed
independently.

A Requirement may declare `Realization: reviewed` and `Verification: reviewed`. No current framework
Requirement declares either. They are specified and owe no realization, Check, Evidence Binding or
review. Ordinary tests still protect the implementation, but they are outside the Azimuth
evidence graph.

A Requirement that declares `Realization:` owes `Realizes` sites, `ImplementsMechanism` sites where a
`realization.md` exists, and a Realization Review. For a Requirement that declares `Verification:`,
`verification.md` owns:

- a Check with one atomic terminal proposition;
- Evidence Bindings from Checks covering every Scenario; and
- optional Verification Challenges that name open objection forms over a semantic scope.

The Verification Review judges whether the Checks, bindings and declared Challenges are adequate.
The latest accepted review of each kind is a sealed head in the module's `verification-review.md` or
`realization-review.md`.

One Check may bind to several Scenarios and one Scenario may receive several Checks. Source only declares
`ImplementsCheck(<project-global-check-id>)`. Workspace or federation assembly attaches semantic
source identity. Evidence meaning never comes from the source marker.

A Verification Review fingerprints its blocking dependencies: the Requirement text, Scenarios, Checks
and bindings, Challenge declarations and the execution units the Checks run. A Realization Review
fingerprints the Requirement text, Scenarios and realization facet; code changed at its sites is
reported as drift and never blocks. Challenge scope names stable Scenario, Requirement, realization,
mechanism, Check and binding relations. Paths, line numbers and
globs are not semantic selectors, and zero selection never widens to a suite.

## Implementation map

- `spec.rs` parses strict parent Requirements and normative Scenarios.
- `realization.rs` and `account_realization.rs` parse the realization facet's mechanisms and structural bindings.
- `verification.rs` and `account_verification.rs` parse verification declarations.
- `review_heads.rs` reads and scans the review heads stored in the model.
- `manifest.rs` reads strict linkage collections.
- `workspace.rs` derives areas, surfaces and realization obligations.
- `validation.rs` reports categorized Findings through one exhaustive registry.
- `traceability.rs` derives traceability and impact edges.
- `run.rs` parses, fingerprints, reduces, verifies and inspects standalone Run bundles.
- `adapter.rs` parses strict configuration and derives adapter and capability identities.
- `run_plan.rs` resolves complete-model Checks and Challenges and builds semantic and launch plans.
- `adapter_host.rs` stages content, hosts bounded processes and validates returned bundles.
- `model.rs` owns the graph and export version 5.
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
Verification Target and semantic plan. The bundle records actual selection, physical activities, ordered
attempts, one terminal Check Result per selected Check-to-Scenario projection and one Challenge Result
per selected Verification Challenge target. Canonical fingerprints and full-replacement corrections make the
standalone account deterministic without making it current repository acceptance.

The semantic Plan is provider-neutral. A separate launch plan binds it to one configured adapter and exact capability routes so provider substitution changes launch identity and the derived Run id. Adapters expose a closed semantic capability dictionary with open configured addresses: `model.extract`, `check.execute`, `check.import`, `challenge.execute` and `challenge.import`.

Current planning resolves declared Checks and Verification Challenges from the complete model. Every
Challenge selection contains its lane, target and semantic scope; every Challenge route repeats the
exact form and explicit capability and carries accountable inputs for source-backed scope. A Check
request does not require an accepted review, because executing a Check and judging its verification
remain separate meanings.

Challenge Results are exactly `clean | findings | inconclusive`. Clean records only a negative
search fact and creates no evidence or review. Every planned Challenge omitted from a partial,
cancelled or timed-out Run has one selection-scoped execution diagnostic and no fabricated Result;
omission is an honest execution failure. Added or substituted targets, context, units or scope are
mismatches.

`model.extract` execution is not implemented. Adapters are bounded short-lived processes; there is no long-running adapter, daemon, service bridge or webhook boundary.

The current Run bundle version 2 requires adapter provenance and rejects the unpublished earlier shape without a compatibility reader.

The optional Assurance Service remains isolated on its alpha 1 v1 wire until a Run-ledger change replaces it. Core does not ingest Run bundles or service execution records, and the service is not model authority. Authorization, durable storage, retention and target-specific Assurance State remain ledger responsibilities. Current planning defines no cache validity, cadence or cross-target reuse. `azimuth run ingest` is unknown.

## Federation

Federated projects use a versioned project catalog plus a complete workset. Repository manifests carry typed area/address source identity, exact revision, owned model-source digests and producer identity. Complete assembly rejects missing inputs, ownership conflicts, revision skew and duplicate change authority. A local assembly is explicitly incomplete and cannot be finalized.

## Tests

Run `cargo test --manifest-path tools/azimuth/Cargo.toml`. Fixtures are synthetic and independent of consumer repositories.
