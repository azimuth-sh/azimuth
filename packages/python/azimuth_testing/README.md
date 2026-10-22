# Public typed-test normalization and publishing

These standard-library Python modules connect public xUnit/Playwright capture integrations to
Azimuth's existing configured adapter protocol. Harness reports are execution facts, not Runs.
Explicit project mappings give those facts their Check/Scenario meaning. Publication is independent.
Expose the commit-pinned `packages/python` directory through `PYTHONPATH`; nothing uses an
installed floating release, ambient source discovery, or a connection from tests to ASS.

## Mapping

```json
{
  "format": "azimuth-test-mapping",
  "version": 2,
  "framework": "xunit-v2",
  "source_system": "example-ci",
  "tests": [{
    "id": "Example.Tests.RejectsMissingPermission",
    "native_id": "Example.Tests.RejectsMissingPermission|Example.Tests.RejectsMissingPermission",
    "check": "caller-permissions",
    "unit": "rejects-missing-permission",
    "scenarios": ["missing-permission"],
    "components": ["api"],
    "contexts": [],
    "failed_outcome": "inconclusive"
  }]
}
```

`id` is the exact typed-reporter identity. `native_id` is an exact native identity:

- TRX: `TestDefinitions/UnitTest/TestMethod@className` + `.` + `@name` + `|` +
  `UnitTestResult@testName`. Theory-row display arguments are part of that exact name.
- Playwright JSON reporter: `spec.id` + `|` + `test.projectId` (empty project ID is allowed).

Use `python -m azimuth_testing inventory --trx results.trx` or
`--playwright-json results.json` to inspect identities. Inventory does not assign Requirements/Checks.
No fuzzy matching or display-name inference is performed. A typed identity can map to several
Check/unit/Scenario bindings, but one native identity cannot stand for different typed identities.

Passing tests contribute satisfied only for their declared Scenarios. Failure defaults to inconclusive.
`failed_outcome: violated` is an explicit terminal interpretation whose suitability must be pinned
by independent Requirement Review; generic assertion/setup failures are not automatically counterexamples.
Skipped, interrupted, timed-out, missing or diagnostically incomplete executions are inconclusive.
A static xUnit skip may have no harness fact: its TRX execution is still reconciled and retained as
inconclusive. Missing selected native tests never become success. Native runner outcome disagreement
with typed facts rejects normalization. Result intervals must overlap within a fixed five-second
runner-delivery tolerance: TRX can reconstruct failure start times from delayed event delivery.
Effective context intervals always come from actual harness capture timestamps, not TRX event time. Retries retain all attempts and a declared violation cannot
be erased by a subsequent passing attempt. Multiple required tests in a unit combine conservatively.

## Native report envelope

Retain raw reports and provide their exact content as an import input:

```json
{
  "format": "azimuth-native-test-results",
  "version": 1,
  "framework": "xunit-v2",
  "documents": [{"format": "trx", "content": "<TestRun>...</TestRun>"}]
}
```

For Playwright use `framework: playwright`, `format: playwright-json`, and an object as `content`.
Provide all native retry reports in explicit chronological order; TRX documents contribute attempt
ordinals in that order. Duplicate identities within one document are rejected as ambiguous.
Typed harness report format is the shared `azimuth-test-report` version 1.

## Contexts and explicit partitions

`contexts` in the mapping lists the required context leaves per component as
`[{"component": "api", "leaves": ["auth.registered_schemes"]}]`; a component serves one context named by
its id and leaves are dotted names. A mapping that still lists contract fingerprints is rejected.
Captures require identical before/after process-lifetime identity and selected effective values.
The planned required Execution Context must already contain
`context/<component>/<leaf>/activity/<activity-id>` for every required leaf, with the canonical
SHA-256 fingerprint of that leaf's scalar value. `activity_identity(check, unit, ordinal)` derives the stable ID
from the exact mapped Check, unit and zero-based attempt ordinal. No ambient setting is inferred.
Capture diagnostics make outcomes inconclusive. Contracts and secret-field declarations are
validated by core/ASS; adapters do not manufacture contract acceptance.

