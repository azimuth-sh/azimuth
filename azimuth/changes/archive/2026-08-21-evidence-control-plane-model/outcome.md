# Outcome: evidence-control-plane-model

Status: accepted

## Result

D43 now establishes Azimuth's evidence-control-plane semantics. Requirement-level Claims own
normative propositions and criticality; case-level Claims remain independently addressable. A
deliberately enrolled Check executes inside a Subject-bound Run and produces one terminal
Observation. Evidence Bindings, Qualifications, Claim Judgments, Challengers, Challenge Results and
Assurance State have distinct ownership and cardinality.

The public glossary, derived framework account, assurance-extension guidance and model-package
orientation use the same vocabulary. Provider adapters translate bounded core plans but do not
interpret the repository model. Dependent changes own exact formats, commands, transports and
ledger schemas.

The current model adds six routine evidence-control-plane requirements. The nine previously
standard or critical framework requirements are now routine. Their obsolete verification and
judgment facets were removed, while useful design accounts remain. Every active requirement is now
intent-only for Azimuth purposes during the fast-moving alpha.

## Evidence executed

- `azimuth change check evidence-control-plane-model` parses six additions and nine criticality
  transitions and reports every transition applied.
- Work-package validation reports the semantic predecessor and five delegated packages complete.
- A clean-target `cargo test --manifest-path tools/azimuth/Cargo.toml` passed all 177 unit,
  integration and CLI tests.
- Manifest-backed `azimuth check rtm` derives 37 Claims in five specs with no holes, errors or
  warnings.
- A criticality inventory reports all sixteen active requirements as `routine`.
- Documentation contradiction searches found no active statement that a Check emits a Run, that an
  Observation projects into `Covers`, or that provider adapters interpret the model.
- Relative-link, whitespace, 100-column and `git diff --check` reviews passed for changed current
  material. The preserved immutable citation URL remains longer than 100 columns.
- The repository content-boundary scan returned no prohibited domain material.

These are ordinary engineering checks. The routine Claims intentionally have no Azimuth evidence,
Qualifications or Claim Judgments.

## Departures

- The approved exploration initially applied routine criticality only to requirements introduced
  by alpha 2. AFC direction clarified that every active requirement must be routine until the
  codebase stabilizes. This change therefore lowers the existing release and deployment
  requirements and removes their obsolete assurance facets.
- The change retains existing design facets because they preserve useful mechanism knowledge.
  Their implementation bindings continue to resolve through generated manifests even though the
  routine Claims owe no design or realization obligation.
- Exact `verification.md` syntax and Qualification file placement remain deferred. The public
  account states semantic ownership without inventing that format.
- The proposal skill names `azimuth/changes/README.md`, which is absent. The change used
  `azimuth/README.md`, the CLI contract and archived precedents instead.
- The first Rust test invocation reused a target directory compiled in an older checkout. Four CLI
  tests could not launch that obsolete absolute binary path. Removing only generated Cargo target
  artifacts and rerunning from a clean target made the complete suite pass.

## Residual decisions

- Freeze the exact Check, Evidence Binding and Qualification grammar in the breaking verification
  change.
- Define Run and Subject schemas, outcome aggregation and temporal correction in the Run protocol
  change.
- Define adapter transport and namespaced capability configuration after the Run schema is stable.
- Decide ledger acceptance and retention independently of gate policy.
- Raise individual requirement criticality only through a later accepted transition after the
  codebase stabilizes and the consequence warrants assurance obligations.

## Measurements

- New requirement-level Claims: 6 routine.
- New case-level Claims: 8.
- Existing requirements lowered to routine: 9.
- Active requirements above routine: 0.
- Current verification facets removed: 3.
- Current judgment facets removed: 3.
- Delegated implementation packages: 5.
- Manifest-backed model findings: 0 errors and 0 warnings.
