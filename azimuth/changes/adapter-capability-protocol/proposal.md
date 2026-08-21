# Change: adapter-capability-protocol

Status: proposed

Exploration: evidence-control-plane-alpha-2
Carries decisions: E10, E11, E12, E13
Depends on change: run-bundle-protocol

## Problem

D43 assigns semantic selection to core and provider interaction to adapters, while D46 deliberately
stops at validation of a provider-neutral Run bundle. There is no current contract that binds an
exact D46 plan to configured capabilities, launches a bounded provider process, imports an existing
native report or proves that the returned bundle used the planned adapter route.

Leaving that seam implicit would let adapters reinterpret the repository model, substitute tools
without changing execution identity, discover uncontrolled executables from `PATH`, or write a
partially validated bundle. It would also make each provider invent its own process and failure
contract.

## Outcome

Azimuth has an explicit adapter configuration, an `azimuth-run-launch-plan` version 1 and a strict
short-lived process protocol. Core loads the complete unselected model, selects exact Checks and
their complete implementations, creates the D46 semantic plan, binds each selection to a
configured capability and validates the normalized returned bundle before atomic output.

`azimuth adapter verify`, `azimuth run plan`, `azimuth run execute` and `azimuth run import` expose
the boundary. One synthetic adapter executes planned Checks and another imports an existing native
report through one conformance suite. Durable ingest remains absent.

## Scope

In scope:

- define strict `azimuth/adapters.json` configuration with no discovery from `PATH`;
- pin adapter id, version, build, executable and resource content digests, descriptor fingerprint,
  semantic settings, exact non-secret environment literals, allowed inherited environment names,
  process limits and a capability dictionary;
- define closed capability classes with open `<adapter-id>/<capability-id>` addresses and open
  challenge forms;
- define canonical adapter, descriptor, configuration and capability fingerprints;
- define `azimuth-run-launch-plan` version 1 over one exact Subject, planned time, operation,
  complete D46 plan and sorted capability routes;
- generate Check-only launch plans from a strict request and the complete unselected model;
- preserve D46 semantic selection while extending returned bundle provenance with exact adapter,
  descriptor, configuration, launch, route and per-revision import-input identities;
- invoke adapters as bounded short-lived processes with one JSON request on standard input and one
  JSON response on standard output;
- validate protocol handshake, configured description, class support, request identity, returned
  provenance, actual selection and the complete D46 bundle before atomic output;
- import core-content-addressed exact native report files;
- expose adapter verification and Run planning, execution and import commands with stable exit
  classes;
- conformance-test Check and Challenge capability transport with hand-authored launch plans; and
- provide one executing and one report-importing synthetic adapter through a shared gate.

Out of scope:

- resolving authored Challenge Plans, Qualifications or Claim Judgments into Run selections;
- planning Challenges from the current model or applying Challenge Results to current decisions;
- deriving a Subject, operation, context, work units or capability choice automatically;
- partial-model or `--only` Run planning and federated project planning;
- requiring a current Qualification or matching a binding context before a Check can execute;
- defining secret values, secret-provider policy or putting secrets in fingerprints;
- migrating existing source extractors to `model.extract`;
- production provider adapters, daemons, webhooks or a generic event gateway;
- durable Run ingestion, authorization, retention, Assurance State or service migration;
- package-version or release-authority changes; and
- any alpha 1 compatibility reader, alias or migration.

## Affected claims

Add eight routine requirements under `framework/adapter-capability-protocol`:

- adapter configuration and descriptor identity are explicit and pinned;
- the planner selects Checks from the complete current model without inventing challenge authority;
- a launch plan binds an exact D46 semantic plan to explicit capability routes;
- adapter invocation is strict, bounded and shell-free;
- execute validates returned provenance, selection and outcomes before publication;
- import binds exact native files by content before normalization;
- the capability dictionary separates closed semantic classes from open provider identities; and
- the public command and exit contract preserves honest negative execution facts without ingest.

