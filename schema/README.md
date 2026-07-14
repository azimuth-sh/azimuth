# Azimuth manifest schema

A **manifest** is the language-neutral seam between a codebase and `rtm`. Each codebase emits one
`*.manifest.json` describing its linkage tags; `rtm` reads the specs (markdown) plus one or more
manifests and derives the traceability matrix. This is the polyglot path — the alternative to
`rtm` scanning source comments directly, and the only path once tags come from real language
annotations (C# attributes, TS marker calls) rather than comments.

## Shape

```json
{
  "realizes": [
    { "spec": "public-certificates", "req": "detail", "scenario": "detail-valid",
      "site": "GetPublicCertificate", "file": "Features/Certificates/GetPublicCertificate.cs",
      "lang": "csharp" }
  ],
  "covers": [
    { "spec": "public-certificates", "req": "detail", "scenario": "detail-revoked-void",
      "scope": "component", "quantification": "invariant", "oracle": "direct",
      "site": "RevokedCertificateReturns404", "file": "Features/Certificates/GetPublicCertificateTests.cs",
      "lang": "csharp" }
  ]
}
```

- **`realizes`** — production-code sites on a scenario's path. Keyed by the `(spec, req, scenario)`
  triple. **No form**: form is how a *test* checks, not a property of code.
- **`covers`** — tests, each carrying the scenario's `(scope, quantification)` form and an optional
  descriptive `oracle`.

## What lives here vs in the spec

The split is deliberate — **authored intent** stays in the spec, **derived facts** come from the
manifest:

| In the spec (markdown) | In the manifest (emitted) |
|---|---|
| scenario declarations + their required form (`scope`/`quant`) | `realizes` / `covers` entries |
| `## Invariant` declarations (`over`, `references`) | — |
| scenario-level `exposes` / `upholds` | — |

Class membership (which sites are in an invariant's surface class) and guard discharge are **not**
in the manifest: they fall out of *which scenarios a site realizes*, joined against the spec. A
site can't escape an invariant by forgetting a tag, because membership rides on the exposure
scenario it already realizes.

## Fields

`spec`, `req`, `scenario` — the stable id triple.
`site` — the enclosing symbol (function/method/class) the tag sits on.
`file`, `lang` — provenance, for the polyglot fan-out and code-map views.
`scope` ∈ `unit | component | e2e` — how much of the real system runs (covers only).
`quantification` ∈ `example | invariant` — one case (∃) vs a property over all (∀) (covers only).
`oracle` ∈ `direct | golden | metamorphic | model-based | contract` — optional, descriptive, never
gated.

Both top-level arrays are optional; a manifest may carry only `realizes`, only `covers`, or both.
