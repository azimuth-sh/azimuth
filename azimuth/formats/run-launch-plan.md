# Run launch-plan format

A Run launch plan binds one provider-neutral D46 semantic Plan to explicit configured adapter
capabilities. The file is strict JSON with format `azimuth-run-launch-plan` and version `1`.
Unknown fields, duplicate object keys, invalid numbers, non-canonical arrays and duplicate
identities fail. There is no alpha 1 reader.

## Planning request

`azimuth run plan --request <file>` accepts exactly:

```json
{
  "format": "azimuth-run-plan-request",
  "version": 1,
  "operation": "execute",
  "planned_at_ms": 1787300000000,
  "subject": {
    "kind": "workspace",
    "repositories": [
      {
        "id": "root",
        "revision": "8ca4c0f",
        "content_fingerprint": "sha256:<64-lowercase-hex>"
      }
    ]
  },
  "required_context": {
    "platform": "linux-x86_64"
  },
  "checks": [
    {
      "id": "payments/recovery-under-broker-loss",
      "capability": "synthetic/checks",
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
      "id": "payments/recovery-credibility",
      "capability": "synthetic/challenges",
      "max_candidates": 128,
      "units": [
        {
          "id": "whole",
          "parameters": {}
        }
      ]
    }
  ]
}
```

Operation is `execute | import`. `planned_at_ms` is the non-negative integral safe Unix-millisecond
time at which core creates the bounded execution plan. Subject has one exact D46 Subject shape.
`required_context` and unit parameters are exact objects from unique non-empty strings to strings;
`{}` is valid. `checks` and `challenges` are both required arrays; either may be empty, but their
combined selection is non-empty.

Checks sort by unique project-global Check id. Challenge requests sort by unique authored Challenge
Plan id. Both name an exact configured `<adapter-id>/<capability-id>` address and non-empty units
sorted by unique lower-kebab path id. `max_candidates` is required only for Challenge requests and
is an integer from 1 through the D46 safe-integer maximum. It counts unique candidate records from
that authored Plan after duplicate selectors are removed and before selections from different
requested Plans deduplicate. Every disposition counts. Exceeding the cap fails before a launch is
created; it never truncates.

The planner loads the complete unselected current model before selection. It derives the Subject
fingerprint, complete-model fingerprint, each selected current Check and its complete
implementation set, and every candidate, current decision, Challenger, lane and semantic scope
reached by the fixed union of requested Challenge Plans. It never accepts caller-supplied
fingerprints, implementations, Challenger forms, targets, lanes, scopes or launch inputs.

Only a `selected` candidate is runnable. Any other disposition in a requested Plan fails planning.
For every selected decision, the requested Plan union contains at least one runnable selection for
every form required by its Decision Policy. Extra forms strengthen the search. Qualification
context equals `required_context` exactly; selected Qualifications from different contexts fail
with guidance to create separate Runs. Claim Judgment repository identity remains independent of
Run context and uses the one request context for execution.

Core verifies that each explicit capability has the operation's Challenge class and the current
Challenger's exact form. It never chooses a capability lexically or trusts a form in the request.
The one-adapter prefix rule spans Check and Challenge requests. Duplicate selections with different
capabilities or units fail. There is no complete-model broadening, partial-model or `--only` path.

## Launch plan

The complete shape is:

```json
{
  "format": "azimuth-run-launch-plan",
  "version": 1,
  "operation": "execute",
  "planned_at_ms": 1787300000000,
  "subject": {},
  "subject_fingerprint": "sha256:<subject-fingerprint>",
  "plan": {},
  "adapter": {
    "id": "synthetic",
    "adapter_version": "0.1.0-alpha.2",
    "adapter_fingerprint": "sha256:<adapter-fingerprint>",
    "descriptor_fingerprint": "sha256:<descriptor-fingerprint>",
    "configuration_fingerprint": "sha256:<configuration-fingerprint>"
  },
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
  "fingerprint": "sha256:<launch-fingerprint>"
}
```

