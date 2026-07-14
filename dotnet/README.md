# Azimuth — .NET annotations + manifest emitter

The C# side of azimuth's polyglot path. A codebase tags its production code and tests with
attributes, then emits a language-neutral `*.manifest.json` (see `../schema/manifest.schema.json`)
that the `rtm` core ingests alongside the specs to build the traceability matrix.

Two projects:

- **`Azimuth.Annotations`** (`netstandard2.0`) — the `[Realizes]` / `[Covers]` attributes and the
  form enums. Referenced by any codebase whose linkage you want traced. Wide target so it drops
  into net8/9/10 (and older) consumers unchanged.
- **`Azimuth.Manifest`** (`net9.0`) — the reflection emitter: a static API plus a console entry
  point that reads the attributes off one or more built assemblies and writes the manifest JSON.

## Tagging code

`[Realizes]` marks a **production-code site** on a scenario's path, by the stable
`(spec, req, scenario)` triple. No form — form is how a *test* checks, not a property of code.
It sits on a class or a method and may repeat.

```csharp
using Azimuth.Annotations;

[Realizes("public-certificates", "detail", "detail-valid")]
public sealed class GetPublicCertificate
{
    [Realizes("public-certificates", "detail", "detail-revoked-void")]
    public CertificateView Handle(string slug) => /* ... */;
}
```

`[Covers]` marks a **test** that verifies a scenario, at a declared form. Form is the pair
`(scope, quantification)`: `scope ∈ {Unit, Component, E2e}` (how much of the real system runs) and
`quantification ∈ {Example, Invariant}` (∃ one case vs ∀ a property). The optional `oracle`
(`Direct` default, `Golden`, `Metamorphic`, `ModelBased`, `Contract`) is descriptive only —
recorded for the code-map, never gated.

```csharp
[Covers("public-certificates", "detail", "detail-revoked-void",
    RtmScope.Component, RtmQuantification.Invariant, RtmOracle.Direct)]
public void RevokedCertificateReturns404() { /* ... */ }
```

Spec-side scenario attributes (`exposes` / `upholds`) live in the spec, not in code — code only
`realizes` / `covers`.

## Referencing the package

Local project reference (until the package is published to a feed):

```xml
<ItemGroup>
  <ProjectReference Include="path/to/azimuth/dotnet/src/Azimuth.Annotations/Azimuth.Annotations.csproj" />
</ItemGroup>
```

Both production and test projects reference `Azimuth.Annotations` — the same attributes tag both
`realizes` (production) and `covers` (test) sites.

## Running the emitter

Build the target assemblies first (their `.pdb` next to the `.dll` gives best-effort source paths),
then run the console tool over them:

```bash
dotnet run --project path/to/azimuth/dotnet/src/Azimuth.Manifest -- \
  --output out/csharp.manifest.json \
  --root /path/to/consumer/repo/root \
  path/to/Consumer.dll path/to/Consumer.Tests.dll
```

- `--output` / `-o` — where the manifest JSON is written (required).
- `--root` / `-r` — repo root the emitted `file` paths are made relative to (defaults to CWD).
- positional args — one or more built assemblies to reflect over.

Point `rtm` at the emitted manifest to fold the C# linkage into the matrix.

### Or from code

```csharp
using Azimuth.Manifest;

// static API — collect from loaded assemblies and serialize
Manifest manifest = ManifestCollector.Collect(new[] { productionAssembly, testAssembly }, repoRoot);
string json = ManifestEmitter.ToJson(manifest);

// or collect + write in one call
ManifestEmitter.Emit(new[] { productionAssembly, testAssembly }, "out/csharp.manifest.json", repoRoot);
```

## Build & test

```bash
dotnet build path/to/azimuth/dotnet/Azimuth.slnx
dotnet test  path/to/azimuth/dotnet/Azimuth.slnx
```

## How it works

`ManifestCollector` reflects over each assembly. It matches the attributes by **full type name**
(via `CustomAttributeData`), not CLR identity, so the emitter still sees tags when a target
assembly was loaded from a different path than the emitter's own `Azimuth.Annotations` reference.
Site names follow the harness convention: a type-level tag → the type name, a method-level tag →
`Type.Method`. `SourceFileResolver` reads the portable PDB (standalone or embedded) to map each
tagged method to its source file — best-effort: with no PDB the `file` is left empty.
