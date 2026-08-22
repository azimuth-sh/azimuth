# Repository manifest format

`azimuth-repository-manifest` is the strict JSON a language extractor emits and core reads. It is
the only surface between an ecosystem and core: core learns no language syntax, and an extractor
learns no model rules. Every `--manifest <file>` argument names one such document.

This contract describes the document as the reader accepts it. Anything not described here is a
parse error.

## JSON dialect

The document is one JSON object encoded in UTF-8. The parser accepts objects, arrays, strings,
numbers, `true`, `false` and `null`, and nothing else. Comments, trailing commas and any trailing
content after the top-level value are errors. A malformed document is reported as
`malformed manifest: line <n>: <what was expected>` against the manifest path, and no record in it
is read.

Object keys keep their emitted order. Order is never identity: no collection is required to be
sorted, and core derives nothing from position. Producers that sort do so for diffability only.

## Root envelope

The root is an object whose keys are drawn from exactly these six collections:

```json
{
  "realizes": [],
  "check_implementations": [],
  "mechanism_implementations": [],
  "class_members": [],
  "enumerations": [],
  "artifacts": []
}
```

There is no `format` or `version` key. The document carries no envelope identity of its own; a
`format`, `version` or any other extra top-level key is an unknown key and fails.

Every collection is optional and every present collection may be empty, but at least one of the six
must be present. A root that is not an object, a root that declares none of the six, a repeated
top-level key and a collection whose value is not an array are each errors.

Three alpha-era collections are retired and rejected by name:

```text
covers
mechanism_covers
observations
```

Each produces `legacy manifest key <key> is not supported; use Check implementations and repository
verification declarations`.

## Common field rules

Records are objects. A non-object element of a collection is an error, as is an unknown field name
or a repeated field name inside a record.

A required field must be present and must be a non-empty string. An optional field may be omitted
entirely; when present it must have the declared type. An explicit `null` is never a valid value —
it is a type error, not an omission.

A fingerprint is exactly `sha256:` followed by 64 lowercase hexadecimal digits. No other length,
case or algorithm prefix is accepted.

An identifier validated as an id is lowercase kebab-case: each segment contains only ASCII
lowercase letters, digits and interior hyphens, and no segment is empty or starts or ends with `-`.
A path id may contain `/` between segments; a segment id may not.

### Emitted source identity

Any record may carry a pre-assembled source identity as four sibling fields:

```json
{
  "area": "payments",
  "address_kind": "rust-symbol",
  "address": "cargo:lib:pay:pay::Capture::complete fn(&self)->bool",
  "mount": "code"
}
```

All four are required together. Any proper non-empty subset is a partial source identity and fails.
All four are omitted in the ordinary case, and an extractor that cannot resolve area and mount from
its own inputs must omit them rather than guess.