The requirements are routine and owe no Azimuth evidence, Qualification or Claim Judgment.
Parser, transport, CLI and conformance suites are ordinary engineering tests.

The current Claim
`framework/run-bundle-protocol#future-runtime-verbs-remain-absent` is temporally superseded. The
same implementation revision must replace it with
`framework/run-bundle-protocol#ingest-remains-absent`, because plan, execute and import become
current while durable ingest does not. The change parser cannot express replacement or removal, so
the additive intent delta alone cannot project this transition. Current intent must be edited
atomically with implementation and the limitation must remain visible in the change account.

## Completion conditions

- Configuration rejects unknown fields, duplicate adapter or capability identities, relative
  escape, unpinned executable or resource content, unsupported classes and executable discovery
  from `PATH`.
- An executable is absolute or relative to the configuration file, is invoked directly without a
  shell and is hashed before spawn. The child starts with a cleared environment and receives only
  exact configured literals plus explicitly allowed inherited names.
- `azimuth adapter verify` performs the protocol-v1 description exchange and fails if adapter id,
  version, build, descriptor fingerprint or capability dictionary differs from configuration.
- Capability and configuration fingerprints bind behavior-changing executable and resource
  digests, protocol, adapter version and build, declarations and non-secret semantic settings while
  excluding locators, prose and secret values.
- A strict plan request supplies one exact D46 Subject, planned time, operation, exact string
  context and sorted Check selections with finite explicit units and capability addresses.
- The public surface is `azimuth adapter verify [--config <file>]`,
  `azimuth run plan --request <file> [model options] [--config <file>] [--out <file>]`,
  `azimuth run execute --plan <file> [--config <file>] [--out <file>]` and
  `azimuth run import --plan <file> --input <id>=<file>... [--config <file>] [--out <file>]`.
- Planning loads and fingerprints the complete unselected model, resolves every Check fingerprint
  and complete stable implementation set, emits `challenges: []` and has no `--only` path.
- Planning does not require a current Qualification or compare request context to an Evidence
  Binding; Change F owns decision resolution and applicability.
- A launch plan freezes the Subject, planned time, operation, complete D46 semantic plan and one
  configured capability route per selection. Any capability substitution changes its canonical
  fingerprint.
- Exactly one configured adapter id serves a Run, while its routes may name several capabilities;
  one capability may implement several classes and one physical activity may return separate Check
  and Challenge outputs.
- Execute and import use one request/response process, enforce configured timeout and output bounds,
  drain bounded standard output and standard error concurrently, and treat exit status, malformed
  response or timeout as transport failure. Execute is never retried automatically after timeout.
- Import inputs are exact files whose digests and sizes core computes before invocation. Returned
  bundle provenance repeats those identities; adapters cannot replace them with locators or native
  run ids. A correction may carry later bytes from the same native execution while its launch route
  remains fixed.
- Execute and import validate configuration, descriptor, class, launch identity, Subject, semantic
  plan, adapter provenance, routes, actual selection and the complete D46 bundle before atomic
  output.
- Valid violated, findings, partial, cancelled and timed-out execution facts exit zero. Semantic,
  model, identity and transport mismatches exit one. CLI, configuration, request and response
  schema errors exit two. Neither nonzero class leaves an output file.
- `model.extract` is a declared capability class but is not a Run operation in this change.
- `azimuth run ingest` remains unknown, while existing `run verify` and `run inspect` retain their
  standalone protocol behavior.
- The sole current unpublished Run bundle version 1 requires D47 adapter provenance. The earlier
  pre-D47 shape is rejected without a compatibility reader.
- The executing and importing synthetic adapters pass one conformance suite, including
  configuration drift, capability substitution, bounded failures, honest negative facts and a
  dual-role hand-authored launch plan.
- The current Run-protocol Claim is narrowed atomically to `ingest-remains-absent`, all current
  requirements remain routine, and complete Rust, conformance, isolation and composed-model suites
  pass.