A Run may contain many independently captured test units and retries. Each context-bearing unit
must map exactly one native test; every attempt has its own activity interval and context pin.
Different fixtures may use different process instances and effective values without inventing a
suite-wide lifetime. No-context units may aggregate multiple tests. Missing declared units remain
inconclusive. `partition_report(...)` offers explicit per-attempt selections when desired;
`selection=[{id, attempt}]` imports the selected attempt under a distinct source execution identity.
Plan each selection before importing; partitioning never silently discards retries.

Capture sidecars bind exact Run ID, bundle fingerprint, Verification Target fingerprint, **activity**, component, instance
and actual before/after snapshots. They never assert `trusted`; ASS derives attribution from
authentication. Startup-fixed context is supported. Each capture covers its referenced activity's
actual body interval, rather than a fictitious whole-suite interval. Native producers may instead
use `activity:null` only when a genuine capture brackets the entire Run.

A Check's reviewed deployment policy may pin an exact `units` inventory and explicitly declare
`controls:[{unit,contexts:[{component,leaves}]}]`. Only those unit/leaf pairs may intentionally
differ from deployed context; their capture stability and Run context pins are still required.
The full declared unit inventory must match both plan and execution. Controls cannot alone
establish positive Scenario support. The adapter never infers controls from mismatching settings.

## Configure and import

```sh
export PYTHONPATH="$AZIMUTH_CHECKOUT/packages/python"
python -m azimuth_testing configuration > adapters.json
"$AZIMUTH" adapter verify --config adapters.json
"$AZIMUTH" run plan --request request.json --model model --config adapters.json --out launch.json
"$AZIMUTH" run import --plan launch.json --config adapters.json   --input mapping=mapping.json --input report=report.json   --input native-results=native-results.json --out bundle.json
"$AZIMUTH" run verify --bundle bundle.json
```

The generated configuration pins executable, Run helper and normalizer bytes from the same
canonical checkout. If combining with other adapters, include this configured adapter explicitly;
never replace another project's configuration implicitly. After any source modification regenerate
the configuration and plan. Core stages/hash-verifies opened resources and imports before spawn.
For a partition add `--input selection=selection.json`. Corrections require native core predecessor
arguments and an additional `--input correction=correction.json` containing an explicit `reason`.
Unchanged source execution, Verification Target, launch, Execution Context and start anchors remain mandatory.

Generate the deterministic sidecar and confirm equality with the core-imported bundle:

```sh
python -m azimuth_testing normalize --launch launch.json --report report.json   --mapping mapping.json --native-results native-results.json   --verified-bundle bundle.json --out reconstructed.json --captures-out captures.json
```

Add matching `--selection`, `--correction` and complete `--predecessor` history where used.
The standalone normalizer does not replace core's strict verification or authority validation.

## Separate publisher

```sh
python -m azimuth_testing.publisher --bundle bundle.json --cli "$AZIMUTH"   --base-url https://assurance.example/api --project example   --token-file "$RUNNER_TEMP/producer-token" --captures captures.json
```

A project-scoped producer credential is supplied only to the protected publisher step. The CLI
requires successful native `run verify` before any network upload. Retries preserve exact Run
bytes; upload does not spawn tests. Sidecars upload only after their Run receipt identifies the
same immutable Run. Partial upload can be retried idempotently. HTTP 409/authentication/schema
errors are terminal; transient transport/429/5xx failures have bounded exponential retries.
Redirects are refused to prevent forwarding credentials. HTTPS is mandatory outside explicit
loopback integration tests. Tokens and response bodies are never included in error messages.
Use an always-run artifact step after test execution, then normalize/publish unsuccessful facts
where possible without granting ASS credentials to untrusted PR execution.

## Verification

```sh
PYTHONPATH=packages/python python3 experiments/test-report-adapter/test_adapter.py
```

Synthetic tests exercise the real CLI descriptor/plan/staged-import/verifier path and actual local
HTTP servers for byte-identical retry, receipt validation and terminal conflict handling. They do
not depend on a consumer checkout, use mocked runner behavior, or contact production.
