# Run bundle format

## Current Verification Target and machine Challenge semantics

New Run requests use three Verification Target kinds: `source`, `artifact`, `deployment`. `source` is the exact source snapshot, with the same repository state fields required for historical workspace capture:

```json
{"kind":"source","repositories":[{"id":"example","revision":"immutable-base-revision","content_fingerprint":"sha256:<64-lowercase-hex>"}]}
```

Local/CI origin belongs to Run provenance, not a different source identity. Complete relevant tracked/staged/modified/untracked capture remains a producer obligation; a digest proves identity only, not capture completeness. Service/component selection and observation intervals are explicit examination scope carried in exact planned Execution Context and native evidence. They do not erase deployment differences or authorize cross-target evidence transfer. Existing `workspace`, `ci-candidate`, `service` and `monitoring-window` Run descriptors remain readable with their original meanings. Their descriptor hashes are not rewritten to new types. They are rejected for newly authored Run requests.

Challenge target `kind` additionally accepts `requirement-review` and `requirement-status`. Its `id` identifies the assessed Requirement and `fingerprint` identifies the exact native assessment record. The execution Verification Target remains the Run Verification Target and is independent of that assessment target. Scope anchors additionally accept these assessment kinds; source-backed scope inputs retain exact selected source identities and fingerprints.

Machine Challenge Results remain `clean | findings | inconclusive` with native objections, artifacts, diagnostics and measured work-unit attempts. Findings may coexist with incomplete search; diagnostics must retain those gaps. Clean never establishes product satisfaction or a supported Requirement Status. Surviving mutations require equivalence/relevance assessment, while timeout/compile/discovery failures remain visible.

The initial Stryker integration records the exact captured source input Verification Target for the batch Verification Challenge execution, native mutation identities/statuses and derivation locations. Its native report does not capture exact materialized source snapshots per mutant. It therefore produces no fabricated mutant Verification Targets or mutant-specific product Check Results. A provider that does execute product Checks against captured mutants must identify each distinct mutant Verification Target explicitly before those Check Results can be used. A mutation result cannot characterize the unmodified deployment's Requirement Status.


A Run bundle is one strict provider-neutral JSON account of a bounded execution. The file suffix is `.json`; format identity comes from its fields, not its path.

## Root

