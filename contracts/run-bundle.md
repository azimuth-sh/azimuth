# Run bundle format

A Run bundle is one strict provider-neutral JSON account of a bounded execution. The file suffix is `.json`; format identity comes from its fields, not its path.

## Root

```json
{
  "format": "azimuth-run-bundle",
  "version": 1,
  "run_id": "sha256:<64-lowercase-hex>",
  "bundle_revision": 0,
  "bundle_fingerprint": "sha256:<64-lowercase-hex>",
  "subject": {},
  "subject_fingerprint": "sha256:<64-lowercase-hex>",
  "planned_at_ms": 1787300000000,
  "started_at_ms": 1787300010000,
  "finished_at_ms": 1787300020000,
  "status": "complete",
  "plan": {},
  "actual_selection": {},
  "provenance": {},
  "artifacts": [],
  "diagnostics": [],
  "activities": [],
  "check_executions": [],
  "challenger_executions": []
}
```

These are the only common root fields. Revision zero has exactly those fields. Later revisions add exactly `corrects` and `correction_reason`. `format` and `version` have exactly the values shown. Status is `complete | partial | cancelled | timed-out`. Timestamps are non-negative integral Unix milliseconds no greater than `9007199254740991`; they satisfy `planned_at_ms <= started_at_ms <= finished_at_ms`.

Revision zero forbids `corrects` and `correction_reason`. A later revision requires both:

```json
{
  "bundle_revision": 1,
  "corrects": "sha256:<immediate-predecessor-bundle-fingerprint>",
  "correction_reason": "a late shard completed the source execution"
}
```

Unknown fields, duplicate object keys, invalid numbers and duplicate identities fail. Every number in the format is a non-negative integral safe integer no greater than `9007199254740991`; fields with narrower rules state them. Explanatory strings are non-empty. All ids described as lower kebab path ids use lowercase ASCII segments separated by `/`; segments contain lowercase letters, digits and interior hyphens.

## Subject

The Subject is exactly one of the following tagged objects. Arrays are non-empty, sorted by id and have unique ids.

### Workspace

```json
{
  "kind": "workspace",
  "repositories": [
    {
      "id": "root",
      "revision": "8ca4c0f",
      "content_fingerprint": "sha256:<64-lowercase-hex>"
    }
  ]
}
```

`revision` is the repository's immutable base revision syntax. `content_fingerprint` is the producer's accountable assertion over the complete relevant tracked, staged, modified and untracked content. A clean workspace still carries the content fingerprint.

### CI candidate

```json
{
  "kind": "ci-candidate",
  "repositories": [
    {
      "id": "application",
      "revision": "8ca4c0f",
      "content_fingerprint": "sha256:<64-lowercase-hex>"
    }
  ]
}
```

Repository content, not a pull-request number or mutable merge ref, makes the Subject exact. The provider's candidate number or ref belongs in provenance attributes.

### Artifact

```json
{
  "kind": "artifact",
  "artifacts": [
    {
      "id": "api-image",
      "digest": "sha256:<64-lowercase-hex>"
    }
  ]
}
```

### Deployment

```json
{
  "kind": "deployment",
  "environment": "production",
  "deployment": "orders/2026-08-21-17",
  "deployment_fingerprint": "sha256:<64-lowercase-hex>",
  "artifacts": [
    {
      "id": "api-image",
      "digest": "sha256:<64-lowercase-hex>"
    }
  ]
}
```

`environment` and `deployment` are lower kebab path ids. The fingerprint covers the immutable deployed configuration; the artifact array is non-empty.

### Service

```json
{
  "kind": "service",
  "environment": "production",
  "service": "orders/api",
  "deployment": "orders/2026-08-21-17",
  "deployment_fingerprint": "sha256:<64-lowercase-hex>"
}
```

### Monitoring window

```json
{
  "kind": "monitoring-window",
  "environment": "production",
  "services": [
    {
      "service": "orders/api",
      "deployment": "orders/2026-08-21-17",
      "deployment_fingerprint": "sha256:<64-lowercase-hex>"
    }
  ],
  "window_start_ms": 1787299200000,
  "window_end_ms": 1787300100000
}
```

The window is half-open and closed at normalization time: `window_end_ms > window_start_ms` and `window_end_ms <= finished_at_ms`. A service deployment change splits the interval into separate Subjects. Alert silence can be satisfied only through explicit planned work units establishing complete and healthy measurement; the Subject alone gives silence no meaning.

