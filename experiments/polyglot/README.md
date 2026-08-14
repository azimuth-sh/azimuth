# Polyglot extractor conformance

This experiment asks one narrow question: can independently built services in materially different
language ecosystems emit one Azimuth model without the core learning any language?

Seven small HTTP services expose `/identity`. Their unit-level identity capabilities realize and
cover seven standard claims in one synthetic spec. `check.sh` compiles or syntax-checks each
service, runs its test, emits seven manifests through six extractor paths, and feeds their union to
the unchanged Rust core.

| Language | Marker shape | Extractor authority | Fingerprint boundary |
|---|---|---|---|
| Go | typed no-op calls | Go parser AST, followed by `go test` | enclosing function |
| Java | runtime annotations | compiled JVM reflection | source file fallback |
| Kotlin | runtime annotations | compiled JVM reflection | source file fallback |
| Python | no-op decorators | Python `ast` | decorated symbol |
| JavaScript | no-op calls | TypeScript compiler parser in JS mode | enclosing symbol/test |
| Rust | inert attribute macros | compile gate plus Rust attribute parser | enclosing function |
| C++ | Clang annotations | Clang AST dump after semantic analysis | source file fallback |

Java and Kotlin deliberately share a JVM extractor because class metadata is their common trusted
boundary. JavaScript deliberately shares the TypeScript extractor while retaining `lang:
javascript`. Copying either parser would demonstrate code duplication, not language support.

Run:

```text
./experiments/polyglot/check.sh
```

The Kotlin compiler is obtained through Gradle on the first run. Go and Gradle caches default to
task-specific paths under `/tmp` so the experiment does not depend on a developer's global cache.

## Result boundary

A green run establishes compiler-compatible annotation packages, extraction of claim identity,
site, language, actual evidence form and source freshness, plus composition through the existing
manifest contract. It does not establish framework adoption ergonomics in seven production
codebases. The C++ and JVM extractors conservatively fingerprint a complete source file; unrelated
edits can therefore expire judgments until their compiler APIs expose a stable source span.
