# azimuth

The dependency-free Rust core for Azimuth's evidence control plane. It derives repository-owned
Claims and their graph, validates that graph, reports traceability and exports version 2 JSON.

Install a checkout with `cargo install --path tools/azimuth`.

## Current commands

```text
azimuth validate
azimuth validate --only 'billing/**'
azimuth report traceability
azimuth report traceability --out traceability.json
azimuth export --out model.json
azimuth init

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
- `model.rs` owns the graph and export version 2.
- `change.rs` handles change projection, finalization and archive gates.
- `federation.rs` assembles revision-bound repository accounts.
- `workflow.rs` scaffolds changes and validates path-isolated work packages.

The strict manifest collections are `realizes`, `check_implementations`,
`mechanism_implementations`, `class_members`, `enumerations` and `artifacts`. Source fingerprints
have the exact lexical form `sha256:<64-lowercase-hex>`. Removed alpha-era collections are rejected;
there is no compatibility reader.

## Deferred execution plane

D43 defines a future Run as a bounded envelope over one exact Subject. It may contain Check or
Challenger executions, while an adapter translates semantic selections to provider-native work and
reports actual selection. Those Run, adapter and normalized-bundle formats are deliberately not
implemented by this change.

The optional Assurance Service remains isolated on its D42 v1 wire until a Run-ledger change
replaces it. Core does not currently ingest service execution records, and the service is not model
authority.

## Federation

Federated projects use a versioned project catalog plus a complete workset. Repository manifests
carry typed area/address source identity, exact revision, owned model-source digests and producer
identity. Complete assembly rejects missing inputs, ownership conflicts, revision skew and
duplicate change authority. A local assembly is explicitly incomplete and cannot be finalized.

## Tests

Run `cargo test --manifest-path tools/azimuth/Cargo.toml`. Fixtures are synthetic and independent of
consumer repositories.
