# Extractors

Each language extractor finds structural Azimuth markers in its own ecosystem and emits the same
language-neutral linkage manifest. Core reads manifests and learns no language-specific syntax.

## Strict v2 manifest

The manifest has exactly six collections:

```json
{
  "realizes": [
    {
      "spec": "payments/capture",
      "scenario": "duplicate-completion-is-idempotent",
      "site": "capture::complete",
      "file": "src/capture.rs",
      "lang": "rust",
      "source_fingerprint": "sha256:<64-lowercase-hex>"
    }
  ],
  "check_implementations": [
    {
      "check": "payments/duplicate-completion",
      "site": "capture_tests::duplicate_completion",
      "file": "tests/capture.rs",
      "lang": "rust",
      "source_fingerprint": "sha256:<64-lowercase-hex>"
    }
  ],
  "mechanism_implementations": [
    {
      "spec": "payments/capture",
      "mechanism": "completion-guard",
      "site": "payments::capture::Capture::complete",
      "binding": "rust-symbol:payments::capture::Capture::complete",
      "file": "src/capture.rs",
      "lang": "rust",
      "source_fingerprint": "sha256:<64-lowercase-hex>"
    }
  ],
  "class_members": [],
  "enumerations": [],
  "artifacts": [
    {
      "id": "rust-symbol:payments::capture::Capture::complete",
      "kind": "rust-symbol",
      "file": "src/capture.rs"
    }
  ]
}
```

Every `source_fingerprint` is SHA-256 in the exact lexical form shown. Extractors fingerprint the
smallest trustworthy enclosing semantic site. Ecosystems without a stable source span may use a
complete-file boundary, but they may not invent a short or provider-specific fingerprint.

`realizes` links production implementation to a case-level Claim and carries no verification form.
`check_implementations` links source to one project-global Check id and carries no Claim, form,
context or Qualification. Workspace or project assembly attaches area, mount and semantic source
address. Several implementation records may compose one Check.

`mechanism_implementations` links source to a declared design mechanism. Its seven raw fields are
all required: `spec` is a lower-kebab path id and `mechanism` is one lower-kebab segment. The old
record without `site` is invalid. The extractor derives a compiler-semantic qualified site, exact
typed binding and companion Artifact as one atomic account. Enumeration witnesses and class
members establish complete site domains independently of linkage markers. Artifacts provide exact
structural binding targets.

For the marker-derived pair:

- `site` includes module or package, declaring type or receiver and the overload signature or
  generic arity wherever the compiler exposes them;
- `binding`, split at its first `:`, is exactly `<address-kind>:<site>`; the raw `file` locator does
  not participate;
- the raw companion Artifact id equals `binding`, its kind equals the binding prefix and its file
  equals the implementation file; and
- assembly rewrites both binding and companion id to `<area>|<address-kind>|<site>`.

The marker address-kind mapping is `csharp` to `dotnet-symbol`; `cpp | go | java | javascript |
kotlin | python | rust | typescript` map to `<lang>-symbol`. Untyped, mismatched and
`<kind>:<file>#<site>` bindings fail. A semantic language package or module name may use its native
separators, including `/`; path-free means that the workspace-relative `file` is not used as or
appended to the qualified site.

The raw companion requires `id`, `kind` and `file` and permits only optional typed `unique`,
`columns` and `predicate`. It is paired by `(id, kind, file)`. A repeated or ambiguous triple fails;
the same raw id in different files survives until area assembly only when every collision is an
exact marker companion. An unmatched or ordinary Artifact collision fails. Assembly resolves the
file to one area and mount, derives the SourceIdentity key and atomically rewrites both
implementation binding and companion id to that key before model identity, resolution or
fingerprinting. Optional properties are preserved; absence becomes `null`, `[]` and `null` in the
canonical Artifact account.

A raw companion is marker-only. Before rewrite, core rejects any explicit Design `Binding:` equal
to its raw id or derived assembled key. One MechanismImplementation and companion resolve only the
exact `spec` and `mechanism` named by that record; an artifact-id match cannot fan out to another
target. Only ordinary non-companion Artifacts may be reused by several explicit Design bindings.

The companion is the one Artifact identity exception. Its assembled id is already the
SourceIdentity key and is not expanded again as `<area>|<kind>|<id>`. Other Artifacts, including
explicit schema or index bindings, retain their authored kind and id as semantic input.

One qualified site is unique within `(area, address-kind)` and belongs to one marker target. A
duplicate target, another target at the same SourceIdentity, a conflicting qualified-site account
or several distinct sites for one mechanism fails closed. The same kind/site in different areas is
legal and produces different assembled binding and Artifact ids.

The accountable emitter derives compiler qualification and fails when its compiler or runtime
account reports ambiguity. Core cannot prove qualification from opaque `site` bytes. It checks a
non-empty trimmed site without control characters or `|`, exact raw binding and companion equality,
then post-assembly uniqueness and consistency. Local and federated assembly use the same area-key
rewrite and produce identical semantic ids; neither uses file, mount, repository or revision as a
semantic disambiguator.

