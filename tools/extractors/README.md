# Extractors

One per ecosystem. Each finds the linkage tags in its own language and writes the same
language-neutral manifest; the core only ever reads manifests.

That seam is why adding a language is a day's work rather than a fork of the core — and it is what
lets the core stay dependency-free (D17) while this side does metadata and AST work in ecosystems
where that tooling is already present and idiomatic.

## Manifest

Keyed on the pair `(spec, scenario)`, not the alpha's triple (D2.2). Scenario ids are unique per
spec, so a requirement id would be redundant information that can go stale — and dropping it is
what makes splitting or merging a requirement free, since no tag moves.

```json
{
  "realizes": [{ "spec": "", "scenario": "", "site": "", "file": "", "lang": "",
                 "source_fingerprint": "" }],
  "covers":   [{ "spec": "", "scenario": "", "site": "", "file": "", "lang": "",
                 "source_fingerprint": "",
                 "scope": "unit|component|e2e",
                 "quantification": "example|universal",
                 "oracle": "direct|golden|relational|metamorphic|model-based|contract" }],
  "mechanism_implementations": [
    { "spec": "", "mechanism": "", "binding": "", "file": "", "lang": "",
      "source_fingerprint": "" }
  ],
  "mechanism_covers": [
    { "spec": "", "mechanism": "", "site": "", "file": "", "lang": "",
      "source_fingerprint": "", "scope": "unit|component|e2e",
      "quantification": "example|universal",
      "oracle": "direct|golden|relational|metamorphic|model-based|contract" }
  ],
  "enumerations": [{ "class": "", "kind": "", "source": "",
                     "source_fingerprint": "" }],
  "class_members": [{ "class": "", "site": "", "file": "", "lang": "" }],
  "artifacts": [{ "id": "", "kind": "", "file": "" }]
}
```

External tools use an `observations` collection. One immutable execution carries producer, report,
configuration inputs and fingerprint once; explicit bindings interpret it per claim. `evidence`
bindings project into `covers`. `challenge` bindings become agent judgment context and never cover
a claim. This many-to-many boundary avoids one core collection per tool and avoids repeating one
execution receipt for every claim.

`realizes` carries no form: form is how a test checks, not a property of code. The core rejects a
`realizes` that carries one, and rejects any entry carrying `req`, rather than ignoring it — a
stale emitter must not be able to produce tags that look fine and are not.

`source_fingerprint` hashes the compiler-resolved enclosing symbol or test. Judgment freshness uses
it to distinguish a changed evidence site from an unrelated edit in a shared file. It is optional:
when an extractor cannot resolve a site, the core conservatively hashes the complete file.

An enumeration witness says where a class came from independently of linkage tags. Its members are
authoritative only when the source was read completely; a missing build output or unresolved member
fails emission. Artifacts are exact binding targets for current design entries.

`mechanism_implementations` derives a concrete binding from a tag referring to an independent
design mechanism id. `mechanism_covers` records tests of that mechanism's contract. Neither array
is claim linkage, and mechanism evidence is never promoted automatically into `covers`.

## dotnet

Reflects over built assemblies. Running after a build rather than scanning source is deliberate:
reflection resolves inheritance and generics that a text scan gets wrong, and a tag the compiler
rejected is not a tag. Source paths come from the portable PDB, best-effort — an assembly without
one emits no paths and says so, rather than inventing them.

The .NET extractor also emits type/method symbols and executes compiled EF migration metadata to
derive database indexes, including uniqueness, ordered columns and predicates. It does not infer
that a method is an exclusive choke point.

```
azimuth-emit-dotnet --output m.json --root . path/to/Assembly.dll
```

## typescript

Static scan over the compiler API. The front end is functions, not classes, so the tags are typed
no-op function calls rather than decorators; the emitter resolves each call's enclosing named
symbol as the site, which makes a `covers` inside `test('…')` name the test.
The same AST node is the source-fingerprint boundary.

```
azimuth-emit-ts --output m.json --root . src
```

`--next-app <class>=<dir>` derives route members from Next's built route manifest. The option fails
closed when the build manifest is absent or a route cannot be resolved to project source.

`--prometheus <rules.yml>,<rules.test.yml>` emits `prometheus-alert:<name>` and
`prometheus-rule-test:<name>` artifacts. The repository runs `promtool test rules` before emission;
the extractor supplies machine addresses after Prometheus has validated the files, not a substitute
YAML interpretation. An immediately preceding `# azimuth-realizes: <spec> <scenario>` or
`# azimuth-covers: <spec> <scenario> <scope> <quantification> <oracle>` comment opts the named rule
or rule-test case into claim linkage. Federated source identity distinguishes `prometheus-alert`
from `prometheus-rule-test`; sharing the alert name does not make them one artifact.

The same compiler parser accepts `.js`, `.jsx`, `.mjs` and `.cjs` and emits `lang: javascript`.
JavaScript is an explicit mode of this extractor, not a copied text scanner.

## Go, JVM, Python, Rust and C++

The polyglot conformance experiment adds five extractor paths and language-native annotation
packages:

- Go uses typed no-op calls resolved against the enclosing Go AST function.
- Java and Kotlin use repeatable runtime annotations read from compiled JVM classes. Source lookup
  fails on ambiguity; fingerprints conservatively cover the complete source file.
