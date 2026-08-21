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
  ]
}
```

Operation is `execute | import`. `planned_at_ms` is the non-negative integral safe Unix-millisecond
time at which core creates the bounded execution plan. Subject has one exact D46 Subject shape.
`required_context` and unit parameters are exact objects from unique non-empty strings to strings;
`{}` is valid. Checks sort by unique project-global Check id and are non-empty. Capability is an
exact configured `<adapter-id>/<capability-id>` address. Units sort by unique lower-kebab path id
and are non-empty. Every number follows the D46 safe-integer boundary.

The planner loads the complete unselected current model before selecting Checks. It derives the
Subject fingerprint, complete-model fingerprint, each current Check fingerprint and its complete
sorted non-empty implementation set. It never accepts caller-supplied fingerprints or
implementations. It rejects unknown Checks, capabilities or classes, duplicate selections,
different adapter prefixes, missing implementations and a capability whose class does not match
the operation.

The planner emits the D46 Plan with the selected Checks and `challenges: []`. It does not require a
current Qualification, compare context to an Evidence Binding or resolve a Challenge Plan. There
is no partial-model or `--only` planning path. A hand-authored strict launch plan may contain
Challenges for transport conformance, but this planner cannot create them.

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
    "id": "recovery-credibility"
  },
  "capability": {
    "address": "synthetic/challenges",
    "class": "challenge.execute",
    "challenge_form": "implementation-perturbation",
    "fingerprint": "sha256:<capability-fingerprint>"
  }
}
```

The Challenge selection id is its D46 plan-local id. `challenge_form` is the producer-accountable
open lower-kebab path form paired with that Challenger fingerprint. It is required on Challenge
routes and forbidden on Check routes. Standalone format validation proves capability coverage for
the declared form, not that the Challenger or form is current. Change F owns that model join.

All addresses start with the launch adapter id followed by `/` and name one configured capability.
For operation `execute`, route class is exactly `check.execute` or `challenge.execute` according to
selection kind. For `import`, it is `check.import` or `challenge.import`. The capability declaration
must contain that class and, for a Challenge, the exact form. Capability fingerprints equal the
configured values. `model.extract` is never a Run route.

Several routes may name one capability, and several capabilities of the one adapter may occur.
One physical activity may later support both Check and Challenge executions, but routes and result
records remain separate. No launch may contain routes from two adapter ids.

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
unchanged configured content does not.

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
D46 correction chain and sends the sorted revision/fingerprint identities in the adapter request.
No predecessor requires revision zero. Otherwise the response is exactly the next full revision,
names the terminal predecessor and validates with the supplied chain. The returned bundle's Run id
includes this launch fingerprint, so a predecessor from another route cannot join the chain.

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