Moving unchanged source within one area preserves the qualified site, SourceIdentity, Claim
Judgment and semantic Challenge scope. The new file changes complete-model and accountable launch
locator fingerprints. Moving across areas or changing language, site or source content changes
semantic identity or content identity.

Unknown collections and removed alpha-era fields fail closed. There are no result-import binaries
in the extractor packages. Run normalization and provider adapters belong to dependent execution
plane changes, not to language extraction.

## Source opt-in

Production source uses `Realizes(<spec>, <case>)`. A deliberately enrolled Check implementation
uses `ImplementsCheck(<project-global-check-id>)`; language packages expose the idiomatic spelling
for that ecosystem. The source marker says only which Check the site implements. Evidence Bindings
in `verification.md` own every Check-to-Claim relationship and its form.

A mechanism implementation continues to use `ImplementsMechanism(<spec>, <mechanism>)` in the
existing ecosystem spelling. D48 adds no annotation argument. The extractor, not source authors,
owns qualified `site`, typed `binding`, companion Artifact and source fingerprint derivation.

An unmarked native test emits nothing. This is the normal state for ordinary engineering tests and
for every test of a routine Claim. All current framework Claims are routine, so canonical synthetic
fixtures emit Realizes linkage only.

## Ecosystems

### .NET

The .NET extractor reflects over built assemblies. Compiled metadata resolves repeatable
attributes, inheritance and generics; portable PDBs provide source paths where available. It also
derives type and method symbols plus Entity Framework migration indexes. A mechanism site uses the
namespace, declaring type, method and complete metadata parameter signature; a PDB path never
disambiguates it.

```text
azimuth-emit-dotnet --output manifest.json --root . path/to/Assembly.dll
```

### TypeScript and JavaScript

The TypeScript extractor uses the compiler API and resolves marker calls to their enclosing named
symbol. A mechanism site uses the resolved module, declaring type or receiver, symbol and checker
signature. The same parser accepts JavaScript extensions while retaining `lang: javascript` and
fails when the available JavaScript semantics cannot prove that identity unique.

```text
azimuth-emit-ts --output manifest.json --root . src
```

`--next-app <class>=<dir>` derives route membership from a built Next.js manifest and fails closed
when build output or source resolution is incomplete. Prometheus inputs may emit structural rule
and rule-test artifacts; they do not create Check enrollment from provider-specific metadata.

### Go, JVM, Python, Rust and C++

- Go resolves typed no-op calls against enclosing AST functions and uses package import identity,
  receiver, function and `go/types` signature.
- Java and Kotlin read repeatable runtime annotations from compiled classes and use binary class
  identity, method and JVM descriptor.
- Python parses no-op decorators with the standard `ast` module and uses importable module plus
  enclosing class and function identities; an unresolved import identity or collision fails.
- Rust requires the crate to compile, binds inert attributes to enclosing functions and uses crate
  and module path plus enclosing type or trait and item identity.
- C++ consumes Clang's semantic AST and `clang::annotate` metadata, using the qualified declaration
  plus canonical parameter and template signature.

`experiments/polyglot/check.sh` builds all seven language fixtures, runs ordinary tests, emits seven
strict manifests, validates their union and asserts export version 2.

## Tests

```text
dotnet test tools/extractors/dotnet/Azimuth.Emit.Tests
(cd tools/extractors/typescript && npm run build && npm test)
python3 -m unittest discover -s tools/extractors/python -p 'test_*.py'
(cd tools/extractors/go && go test ./...)
cargo test --manifest-path tools/extractors/rust/Cargo.toml
python3 -m unittest discover -s tools/extractors/cpp -p 'test_*.py'
```

Fixtures are synthetic. Tests assert the complete manifest shape, source identities, exact
fingerprint contract, multiple implementations of one Check and absence of linkage from unmarked
tests.

Every ecosystem's mechanism tests also prove:

- the emitted record has required qualified `site`, exact raw typed `binding` and one path-free
  companion Artifact, including optional-property preservation, without changing the annotation
  API;
- nested modules, declaring types or receivers and overload signatures produce distinct sites;
- two declarations that would collapse to one site fail extraction rather than adding a file path;
- missing site, untyped or path-bearing binding, raw prequalified area key, prefix/address mismatch
  and companion id, kind or file mismatch are rejected;
- an explicit Design binding to either a marker companion's raw id or assembled key, and two marker
  targets sharing one companion, are rejected before rewrite;
- one ordinary non-companion Artifact may still serve several explicit Design bindings;
- duplicate targets, same-area cross-manifest qualified-site conflicts and several sites for one
  mechanism fail, while the same kind/site in two areas produces distinct assembled ids;
- local and federated assembly produce the same atomic binding and companion-id rewrite; and
- relocation inside one area preserves SourceIdentity, Claim Judgment and semantic scope while the
  complete-model and launch locator fingerprints change.

An ecosystem without overloads still tests declaring-type or receiver qualification. A compiler or
runtime metadata reader that cannot prove a unique qualified declaration must reject that marker;
it may not fall back to a simple name or path.