There is no historical Subject variant. Historical data uses `provenance.mode = import` with one of the exact Subjects above.

Subject digests and revisions are opaque protocol values. Adapters define their byte-level derivation and provenance; the standalone verifier recomputes only the Subject envelope fingerprint and never claims to have re-derived repository, artifact or deployment content.

## Plan

```json
{
  "model_fingerprint": "sha256:<complete-model-fingerprint>",
  "required_context": {
    "platform": "linux-x86_64",
    "storage": "postgres-17"
  },
  "checks": [
    {
      "id": "payments/recovery-under-broker-loss",
      "fingerprint": "sha256:<check-fingerprint>",
      "implementations": [
        {
          "identity": "payments|rust-symbol|recovery::replay-after-loss",
          "source_fingerprint": "sha256:<64-lowercase-hex>"
        }
      ],
      "units": [
        {
          "id": "whole",
          "parameters": {}
        }
      ]
    }
  ],
  "challenges": [
    {
      "id": "challenge/91f69477f56b9a2ba588fb045529ddbaa7184c79f473eab50f73a0c5f70d038b",
      "challenger": {
        "id": "mutation/implementation-perturbation",
        "fingerprint": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
      },
      "target": {
        "kind": "qualification",
        "id": "payments/recovery-replay-edge",
        "fingerprint": "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
      },
      "lane": "gate",
      "scope": {
        "anchors": [
          {
            "kind": "realization",
            "id": "payments|rust-item|recovery::replay",
            "fingerprint": "sha256:<source-fingerprint>"
          }
        ],
        "inputs": [
          {
            "kind": "check-implementation",
            "id": "payments|rust-item|recovery::replay-after-loss",
            "fingerprint": "sha256:<source-fingerprint>"
          }
        ],
        "fingerprint": "sha256:<scope-fingerprint>"
      },
      "units": [
        {
          "id": "whole",
          "parameters": {}
        }
      ]
    }
  ],
  "fingerprint": "sha256:<plan-fingerprint>"
}
```

`required_context` and every unit `parameters` value are exact JSON objects from non-empty strings to strings. Empty objects are valid. The combined Check and Challenge arrays are non-empty.

Checks sort by `(id, fingerprint)` and ids are unique within a plan. Implementations sort by `(identity, source_fingerprint)` and are non-empty. Units sort by id, are non-empty and have unique ids. One `whole` unit represents native work whose internal population is not separately planned.

Implementation `identity` is the stable semantic SourceIdentity `<area>|<address-kind>|<address>`. Area and address kind are lower kebab ids. Address is non-empty, glob-free semantic identity rather than a file, path, numeric line, or path-plus-line locator; it uses the realization-selector boundary from [verification.md](verification.md).

Challenges sort by their plan-local id, which is unique. `lane` is exactly `gate | scheduled` and is derived from the current Challenge Schedule. Scope is the strict semantic Challenge scope from [verification.md](verification.md); its arrays and fingerprint are validated independently.

The semantic tuple of Challenger fingerprint, target kind and target fingerprint is also unique within the plan. Target kind is exactly `qualification | claim-judgment`. Qualification target ids are Evidence Binding ids; Claim Judgment target ids have exact case-level Claim form `<spec-id>#<case-id>`. The fingerprint, not the display id, is the exact decision target.

Generated Challenge ids are `challenge/<64-lowercase-hex>`, where the suffix is the raw SHA-256 of RFC 8785 canonical UTF-8 for:

```json
{
  "format": "azimuth-challenge-selection-identity",
  "version": 1,
  "challenger_fingerprint": <challenger-fingerprint>,
  "target_kind": <target-kind>,
  "target_fingerprint": <target-fingerprint>
}
```

The id is independent of selector order, authored Plan id, capability, work units, lane and scope. Selections with the same exact tuple union their anchors and inputs. Conflicting capabilities or work units fail planning; neither arrival order nor one lexically smaller Plan wins.

For Challenger fingerprint `sha256:` followed by 64 `a` characters, Qualification fingerprint `sha256:` followed by 64 `b` characters and target kind `qualification`, the canonical preimage is:

```json
{"challenger_fingerprint":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","format":"azimuth-challenge-selection-identity","target_fingerprint":"sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","target_kind":"qualification","version":1}
```

