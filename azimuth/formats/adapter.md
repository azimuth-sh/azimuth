# Adapter protocol format

Azimuth adapters are explicitly configured short-lived processes. This document defines the
strict configuration, description handshake and request/response protocol for version 1. Unknown
fields, duplicate JSON object keys, invalid numbers and duplicate identities fail.

Every number is a non-negative integral JSON safe integer no greater than `9007199254740991`.
Fields with a positive requirement additionally exclude zero. Fingerprints are
`sha256:<64-lowercase-hex>`. Lower-kebab segments contain lowercase ASCII letters and digits with
interior hyphens. Lower-kebab path ids contain one or more such segments separated by `/`.

## Configuration

The default path is `azimuth/adapters.json`. Its exact root shape is:

```json
{
  "format": "azimuth-adapter-configuration",
  "version": 1,
  "adapters": []
}
```

`adapters` sorts by unique adapter `id`. An empty array is valid. Each adapter has exactly this
shape:

```json
{
  "id": "synthetic",
  "provider_family": "synthetic/provider",
  "protocol_version": 1,
  "adapter_version": "0.1.0-alpha.2",
  "build": "synthetic-build-1",
  "content": {
    "executable": {
      "locator": "adapters/synthetic/adapter",
      "digest": "sha256:<64-lowercase-hex>"
    },
    "resources": [
      {
        "id": "normalization-rules",
        "locator": "adapters/synthetic/rules.json",
        "digest": "sha256:<64-lowercase-hex>"
      }
    ]
  },
  "semantic_settings": {
    "dialect": "synthetic-v1"
  },
  "environment": {
    "literals": {
      "LANG": "C.UTF-8"
    },
    "inherit": ["TMPDIR"]
  },
  "limits": {
    "timeout_ms": 30000,
    "stdout_bytes": 10485760,
    "stderr_bytes": 1048576
  },
  "capabilities": [
    {
      "id": "checks",
      "classes": ["check.execute", "check.import"],
      "challenge_forms": [],
      "semantic_settings": {},
      "fingerprint": "sha256:<64-lowercase-hex>"
    }
  ],
  "adapter_fingerprint": "sha256:<64-lowercase-hex>",
  "descriptor_fingerprint": "sha256:<64-lowercase-hex>",
  "configuration_fingerprint": "sha256:<64-lowercase-hex>"
}
```

Adapter and capability ids are each one lower-kebab segment. The open provider-family identity is
a lower-kebab path id. Protocol version is exactly `1`. Adapter version and build are non-empty
exact strings.

The executable and every resource are regular files. A locator is either host-absolute or a
relative path resolved beneath the directory containing the configuration file. A relative
locator uses `/`, has no empty, `.` or `..` component and cannot escape after symlink-aware
resolution. Core never searches `PATH`, expands a shell expression or invokes a shell. Resource
arrays sort by unique lower-kebab path `id`. Core hashes the executable and every resource before
each spawn and requires the configured digest.

Core invokes the resolved executable with no protocol command-line arguments and uses the resolved
configuration directory as the child working directory. Every resource and import locator sent on
standard input is absolute. A conforming adapter derives behavior from the request rather than the
working-directory locator, which is excluded from identity.

`semantic_settings`, `environment.literals` and all other string maps use unique keys. RFC 8785
orders their keys for fingerprints; source object-key order is immaterial. Values are exact strings
and may be empty. Version 1 supports no secret value, secret reference or interpolation syntax.
Environment names match `[A-Za-z_][A-Za-z0-9_]*`.
`inherit` sorts by unique environment name and is disjoint from `literals`. Core clears the child
environment, then supplies the literal map and the current values of allowlisted inherited names.
An inherited name that is absent in the parent stays absent in the child.

Semantic settings configure provider translation. They cannot replace D46 required context or
change what a selected Check or Challenger means. A behavior constraint needed for evidentiary
interpretation belongs in the semantic Plan's exact context.

Every limit is a positive safe integer. `timeout_ms` bounds the complete child lifetime.
`stdout_bytes` and `stderr_bytes` independently bound bytes read from the two streams, including a
final unterminated fragment. The host drains both streams concurrently.

Capabilities sort lexically by unique `id`. `classes` is non-empty, lexically sorted and unique.
Its closed values are:

- `model.extract`;
- `check.execute`;
- `check.import`;
- `challenge.execute`; and
- `challenge.import`.

`challenge_forms` contains lexically sorted unique open lower-kebab path ids. It is non-empty when
either Challenge class is present and empty otherwise. A capability supporting both Check and
Challenge classes may therefore serve both semantic roles while returning separate result records.
Its address is derived as `<adapter-id>/<capability-id>` and is not stored redundantly.

