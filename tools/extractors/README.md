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
  "mechanism_implementations": [],
  "class_members": [],
  "enumerations": [],
  "artifacts": []
}
```

Every `source_fingerprint` is SHA-256 in the exact lexical form shown. Extractors fingerprint the
smallest trustworthy enclosing semantic site. Ecosystems without a stable source span may use a
complete-file boundary, but they may not invent a short or provider-specific fingerprint.

`realizes` links production implementation to a case-level Claim and carries no verification form.
`check_implementations` links source to one project-global Check id and carries no Claim, form,
context or Qualification. Workspace or project assembly attaches area, mount and semantic source
address. Several implementation records may compose one Check.

`mechanism_implementations` links source to a declared design mechanism. Enumeration witnesses and
class members establish complete site domains independently of linkage markers. Artifacts provide
exact structural binding targets.

Unknown collections and removed alpha-era fields fail closed. There are no result-import binaries
in the extractor packages. Run normalization and provider adapters belong to dependent execution
plane changes, not to language extraction.

## Source opt-in

Production source uses `Realizes(<spec>, <case>)`. A deliberately enrolled Check implementation
uses `ImplementsCheck(<project-global-check-id>)`; language packages expose the idiomatic spelling
for that ecosystem. The source marker says only which Check the site implements. Evidence Bindings
in `verification.md` own every Check-to-Claim relationship and its form.

An unmarked native test emits nothing. This is the normal state for ordinary engineering tests and
for every test of a routine Claim. All current framework Claims are routine, so canonical synthetic
fixtures emit Realizes linkage only.

## Ecosystems

### .NET

The .NET extractor reflects over built assemblies. Compiled metadata resolves repeatable
attributes, inheritance and generics; portable PDBs provide source paths where available. It also
derives type and method symbols plus Entity Framework migration indexes.

```text
azimuth-emit-dotnet --output manifest.json --root . path/to/Assembly.dll
```

### TypeScript and JavaScript

The TypeScript extractor uses the compiler API and resolves marker calls to their enclosing named
symbol. The same parser accepts JavaScript extensions while retaining `lang: javascript`.

```text
azimuth-emit-ts --output manifest.json --root . src
```

`--next-app <class>=<dir>` derives route membership from a built Next.js manifest and fails closed
when build output or source resolution is incomplete. Prometheus inputs may emit structural rule
and rule-test artifacts; they do not create Check enrollment from provider-specific metadata.

### Go, JVM, Python, Rust and C++

- Go resolves typed no-op calls against enclosing AST functions.
- Java and Kotlin read repeatable runtime annotations from compiled classes.
- Python parses no-op decorators with the standard `ast` module.
- Rust requires the crate to compile and binds inert attributes to enclosing functions.
- C++ consumes Clang's semantic AST and `clang::annotate` metadata.

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
