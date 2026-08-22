# Outcome: adapter-capability-protocol

Status: accepted

## Result

Azimuth now plans Check Runs from the complete current model and invokes explicit short-lived
provider adapters through one strict protocol. A provider-neutral semantic Plan remains separate
from the launch plan that binds an exact Subject, planned time, operation, configured adapter and
one capability route per selection. Capability or configuration substitution therefore changes
launch identity and the derived Run id without polluting reusable semantic selection.

`azimuth adapter verify`, `azimuth run plan`, `azimuth run execute` and `azimuth run import` expose
the boundary. Core pins adapter content and description, stages executable, resources and import
inputs from the streams it hashes, invokes without a shell or ambient environment, bounds one
request/response exchange and validates the complete returned Run bundle before atomic output.
Valid adverse or incomplete execution facts remain successful protocol exchanges.

The executing and importing synthetic adapters exercise the complete public journey. Exact import
identities, stateless correction predecessors, capability substitution, Challenge findings,
transport failures, schema failures and no-output failure paths share one conformance gate.

## Acceptance checks

- The canonical root gate passed on revision `376ac49`. It ran 246 Rust core and CLI tests,
  including 16 adapter-contract, 19 planner, 15 bounded-host and 16 adapter/Run CLI cases.
- The host suite includes a deliberate Unix `setsid` escape. Core still returns exit one at its
  deadline, and the test explicitly cleans the escaped process instead of claiming it was
  contained.
- The adapter capability and migrated Run-bundle conformance gates passed through the public CLI.
  The adapter gate independently recomputes request and predecessor identities and preserves a
  successful Challenge `findings` result separately from its Check Observation.
- The root gate passed every extractor and language fixture, the Assurance Service and web build,
  PostgreSQL-backed lifecycle, private deployment lifecycle, all 66 release tests and package
  candidate qualification.
- Release isolation discovered 11 experiment roots and 51 executable inputs with one immutable
  documentary citation. Five package candidates, two image contracts and three native targets
  qualified without publication.
- Five release manifests validated 95 Claims across nine specs with zero Findings. Selected
  adapter traceability contains all 27 case Claims, and selected export version 2 contains one
  adapter spec and no retired evidence collections.
- All 34 current framework requirements are routine. The current model has no `verification.md`,
  Evidence Binding, Qualification or Claim Judgment facet for these ordinary engineering tests.
- Work-package dependency, strict-command, public-link, skill-frontmatter, all-routine,
  no-compatibility, line-width, diff and prohibited-name audits passed.

## Departures

- D46's semantic Plan deliberately remains provider-neutral. An early alternative put capability
  identity only in invocation provenance; the accepted design adds a separate launch plan and
  binds its fingerprint into Run identity so provider substitution cannot be invisible.
- The unpublished pre-D47 Run bundle version 1 shape was replaced in place. Adapter provenance is
  now required, the older shape is rejected and the existing standalone Run fixtures were migrated
  rather than read through a compatibility path.
- Stateless corrections initially received only predecessor fingerprints, which was insufficient
  to preserve source and time anchors. Requests now also carry the complete verified terminal
  predecessor while fingerprints continue to bind its ordered immutable identity.
- Contract review made executable, resource and import identity use the same staged byte streams.
  The child environment became an exact literal allowlist, and response parsing gained a strict
  typed envelope before Run interpretation.
- Planner review preserved the established raw finalization model digest while wrapping that same
  digest in the D46 `sha256:` lexical form for Run plans. Neither authority now changes the other's
  public representation.
- CLI review found that lexical output-path comparison missed symlink and `..` aliases. Planning,
  execution, import and inspection now resolve input/output aliases and preserve existing outputs
  on every failure.
- Experiment review found that an echoed request id did not independently prove predecessor
  binding and that Challenge findings were unexercised. The synthetic adapter now recomputes every
  request identity, rejects malformed predecessor accounts and returns an asserted findings case.
- Final adversarial review falsified a stronger claim that Unix process groups guarantee complete
  descendant termination. D47 now promises a fresh process group and bounded core exchange only.
  Authorized descendants may escape with `setsid` or `setpgid`; they cannot extend core's deadline,
  but Azimuth does not claim to terminate or sandbox them.

## Residual decisions

- Generated Challenge selection remains deferred. Repository Challenge Plans can resolve authored
  Qualification targets, but a later change must project current applicability into Run selections
  and add Claim Judgment targets without a whole-suite fallback.
- `model.extract` is a declared capability class without a current invocation command. Existing
  extractors remain outside the Run adapter path until a dedicated migration is accepted.
- The current bounded host implements fresh process groups on Unix. Other platforms fail before
  spawn until an honest platform-specific group or job primitive is implemented and tested.
- Adapters are authorized short-lived code, not hostile workloads. Stronger filesystem, network or
  non-escapable process isolation, daemon supervision, secrets and production provider packages
  require separate authority.
- Durable ingest, authorization, correction acceptance, revocation, retention and Subject-specific
  Assurance State remain Run-ledger work. The D42 service wire stays isolated without a bridge.
- Event gateways, webhooks and long-running monitoring ingress remain separate from both the
  short-lived adapter protocol and the future ledger.
- Package versions and release authority intentionally remain at alpha 1. The coordinated alpha 2
  release change owns the version transition, hosted execution accounts and publication.

## Measurements

- adapter requirements: 8 routine;
- adapter case Claims: 27;
- current framework requirements: 34 routine;
- current accepted Claims: 95 across 9 specs;
- current model Findings and verification facets: 0 and 0;
- Rust tests in the canonical root gate: 246;
- adapter contract, planner, host and CLI tests: 66;
- release tests: 66;
- isolated experiment roots and executable inputs: 11 and 51; and
- qualified package, image and native subjects: 5, 2 and 3.