The generated id is `challenge/91f69477f56b9a2ba588fb045529ddbaa7184c79f473eab50f73a0c5f70d038b`.

The standalone verifier recomputes plan identity and lexical exactness. It does not assert that the named model, Check, Challenger or decision is current. Model-aware generated planning performs the current-authority join; the ledger later joins accepted facts.

## Actual selection

```json
{
  "context": {
    "platform": "linux-x86_64",
    "storage": "postgres-17"
  },
  "plan_fingerprint": "sha256:<plan-fingerprint>",
  "checks": [],
  "challenges": [],
  "fingerprint": "sha256:<actual-selection-fingerprint>"
}
```

Actual Check and Challenge entries have the same exact shapes as plan entries. A selected Check repeats its complete planned implementation array and only its units may be a non-empty subset. A selected Challenge repeats its semantic identity, lane and complete scope, and only its units may be a non-empty subset. Context equals `plan.required_context` exactly, and `plan_fingerprint` equals the recomputed plan fingerprint.

Every actual entry resolves to the plan entry with the same identity. Units are a non-empty subset of that entry. Additional or changed entries, fingerprints, implementations, target, lane, scope, unit parameters or context are material mismatches. Status `complete` requires actual `checks` and `challenges` to equal the plan arrays and actual context to equal required context; other terminal statuses may carry a subset of entries or units.

## Provenance

```json
{
  "mode": "execute",
  "source": {
    "system": "github-actions",
    "execution": "repository/actions/runs/991/attempts/2",
    "uri": "https://example.invalid/native-run/991"
  },
  "normalizer": {
    "id": "adapter/synthetic",
    "version": "0.1.0-alpha.2",
    "build_fingerprint": "sha256:<adapter-fingerprint>"
  },
  "adapter": {
    "id": "synthetic",
    "adapter_version": "0.1.0-alpha.2",
    "adapter_fingerprint": "sha256:<adapter-fingerprint>",
    "descriptor_fingerprint": "sha256:<descriptor-fingerprint>",
    "configuration_fingerprint": "sha256:<configuration-fingerprint>",
    "launch_fingerprint": "sha256:<launch-fingerprint>",
    "routes": [
      {
        "selection": {
          "kind": "check",
          "id": "payments/recovery-under-broker-loss"
        },
        "capability": {
          "address": "synthetic/checks",
          "class": "check.execute",
          "fingerprint": "sha256:<capability-fingerprint>"
        }
      }
    ],
    "import_inputs": []
  },
  "generated_at_ms": 1787300021000,
  "principal": "ci/workload-identity",
  "attributes": {
    "runner-image": "ubuntu-24.04"
  }
}
```

Mode is `execute | import`. `source.system` and `normalizer.id` are lower kebab path ids. `source.execution`, version and principal are non-empty strings. Source URI, principal and attributes are optional; no other fields are. Attributes are an exact string map. `generated_at_ms >= finished_at_ms`. Verification never dereferences the URI.

`adapter` is required. Its identity and routes equal the strict launch plan from [run-launch-plan.md](run-launch-plan.md). Route shapes, ordering, capability classes, Challenge forms and fingerprints follow that format exactly. Every address uses the one adapter id. There is exactly one route for every semantic Plan selection and no other route.

`normalizer.id` is exactly `adapter/<configured-adapter-id>`. Its version equals the returned adapter description's `adapter_version` and `adapter.adapter_version`, and its required build fingerprint equals `adapter.adapter_fingerprint`. The distinct `source` object retains the native provider execution.

For `mode: execute`, `import_inputs` is exactly `[]`. For `mode: import`, it is a non-empty array sorted by unique lower-kebab path `id`:

```json
{
  "id": "native-report",
  "digest": "sha256:<64-lowercase-hex>",
  "size_bytes": 18423
}
```

These are the exact identities core computed and supplied to the adapter. Paths, URIs and native execution ids are not input identities and do not occur in this array. The launch operation equals provenance mode. The bundle's Subject, Subject fingerprint and Plan equal the launch plan. The adapter response request id and description are transport fields protected before publication; they are not copied into the provider-neutral bundle.

## Artifacts

```json
{
  "id": "native-report",
  "kind": "test-report",
  "media_type": "application/json",
  "digest": "sha256:<64-lowercase-hex>",
  "size_bytes": 18423,
  "locator": {
    "kind": "uri",
    "value": "https://example.invalid/artifacts/report.json"
  }
}
```