```json
{
  "format": "azimuth-run-bundle",
  "version": 2,
  "run_id": "sha256:<64-lowercase-hex>",
  "bundle_revision": 0,
  "bundle_fingerprint": "sha256:<64-lowercase-hex>",
  "verification_target": {},
  "verification_target_fingerprint": "sha256:<64-lowercase-hex>",
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
  "verification_challenge_executions": []
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

## Verification Target

The Verification Target is exactly one of the following tagged objects. Arrays are non-empty, sorted by id and have unique ids.

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

Repository content, not a pull-request number or mutable merge ref, makes the Verification Target exact. The provider's candidate number or ref belongs in provenance attributes.

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

The window is half-open and closed at normalization time: `window_end_ms > window_start_ms` and `window_end_ms <= finished_at_ms`. A service deployment change splits the interval into separate Verification Targets. Alert silence can be satisfied only through explicit planned work units establishing complete and healthy measurement; the Verification Target alone gives silence no meaning.

There is no historical Verification Target variant. Historical data uses `provenance.mode = import` with one of the exact Verification Targets above.

Verification Target digests and revisions are opaque protocol values. Adapters define their byte-level derivation and provenance; the standalone verifier recomputes only the Verification Target envelope fingerprint and never claims to have re-derived repository, artifact or deployment content.

## Plan

```json
{
  "model_fingerprint": "sha256:<complete-model-fingerprint>",
  "required_execution_context": {
    "platform": "linux-x86_64",
    "storage": "postgres-17"
  },
  "checks": [
    {
      "id": "recovery-under-broker-loss",
      "fingerprint": "sha256:<check-fingerprint>",
      "scenarios": ["replayed-after-broker-loss"],
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
      "id": "challenge/5fb09eceaf657d64c39b90dbd28a3455045be1cd55f15bbebe5bcb0d0881d635",
      "verification_challenge": {
        "id": "mutation/implementation-perturbation",
        "fingerprint": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
      },
      "target": {
        "kind": "method-qualification",
        "id": "payments/recovery-method",
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

`required_execution_context` and every unit `parameters` value are exact JSON objects from non-empty strings to strings. Empty objects are valid. The combined Check and Challenge arrays are non-empty.

Checks sort by `(id, fingerprint)` and ids are unique within a plan. `scenarios` is a non-empty,
sorted, unique array of stable Scenario ids. Implementations sort by `(identity,
source_fingerprint)` and are non-empty. Units sort by id, are non-empty and have unique ids. One
`whole` unit represents native work whose internal population is not separately planned. One Check
selection may name several Scenarios without duplicating its physical execution.

Implementation `identity` is the stable semantic SourceIdentity `<area>|<address-kind>|<address>`. Area and address kind are lower kebab ids. Compiler symbol address kinds retain their ecosystem syntax: .NET array/generic/pointer type notation and ECMAScript package/module/binding and package/module/receiver/member signatures are semantic addresses, including a relative logical module and optional-property/array syntax; these are not file locators or wildcard selections. Address is non-empty, glob-free semantic identity rather than a file, path, numeric line, or path-plus-line locator; it uses the realization-selector boundary from [verification.md](verification.md).

Challenges sort by their plan-local id, which is unique. `lane` is exactly `gate | scheduled` and is derived from the current Challenge Schedule. Scope is the strict semantic Challenge scope from [verification.md](verification.md); its arrays and fingerprint are validated independently.

The semantic tuple of Verification Challenge fingerprint, target kind and target fingerprint is also unique
within the plan. Target kind is exactly `method-qualification | applicability-decision |
requirement-judgment`, plus the assessment kinds `requirement-review | requirement-status` described above. Method Qualification ids are project-global, Applicability Decision ids are exact
Evidence Binding ids retain their declared identity; Requirement Judgment ids equal their stable Requirement ids. The
fingerprint, not the display id, is the exact decision target.

Generated Challenge ids are `challenge/<64-lowercase-hex>`, where the suffix is the raw SHA-256 of RFC 8785 canonical UTF-8 for:

```json
{
  "format": "azimuth-challenge-selection-identity",
  "version": 2,
  "verification_challenge_fingerprint": <verification-challenge-fingerprint>,
  "target_kind": <target-kind>,
  "target_fingerprint": <target-fingerprint>
}
```

The id is independent of selector order, authored Plan id, capability, work units, lane and scope. Selections with the same exact tuple union their anchors and inputs. Conflicting capabilities or work units fail planning; neither arrival order nor one lexically smaller Plan wins.

For Verification Challenge fingerprint `sha256:` followed by 64 `a` characters, Method Qualification
fingerprint `sha256:` followed by 64 `b` characters and target kind `method-qualification`, the
canonical preimage is:

```json
{"format":"azimuth-challenge-selection-identity","target_fingerprint":"sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","target_kind":"method-qualification","verification_challenge_fingerprint":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","version":2}
```

The generated id is `challenge/5fb09eceaf657d64c39b90dbd28a3455045be1cd55f15bbebe5bcb0d0881d635`.

The standalone verifier recomputes plan identity and lexical exactness. It does not assert that the named model, Check, Verification Challenge or decision is current. Model-aware generated planning performs the current-authority join; the ledger later joins accepted facts.

## Actual selection

```json
{
  "execution_context": {
    "platform": "linux-x86_64",
    "storage": "postgres-17"
  },
  "plan_fingerprint": "sha256:<plan-fingerprint>",
  "checks": [],
  "challenges": [],
  "fingerprint": "sha256:<actual-selection-fingerprint>"
}
```

Actual Check and Challenge entries have the same exact shapes as plan entries. A selected Check repeats its complete planned implementation array and only its units may be a non-empty subset. A selected Challenge repeats its semantic identity, lane and complete scope, and only its units may be a non-empty subset. `execution_context` equals `plan.required_execution_context` exactly, and `plan_fingerprint` equals the recomputed plan fingerprint.

Every actual entry resolves to the plan entry with the same identity. Units are a non-empty subset of that entry. Additional or changed entries, fingerprints, implementations, target, lane, scope, unit parameters or Execution Context are material mismatches. Status `complete` requires actual `checks` and `challenges` to equal the plan arrays and actual Execution Context to equal the required Execution Context; other terminal statuses may carry a subset of entries or units.

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
    "version": "0.1.0-alpha.6",
    "build_fingerprint": "sha256:<adapter-fingerprint>"
  },
  "adapter": {
    "id": "synthetic",
    "adapter_version": "0.1.0-alpha.6",
    "adapter_fingerprint": "sha256:<adapter-fingerprint>",
    "descriptor_fingerprint": "sha256:<descriptor-fingerprint>",
    "configuration_fingerprint": "sha256:<configuration-fingerprint>",
    "launch_fingerprint": "sha256:<launch-fingerprint>",
    "routes": [
      {
        "selection": {
          "kind": "check",
          "id": "recovery-under-broker-loss"
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

`adapter` is required. Its identity and routes equal the strict launch plan from [run-launch-plan.md](run-launch-plan.md). Route shapes, ordering, capability classes, Challenge forms and fingerprints follow that format exactly. A direct adapter bundle uses that adapter id. A coordinated bundle retains child provenance and uses the deterministic coordinator identity described below. There is exactly one route for every semantic Plan selection and no other route.

`normalizer.id` is exactly `adapter/<configured-adapter-id>`. Its version equals the returned adapter description's `adapter_version` and `adapter.adapter_version`, and its required build fingerprint equals `adapter.adapter_fingerprint`. The distinct `source` object retains the native provider execution.

For `mode: execute`, `import_inputs` is exactly `[]`. For `mode: import`, it is a non-empty array sorted by unique lower-kebab path `id`:

```json
{
  "id": "native-report",
  "digest": "sha256:<64-lowercase-hex>",
  "size_bytes": 18423
}
```

These are the exact identities core computed and supplied to the adapter. Paths, URIs and native execution ids are not input identities and do not occur in this array. The launch operation equals provenance mode. The bundle's Verification Target, Verification Target fingerprint and Plan equal the launch plan. The adapter response request id and description are transport fields protected before publication; they are not copied into the provider-neutral bundle.

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

Artifact ids and kinds are lower kebab path ids. Media type and locator value are non-empty. `size_bytes` is a non-negative safe integer. Locator kind is `uri | bundle-relative | unavailable`. An unavailable locator retains the artifact digest as its value and asserts no retrievable location; it does not discard measurements required for a Check Result. Bundle-relative values use `/`, are relative, contain no empty, `.` or `..` segment and do not start with `/`. Artifact arrays sort by unique id. Verification never reads a locator.

## Diagnostics

```json
{
  "id": "mutation/survivor-17",
  "class": "objection",
  "severity": "error",
  "code": "mutation/survived",
  "message": "The recovery implementation mutation survived.",
  "scope": {
    "kind": "verification-challenge-execution",
    "verification_challenge_fingerprint": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
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
{"kind":"check-execution","check":"recovery-under-broker-loss"}
{
  "kind": "challenge-selection",
  "id": "challenge/5fb09eceaf657d64c39b90dbd28a3455045be1cd55f15bbebe5bcb0d0881d635"
}
{
  "kind": "verification-challenge-execution",
  "verification_challenge_fingerprint": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  "target_fingerprint": "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
}
```

Artifact references sort and are unique. Details are an exact string map. Diagnostics explain facts but do not determine outcomes.

For status `partial | cancelled | timed-out`, every planned Challenge omitted from actual selection has exactly one diagnostic whose class is `execution`, whose scope is that `challenge-selection` id and whose non-empty message states the reason. No Verification Challenge execution or Challenge Result exists for the omission. No such omission is valid for `complete`. A diagnostic does not turn deferred work into a result and does not make the outstanding selection disappear.

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

## Check executions and Check Results

```json
{
  "check": {
    "id": "recovery-under-broker-loss",
    "fingerprint": "sha256:<check-fingerprint>"
  },
  "units": [
    {
      "id": "whole",
      "attempts": [
        {
          "ordinal": 1,
          "activity": "fault-probe/attempt-1",
          "outcomes": {
            "replayed-after-broker-loss": "satisfied"
          }
        }
      ]
    }
  ],
  "results": [
    {
      "scenario": "replayed-after-broker-loss",
      "outcome": "satisfied",
      "observed_at_ms": 1787300019000,
      "fingerprint": "sha256:<check-result-fingerprint>",
      "artifacts": ["native-report"],
      "diagnostics": []
    }
  ]
}
```

Exactly one Check execution exists for every actual Check and none exists for an omitted planned
Check. Execution units equal that Check's actual units. Unit ids sort and are unique. Attempts are
non-empty with contiguous positive ordinals beginning at one and valid activity references. Every
attempt's `outcomes` object has exactly one key for every selected Scenario and no other key; each value
is `satisfied | violated | inconclusive`. A non-completed activity requires every Scenario outcome to be
`inconclusive`. One activity may occur at most once within one execution unit's attempt sequence,
while the same activity may support a Check execution and a Verification Challenge execution.

Reduction is independent for every selected Scenario. A Scenario/unit preserves any `violated`; otherwise a
final `satisfied` attempt produces `satisfied`, and the unit is `inconclusive` otherwise. The Scenario
Check Result preserves any violated unit, produces satisfied only when actual selection exactly
matches the planned Check—including Scenarios—and all units are satisfied for that Scenario, and is
inconclusive otherwise. `results` has exactly one entry per selected Scenario and each declared
outcome must equal the derived outcome.

Check Result time (`observed_at_ms`) falls within the Run interval and is no earlier than every referenced activity finish. Artifact and diagnostic references resolve. Execution Context is the one actual Run Execution Context and is not repeated here. There is no lifecycle stage or expiry.

## Verification Challenge executions and Challenge Results

```json
{
  "challenge": "challenge/5fb09eceaf657d64c39b90dbd28a3455045be1cd55f15bbebe5bcb0d0881d635",
  "verification_challenge": {
    "id": "mutation/implementation-perturbation",
    "fingerprint": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
  },
  "target": {
    "kind": "method-qualification",
    "id": "payments/recovery-method",
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

Exactly one Verification Challenge execution exists for every actual Challenge entry, matched by its plan-local id, and none exists for an omitted entry. Unit and attempt rules mirror Checks. Attempt outcome is `clean | findings | inconclusive`; a non-completed activity requires `inconclusive`.

Unit reduction preserves any findings. Otherwise a final clean attempt produces clean; otherwise the unit is inconclusive. Result reduction preserves any findings, produces clean only when actual units equal all planned units and all are clean, and is inconclusive otherwise. The declared result must equal the derived outcome.

`findings` requires at least one objection id. Every objection resolves to a diagnostic whose class is `objection` and whose Verification Challenge and target fingerprints match the execution. `clean` and `inconclusive` require an empty objection array. References and `observed_at_ms` rules mirror Check Results.

## Canonical fingerprints

Canonical JSON is the JSON Canonicalization Scheme defined by RFC 8785. It uses UTF-8, no insignificant whitespace, ECMAScript JSON string escaping and UTF-16 object-key ordering; input strings are preserved as-is with no Unicode normalization. This format's numbers are safe integers, so RFC 8785 number serialization always writes them without exponent or fraction syntax. Input arrays remain in their declared canonical order and duplicate identities fail; fingerprinting never sorts or deduplicates invalid input. SHA-256 hashes the canonical UTF-8 bytes of the following exact preimage objects, where angle-bracket values stand for the normalized JSON value named by the field:

```json
{"format":"azimuth-verification-target-fingerprint","version":2,"verification_target":<verification-target>}
{
  "format": "azimuth-run-plan-fingerprint",
  "version": 2,
  "verification_target_fingerprint": <verification-target-fp>,
  "model_fingerprint": <model-fp>,
  "required_execution_context": <execution-context>,
  "checks": <checks>,
  "challenges": <challenges>
}
{
  "format": "azimuth-run-selection-fingerprint",
  "version": 2,
  "plan_fingerprint": <plan-fp>,
  "execution_context": <execution-context>,
  "checks": <checks>,
  "challenges": <challenges>
}
{
  "format": "azimuth-run-identity",
  "version": 2,
  "source_system": <system>,
  "source_execution": <execution>,
  "verification_target_fingerprint": <verification-target-fp>,
  "plan_fingerprint": <plan-fp>,
  "launch_fingerprint": <launch-fp>
}
{
  "format": "azimuth-check-result-fingerprint",
  "version": 2,
  "run_id": <run-id>,
  "verification_target_fingerprint": <verification-target-fp>,
  "check": <check>,
  "scenario": <scenario-id>,
  "execution_context": <execution-context>,
  "outcome": <outcome>,
  "observed_at_ms": <time>
}
{
  "format": "azimuth-challenge-result-fingerprint",
  "version": 2,
  "run_id": <run-id>,
  "challenge": <plan-local-id>,
  "verification_challenge": <verification-challenge>,
  "target": <target>,
  "outcome": <outcome>,
  "observed_at_ms": <time>
}
{
  "format": "azimuth-run-bundle-fingerprint",
  "version": 2,
  "bundle": <complete-bundle-without-bundle-fingerprint>
}
```

For the fixed opaque launch and plan fingerprints below, source system `synthetic`, source execution `run-1` and an artifact Verification Target with one artifact `image` whose digest is `sha256:` followed by 64 `4` characters (Verification Target fingerprint `sha256:339156da3a55df33136e71859f1a3482bb39ee9aa45c11a30780334ee9e63759`), the canonical Run-id preimage is:

```json
{"format":"azimuth-run-identity","launch_fingerprint":"sha256:980dc9e544f41414e3a2735e84a6d9733aee85b2961899bb538f1f34c4347237","plan_fingerprint":"sha256:b75606956b9c1857f8b401d9bad207253b90f6948efddb5532a769b9f488fbfb","source_execution":"run-1","source_system":"synthetic","verification_target_fingerprint":"sha256:339156da3a55df33136e71859f1a3482bb39ee9aa45c11a30780334ee9e63759","version":2}
```

Its SHA-256 value is `sha256:2bf29414172cc5dffce83955f95be42a9d56fd3ed098a25f6c7cc070547395f5`.

`check`, `verification_challenge` and `target` are the complete corresponding normalized objects, not joined strings. A plan preimage excludes the plan's `fingerprint`; a selection preimage excludes the selection's `fingerprint`; and the bundle preimage excludes only `bundle_fingerprint`.

Every array has one canonical order:

- Verification Target repositories and artifacts sort by id; monitoring services sort by service id, which is unique in that Verification Target.
- Plan and actual Checks sort by id; Check implementations sort by identity; units sort by id.
- Plan and actual Challenges sort by plan-local id; Challenge units sort by id.
- Provenance adapter routes use launch-route order. Import inputs sort by id.
- Root artifacts, diagnostics and activities sort by id. Check executions sort by Check id; Verification Challenge executions sort by plan-local Challenge id.
- Execution units sort by id. Attempts are an ordered sequence by contiguous ordinal and are not a set. Artifact, diagnostic and objection reference arrays sort by referenced id.

Every identity named by those sort keys is unique in its containing array. Where a member also has a fingerprint or parameters, two records with the same identity are a duplicate rather than a secondary sort case.

Each supplied fingerprint must equal the recomputed value. The complete bundle fingerprint protects artifact locators, diagnostic details, activities, attempts and correction metadata even when those fields do not affect a semantic result fingerprint.

## Bundle-set verification

Input order is irrelevant. Exact bundle-fingerprint duplicates deduplicate. For one Run id:

- revision zero occurs exactly once;
- each later revision increments by one and corrects the previous revision fingerprint;
- `(run_id, bundle_revision)` cannot name different content;
- one predecessor cannot have several successors;
- Verification Target, Verification Target fingerprint, plan, plan fingerprint, required and actual Execution Context, source system, source execution, complete normalizer, adapter identity, version, descriptor, configuration, launch, routes, planned time and started time are correction anchors and do not change; and
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

This version deliberately replaces the earlier unpublished pre-release shape that lacked adapter provenance. A bundle without the required adapter account is rejected; there is no compatibility reader and no second interpretation of the current schema. Version 2 renames the Verification Target, Check Result, Scenario, Execution Context and Verification Challenge fields and fingerprint formats; a version 1 bundle is rejected rather than upgraded or read twice.

It likewise replaces the unpublished Challenge shape in place. Every Challenge selection now has `lane` and `scope`, actual selection repeats both, and an omitted Challenge uses the strict selection-scoped diagnostic. A prior Challenge entry without those fields is rejected rather than upgraded or interpreted twice.

## Stable entity identities: 2026-10-02

All current Requirement, Scenario, Mechanism and Check references use stable project-wide IDs and retain explicit entity kind. Module membership and Scenario parent relationships do not determine identity. Other activities, units, adapters, capabilities and binding identities retain their contracts. Historical artifacts remain immutable; current alpha artifacts are regenerated without aliases or syntax-only version bumps. See `contracts/entity-identity.md`.

## Coordinated bounded contributions: 2026-10-06

A coordinated Run adds `contributions`: two through 64 exact child Run bundles sorted by unique adapter id. A child cannot itself contain coordinated contributions. A direct single-adapter Run omits this field. The root fingerprint covers retained child records.

Every child must pass standalone protocol verification and share the parent's exact Verification Target, model fingerprint, required Execution Context, planning time and operation. Their planned Check and Challenge selections partition the parent's global Plan exactly. Parent facts are the deterministic projection of all child facts: artifact, activity and diagnostic local IDs gain the adapter prefix; references follow those prefixes; Check Result and Challenge Result fingerprints are recomputed for the global account. Original child bundles remain intact for audit. Extra, duplicate, omitted, mismatched or excessive-depth contributions fail.

The parent uses the deterministic `azimuth-coordinator` identity derived from all exact child adapter identities. It records the complete global routes, union of import input identities and global launch fingerprint. Parent status is complete only if every contribution is complete; otherwise it remains partial. Adverse facts remain adverse. Bounded exchange failures retain explicit execution diagnostics and missing selection without fabricating positive Check Results.

Corrections retain parent and corresponding child immediate-predecessor histories. No coordinated Run joins source, artifact and deployment Verification Targets or implies cross-target evidence applicability.

## Unit input fingerprints and executed tests

A plan unit derived from the project partition rule ([unit-inputs.md](unit-inputs.md)) carries `input_fingerprint` and `hermetic` in addition to `id` and `parameters`; both belong to the unit's canonical form in the Plan fingerprint. The actual selection repeats each selected unit's `input_fingerprint` exactly, and bundle verification rejects any difference, an omission or an addition. Plan, actual selection and executions of a `whole` or pinned unit have neither field.

Every Check execution unit of a rule unit additionally carries `executed_tests`, sorted by `(file, native_id)`:

```json
"executed_tests": [
  {"native_id": "Ns.Class.Method", "file": "backend/src/Tests/Features/X/Y.cs", "outcome": "passed"}
]
```

`outcome` is `passed | failed | skipped | not-executed`; `file` is the repository-relative normalized path of the source file that defines the test. Every selected test appears, including skipped and not-executed tests, so a silent skip is visible in the Run. Verification rejects a bundle in which an executed test's `file` lies inside the private paths of a unit other than the reporting one. When any listed test is `skipped` or `not-executed`, every Scenario outcome of that unit's attempts is `inconclusive` and a diagnostic names the test; a skipped test contributes no evidence. A `whole` unit of an adapter that cannot report defining files omits `executed_tests`. `executed_tests` belongs to the execution record and is covered by the bundle fingerprint.

