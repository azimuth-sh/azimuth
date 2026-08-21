# Design: Explicit Adapter Capabilities and Launch Plans

## Authority split

The D46 semantic Plan remains provider-neutral. It says which exact Checks or Challenger targets
belong to a Run, with complete Check implementation sets and finite work units. It does not name an
adapter. Core remains authority for that plan and for validation of the returned D46 bundle.

A separate `azimuth-run-launch-plan` version 1 binds the semantic plan to provider interaction. It
contains the exact Subject, planned time, `execute | import` operation, the complete D46 Plan, one
configured adapter identity and a sorted route for every selection. A route names a capability
address and class but does not change the D46 semantic selection. The launch fingerprint changes
when any route, capability fingerprint, adapter identity, planned time, operation, Subject or
semantic plan changes.

This two-layer shape avoids either bad substitution: provider identities do not pollute reusable
semantic plans, and an adapter choice cannot change invisibly around an unchanged execution
account.

## Explicit configuration and capability identity

The default configuration is the strict JSON file `azimuth/adapters.json`. There is no `PATH`
search or executable-name fallback. An executable locator is absolute or resolved relative to the
configuration file, and core starts it directly without a shell.

Each configured adapter pins:

- one project-local configured adapter id;
- adapter protocol version, provider-family id, adapter version and build identity;
- expected executable and declared resource content digests;
- the expected descriptor fingerprint;
- exact non-secret semantic string settings;
- exact non-secret environment literal values;
- timeout, standard-output and standard-error limits; and
- a dictionary of named capabilities.

Capability classes are closed in version 1:

- `model.extract`;
- `check.execute`;
- `check.import`;
- `challenge.execute`; and
- `challenge.import`.

Addresses remain open as `<adapter-id>/<capability-id>`, and challenge forms remain open policy
identities. One capability may declare several classes. One Run uses exactly one configured adapter
id but may route selections through several capabilities from that adapter. This permits one shared
physical activity to yield Check and Challenge outputs without giving the activity a combined
semantic verdict.

A capability fingerprint binds the protocol version, adapter version and build, behavior-changing
executable and resource digests, capability declaration, supported classes, supported open forms
and non-secret semantic settings. It excludes executable and resource locators and explanatory
prose. The configuration fingerprint also binds behavior-changing digests, exact non-secret
literals and process limits while excluding locators. This separation lets relocation leave
semantic identity stable while behavior or a bounded invocation choice remains attributable.

## Strict process protocol

Every interaction opens each configured executable, resource and import input once, copies the
bytes to a private invocation stage while hashing that same stream and verifies the expected
digest before launching a short-lived configured process from the stage. Staged content is
read-only, with executable permission only where required. This prevents content substitution
between hashing and invocation; it is not a filesystem or network sandbox for authorized adapter
code.

Core clears the child environment and supplies only exact configured non-secret literals. Version
1 has no ambient inheritance, secret reference or interpolation. It sends one complete JSON
request on standard input, closes it and accepts exactly one JSON response on standard output.
Standard output and standard error are drained concurrently into separate capped buffers so one
full pipe cannot deadlock the process. Core enforces the configured timeout. Exit status and
standard error never encode product outcomes.

The protocol-v1 description operation returns adapter, version, build and capability declarations.
`azimuth adapter verify` compares that exact description and its fingerprint with configuration.
Execute and import responses repeat the same handshake identity, so a binary substituted after
verification still fails closed.

A nonzero exit, timeout, truncated exchange or extra standard-output content is a transport
failure. Malformed or schema-invalid response JSON is both a transport-contract failure and a
response-schema exit. Core does not automatically retry an execute request after timeout because
the native activity may have happened. A caller may create a new source-execution identity and Run
explicitly.

## Complete-model Check planning

`azimuth run plan --request <file> --config <file>` accepts a strict request containing:

- the exact D46 Subject;
- the exact planned Unix-millisecond time;
- `execute | import`;
- the exact string-to-string Run context; and
- a sorted non-empty list of Check ids, finite non-empty work-unit ids and explicit capability
  addresses.

Core loads the complete model before applying any request selection and fingerprints that complete
account. There is no `--only` option. It resolves each named Check, its current Check fingerprint
and the complete sorted stable implementation set, then emits the D46 Plan and launch routes.
Unknown, duplicate, inapplicable or mismatched capability selections fail planning.

The E planner selects Checks only and always emits an empty Challenge selection. It does not
resolve a Challenge Plan, Qualification or Claim Judgment, and it does not require a current
Qualification or compare the request context with a binding. Executing a Check is operationally
useful even before an evidentiary decision applies. Change F owns decision-target traversal,
qualification policy and authored Challenge planning.