An emitted source identity does not survive assembly except on an Artifact. Both local checking and
federated workset assembly derive `area`, `address_kind`, `address` and `mount` from the source
locator and the declared area mounts, and overwrite whatever the manifest carried, for `realizes`,
`check_implementations`, `mechanism_implementations`, `class_members` and `enumerations`. An
Artifact's emitted source identity is kept and is derived only when absent. The derivation is
described under [Assembly](#assembly).

## `realizes`

A production realization site for one case-level Claim.

```json
{
  "spec": "payments/capture",
  "scenario": "duplicate-completion-is-idempotent",
  "site": "capture::complete",
  "file": "src/capture.rs",
  "lang": "rust",
  "source_fingerprint": "sha256:<64-lowercase-hex>"
}
```

`spec`, `scenario`, `site`, `file` and `lang` are required. `source_fingerprint` is optional; when
present it must be a fingerprint. The four source-identity fields are permitted. No other field is.

Neither `spec` nor `scenario` is id-validated by the reader; both are matched against the model,
where an unknown pair becomes a dangling-linkage finding rather than a parse error. `lang` is
accepted as written and selects the address kind (see [Address kinds](#address-kinds)).

Within one manifest the identity is `spec|scenario|site|file|lang`, and a repeat is a duplicate
realization.

## `check_implementations`

One source site that implements one project-global Check. The marker carries implementation
identity only: no Claim, form, context or Qualification. Several records may implement one Check.

```json
{
  "check": "payments/duplicate-completion",
  "site": "capture_tests::duplicate_completion",
  "file": "tests/capture.rs",
  "lang": "rust",
  "source_fingerprint": "sha256:<64-lowercase-hex>"
}
```

All five fields are required, including `source_fingerprint`. The four source-identity fields are
permitted. No other field is.

`check` is a lower-kebab path id. Within one manifest the identity is `check` plus the emitted
source key `area|address_kind|address` when a source identity is present, and
`check|file|site|lang` when it is not.

## `mechanism_implementations`

One compiler-resolved site that implements one design-owned mechanism identity. The record, its
binding and its companion Artifact are one atomic account: an extractor emits all three or none.

```json
{
  "spec": "payments/capture",
  "mechanism": "completion-guard",
  "site": "cargo:lib:pay:pay::Capture::complete fn(&self)->bool",
  "binding": "rust-symbol:cargo:lib:pay:pay::Capture::complete fn(&self)->bool",
  "file": "src/capture.rs",
  "lang": "rust",
  "source_fingerprint": "sha256:<64-lowercase-hex>"
}
```

All seven fields are required. The four source-identity fields are permitted. No other field is. A
record without `site` is the retired alpha-era shape and fails.

`spec` is a lower-kebab path id and `mechanism` is one lower-kebab segment; both are id-validated
here, unlike the ids in `realizes`. `file` must be a non-empty normalized workspace-relative path:
not absolute, containing no `.`, `..` or empty component and no backslash.

`site` is one trimmed, non-empty, path-free semantic identity. It contains no leading or trailing
whitespace, no control character and no `|`. Core does not parse it further: qualification is the
extractor's obligation, and `contracts/verification.md` fixes each ecosystem's exact site profile.

`lang` must be a supported mechanism language. The mapping to address kind is closed:

| `lang` | address kind |
|---|---|
| `csharp` | `dotnet-symbol` |
| `cpp`, `go`, `java`, `javascript`, `kotlin`, `python`, `rust`, `typescript` | `<lang>-symbol` |

Any other `lang` is an unsupported language and the record fails.

### Binding and companion

With no emitted source identity, `binding` is exactly `<address-kind>:<site>` and `site` contains
no `#`. The raw `file` locator never participates in the binding. Untyped, mismatched and
`<kind>:<file>#<site>` bindings fail.

With an emitted source identity, `address_kind` must equal the address kind implied by `lang`,
`address` must equal `site`, and `binding` must equal the source key `area|address_kind|address`.

Either way the record requires exactly one companion Artifact in the same manifest whose `id`
equals that same binding value, whose `kind` equals the address kind, whose `file` equals the
implementation's `file`, and whose source identity is present or absent exactly as the
implementation's is and equal to it when present. Zero matches, several matches, or one companion
claimed by two implementations each fail.

Within one manifest, `(spec, mechanism)` is unique: two marker implementations of one mechanism
fail. The full duplicate identity is `spec|mechanism|site|binding|file|lang`.

## `class_members`

A member of a class enumerated from what the build produced — a route table, a container, a
migration set — rather than from a marker. Identity is the **file**: the member is the file, and a
discharge anywhere in it discharges the member.

```json
{
  "class": "trips/rider-view",
  "site": "/trips/[id]",
  "file": "app/web/rider/src/app/trips/[id]/page.tsx",
  "lang": "typescript"
}
```

`class`, `site`, `file` and `lang` are required. The four source-identity fields are permitted. No
other field is — in particular this record has no `source_fingerprint`; the enumeration witness
carries the fingerprint for the whole contribution.

Within one manifest the identity is `class|site|file|lang`.

## `enumerations`

The witness that a class was enumerated from a system-produced source rather than reconstructed
from the declarations whose omissions the enumeration exists to find. Without a witness, tag-derived
membership is incomplete and a site-domain Claim reports
`enumerator-unsound-or-underived` rather than green.

```json
{
  "class": "trips/rider-view",
  "kind": "next-routes",
  "source": "app/web/rider/.next/app-path-routes-manifest.json",
  "source_fingerprint": "sha256:<64-lowercase-hex>"
}
```

All four fields are required. The four source-identity fields are permitted. No other field is —
this record has no `file`; `source` is the locator of the enumerated system source, and it is the
locator whose area and mount are resolved.

`kind` names the enumerator and must equal the `enumerator` of the workspace surface contribution
it witnesses. Within one manifest the identity is `class|kind|source`.

## `artifacts`

A machine-addressable artifact emitted from a compiler or schema model. Optional properties carry
only facts the extractor derived; semantic claims stay in design prose.

```json
{
  "id": "postgres-index:payments.ix_payments_idempotency",
  "kind": "database-index",
  "file": "src/Migrations/20250101_AddIdempotency.cs",
  "unique": true,
  "columns": ["tenant_id", "idempotency_key"],
  "predicate": "state = 'completed'"
}
```

`id`, `kind` and `file` are required. `unique` is an optional boolean, `columns` an optional array
of strings and `predicate` an optional string. The four source-identity fields are permitted. No
other field is. An omitted optional property becomes `null`, `[]` and `null` in the canonical
Artifact account; it is not written as `null` in the manifest.

Within one manifest the identity is the triple `(id, kind, file)`.

An Artifact is either a marker companion — claimed by exactly one `mechanism_implementations`
record as described above — or an ordinary Artifact. An ordinary Artifact may not reuse the id of a
companion, and may not use a reserved raw marker id `<address-kind>:<site>` for any mechanism site
in the manifest. Ordinary artifact ids are unique among themselves. Only ordinary Artifacts may be
named by an explicit `Binding:` in a design; an explicit binding to a companion's raw id or its
assembled key is rejected.

## Address kinds

For records other than `mechanism_implementations`, core derives the address kind and address from
`lang`, `file` and `site` rather than from a declared field:

For `realizes` and `check_implementations` the first matching rule applies:

- `lang` is `csharp`: kind `dotnet-symbol`, address `site`;
- `lang` is `prometheus` and `file` ends `.rules.test.yml`: kind `prometheus-rule-test`, address
  `site`;
- `lang` is `prometheus`: kind `prometheus-alert`, address `site`;
- `lang` is `typescript`, `file` contains `/app/` and ends `/route.ts`, and `site` is one of `GET`,
  `POST`, `PUT`, `PATCH` or `DELETE`: kind `next-route`, address `<site> /<route>`, where `<route>`
  is the portion of `file` after the first `/app/` with a trailing `/route.ts` removed. Backslashes
  in `file` are normalized to `/` before both the match and the split, so a Windows-emitted locator
  derives the same address;
- otherwise: kind `<lang>-symbol`, address `site`.

For the remaining collections the kind and address are fixed:

| record | address kind | address |
|---|---|---|
| `class_members` | `class-member` | `<class>#<site>` |
| `enumerations` | `enumerator` | `<class>#<kind>` |
| `artifacts` | the Artifact's `kind` | the Artifact's `id` |

Unlike `mechanism_implementations`, these records accept any `lang` string: an unrecognized
language yields `<lang>-symbol` rather than an error.

## Assembly

Core resolves each record's locator — `file`, or `source` for an enumeration — against the declared
area mounts and takes the longest containing mount. Locally the mounts come from
`azimuth/workspace.json`; in a federated workset they come from the project catalog's areas for the
emitting repository. Local and federated assembly use the same rewrite and produce identical
semantic ids. Neither uses file, mount, repository or revision as a semantic disambiguator.

A locator matching no mount yields no source identity locally, and is an error in a federated
workset, as is a locator matching two mounts of equal path length. A `mechanism_implementations`
record whose locator matches no mount is an error in both.

For a mechanism implementation, assembly atomically rewrites the implementation's `binding` and its
companion Artifact's `id` to the source key `area|address_kind|address`, and sets both source
identities. The companion is the one Artifact identity that is not further expanded: its assembled
id is already the source key. Every other Artifact retains its authored kind and id as semantic
input.

## Merged manifests

Several manifests are read together. Duplicate detection inside one manifest is by the raw identity
tuples above; after assembly the following must additionally hold across all manifests:

- one realization per `(spec#scenario, source key)`;
- one Check implementation per `(check, source key)`;
- one marker implementation per `(spec, mechanism)`, and one marker target per mechanism source
  key;
- one enumeration witness per `(class, area, mount, kind)`;
- one class member per `(class, file)`; and
- one Artifact per `id`, with no ordinary Artifact colliding with a reserved raw marker id.

The same qualified site in two different areas is legal and produces different assembled binding and
Artifact ids. A locator-only move within one area that preserves the qualified site and the content
preserves source identity, Claim Judgment and semantic Challenge scope; only the locator
fingerprints change.

## Diagnostics

A manifest diagnostic names the manifest path and, for a record-level failure, the record position
as `<collection>[<index>]`, together with what was expected. Assembly diagnostics name the source
locator or the producer instead, because by then the record no longer belongs to one input file.

## Related contracts

- `contracts/markers.md` — the source annotations an extractor reads to produce these records.
- `contracts/verification.md` — the exact per-ecosystem semantic-site profile each extractor must
  satisfy, and the Check linkage rules.
- `contracts/workspace.md` — areas, mounts, surfaces and contributions used by assembly.
- `contracts/design.md` — explicit mechanism `Binding:` declarations that resolve against Artifacts.
