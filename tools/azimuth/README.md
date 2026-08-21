# azimuth

The dependency-free Rust core for Azimuth's evidence control plane. It derives repository-owned
Claims and their graph, validates that graph, reports traceability, exports version 2 JSON and
plans and hosts bounded adapter exchanges for provider-neutral Run bundles.

Install a checkout with `cargo install --path tools/azimuth`.

## Current commands

```text
azimuth validate
azimuth validate --only 'billing/**'
azimuth report traceability
azimuth report traceability --out traceability.json
azimuth export --out model.json
azimuth init

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

azimuth change create <id> --title <text>
azimuth change list
azimuth change show <id>
azimuth change status <id>
azimuth change work-packages <id>
azimuth change instructions <id> --package <package-id>
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
not be derived. `azimuth report traceability` is a pure projection over selected case-level Claims;
it creates no authored authority or execution fact. `azimuth export` writes model version 2.

Model input defaults to `azimuth/model`. Qualification policy defaults to
`azimuth/standards/verification.md`, the workspace defaults beside the model root, and `--manifest`
is repeatable. Selection operates on declared ids, not paths.

The former top-level validator alias and positional selector are absent. There is currently no
command for Claim Judgment or Assurance Service export. Nested change and project commands retain
their bounded lifecycle meanings.

`azimuth run verify` validates one or more revisions of the strict
[`azimuth-run-bundle` version 1](../../azimuth/formats/run-bundle.md) protocol. Protocol-consistent
violations, challenge findings and partial Runs exit `0`; internal protocol Findings exit `1`; and
malformed JSON, schema errors or usage exit `2`. `azimuth run inspect` emits a deterministic text or
JSON account, including protocol Findings on exit `1`, and labels repository authority and
Assurance State unresolved.

Adapter configuration defaults to strict
[`azimuth/adapters.json`](../../azimuth/formats/adapter.md). `azimuth adapter verify` stages each
configured executable and resource, performs the version 1 description handshake and fails closed
on content, identity or capability drift. Core never searches `PATH`, invokes a shell or inherits
the ambient environment.

`azimuth run plan` loads the complete unselected model and resolves the exact fingerprint and full
stable implementation set for every requested Check. It creates a provider-neutral semantic Plan,
then freezes Subject, operation, configured adapter and one capability route per selection in a
separate [launch plan](../../azimuth/formats/run-launch-plan.md). The planner is Check-only, emits
`challenges: []` and has no partial-model or `--only` path.

`azimuth run execute` invokes an execute launch. `azimuth run import` invokes an import launch and
requires one or more exact `<id>=<file>` inputs. Executable, resource and import bytes are staged
and hashed from the same opened streams. Both operations contain the descendant process tree,
bound time and output streams, validate the complete response and publish only by atomic
replacement. Repeatable predecessors must form one exact correction chain; a response is revision
zero or exactly the next revision with the terminal correction anchor.

A valid violated Observation, Challenge finding, partial or cancelled Run, or adapter-returned
protocol-valid `timed-out` Run fact exits `0`. A host-enforced process deadline is a transport
timeout and exits `1`, as does a semantic, identity, content, other transport or bundle mismatch.
CLI and schema failures exit `2`. Neither nonzero class leaves the requested output file.

## Model

The intent graph has two Claim levels:

- a requirement-level Claim states the normative proposition and owns criticality;
- a case-level Claim refines one observable condition and has identity `<spec>#<case>`.

All current framework Claims are routine. They owe no realization, Check, Evidence Binding or
Qualification. Ordinary tests still protect the implementation, but they are outside the Azimuth
evidence graph.

For a future non-routine Claim, `verification.md` owns:

- a Check with one atomic terminal proposition;
- sparse Evidence Bindings from that Check to individual case Claims;
- exactly one Qualification for each binding;
- Challengers that name open objection forms; and
- Challenge Plans with semantic selectors over the graph.

One Check may bind to several Claims and one Claim may receive several Checks. Source only declares
`ImplementsCheck(<project-global-check-id>)`. Workspace or federation assembly attaches semantic
source identity. Evidence meaning never comes from the source marker.

Qualification fingerprints compose canonical Check, binding and required-context fingerprints.
Challenge selection traverses stable Claim, realization, mechanism, Check and binding relations.
Paths, line numbers and globs are not semantic selectors. Claim Judgment selectors are reserved
until a total-composition format is accepted.

## Implementation map

- `spec.rs` parses strict requirement and case Claims.
- `design.rs` parses current mechanisms and structural bindings.
- `verification.rs` parses Qualification policy and verification declarations.
- `manifest.rs` reads strict v2 linkage collections.
- `workspace.rs` derives areas, surfaces and realization obligations.
- `validation.rs` reports categorized Findings through one exhaustive registry.
- `traceability.rs` derives deterministic case-level traceability.
- `run.rs` parses, fingerprints, reduces, verifies and inspects standalone Run bundles.
- `adapter.rs` parses strict configuration and derives adapter and capability identities.
- `run_plan.rs` resolves complete-model Checks and builds semantic and launch plans.
- `adapter_host.rs` stages content, hosts bounded processes and validates returned bundles.
- `model.rs` owns the graph and export version 2.
- `change.rs` handles change projection, finalization and archive gates.
- `federation.rs` assembles revision-bound repository accounts.
- `workflow.rs` scaffolds changes and validates path-isolated work packages.

The strict manifest collections are `realizes`, `check_implementations`,
`mechanism_implementations`, `class_members`, `enumerations` and `artifacts`. Source fingerprints
have the exact lexical form `sha256:<64-lowercase-hex>`. Removed alpha-era collections are rejected;
there is no compatibility reader.

## Run and adapter execution plane

D46 implements one immutable provider-neutral bundle revision for a bounded Run over one exact
Subject and semantic plan. The bundle records actual selection, physical activities, ordered
attempts, one terminal Observation per actually selected Check and one Challenge Result per
selected Challenger target. Canonical fingerprints and full-replacement corrections make the
standalone account deterministic without making it current repository acceptance.

The semantic Plan is provider-neutral. A separate launch plan binds it to one configured adapter
and exact capability routes so provider substitution changes launch identity and the derived Run
id. Adapters expose a closed semantic capability dictionary with open configured addresses:
`model.extract`, `check.execute`, `check.import`, `challenge.execute` and `challenge.import`.

Current planning selects Checks only. Repository Challenge Plans already resolve authored
Qualification targets, but the planner does not project those targets or their current
applicability into generated Run selections. Claim Judgment target resolution remains later. The
planner does not require a current Qualification to execute a Check. Strict hand-authored launch
plans may exercise Challenge execute or import transport without establishing current model
authority. `model.extract` execution, long-running adapters and service or webhook bridges are not
implemented.

The current Run bundle version 1 requires D47 adapter provenance and rejects the unpublished
pre-D47 shape without a compatibility reader.

The optional Assurance Service remains isolated on its D42 v1 wire until a Run-ledger change
replaces it. Core does not ingest Run bundles or service execution records, and the service is not
model authority. Authorization, durable storage, retention and Subject-specific Assurance State
remain ledger responsibilities. `azimuth run ingest` is unknown.

## Federation

Federated projects use a versioned project catalog plus a complete workset. Repository manifests
carry typed area/address source identity, exact revision, owned model-source digests and producer
identity. Complete assembly rejects missing inputs, ownership conflicts, revision skew and
duplicate change authority. A local assembly is explicitly incomplete and cannot be finalized.

## Tests

Run `cargo test --manifest-path tools/azimuth/Cargo.toml`. Fixtures are synthetic and independent of
consumer repositories.