Operation is `execute | import`. `planned_at_ms` equals the planning request and later equals the
D46 bundle field. Subject and Subject fingerprint obey D46. `plan` is the complete D46 Plan object,
including its supplied and recomputed fingerprint. The Plan is unchanged from D46: it contains no
Subject, adapter, capability, launch or import-input field. Core supplies the separately carried
Subject fingerprint when recomputing the D46 Plan fingerprint.

The adapter id is one lower-kebab segment. Adapter version is the exact non-empty configured
version. Every fingerprint has exact `sha256:<64-lowercase-hex>` shape and equals the selected
configuration entry.

## Routes

There is exactly one route for every Check and Challenge entry in the semantic Plan and no other
route. Routes sort first by selection kind in the fixed order `check`, `challenge`, then by id.
Selection ids are unique within their kind. A Check route has exactly the shape shown above.

A Challenge route is:

```json
{
  "selection": {
    "kind": "challenge",
    "id": "challenge/91f69477f56b9a2ba588fb045529ddbaa7184c79f473eab50f73a0c5f70d038b"
  },
  "capability": {
    "address": "synthetic/challenges",
    "class": "challenge.execute",
    "challenge_form": "implementation-perturbation",
    "fingerprint": "sha256:<capability-fingerprint>"
  },
  "inputs": [
    {
      "kind": "check-implementation",
      "id": "payments|rust-item|recovery::replay-after-loss",
      "fingerprint": "sha256:<source-fingerprint>",
      "source": {
        "kind": "source",
        "file": "tests/recovery.rs",
        "language": "rust",
        "site": "recovery::replay-after-loss"
      }
    },
    {
      "kind": "realization",
      "id": "payments|rust-item|recovery::replay",
      "fingerprint": "sha256:<source-fingerprint>",
      "source": {
        "kind": "source",
        "file": "src/recovery.rs",
        "language": "rust",
        "site": "recovery::replay"
      }
    }
  ]
}
```

The example addresses the exact `a`/`b` selection-identity vector in the D46 format and projects
both source-backed scope items. The Challenge selection id is its D46 plan-local id.
`challenge_form` is the producer-accountable
open lower-kebab path form paired with that Challenger fingerprint. It is required on Challenge
routes and forbidden on Check routes. `inputs` is required on Challenge routes and forbidden on
Check routes. Standalone format validation proves capability coverage for the declared form and
the launch-input shape; generated planning proves their current model authority.

All addresses start with the launch adapter id followed by `/` and name one configured capability.
For operation `execute`, route class is exactly `check.execute` or `challenge.execute` according to
selection kind. For `import`, it is `check.import` or `challenge.import`. The capability declaration
must contain that class and, for a Challenge, the exact form. Capability fingerprints equal the
configured values. `model.extract` is never a Run route.

Several routes may name one capability, and several capabilities of the one adapter may occur.
One physical activity may later support both Check and Challenge executions, but routes and result
records remain separate. No launch may contain routes from two adapter ids.

## Accountable launch inputs

For one Challenge route, `inputs` is the exact sorted projection of every source-backed item in its
semantic scope. The projectable semantic kinds are `check-implementation | realization |
mechanism-implementation | artifact | enumeration | surface-member`. Other scope kinds are already
fully represented by semantic identity and do not acquire repository paths. Every projectable
scope item appears exactly once and no other input appears. Inputs sort and are unique by
`(kind, id, fingerprint)` using the semantic-scope kind order. A conflicting source account is a
launch mismatch. When one item occurs in both scope arrays it projects once; anchors and inputs are
not two launch namespaces.

Every input repeats the scope item's exact `kind`, `id` and `fingerprint`. A realization, Check
implementation or mechanism implementation uses the `source` object shown above. `file` is a
normalized workspace-relative locator, `language` is a lower kebab id and `site` is the exact
extractor-resolved non-empty site. Area, mount and SourceIdentity are not repeated because the
outer semantic id already owns identity.

