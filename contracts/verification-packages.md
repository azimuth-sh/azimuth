# Verification package contracts

Packages extend the authoring and source contracts of a Check. They do not introduce another Check identity or establish qualification, applicability, execution or a Claim Judgment.

## Enablement and identity

The project catalog and local workspace accept `packages: ["azimuth.surface", "azimuth.network"]`. Enablement is explicit. Unknown packages, duplicate enablement and use of a disabled package are diagnostics. Federation preserves the project package selection. Legacy workspace surfaces remain distinct from the surface package.

Package-owned entity kinds and source contracts have canonical package identities. Authored entity IDs are stable lower-kebab names. Core Claim, Case, Mechanism and Check IDs retain their project-wide typed identities; moving a declaration changes membership only.

## Compact authoring

Shared declarations are outside Claim verification scopes:

```markdown
## Shared inputs and probes

### Surface: operations

- Fields:
  - `Key`
  - `Transport`
- Area: `api`

Describe the independently discovered population and completeness boundary.

### Expectation set: admission

- Surface: `operations`
- Fields:
  - `Key`
  - `Allowed`

Describe the independently authored expectations and their constraints.

### Probe: connectivity

- Area: `deployment`
- Origins:
  - `external`

Describe measurements, origin provenance, controls and incomplete outcomes.
```

Surface and Expectation Set field lists publish names without duplicating language types. A Surface has an Area; an Expectation Set selects one Surface. A Probe has an Area and nonempty origin names. Ordinary captions are prose, with no semantic identity.

A Check retains its own method prose and may declare `Surface`, `Expectations`, `Select`, `Probe`, `Origin` and `Mechanisms`. Selectors name `Member.<field>` or `Expected.<field>`. Equality selects a supported scalar value; `includes <value>` selects a supported member of a collection, including nested map value collections. Unknown fields and incompatible values fail validation against the producer descriptor. A selector neither proves discovery completeness nor supplies an outcome oracle.

```markdown
### Check: operations-have-expected-admission

- Surface: `operations`
- Expectations: `admission`
- Evidence bindings:
  - Case: `designated-caller-is-admitted`
    - Contribution: Establishes equality of declared admission; runtime enforcement needs separate evidence.

Compare independently enumerated registrations with the expected records, preserving omissions and unexpected members.
```

An inline binding owns a Case reference and nonempty Contribution. Inside a Case verification, omitting Case selects that enclosing Case. Outside a Case, Case is explicit. The typed `(Check, Case)` pair identifies the relation; no authored binding ID, Context, Decision Policy or Method Qualification field is required in this compact account. Source tags supply implementation identity only. They cannot generate the Case contribution or any approved decision. Separate review records remain separate authority.

## Source producer records

The strict extraction manifest accepts an `extensions` collection:

```json
{
  "extensions": [{
    "package": "azimuth.surface",
    "contract": "surface-enumerator",
    "entity": "operations",
    "site": "Example.Api.OperationEnumeration.Enumerate(...) -> OperationMember[]",
    "file": "src/OperationEnumeration.cs",
    "lang": "csharp",
    "source_fingerprint": "sha256:<64 lowercase hex>",
    "schema": {"fields": [
      {"name": "Key", "schema": {"kind": "string"}},
      {"name": "Transport", "schema": {"kind": "enum", "values": ["http", "signalr"]}}
    ]}
  }]
}
```

Supported contracts are `azimuth.surface/surface-enumerator`, `azimuth.surface/surface-expectations` and `azimuth.network/probe-implementation`. Surface producers require an output schema; Probe implementations have none. Source records retain exact semantic identity and fingerprint; optional source identity fields follow the core manifest contract. A locator cannot resolve an ambiguous semantic site. Package records resolve to declared entities and enabled packages.

Schema fields have unique nonempty `name` and `schema`. Schema kinds are `string`, `boolean`, `number`, `enum` with unique supported `values`, `list` with `items`, `map` with `keys` and `values`, `record` with `fields`, and `nullable` with `value`. Unknown fields and unsupported types fail. Types and wire enum values derive from the language producer contract. A descriptor establishes the declared data shape, not successful enumeration or correct application policy.

The .NET package namespaces are `Azimuth.Surface.Annotations` and `Azimuth.Network.Annotations`. `SurfaceEnumerator(surface)`, `SurfaceExpectations(expectationSet)` and `ImplementsProbe(probe)` identify package-owned source relations. Emitters resolve canonical attribute types, not arbitrary attributes with the same short name. Producer invocation remains explicit; extracting a descriptor does not run application code. Endpoints do not repeat facts already present in registration.

## Execution boundary

Enumeration and expectation results must retain the exact Subject, producer identity, members, completeness and accountable membership fingerprint. Missing discovery cannot appear as a successful empty population. Every actual member must join one expected record before Check selection; duplicate and absent expectations, obsolete expected keys and unclassified registrations remain findings. Expectations cannot be generated from the policy under examination.

Probe reports retain actual origins, target identities, replicas, controls, timestamps and measurement failures. A configured origin label alone proves no origin. A bounded adapter interprets native reports under the selected Check, preserves violated and inconclusive outcomes, and produces the existing Run protocol. Package validation and successful extraction do not establish a product proposition.