- Python uses no-op decorators parsed by the standard `ast` module.
- Rust uses inert procedural attributes, requires the crate to compile, and binds attributes to
  their enclosing function in the source accepted by that build.
- C++ uses `clang::annotate`; the extractor consumes Clang's semantic AST rather than matching
  macros as text. Its fingerprint conservatively covers the complete source file.

`experiments/polyglot/check.sh` builds seven services, runs their evidence, emits seven manifests
through six extractor implementations and proves that their union closes one unchanged Azimuth
model.

### External manual results

`azimuth-import-manual <export.json> <manifest.json>` converts a provider-neutral manual-run export
into covering evidence receipts. A TestRail, Qase, Zephyr or similar adapter maps its API response
to this boundary:

```json
{
  "provider": "testrail",
  "run_id": "run-7",
  "observed_at": "2026-08-08T01:00:00Z",
  "expires_at": "2026-09-08T01:00:00Z",
  "results": [{
    "case_id": "case-42",
    "spec": "payments/capture",
    "scenario": "receipt-explains-payment-state",
    "status": "passed",
    "scope": "e2e",
    "quantification": "example",
    "url": "https://tracker.example/runs/7#42"
  }]
}
```

Only `passed` and `failed` cross the boundary; provider-specific states must be mapped explicitly.
The importer preserves failures, result attribution, observation time, expiry and a payload
fingerprint. A failed or expired receipt is a hole and does not count as coverage. A charter or test
case without an executed result emits nothing.

### Assurance observations

`azimuth-import-observation` validates the provider-neutral boundary used by load, chaos, recovery
and other execution adapters. Every evidence binding declares its own assertion, outcome, scope,
quantification and oracle. A shared run-level `passed` bit is rejected.

`azimuth-import-sarif` consumes SARIF 2.1.0. It intersects analyzed artifacts with existing
`Realizes` sites and emits one challenge binding per affected claim. Findings remain in the
fingerprinted payload; a clean scan creates judgment context, not claim evidence.

### Mutation challenge

`azimuth-import-mutation` consumes a Stryker mutation-testing-elements schema v2 JSON report, an
ordinary linkage manifest and the exact Stryker configuration:

```sh
azimuth-import-mutation report.json linkage.json mutation.json \
  --root . --config tests/stryker-config.json --tool-version 4.16.0
```

The adapter derives bindings by intersecting selected test names with existing `Covers` sites and
mutated files with existing `Realizes` sites, avoiding a second hand-maintained map. Its payload
carries every final mutant-state count and review metadata for non-killed executable mutants.
Unknown schemas or statuses fail closed. A renamed target or test becomes an
`unresolved-observation-binding` hole. Survivors do not automatically fail `azimuth check`: only the
agent can decide whether a generated wrong implementation is relevant to the claim.

`azimuth-import-pit` consumes PIT's `mutations.xml`, an ordinary JVM linkage manifest, the exact
native configuration and a resolved source-selection input:

```sh
azimuth-import-pit target/pit-reports/mutations.xml linkage.json mutation.json \
  --root . --config pom.xml --selection tests/pit-selection.json --tool-version 1.25.3
```

```json
{
  "schema": "azimuth-pit-selection/1",
  "target_classes": ["example.Policy"],
  "selected_tests": [{
    "site": "example.PolicyTest.rejectsInvalid",
    "pit_names": ["rejectsInvalid(example.PolicyTest)"]
  }]
}
```

`target_classes` and each `site` are exact JVM symbols, not PIT globs. `pit_names` is an optional
bridge from PIT's native test display name to the same test symbol; it carries no claim identity.
The adapter requires every target and selected test to resolve through existing `Realizes` and
`Covers` linkage. A killing test named by the XML must also resolve to the selection. For survivors,
no-coverage results and other records without an exact test name, the fingerprinted selection is
the fail-closed account of which tests ran. This is required because `mutations.xml` does not carry
a complete selected-test inventory.

PIT remains a challenge under D38 and D39. Killed, survived, no-coverage, timed-out, non-viable,
runtime-error, pending and equivalent outcomes stay in the observation payload; every outcome
other than killed is a review item. Unknown XML structure, selection schemas or PIT statuses fail
before any binding is emitted. The report, native configuration, resolved selection and PIT version
all contribute to the observation fingerprint.

## Linkage opt-in

`covers` opts a test into the evidence model. An untagged test emits nothing: it may exercise
routine behavior, infrastructure or a project rule that no Azimuth claim names. `uncovered` is
derived in the other direction, from a standard or critical claim with no sufficient evidence.
Likewise, mechanism markers participate only when a design declares the named identity; orphaned
implementation or evidence markers are dangling holes.

## Tests

```
dotnet test tools/extractors/dotnet/Azimuth.Emit.Tests
(cd tools/extractors/typescript && npx tsc -p tsconfig.json && node --test dist/emitter.test.js)
```

Each extractor has a synthetic fixture beside it, synthetic by decision (D2): the moment an
extractor's tests assert against the real demo app, the two are welded together and neither can
move independently.

The tests assert on the *shape* of what is emitted rather than merely that something was, because
a silently wrong emitter produces a green matrix — the exact failure the framework exists to
prevent. The .NET suite includes a regression for the enum-boxing bug that first emitted
`"scope": "1"` instead of `"component"`.