## Description

Configuration implies one exact expected description. The adapter returns that description on a
`describe` request and repeats it in every execute or import response:

```json
{
  "format": "azimuth-adapter-description",
  "version": 1,
  "protocol_version": 1,
  "id": "synthetic",
  "provider_family": "synthetic/provider",
  "adapter_version": "0.1.0-alpha.2",
  "build": "synthetic-build-1",
  "content": {
    "executable_digest": "sha256:<64-lowercase-hex>",
    "resources": [
      {
        "id": "normalization-rules",
        "digest": "sha256:<64-lowercase-hex>"
      }
    ]
  },
  "adapter_fingerprint": "sha256:<64-lowercase-hex>",
  "capabilities": [
    {
      "id": "checks",
      "classes": ["check.execute", "check.import"],
      "challenge_forms": [],
      "semantic_settings": {},
      "fingerprint": "sha256:<64-lowercase-hex>"
    }
  ],
  "descriptor_fingerprint": "sha256:<64-lowercase-hex>"
}
```

Description content omits locators. Resource and capability arrays obey configuration ordering
and cardinality. All shared scalar, content, capability and fingerprint fields must equal the
configured values exactly. Relocating identical content therefore changes neither the description
nor any semantic fingerprint.

## Requests

Core writes one complete request to standard input and closes it. A description request is:

```json
{
  "format": "azimuth-adapter-request",
  "version": 1,
  "request_id": "sha256:<request-fingerprint>",
  "operation": "describe",
  "adapter": {
    "id": "synthetic",
    "configuration_fingerprint": "sha256:<configuration-fingerprint>"
  },
  "configuration": {
    "fingerprint": "sha256:<configuration-fingerprint>",
    "semantic_settings": {
      "dialect": "synthetic-v1"
    },
    "resources": [
      {
        "id": "normalization-rules",
        "digest": "sha256:<64-lowercase-hex>",
        "locator": "/workspace/experiments/adapter-capabilities/rules.json"
      }
    ],
    "capabilities": [
      {
        "address": "synthetic/checks",
        "fingerprint": "sha256:<capability-fingerprint>",
        "semantic_settings": {}
      }
    ]
  }
}
```

An execute or import request is:

```json
{
  "format": "azimuth-adapter-request",
  "version": 1,
  "request_id": "sha256:<request-fingerprint>",
  "operation": "import",
  "launch_plan": {},
  "configuration": {
    "fingerprint": "sha256:<configuration-fingerprint>",
    "semantic_settings": {
      "dialect": "synthetic-v1"
    },
    "resources": [
      {
        "id": "normalization-rules",
        "digest": "sha256:<64-lowercase-hex>",
        "locator": "/workspace/experiments/adapter-capabilities/rules.json"
      }
    ],
    "capabilities": [
      {
        "address": "synthetic/checks",
        "fingerprint": "sha256:<capability-fingerprint>",
        "semantic_settings": {}
      }
    ]
  },
  "inputs": [
    {
      "id": "native-report",
      "digest": "sha256:<64-lowercase-hex>",
      "size_bytes": 18423,
      "locator": "/private/tmp/report.json"
    }
  ]
}
```

`launch_plan` is the complete strict object from [run-launch-plan.md](run-launch-plan.md), and its
operation equals the request operation. `configuration.fingerprint` equals the launch configuration
fingerprint. Semantic settings equal the configured adapter-wide map. Resources equal the complete
configured resource identities and use host-absolute resolved locators. Capabilities sort by unique
address and equal the configured fingerprint and semantic settings for exactly the capabilities
named by one or more launch routes. This gives the process all selected behavior-changing values
without granting access to the configuration file.

A describe request carries the same configuration object with every configured capability. Its
adapter id and both configuration-fingerprint fields equal the selected entry. This lets the
adapter read pinned resources and validate configured settings during the handshake. An execute or
import request omits the `adapter` field and carries only capabilities used by its launch routes.

Execute requires `inputs: []`. Import requires a non-empty array sorted by unique lower-kebab path
`id`. Each input locator is a host-absolute regular-file path. Core computes digest and size before
launch. The adapter verifies that the bytes it consumes have that exact digest and size; a
mismatched or changing file produces failure, never a bundle.

Resource and input locators are transport-only and do not enter request or bundle identity. An
adapter cannot replace the supplied input identity with a native URI, display path or provider
execution id.

## Responses

