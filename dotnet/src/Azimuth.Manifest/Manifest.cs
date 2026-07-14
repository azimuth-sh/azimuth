using System.Text.Json.Serialization;

namespace Azimuth.Manifest;

/// <summary>
/// The language-neutral linkage manifest a C# codebase emits — the seam <c>rtm</c> reads
/// (see <c>schema/manifest.schema.json</c>). Only the emitted tags live here: derived intent
/// (scenario form, invariant declarations, exposes/upholds) stays in the spec.
/// </summary>
public sealed record Manifest
{
    [JsonPropertyName("realizes")]
    public IReadOnlyList<RealizesEntry> Realizes { get; init; } = [];

    [JsonPropertyName("covers")]
    public IReadOnlyList<CoversEntry> Covers { get; init; } = [];

    [JsonPropertyName("untraced_tests")]
    public IReadOnlyList<UntracedTestEntry> UntracedTests { get; init; } = [];
}

/// <summary>A production-code site on a scenario's path. No form — form is a property of tests.</summary>
public sealed record RealizesEntry(
    [property: JsonPropertyName("spec")] string Spec,
    [property: JsonPropertyName("req")] string Req,
    [property: JsonPropertyName("scenario")] string Scenario,
    [property: JsonPropertyName("site")] string Site,
    [property: JsonPropertyName("file")] string File,
    [property: JsonPropertyName("lang")] string Lang);

/// <summary>A test that verifies a scenario, at its declared (scope, quantification) form.</summary>
public sealed record CoversEntry(
    [property: JsonPropertyName("spec")] string Spec,
    [property: JsonPropertyName("req")] string Req,
    [property: JsonPropertyName("scenario")] string Scenario,
    [property: JsonPropertyName("site")] string Site,
    [property: JsonPropertyName("file")] string File,
    [property: JsonPropertyName("lang")] string Lang,
    [property: JsonPropertyName("scope")] string Scope,
    [property: JsonPropertyName("quantification")] string Quantification,
    [property: JsonPropertyName("oracle")] string Oracle);

/// <summary>
/// A test method in a tracing class (one with ≥1 <c>[Covers]</c>) that declares no scenario and is
/// not explicitly opted out — the dual of an uncovered scenario. No form: it names no behavior to
/// have a form for.
/// </summary>
public sealed record UntracedTestEntry(
    [property: JsonPropertyName("site")] string Site,
    [property: JsonPropertyName("file")] string File);
