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
    }
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
arrays sort by unique lower-kebab path `id`.

For each invocation, core creates one private staging directory. It opens each configured
executable and resource exactly once, copies and hashes bytes from that same open handle, rejects a
digest mismatch, and makes the completed staged files non-writable. Resources are read-only; the
executable retains only the owner permissions needed to read and execute it. Core invokes only that
staged executable, with no protocol command-line arguments, and passes only staged absolute resource
paths. The staging directory is the child working directory and is removed after the exchange. This
removes a hash-then-open substitution window.

`semantic_settings`, `environment.literals` and all other string maps use unique keys. RFC 8785
orders their keys for fingerprints; source object-key order is immaterial. Values are exact strings
and may be empty. Version 1 supports no secret value, secret reference or interpolation syntax.
Environment names match `[A-Za-z_][A-Za-z0-9_]*`.
Core clears the child environment and supplies only this exact non-secret literal map. Inherited
environment names and values are not supported in version 1.

Semantic settings configure provider translation. They cannot replace the Run-bundle required
context or change what a selected Check or Challenger means. A behavior constraint needed for
evidentiary interpretation belongs in the semantic Plan's exact context.

Every limit is a positive safe integer. `timeout_ms` bounds the complete core exchange lifetime.
`stdout_bytes` and `stderr_bytes` independently bound bytes read from the two streams, including a
final unterminated fragment. One deadline derived from `timeout_ms` bounds request writing,
concurrent response and diagnostic reading, and core's own process wait. Core never waits past that
deadline for a descendant-held pipe to close.

On a supported host, process creation places the adapter in a fresh process group before adapter
code begins. Core signals that group on every terminal path, including success, process failure,
timeout and stream overflow. It cleans group members and their inherited pipes while they retain
group membership. If the host cannot provide the required process-group primitive, invocation
fails before spawn as an exit-one transport failure and creates no process or output bundle. The
host primitive is an implementation fact and does not enter configuration, capability, launch or
Run identity.

This is not non-escapable descendant containment. Authorized adapter code can call facilities such
as `setsid` or `setpgid`, retain ambient filesystem or network access and leave the group. An
escaped descendant cannot make core read or wait beyond the one deadline, but core does not
guarantee that it terminates. Version 1 provides no sandbox, daemon supervision or hostile-code
isolation.

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
        "locator": "/private/staged-invocation/normalization-rules"
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
        "locator": "/private/staged-invocation/normalization-rules"
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
      "locator": "/private/staged-invocation/native-report"
    }
  ],
  "predecessors": [
    {
      "bundle_revision": 0,
      "bundle_fingerprint": "sha256:<predecessor-bundle-fingerprint>"
    }
  ],
  "terminal_predecessor": {}
}
```

`launch_plan` is the complete strict object from [run-launch-plan.md](run-launch-plan.md), and its
operation equals the request operation. `configuration.fingerprint` equals the launch configuration
fingerprint. Semantic settings equal the configured adapter-wide map. Resources equal the complete
configured resource identities and use staged absolute locators. Capabilities sort by unique
address and equal the configured fingerprint and semantic settings for exactly the capabilities
named by one or more launch routes. This gives the process all selected behavior-changing values
through the protocol.

A describe request carries the same configuration object with every configured capability. Its
adapter id and both configuration-fingerprint fields equal the selected entry. This lets the
adapter read pinned resources and validate configured settings during the handshake. An execute or
import request omits the `adapter` field and carries only capabilities used by its launch routes.

Execute requires `inputs: []`. Import requires a non-empty array sorted by unique lower-kebab path
`id`. Core opens each source input once, copies and hashes bytes from that same handle into the
private invocation directory, derives size and digest from that stream and marks the staged file
read-only. Each request locator is the staged absolute regular-file path. The adapter verifies the
bytes it consumes; a mismatch produces failure, never a bundle.

`predecessors` is always present. It is empty for a new Run. Otherwise it contains the full verified
existing correction chain, sorted by contiguous `bundle_revision` from zero, with exactly
`bundle_revision` and `bundle_fingerprint` from each bundle. Core accepts predecessor CLI files in
any order, verifies and deduplicates exact replay through Run-bundle set verification, and rejects
multiple Runs, gaps, forks, changed launch identities or conflicting revisions before invoking the
adapter.

`terminal_predecessor` is always present. It is `null` exactly when `predecessors` is empty.
Otherwise it is the complete strict terminal Run bundle, its revision and fingerprint equal the
last predecessor identity, and it belongs to the verified chain. Core recomputes that bundle
fingerprint and validates the full chain, identity and launch match before spawn. A stateless
adapter uses the terminal bundle to preserve source system, source execution, started time and all
other correction anchors while constructing the next complete revision.

Resource and input locators are transport-only and do not enter request or bundle identity. An
adapter cannot replace the supplied input identity with a native URI, display path or provider
execution id.

Staging, a cleared environment and bounded streams are integrity and process controls, not a
filesystem or network sandbox. A configured adapter is authorized project code and may retain the
ambient operating-system access of the Azimuth process. Projects govern that authority outside
this protocol. Passing a terminal predecessor inline neither dereferences its artifact locators nor
reduces that ambient authority.

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

With no predecessors, the returned bundle has revision zero and forbids `corrects` and
`correction_reason`. With predecessors, it is exactly one complete next revision, increments the
terminal revision by one and names the terminal fingerprint in `corrects`. Core verifies the
combined predecessor-plus-response chain before publishing only the returned bundle. A replay,
skipped revision, changed correction anchor or non-terminal predecessor link is exit one.

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
every literal environment value. There is no inherited environment in version 1.

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
  "inputs": <input-identities>,
  "predecessors": <predecessor-identities>
}
```

