# Run inspection format

`azimuth run inspect --format json` emits one strict JSON account of a set of Run bundles. It is a
deterministic local reading of the bundles it was given and nothing more: it establishes neither
current repository authority nor Assurance State, and it never invokes an adapter, reads an
artifact or input locator, or contacts the isolated alpha 1 service.

Text is the default format; `--format json` selects this one. `--out <file>` writes exactly the
bytes that would otherwise go to stdout and leaves stdout empty.

## Root

```json
{
  "format": "azimuth-run-inspection",
  "version": 1,
  "protocol_consistent": true,
  "model_authority": "unresolved",
  "assurance_state": "unresolved",
  "bundles": [],
  "findings": []
}
```

These are the only root keys, in exactly this order, and all are always present. `format` and
`version` have exactly the values shown.

`protocol_consistent` is `true` when and only when `findings` is empty.

`model_authority` and `assurance_state` are the constant string `unresolved`. Inspection does not
load a model, a Decision Standards file or a workspace, so it cannot say which Claims a bundle's
observations bear on, whether the Checks it names are still current, or what Assurance State
follows. A consumer must not read a consistent inspection as an assurance verdict. The text format
carries the same two labels as `Current model: unresolved` and `Assurance State: unresolved`.

## Bundle set

Input order is irrelevant. Exact duplicates — bundles equal in every field — are collapsed to one
entry. The remaining bundles are sorted by `run_id`, then `bundle_revision`, then
`bundle_fingerprint`. Every distinct revision of a Run appears as its own entry; the inspection
does not reduce a correction chain to its terminal revision.

## Bundles

```json
{
  "run_id": "sha256:<64-lowercase-hex>",
  "bundle_revision": 1,
  "bundle_fingerprint": "sha256:<64-lowercase-hex>",
  "corrects": "sha256:<64-lowercase-hex>",
  "subject_kind": "workspace",
  "subject_fingerprint": "sha256:<64-lowercase-hex>",
  "status": "complete",
  "observations": [],
  "challenge_results": []
}
```

These are the only bundle keys, in this order, and all are always present. `corrects` is the
immediate predecessor bundle fingerprint on a correction, and `null` on revision zero.

`subject_kind` is `workspace | ci-candidate | artifact | deployment | service | monitoring-window`.
Only the kind and the Subject fingerprint are projected; the Subject's own repositories, artifacts,
environment and window fields are not. `status` is `complete | partial | cancelled | timed-out`.

The record deliberately omits the bundle's plan, actual selection, provenance, artifacts,
diagnostics, activities and timestamps. `contracts/run-bundle.md` remains the account of the
complete bundle; this format is a reading of it, not a re-serialization.

## Observations

One entry per Check execution in the bundle, sorted by check id, then check fingerprint.

```json
{
  "check": "billing/invoice-total-suite",
  "check_fingerprint": "sha256:<64-lowercase-hex>",
  "outcome": "satisfied",
  "fingerprint": "sha256:<64-lowercase-hex>"
}
```

All four keys are always present. `outcome` is `satisfied | violated | inconclusive`.
`check_fingerprint` is the Check identity the execution was launched against; `fingerprint` is the
Observation's own fingerprint.

## Challenge results

One entry per Challenger execution in the bundle, sorted by challenge id.

```json
{
  "challenge": "challenge/<plan-local-id>",
  "challenger": "billing/mutation-search",
  "challenger_fingerprint": "sha256:<64-lowercase-hex>",
  "target_kind": "qualification",
  "target": "billing/invoice-total-binding",
  "target_fingerprint": "sha256:<64-lowercase-hex>",
  "outcome": "clean",
  "fingerprint": "sha256:<64-lowercase-hex>"
}
```

All eight keys are always present. `target_kind` is `qualification | claim-judgment`. `outcome` is
`clean | findings | inconclusive`. A `clean` outcome is only a negative search fact; it is not
evidence that the target decision holds, and the inspection assigns it no aggregate score.

An allowed incomplete scheduled omission produces no entry here. It is carried by the bundle's own
`challenge-selection` diagnostic, which this format does not project.

## Findings

Protocol Findings over the bundle set, in the order verification produced them.

```json
{
  "run_id": "sha256:<64-lowercase-hex>",
  "code": "run/history-gap",
  "detail": "revision 1 is followed by revision 3"
}
```

All three keys are always present. `code` is a lower kebab path id naming the violated protocol
rule; `detail` is an explanatory sentence. The rules these codes report are stated under
bundle-set verification in `contracts/run-bundle.md`.

Findings are reported for both per-bundle invariants and cross-bundle correction history. A bundle
carrying unsafe numbers is excluded from the correction-history pass, so its per-bundle Findings
stand alone.

## Command boundary

```text
azimuth run inspect --bundle <file>... [--format text|json] [--out <file>]
```

`--bundle` is repeatable and at least one is required. `--format` accepts exactly `text` or `json`;
any other value is a command failure. The output path may not equal an input path.

Inspect exits zero for a protocol-consistent set. A well-typed set with protocol Findings exits one
and still emits its complete account, including those Findings — the account is the product, and a
nonzero exit does not suppress it. Malformed JSON, schema failure or command usage exits two and
emits no inspection account.