The adapter writes exactly one JSON response to standard output and then closes the stream. Leading
or trailing JSON whitespace is permitted; any second value or non-whitespace content is a
transport failure. A successful description response is:

```json
{
  "format": "azimuth-adapter-response",
  "version": 1,
  "request_id": "sha256:<request-fingerprint>",
  "operation": "describe",
  "status": "ok",
  "description": {}
}
```

A successful execute or import response is:

```json
{
  "format": "azimuth-adapter-response",
  "version": 1,
  "request_id": "sha256:<request-fingerprint>",
  "operation": "import",
  "status": "ok",
  "description": {},
  "launch_fingerprint": "sha256:<launch-fingerprint>",
  "bundle": {}
}
```

The description and bundle are complete strict objects, not references. Request id, operation and
launch fingerprint equal the request. The description equals configured description. The bundle
obeys [run-bundle.md](run-bundle.md), including exact returned adapter provenance.

An adapter that cannot complete the exchange may return:

```json
{
  "format": "azimuth-adapter-response",
  "version": 1,
  "request_id": "sha256:<request-fingerprint>",
  "operation": "import",
  "status": "failed",
  "description": {},
  "launch_fingerprint": "sha256:<launch-fingerprint>",
  "failure": {
    "code": "native-report/unreadable",
    "message": "The native report could not be read.",
    "details": {}
  }
}
```

For `describe`, `launch_fingerprint` is absent. For execute and import it is required. Failure code
is an open lower-kebab path id, message is non-empty and details are an exact string map. A failed
response contains no bundle. It is a transport failure rather than an Observation or Challenge
Result. No response field requests or authorizes automatic retry.

## Canonical fingerprints

Canonical JSON is RFC 8785: UTF-8, no insignificant whitespace, ECMAScript string escaping, UTF-16
object-key ordering and no Unicode normalization. Set-like arrays must already be in their declared
order and contain unique identities; hashing never repairs invalid input.

The adapter fingerprint preimage is:

```json
{
  "format": "azimuth-adapter-fingerprint",
  "version": 1,
  "protocol_version": 1,
  "id": <adapter-id>,
  "provider_family": <provider-family>,
  "adapter_version": <adapter-version>,
  "build": <build>,
  "content": {
    "executable_digest": <digest>,
    "resources": <resource-identities>
  }
}
```

`resource-identities` contains only `id` and `digest`. The capability fingerprint preimage is:

```json
{
  "format": "azimuth-adapter-capability-fingerprint",
  "version": 1,
  "adapter_fingerprint": <adapter-fingerprint>,
  "id": <capability-id>,
  "classes": <classes>,
  "challenge_forms": <challenge-forms>,
  "semantic_settings": <capability-settings>
}
```

The descriptor fingerprint preimage is:

```json
{
  "format": "azimuth-adapter-descriptor-fingerprint",
  "version": 1,
  "descriptor": <complete-description-without-descriptor-fingerprint>
}
```

The configuration fingerprint preimage is:

```json
{
  "format": "azimuth-adapter-configuration-fingerprint",
  "version": 1,
  "adapter_fingerprint": <adapter-fingerprint>,
  "descriptor_fingerprint": <descriptor-fingerprint>,
  "semantic_settings": <adapter-settings>,
  "environment": <complete-environment-object>,
  "limits": <complete-limits-object>,
  "capabilities": <complete-capability-array>
}
```

The description in its fingerprint preimage includes `adapter_fingerprint` and complete
capabilities with their fingerprints. The configuration preimage excludes executable and resource
locators but includes their content transitively through the adapter fingerprint. It includes
literal environment values and inherited names, but no inherited runtime value.

Request fingerprints use one of these exact preimages:

```json
{
  "format": "azimuth-adapter-request-fingerprint",
  "version": 1,
  "operation": "describe",
  "adapter": {
    "id": <adapter-id>,
    "configuration_fingerprint": <configuration-fingerprint>
  }
}
{
  "format": "azimuth-adapter-request-fingerprint",
  "version": 1,
  "operation": <execute-or-import>,
  "launch_fingerprint": <launch-fingerprint>,
  "inputs": <input-identities>
}
```

`input-identities` contains only `id`, `digest` and `size_bytes`; locator relocation does not
change request identity. The launch fingerprint already binds the configuration fingerprint, so
it transitively binds semantic settings and resource content while transport locators stay out of
identity. Every supplied fingerprint equals the recomputed value.

### Canonical vector

The following independent adapter preimage is already in RFC 8785 form:

```json
{"adapter_version":"1","build":"b1","content":{"executable_digest":"sha256:0000000000000000000000000000000000000000000000000000000000000000","resources":[]},"format":"azimuth-adapter-fingerprint","id":"demo","protocol_version":1,"provider_family":"synthetic/demo","version":1}
```