`input-identities` contains only `id`, `digest` and `size_bytes`; locator relocation does not
change request identity. `predecessor-identities` contains only `bundle_revision` and
`bundle_fingerprint` in revision order. `terminal_predecessor` is excluded because the final
predecessor fingerprint already commits its complete content; core must recompute and match it
before spawn. The launch fingerprint already binds the configuration fingerprint, so it
transitively binds semantic settings and resource content while transport locators stay out of
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
{"adapter_fingerprint":"sha256:5274f56569ecfbe3cf6a1d8657ff431f78e99b0b97bf2365ea3d6714f950fa2a","capabilities":[{"challenge_forms":[],"classes":["check.execute"],"fingerprint":"sha256:41d224fdbb6fd9c43e067993ff30beb27eb5fc9793c32c9a7701d8678d3a397f","id":"check","semantic_settings":{"mode":"strict"}}],"descriptor_fingerprint":"sha256:f94a0c51a0050bbadfd0d0cb9b34fd6a696f4b7c06246c890b60310bbcb18670","environment":{"literals":{"LANG":"C"}},"format":"azimuth-adapter-configuration-fingerprint","limits":{"stderr_bytes":1024,"stdout_bytes":4096,"timeout_ms":1000},"semantic_settings":{"dialect":"v1"},"version":1}
```

Its SHA-256 value is
`sha256:8b554d29e9bf8cdaee20699d1d10f64493acba3f2d1466c7523c078922c4f6e1`.

The description request preimage is:

```json
{"adapter":{"configuration_fingerprint":"sha256:1111111111111111111111111111111111111111111111111111111111111111","id":"demo"},"format":"azimuth-adapter-request-fingerprint","operation":"describe","version":1}
```

Its SHA-256 value is
`sha256:4247bd475c6d87a35d495dc1b83f0125c2072d0453db1cd6353406603df18edf`.

A new execute request preimage with no inputs or predecessors is:

```json
{"format":"azimuth-adapter-request-fingerprint","inputs":[],"launch_fingerprint":"sha256:9999999999999999999999999999999999999999999999999999999999999999","operation":"execute","predecessors":[],"version":1}
```

Its SHA-256 value is
`sha256:17730bd1fa89859bb3c4562bc305a9316e079e0daa11756f432afa374e9d19f4`.

An import correction request preimage with one input and predecessor is:

```json
{"format":"azimuth-adapter-request-fingerprint","inputs":[{"digest":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","id":"native-report","size_bytes":12}],"launch_fingerprint":"sha256:9999999999999999999999999999999999999999999999999999999999999999","operation":"import","predecessors":[{"bundle_fingerprint":"sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","bundle_revision":0}],"version":1}
```

Its SHA-256 value is
`sha256:d2ca3469eff8a9bea75a43863fe23103a3b6c137b144b48b581772924a79427d`.

## Host validation and exit boundary

Before spawn, core validates configuration shape, canonical order, all fingerprints and locator
containment, then stages and validates content from its open handles. It invokes the staged
executable directly with a cleared environment and configured bounds. After spawn it validates the
single response shape before interpreting it.
`azimuth adapter verify [--config <file>]` performs that exchange for every configured adapter in
sorted order; an empty valid configuration succeeds without spawning a process.

- Exit zero means a valid description or a fully validated bundle, including honest adverse or
  incomplete execution facts.
- Exit one means content, descriptor, capability, model, request, launch, provenance, selection or
  bundle-invariant mismatch, or nonzero exit, timeout, stream overflow, extra response content or
  explicit adapter failure. Inability to establish the required fresh process group fails here
  before spawn.
- Exit two means CLI, local configuration, plan or request schema failure, or malformed or
  schema-invalid adapter response.

No nonzero exit publishes an output bundle. Standard error, exit status and failure messages are
diagnostics only. Timeout, stream overflow or nonzero child exit is classified as exit one before
response parsing. For an on-time zero child exit, extra non-whitespace standard output is exit one;
a single malformed or schema-invalid JSON value is exit two; and a valid `failed` response is exit
one. Execute is never retried automatically after timeout.