An artifact input is:

```json
{
  "kind": "artifact",
  "id": "postgres-index:payments.recovery_unique",
  "fingerprint": "sha256:<artifact-property-digest>",
  "source": {
    "kind": "artifact",
    "file": "db/schema.sql",
    "artifact_kind": "postgres-index",
    "identity": "payments|postgres-index|payments.recovery_unique",
    "unique": true,
    "columns": ["recovery_key"],
    "predicate": null
  }
}
```

Columns retain extractor order. Nullable values are explicit. An enumeration input is:

```json
{
  "kind": "enumeration",
  "id": "payments/routes|web|app|next-routes|web|next-manifest|app-paths",
  "fingerprint": "sha256:<enumeration-source-fingerprint>",
  "source": {
    "kind": "enumeration",
    "file": ".next/server/app-paths-manifest.json",
    "enumerator_kind": "next-routes",
    "identity": "web|next-manifest|app-paths"
  }
}
```

A surface-member input is:

```json
{
  "kind": "surface-member",
  "id": "payments/routes|enumerated|app/payments/page.tsx",
  "fingerprint": "sha256:<surface-member-digest>",
  "source": {
    "kind": "surface-member",
    "member_kind": "enumerated",
    "file": "app/payments/page.tsx",
    "language": "typescript",
    "site": "GET /payments"
  }
}
```

`member_kind` is exactly `enumerated`; the tagged variant uses the ordinary `source` shape instead.
The D13 enumerated-member file is both model-authoritative identity inside the outer id and an
accountable locator. The derived language and site are repeated for provider translation but do
not replace that identity. A tagged surface member's outer id contains the stable SourceIdentity.
Artifact and enumeration variants repeat their stable SourceIdentity because their outer ids have
different model-authoritative meanings. Launch inputs contain no provider selectors, globs or
native command fragments.

## Canonical launch fingerprint

Canonical JSON is RFC 8785: UTF-8, no insignificant whitespace, ECMAScript string escaping, UTF-16
object-key ordering and no Unicode normalization. Every set-like array must already satisfy its
declared order and uniqueness. The exact preimage is:

```json
{
  "format": "azimuth-run-launch-fingerprint",
  "version": 1,
  "operation": <operation>,
  "planned_at_ms": <planned-time>,
  "subject": <subject>,
  "subject_fingerprint": <subject-fingerprint>,
  "plan": <complete-plan>,
  "adapter": <complete-adapter-identity>,
  "routes": <complete-routes>
}
```

The plan includes its `fingerprint`; the launch object excludes only its own `fingerprint`. A
change to operation, planned time, Subject, semantic Plan, configured adapter identity, route,
class, Challenge form or capability fingerprint therefore changes launch identity. Relocating
unchanged configured adapter content does not; moving an accountable scope source does.

`complete-routes` includes every Challenge route's complete accountable `inputs` array. There is
no separate locator or launch-input fingerprint: semantic item fingerprints remain unchanged by a
move, while any file, language, site or derived-metadata projection change alters the enclosing
launch fingerprint. Check-only launch vectors therefore remain byte-for-byte unchanged.

### Canonical vector

This complete one-Check launch preimage is already in RFC 8785 form. Its Subject fingerprint is
`sha256:22478698e6731ce5984658e366386e466fe173216bc7cb721168e1638d2dee02`, and its D46 Plan
fingerprint is
`sha256:b75606956b9c1857f8b401d9bad207253b90f6948efddb5532a769b9f488fbfb`.