Artifact ids and kinds are lower kebab path ids. Media type and locator value are non-empty. `size_bytes` is a non-negative safe integer. Locator kind is `uri | bundle-relative`. Bundle-relative values use `/`, are relative, contain no empty, `.` or `..` segment and do not start with `/`. Artifact arrays sort by unique id. Verification never reads a locator.

## Diagnostics

```json
{
  "id": "mutation/survivor-17",
  "class": "objection",
  "severity": "error",
  "code": "mutation/survived",
  "message": "The recovery implementation mutation survived.",
  "scope": {
    "kind": "challenger-execution",
    "challenger_fingerprint": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    "target_fingerprint": "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
  },
  "artifacts": ["native-report"],
  "details": {
    "mutation-id": "17"
  }
}
```

Diagnostic ids and codes are lower kebab path ids. Class is `objection | execution | normalization`; severity is `info | warning | error`. Scope is exactly one of:

```json
{"kind":"run"}
{"kind":"activity","id":"fault-probe/attempt-1"}
{"kind":"check-execution","check":"payments/recovery-under-broker-loss"}
{
  "kind": "challenge-selection",
  "id": "challenge/91f69477f56b9a2ba588fb045529ddbaa7184c79f473eab50f73a0c5f70d038b"
}
{
  "kind": "challenger-execution",
  "challenger_fingerprint": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  "target_fingerprint": "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
}
```

Artifact references sort and are unique. Details are an exact string map. Diagnostics explain facts but do not determine outcomes.

For status `partial | cancelled | timed-out`, every planned Challenge omitted from actual selection has exactly one diagnostic whose class is `execution`, whose scope is that `challenge-selection` id and whose non-empty message states the reason. No Challenger execution or Challenge Result exists for the omission. No such omission is valid for `complete`. A diagnostic does not turn deferred work into a result and does not make the outstanding selection disappear.

## Activities

```json
{
  "id": "fault-probe/attempt-1",
  "status": "completed",
  "started_at_ms": 1787300010000,
  "finished_at_ms": 1787300019000,
  "artifacts": ["native-report"],
  "diagnostics": [],
  "attributes": {
    "native-shard": "0"
  }
}
```

Activity ids are unique lower kebab path ids. Status is `completed | failed | timed-out | cancelled`. Times fall within the Run interval and finish no earlier than start. Artifact and diagnostic references sort, are unique and resolve. Attributes are an exact string map.

## Check executions and Observations

```json
{
  "check": {
    "id": "payments/recovery-under-broker-loss",
    "fingerprint": "sha256:<check-fingerprint>"
  },
  "units": [
    {
      "id": "whole",
      "attempts": [
        {
          "ordinal": 1,
          "activity": "fault-probe/attempt-1",
          "outcome": "satisfied"
        }
      ]
    }
  ],
  "observation": {
    "outcome": "satisfied",
    "observed_at_ms": 1787300019000,
    "fingerprint": "sha256:<observation-fingerprint>",
    "artifacts": ["native-report"],
    "diagnostics": []
  }
}
```

Exactly one Check execution exists for every actual Check and none exists for an omitted planned Check. Execution units equal that Check's actual units. Unit ids sort and are unique. Attempts are non-empty with contiguous positive ordinals beginning at one and valid activity references. Attempt outcome is `satisfied | violated | inconclusive`; a non-completed activity requires `inconclusive`. One activity may occur at most once within one execution unit's attempt sequence, while the same activity may support a Check execution and a Challenger execution.

Unit reduction preserves any `violated`. Otherwise a final `satisfied` attempt produces `satisfied`; otherwise the unit is `inconclusive`. Observation reduction preserves any violated unit, produces satisfied only when actual units equal the planned units and all are satisfied, and is inconclusive otherwise. The declared Observation must equal the derived outcome.

Observation time falls within the Run interval and is no earlier than every referenced activity finish. Artifact and diagnostic references resolve. Context is the one actual Run context and is not repeated here. There is no lifecycle stage or expiry.

## Challenger executions and Challenge Results