Its SHA-256 value is
`sha256:5274f56569ecfbe3cf6a1d8657ff431f78e99b0b97bf2365ea3d6714f950fa2a`.

Using that adapter fingerprint, the capability preimage is:

```json
{"adapter_fingerprint":"sha256:5274f56569ecfbe3cf6a1d8657ff431f78e99b0b97bf2365ea3d6714f950fa2a","challenge_forms":[],"classes":["check.execute"],"format":"azimuth-adapter-capability-fingerprint","id":"check","semantic_settings":{"mode":"strict"},"version":1}
```

Its SHA-256 value is
`sha256:41d224fdbb6fd9c43e067993ff30beb27eb5fc9793c32c9a7701d8678d3a397f`.

The corresponding descriptor fingerprint preimage is:

```json
{"descriptor":{"adapter_fingerprint":"sha256:5274f56569ecfbe3cf6a1d8657ff431f78e99b0b97bf2365ea3d6714f950fa2a","adapter_version":"1","build":"b1","capabilities":[{"challenge_forms":[],"classes":["check.execute"],"fingerprint":"sha256:41d224fdbb6fd9c43e067993ff30beb27eb5fc9793c32c9a7701d8678d3a397f","id":"check","semantic_settings":{"mode":"strict"}}],"content":{"executable_digest":"sha256:0000000000000000000000000000000000000000000000000000000000000000","resources":[]},"format":"azimuth-adapter-description","id":"demo","protocol_version":1,"provider_family":"synthetic/demo","version":1},"format":"azimuth-adapter-descriptor-fingerprint","version":1}
```

Its SHA-256 value is
`sha256:f94a0c51a0050bbadfd0d0cb9b34fd6a696f4b7c06246c890b60310bbcb18670`.

The corresponding configuration fingerprint preimage is:

```json
{"adapter_fingerprint":"sha256:5274f56569ecfbe3cf6a1d8657ff431f78e99b0b97bf2365ea3d6714f950fa2a","capabilities":[{"challenge_forms":[],"classes":["check.execute"],"fingerprint":"sha256:41d224fdbb6fd9c43e067993ff30beb27eb5fc9793c32c9a7701d8678d3a397f","id":"check","semantic_settings":{"mode":"strict"}}],"descriptor_fingerprint":"sha256:f94a0c51a0050bbadfd0d0cb9b34fd6a696f4b7c06246c890b60310bbcb18670","environment":{"inherit":["TMPDIR"],"literals":{"LANG":"C"}},"format":"azimuth-adapter-configuration-fingerprint","limits":{"stderr_bytes":1024,"stdout_bytes":4096,"timeout_ms":1000},"semantic_settings":{"dialect":"v1"},"version":1}
```

Its SHA-256 value is
`sha256:28da1f9c4f3262e5fed07c5c0667c6df340bf45351bd0c03ccc037e8a6584275`.

The description request preimage is:

```json
{"adapter":{"configuration_fingerprint":"sha256:1111111111111111111111111111111111111111111111111111111111111111","id":"demo"},"format":"azimuth-adapter-request-fingerprint","operation":"describe","version":1}
```

Its SHA-256 value is
`sha256:4247bd475c6d87a35d495dc1b83f0125c2072d0453db1cd6353406603df18edf`.

## Host validation and exit boundary

Before spawn, core validates configuration shape, canonical order, all fingerprints, locator
containment and content digests. It invokes the executable directly with a cleared environment and
the configured bounds. After spawn it validates the single response shape before interpreting it.
`azimuth adapter verify [--config <file>]` performs that exchange for every configured adapter in
sorted order; an empty valid configuration succeeds without spawning a process.

- Exit zero means a valid description or a fully validated bundle, including honest adverse or
  incomplete execution facts.
- Exit one means content, descriptor, capability, model, request, launch, provenance, selection or
  bundle-invariant mismatch, or nonzero exit, timeout, stream overflow, extra response content or
  explicit adapter failure.
- Exit two means CLI, local configuration, plan or request schema failure, or malformed or
  schema-invalid adapter response.

No nonzero exit publishes an output bundle. Standard error, exit status and failure messages are
diagnostics only. Timeout, stream overflow or nonzero child exit is classified as exit one before
response parsing. For an on-time zero child exit, extra non-whitespace standard output is exit one;
a single malformed or schema-invalid JSON value is exit two; and a valid `failed` response is exit
one. Execute is never retried automatically after timeout.