The capability and transport layers can still be tested for Challenge execution and import with a
hand-authored strict launch plan. That proves the general boundary without pretending E can derive
current decision targets.

## Execute and import

`azimuth run execute` accepts a launch plan and matching configuration. Core validates all
configuration, descriptor, route, class and predecessor identities before invocation. The adapter
translates the already selected semantic targets to native work and returns one complete D46
bundle.

`azimuth run import` additionally accepts explicit `<id>=<file>` inputs. Core stages each input from
the same byte stream used to compute its size and digest, and puts those exact identities in the
request. The adapter receives only staged input locators. It may parse provider-native content but
cannot replace the input identity with a URL, run id or mutable report name. `model.extract` is
part of the capability dictionary for future migration, not an E Run operation.

The returned D46 bundle retains its semantic plan and actual-selection contract. Only its
provenance is extended to repeat:

- the exact `adapter/<configured-id>` normalizer id, configured adapter version and adapter
  fingerprint;
- configured adapter id and adapter version;
- descriptor fingerprint;
- configuration fingerprint;
- launch fingerprint;
- sorted per-selection capability routes and fingerprints; and
- for import, the sorted core-computed input ids, content digests and byte sizes.

The D46 bundle fingerprint protects that provenance. Core checks the response request identity,
Subject, D46 plan, launch identity, routes, adapter description, actual selection, outcomes and the
complete bundle before writing. Output uses a temporary sibling and atomic replacement only after
validation; exit 1 or 2 leaves no output.

Import identities are protected by each bundle revision but are not correction anchors. A later
or completed report from the same provider execution may change those bytes in the next correction;
the Subject, plan, planned time, adapter, configuration, normalizer and routes remain fixed.

Execute and import accept repeatable predecessor bundles. Core validates a complete linear chain
before spawning and binds the sorted revision and bundle-fingerprint identities into the request
fingerprint. No predecessors require revision zero. Otherwise the response must be exactly the
terminal revision plus one, name the terminal bundle fingerprint in `corrects` and preserve every
correction anchor. The request also carries the complete verified terminal predecessor, so a
stateless adapter can copy exact source, start-time and execution-route anchors rather than infer
them. No predecessors use a null terminal account.

The Run id binds the launch fingerprint in addition to its D46 inputs. Changing an adapter,
configuration or capability route therefore creates a different Run rather than a correction of
an execution through another route.

Violated Observations, Challenge findings and explicit partial, cancelled or timed-out Runs are
honest product facts and exit zero when internally valid. A semantic, model, identity, selection or
transport mismatch exits one. CLI, configuration, request and response schema failures exit two.

The exact public surface is:

- `azimuth adapter verify [--config <file>]`;
- `azimuth run plan --request <file> [--model <dir>] [--standards <file>]
  [--workspace <file>] [--manifest <file>...] [--config <file>] [--out <file>]`;
- `azimuth run execute --plan <file> [--predecessor <bundle>...] [--config <file>]
  [--out <file>]`; and
- `azimuth run import --plan <file> --input <id>=<file>...
  [--predecessor <bundle>...] [--config <file>] [--out <file>]`.

Planning rejects `--only`, federated project/workset options and local partial selection. Exact
Check ids in the request are the sole selection surface in this change.

Configuration defaults to `azimuth/adapters.json`; `run verify` and `run inspect` retain their D46
behavior. `run ingest` remains unknown.

D47 deliberately replaces the unpublished pre-D47 Run bundle version 1 shape in place. There is
one current version 1 schema, adapter provenance is required, and no compatibility reader accepts
the earlier shape.

## Temporal intent transition

The current case
`framework/run-bundle-protocol#future-runtime-verbs-remain-absent` was true for D46 and becomes
false when this change adds plan, execute and import. The surviving boundary is
`framework/run-bundle-protocol#ingest-remains-absent`.

The current change parser supports addition and criticality transitions, not replacement or
removal. It cannot honestly project this temporal transition. Implementation must therefore update
the accepted current Run package in the same revision as the command surface, while this design and
proposal preserve why the old case disappeared. Adding a second contradictory case or retaining an
obsolete Claim is not an acceptable parser workaround.

## Deferred boundaries

This change does not derive Subjects, plan federated projects, select Challenges, interpret
Qualifications or Claim Judgments, define secret values, migrate existing extractors, host
webhooks, ingest Runs or derive Assurance State. Production adapters and package publication remain
separate changes. The two synthetic adapters prove the boundary without becoming supported
provider products.