```json
{
  "challenge": "challenge/91f69477f56b9a2ba588fb045529ddbaa7184c79f473eab50f73a0c5f70d038b",
  "challenger": {
    "id": "mutation/implementation-perturbation",
    "fingerprint": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
  },
  "target": {
    "kind": "qualification",
    "id": "payments/recovery-replay-edge",
    "fingerprint": "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
  },
  "units": [
    {
      "id": "whole",
      "attempts": [
        {
          "ordinal": 1,
          "activity": "fault-probe/attempt-1",
          "outcome": "findings"
        }
      ]
    }
  ],
  "result": {
    "outcome": "findings",
    "observed_at_ms": 1787300019000,
    "fingerprint": "sha256:<challenge-result-fingerprint>",
    "objections": ["mutation/survivor-17"],
    "artifacts": ["native-report"],
    "diagnostics": []
  }
}
```

Exactly one Challenger execution exists for every actual Challenge entry, matched by its plan-local id, and none exists for an omitted entry. Unit and attempt rules mirror Checks. Attempt outcome is `clean | findings | inconclusive`; a non-completed activity requires `inconclusive`.

Unit reduction preserves any findings. Otherwise a final clean attempt produces clean; otherwise the unit is inconclusive. Result reduction preserves any findings, produces clean only when actual units equal all planned units and all are clean, and is inconclusive otherwise. The declared result must equal the derived outcome.

`findings` requires at least one objection id. Every objection resolves to a diagnostic whose class is `objection` and whose Challenger and target fingerprints match the execution. `clean` and `inconclusive` require an empty objection array. References and observation-time rules mirror Observations.

## Canonical fingerprints

Canonical JSON is the JSON Canonicalization Scheme defined by RFC 8785. It uses UTF-8, no insignificant whitespace, ECMAScript JSON string escaping and UTF-16 object-key ordering; input strings are preserved as-is with no Unicode normalization. This format's numbers are safe integers, so RFC 8785 number serialization always writes them without exponent or fraction syntax. Input arrays remain in their declared canonical order and duplicate identities fail; fingerprinting never sorts or deduplicates invalid input. SHA-256 hashes the canonical UTF-8 bytes of the following exact preimage objects, where angle-bracket values stand for the normalized JSON value named by the field:

```json
{"format":"azimuth-subject-fingerprint","version":1,"subject":<subject>}
{
  "format": "azimuth-run-plan-fingerprint",
  "version": 1,
  "subject_fingerprint": <subject-fp>,
  "model_fingerprint": <model-fp>,
  "required_context": <context>,
  "checks": <checks>,
  "challenges": <challenges>
}
{
  "format": "azimuth-run-selection-fingerprint",
  "version": 1,
  "plan_fingerprint": <plan-fp>,
  "context": <context>,
  "checks": <checks>,
  "challenges": <challenges>
}
{
  "format": "azimuth-run-identity",
  "version": 1,
  "source_system": <system>,
  "source_execution": <execution>,
  "subject_fingerprint": <subject-fp>,
  "plan_fingerprint": <plan-fp>,
  "launch_fingerprint": <launch-fp>
}
{
  "format": "azimuth-observation-fingerprint",
  "version": 1,
  "run_id": <run-id>,
  "subject_fingerprint": <subject-fp>,
  "check": <check>,
  "context": <context>,
  "outcome": <outcome>,
  "observed_at_ms": <time>
}
{
  "format": "azimuth-challenge-result-fingerprint",
  "version": 1,
  "run_id": <run-id>,
  "challenge": <plan-local-id>,
  "challenger": <challenger>,
  "target": <target>,
  "outcome": <outcome>,
  "observed_at_ms": <time>
}
{
  "format": "azimuth-run-bundle-fingerprint",
  "version": 1,
  "bundle": <complete-bundle-without-bundle-fingerprint>
}
```

For the complete launch vector in [run-launch-plan.md](run-launch-plan.md), source system `synthetic` and source execution `run-1`, the canonical Run-id preimage is:

```json
{"format":"azimuth-run-identity","launch_fingerprint":"sha256:980dc9e544f41414e3a2735e84a6d9733aee85b2961899bb538f1f34c4347237","plan_fingerprint":"sha256:b75606956b9c1857f8b401d9bad207253b90f6948efddb5532a769b9f488fbfb","source_execution":"run-1","source_system":"synthetic","subject_fingerprint":"sha256:22478698e6731ce5984658e366386e466fe173216bc7cb721168e1638d2dee02","version":1}
```

