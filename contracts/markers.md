# Source marker format

Three markers, and only three, put source on the map. `Realizes` names a case-level Claim, `ImplementsCheck` names a project-global Check identity and `ImplementsMechanism` names a design-owned mechanism identity. Every language package spells the same three in its own idiom, and every extractor turns them into the records described in `contracts/manifest.md`.

A marker carries only the identities listed here. It carries no verification form, no scope, no oracle, no Qualification and no Claim relationship. Those live in `verification.md` declarations. An extra argument is an extraction failure, not an extension point.

Anything not described here is a parse error in the ecosystem that reads it.

## What the author writes

```text
Realizes(<spec-id>, <scenario-id>)
ImplementsCheck(<project-global-check-id>)
ImplementsMechanism(<spec-id>, <mechanism-id>)
```

Every argument is a string literal. A computed value, a constant reference, a keyword argument and a wrong argument count each fail extraction rather than emitting a weaker record.

`Realizes` takes the `(spec, scenario)` pair, not a triple. Scenario ids are unique within a spec, so a requirement id would be redundant information that can go stale; omitting it is what lets a requirement be split or merged without touching a marker.

`ImplementsCheck` takes one Check id and nothing else. The marker says which Check the site implements. Every Check-to-Claim edge, its form and its context are declared in `verification.md`. An unmarked test emits nothing, which is the normal state for ordinary engineering tests and for every test of a routine Claim.

`ImplementsMechanism` takes the same two-argument shape it has always had. No annotation argument was added for the qualified site, the binding or the companion Artifact.

## What the extractor derives

The author never writes a symbol path, a locator or a fingerprint. For every marker the extractor derives:

- `file` — a non-empty normalized path below the one resolved `--root`, using `/`, with no empty, `.` or `..` segment and no backslash. An input outside the root fails.
- `lang` — the ecosystem's language token.
- `site` — the enclosing declaration's identity. For `Realizes` and `ImplementsCheck` this is the ecosystem's ordinary site name; for `ImplementsMechanism` it is the qualified, path-free semantic site fixed by that ecosystem's profile in `contracts/verification.md`.
- `source_fingerprint` — SHA-256 over the smallest trustworthy enclosing semantic site, in the exact form `sha256:<64-lowercase-hex>`. An ecosystem without a stable source span may use a complete-file boundary; none may invent a short or provider-specific fingerprint.

For `ImplementsMechanism` the extractor additionally derives the typed `binding` and the companion Artifact as one atomic account, exactly as `contracts/manifest.md` requires. One qualified site implements at most one mechanism: a second `ImplementsMechanism` on the same site fails extraction.

An extractor that cannot prove a unique qualified declaration must reject that marker. It may not fall back to a simple name, and a source path never disambiguates a site.

## Retired markers

`Covers` and `CoversMechanism`, in each ecosystem's spelling, are retired alpha 1 markers. Every extractor rejects them by name with an explicit diagnostic rather than ignoring them. The Python and TypeScript extractors additionally take care not to mistake an ordinary local function or method of the same name for a retired marker.

## Ecosystem spellings

### .NET

`Azimuth.Annotations` supplies `[Realizes(spec, scenario)]`, `[ImplementsCheck(check)]` and `[ImplementsMechanism(spec, mechanism)]`. All three allow multiple instances on one target. `Realizes` and `ImplementsMechanism` target a class, struct, interface, enum or method; `ImplementsCheck` targets a method only. The targets match exactly what the extractor walks, so a marker cannot be placed where it would silently vanish.

Attributes are matched by full attribute-type name, not CLR identity, so the emitter works when the target assembly references a differently located copy of the annotations package.

### JVM

`sh.azimuth.Azimuth` supplies the repeatable runtime annotations `@Azimuth.Realizes(spec=, scenario=)`, `@Azimuth.ImplementsCheck(<check>)` and `@Azimuth.ImplementsMechanism(spec=, mechanism=)`. `Realizes` and `ImplementsMechanism` target a type or a method; `ImplementsCheck` targets a method. Retention is `RUNTIME`, because the extractor reads compiled classes. Java and Kotlin share the annotations and are distinguished by the resolved source file's extension.

### TypeScript and JavaScript

`@azimuth-sh/annotations` exports the typed no-op functions `realizes(spec, scenario)`, `implementsCheck(check)` and `implementsMechanism(spec, mechanism)`. They are function calls rather than decorators because the marked units are functions — route handlers, server components, hooks — and decorators are class-member-only. The call's enclosing named declaration is the site.

`implementsMechanism` emits only when its compiler symbol resolves to that package's export through a direct, aliased or namespace import; a local homonym is ordinary source. `realizes` and `implementsCheck` are recognized by call name.

### Go

`github.com/azimuth-sh/azimuth-go/azimuth` supplies the no-op calls `azimuth.Realizes(spec, scenario)`, `azimuth.ImplementsCheck(check)` and `azimuth.ImplementsMechanism(spec, mechanism)`. A call counts only when `go/types` resolves it to a function in that package; identifier and selector forms both resolve. The enclosing AST function supplies the site, and a marker inside an anonymous function has no stable site and fails.

### Python

`azimuth_annotations` supplies the no-op decorators `@realizes(spec, scenario)`, `@implements_check(check)` and `@implements_mechanism(spec, mechanism)`, applied to a class or a function. Decorators are matched by bare name, so the import must bind the name directly. Arguments must be string literals and keyword arguments are rejected.

### Rust

`azimuth-annotations` supplies the attribute macros `#[realizes(spec, scenario)]`, `#[implements_check(check)]` and `#[implements_mechanism(spec, mechanism)]`. The attribute path is accepted bare or qualified by exactly one of `azimuth` or `azimuth_annotations`; any other path is ordinary code. Arguments must be string literals.

### C++

`azimuth.hpp` supplies the macros `AZIMUTH_REALIZES(spec, scenario)`, `AZIMUTH_IMPLEMENTS_CHECK(check)` and `AZIMUTH_IMPLEMENTS_MECHANISM(spec, mechanism)`, each expanding to a `[[clang::annotate(...)]]` attribute whose payload is pipe-separated:

```text
azimuth|realizes|<spec>|<scenario>
azimuth|implements-check|<check>
azimuth|implements-mechanism|<spec>|<mechanism>
```

The kind token is kebab-case here, unlike every other ecosystem's identifier spelling. A payload that does not begin `azimuth|` with a known kind and the exact argument count is malformed and fails.

## Related contracts

- `contracts/manifest.md` — the records these markers produce and the rules core applies to them.
- `contracts/verification.md` — the exact per-ecosystem semantic-site profiles and Check linkage.