```json
{"adapter":{"adapter_fingerprint":"sha256:0000000000000000000000000000000000000000000000000000000000000000","adapter_version":"1","configuration_fingerprint":"sha256:1111111111111111111111111111111111111111111111111111111111111111","descriptor_fingerprint":"sha256:2222222222222222222222222222222222222222222222222222222222222222","id":"demo"},"format":"azimuth-run-launch-fingerprint","operation":"execute","plan":{"challenges":[],"checks":[{"fingerprint":"sha256:6666666666666666666666666666666666666666666666666666666666666666","id":"demo/check","implementations":[{"identity":"demo|rust-symbol|demo::check","source_fingerprint":"sha256:7777777777777777777777777777777777777777777777777777777777777777"}],"units":[{"id":"whole","parameters":{}}]}],"fingerprint":"sha256:b75606956b9c1857f8b401d9bad207253b90f6948efddb5532a769b9f488fbfb","model_fingerprint":"sha256:8888888888888888888888888888888888888888888888888888888888888888","required_context":{}},"planned_at_ms":1787300000000,"routes":[{"capability":{"address":"demo/check","class":"check.execute","fingerprint":"sha256:3333333333333333333333333333333333333333333333333333333333333333"},"selection":{"id":"demo/check","kind":"check"}}],"subject":{"artifacts":[{"digest":"sha256:4444444444444444444444444444444444444444444444444444444444444444","id":"image"}],"kind":"artifact"},"subject_fingerprint":"sha256:22478698e6731ce5984658e366386e466fe173216bc7cb721168e1638d2dee02","version":1}
```

Its SHA-256 value is
`sha256:980dc9e544f41414e3a2735e84a6d9733aee85b2961899bb538f1f34c4347237`.

## Adapter request and returned bundle

The launch plan does not contain native import files. `azimuth run import` computes their stable
content identities and supplies them in the adapter request defined by
[adapter.md](adapter.md). The request fingerprint combines the launch fingerprint and those input
identities while excluding their locators.

An adapter response repeats the launch fingerprint. Its normalized bundle repeats the launch
adapter identity, routes and stable import-input identities in D47 provenance. Core compares those
objects exactly, verifies actual selection against the semantic Plan and verifies the complete D46
bundle before atomic output.

Execute and import accept repeated predecessor bundle files in any order. Core verifies their full
D46 correction chain and sends the sorted revision/fingerprint identities plus the complete
verified terminal bundle in the adapter request. With no predecessors it sends a null terminal and
requires revision zero. Otherwise the stateless adapter uses the terminal account to preserve every
anchor, and the response is exactly the next full revision. Core validates it with the supplied
chain. The returned bundle's Run id includes this launch fingerprint, so a predecessor from another
route cannot join the chain.

## Command boundary

```text
azimuth run plan --request <file> [--model <dir>] [--standards <file>] \
  [--workspace <file>] [--manifest <file>...] [--config <file>] [--out <file>]
azimuth run execute --plan <file> [--predecessor <bundle>...] \
  [--config <file>] [--out <file>]
azimuth run import --plan <file> --input <id>=<file>... \
  [--predecessor <bundle>...] [--config <file>] [--out <file>]
```

Configuration defaults to `azimuth/adapters.json`. Execute rejects an import launch and import
rejects an execute launch. Import CLI ids sort uniquely after parsing and equal the adapter request
inputs. `--manifest` and `--predecessor` are repeatable. The four listed model inputs are the whole
planning surface: `--only`, project accounts, worksets and local/federated selection modes are
rejected. Planning and successful provider exchange write JSON only after complete validation
using a temporary sibling and atomic replacement.

Valid adverse or incomplete Run facts exit zero. Semantic, model, content, identity, transport or
bundle-invariant mismatch exits one. CLI, configuration, planning-request, launch-plan or adapter-
response schema failure exits two. Neither nonzero class leaves an output file. `run verify` and
`run inspect` remain standalone protocol commands; `run ingest` remains unknown.

D48 replaces the unpublished Check-only request and Challenge-route shape in place. Both request
arrays are required, and every Challenge route requires exact accountable inputs. Prior requests
without `challenges` and prior Challenge routes without `inputs` are rejected; there is no
compatibility reader.