Its SHA-256 value is `sha256:45acaf027cc7abee8a7a8ba8c0ff3ac80c6af61a16dbc904f6406e0fe11642dc`.

`check`, `challenger` and `target` are the complete corresponding normalized objects, not joined strings. A plan preimage excludes the plan's `fingerprint`; a selection preimage excludes the selection's `fingerprint`; and the bundle preimage excludes only `bundle_fingerprint`.

Every array has one canonical order:

- Subject repositories and artifacts sort by id; monitoring services sort by service id, which is unique in that Subject.
- Plan and actual Checks sort by id; Check implementations sort by identity; units sort by id.
- Plan and actual Challenges sort by plan-local id; Challenge units sort by id.
- Provenance adapter routes use launch-route order. Import inputs sort by id.
- Root artifacts, diagnostics and activities sort by id. Check executions sort by Check id; Challenger executions sort by plan-local Challenge id.
- Execution units sort by id. Attempts are an ordered sequence by contiguous ordinal and are not a set. Artifact, diagnostic and objection reference arrays sort by referenced id.

Every identity named by those sort keys is unique in its containing array. Where a member also has a fingerprint or parameters, two records with the same identity are a duplicate rather than a secondary sort case.

Each supplied fingerprint must equal the recomputed value. The complete bundle fingerprint protects artifact locators, diagnostic details, activities, attempts and correction metadata even when those fields do not affect a semantic result fingerprint.

## Bundle-set verification

Input order is irrelevant. Exact bundle-fingerprint duplicates deduplicate. For one Run id:

- revision zero occurs exactly once;
- each later revision increments by one and corrects the previous revision fingerprint;
- `(run_id, bundle_revision)` cannot name different content;
- one predecessor cannot have several successors;
- Subject, Subject fingerprint, plan, plan fingerprint, required and actual context, source system, source execution, complete normalizer, adapter identity, version, descriptor, configuration, launch, routes, planned time and started time are correction anchors and do not change; and
- the set contains one linear chain with no missing predecessor, gap, fork or cycle.

A correction is a complete bundle. Late work updates actual selection, activities, results, finished time and provenance by replacing the complete previous account. Import-input identities are protected by each revision's bundle fingerprint but may change when a later or completed native report for the same source execution is normalized through the frozen launch route. If a correction anchor was wrong, it is a new Run.

An adapter correction request carries the complete verified terminal bundle as well as the ordered revision/fingerprint chain identities. This gives a stateless adapter every fixed field needed to construct the exact next complete revision; core still verifies the combined chain independently.

## Command boundary

```text
azimuth run verify --bundle <file> [--bundle <file> ...]
azimuth run inspect --bundle <file> [--bundle <file> ...] [--format text|json] [--out <file>]
```

Verify exits zero for a protocol-consistent set even when a terminal fact is violated, findings, partial, cancelled or timed out. Exit two covers malformed JSON, duplicate or unknown fields, wrong primitive types, unknown enum values, malformed ids or fingerprints, unsafe numbers and missing or forbidden conditional fields. Exit one covers well-typed invariant Findings such as non-canonical order, duplicate identities, invalid timestamp relations, fingerprint mismatch, selection, reference, cardinality, reduction or correction-history failure. Command usage also exits two.

Inspect produces a deterministic account. Text is the default; JSON is strict and versioned. `--out` writes exactly the bytes that would otherwise go to stdout and leaves stdout empty. An internally consistent bundle is labeled `protocol-valid`; current repository authority and Assurance State are labeled unresolved.

Inspect exits zero for a protocol-consistent set. A well-typed set with protocol Findings exits one and still emits its deterministic account, including those Findings. Malformed JSON, schema or command usage exits two and emits no inspection account.

`plan`, `execute` and `import` use the separate adapter and launch contracts. `ingest` remains unknown. Standalone verification and inspection never invoke an adapter, read an artifact or input locator, contact the isolated alpha 1 service or translate its records.

This version deliberately replaces the earlier unpublished pre-release shape that lacked adapter provenance. A bundle without the required adapter account is rejected; there is no compatibility reader and no second interpretation of the current version 1 schema.

It likewise replaces the unpublished Challenge shape in place. Every Challenge selection now has `lane` and `scope`, actual selection repeats both, and an omitted Challenge uses the strict selection-scoped diagnostic. A prior Challenge entry without those fields is rejected rather than upgraded or interpreted twice.
