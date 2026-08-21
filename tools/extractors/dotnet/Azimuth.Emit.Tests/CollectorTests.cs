using System.Text.Json;
using Xunit;

namespace Azimuth.Emit.Tests;

/// <summary>
/// Tests against the synthetic fixture beside this project (D2). A silently wrong emitter produces
/// a green matrix, which is the exact failure the framework exists to prevent — so these assert on
/// the shape of what is emitted, not merely that something was.
/// </summary>
public sealed class CollectorTests
{
    private static Collector.Result Collect() =>
        Collector.Collect(
            [typeof(Azimuth.Fixture.Production).Assembly],
            Directory.GetCurrentDirectory());

    [Fact]
    public void A_type_level_tag_names_its_site_by_the_type()
    {
        var entry = Assert.Single(
            Collect().Realizes,
            r => r.Scenario == "type-level-thing");
        Assert.Equal("alpha", entry.Spec);
        Assert.Equal("Azimuth.Fixture.Production", entry.Site);
    }

    [Fact]
    public void A_method_level_tag_names_its_site_by_type_and_method()
    {
        var entry = Assert.Single(
            Collect().Realizes,
            r => r.Scenario == "method-level-thing");
        Assert.Equal("Azimuth.Fixture.Production.Method", entry.Site);
    }

    [Fact]
    public void A_site_may_realize_several_claims()
    {
        var branches = Collect()
            .Realizes.Where(r => r.Site == "Azimuth.Fixture.Production.Branching")
            .Select(r => r.Scenario)
            .OrderBy(s => s, StringComparer.Ordinal)
            .ToList();
        Assert.Equal(["first-branch", "second-branch"], branches);
    }

    /// <summary>
    /// The attribute targets must match what the extractor walks, or a tag vanishes silently. A
    /// struct-level tag was rejected by the compiler until the annotation package was widened —
    /// found by tagging Money, which is itself the top-rung enforcement mechanism.
    /// </summary>
    [Fact]
    public void A_value_type_is_a_realization_site()
    {
        var entry = Assert.Single(
            Collect().Realizes,
            r => r.Scenario == "struct-level-thing");
        Assert.Equal("Azimuth.Fixture.Amount", entry.Site);
    }

    [Fact]
    public void Untagged_code_produces_nothing()
    {
        Assert.DoesNotContain(
            Collect().Realizes,
            r => r.Site.EndsWith(".Untagged", StringComparison.Ordinal));
    }

    [Fact]
    public void Symbols_are_emitted_independently_of_linkage_tags()
    {
        var artifact = Assert.Single(
            Collect().Artifacts,
            artifact => artifact.Id == "dotnet-symbol:Azimuth.Fixture.Production.Untagged");
        Assert.Equal("dotnet-method", artifact.Kind);
        Assert.EndsWith("Fixture.cs", artifact.File);
    }

    /// <summary>Form is how a test checks, not a property of code.</summary>
    [Fact]
    public void Realizes_carries_no_form()
    {
        Assert.All(Collect().Realizes, entry =>
        {
            Assert.Null(entry.Scope);
            Assert.Null(entry.Quantification);
            Assert.Null(entry.Oracle);
        });
    }

    [Fact]
    public void Several_sites_can_implement_one_check_deterministically()
    {
        var entries = Collect().CheckImplementations
            .Where(entry => entry.Check == "fixture/component-behavior")
            .ToList();

        Assert.Equal(2, entries.Count);
        Assert.Equal(
            entries.OrderBy(entry => entry.Site, StringComparer.Ordinal),
            entries);
        Assert.Equal(
            [
                "Azimuth.Fixture.Traced.Tests.Covered",
                "Azimuth.Fixture.Traced.Tests.CoveredRelationally",
            ],
            entries.Select(entry => entry.Site));
    }

    [Fact]
    public void An_unmarked_test_emits_no_check_implementation()
    {
        Assert.DoesNotContain(
            Collect().CheckImplementations,
            entry => entry.Site.EndsWith(".Bare", StringComparison.Ordinal)
                || entry.Site.EndsWith(".SmokeCheck", StringComparison.Ordinal));
    }

    [Fact]
    public void Check_implementations_carry_deterministic_exact_site_fingerprints()
    {
        var first = Collect().CheckImplementations;
        var second = Collect().CheckImplementations;

        Assert.Equal(first, second);
        Assert.All(
            first,
            entry => Assert.Matches("^sha256:[0-9a-f]{64}$", entry.SourceFingerprint));
        Assert.NotEqual(first[0].SourceFingerprint, first[1].SourceFingerprint);
    }

    [Fact]
    public void Alpha_one_marker_types_are_absent()
    {
        var annotations = typeof(Azimuth.Annotations.RealizesAttribute).Assembly;

        Assert.Null(annotations.GetType("Azimuth.Annotations.CoversAttribute"));
        Assert.Null(annotations.GetType("Azimuth.Annotations.CoversMechanismAttribute"));
    }

    [Fact]
    public void A_mechanism_implementation_derives_its_symbol_binding()
    {
        var entry = Assert.Single(
            Collect().MechanismImplementations,
            item => item.Mechanism == "branch-selection");
        Assert.Equal("alpha", entry.Spec);
        Assert.Equal("dotnet-symbol:Azimuth.Fixture.Production.Branching", entry.Binding);
    }

    [Fact]
    public void A_constructor_only_type_has_a_navigable_realization_source()
    {
        var entry = Assert.Single(
            Collect().Realizes,
            item => item.Scenario == "constructor-only-thing");

        Assert.EndsWith("tools/extractors/dotnet/fixture/Fixture.cs", entry.File);
        Assert.Matches("^sha256:[0-9a-f]{64}$", entry.SourceFingerprint);
    }

    /// <summary>
    /// A finding reported against "some assembly" is a finding nobody acts on.
    /// </summary>
    [Fact]
    public void Sites_carry_a_source_path()
    {
        Assert.All(Collect().Realizes, entry => Assert.EndsWith("Fixture.cs", entry.File));
    }

    [Fact]
    public void Sites_carry_a_compiler_resolved_source_fingerprint()
    {
        Assert.All(
            Collect().CheckImplementations,
            entry => Assert.Matches("^sha256:[0-9a-f]{64}$", entry.SourceFingerprint));
    }

    /// <summary>
    /// Most tagged methods in a service are async, and an async method's sequence points live on
    /// the compiler-generated state machine rather than on the method the tag sits on. Without the
    /// fallback the manifest carried almost no paths, which stayed invisible until the agent tier
    /// needed to fingerprint over evidence files.
    /// </summary>
    [Fact]
    public void An_async_method_still_carries_a_source_path()
    {
        var entry = Assert.Single(
            Collect().Realizes,
            r => r.Scenario == "async-thing");
        Assert.EndsWith("Fixture.cs", entry.File);
    }

    [Fact]
    public void Entries_are_ordered_so_the_manifest_diffs()
    {
        var scenarios = Collect().Realizes.Select(r => r.Scenario).ToList();
        Assert.Equal(scenarios.OrderBy(s => s, StringComparer.Ordinal), scenarios);
    }

    [Fact]
    public void The_manifest_is_keyed_on_the_pair()
    {
        using var document = JsonDocument.Parse(Collector.ToJson(Collect()));
        var root = document.RootElement;

        var realizes = root.GetProperty("realizes")[0];
        Assert.True(realizes.TryGetProperty("spec", out _));
        Assert.True(realizes.TryGetProperty("scenario", out _));
        Assert.False(realizes.TryGetProperty("req", out _));
        Assert.Equal("csharp", realizes.GetProperty("lang").GetString());
        Assert.Matches(
            "^sha256:[0-9a-f]{64}$",
            realizes.GetProperty("source_fingerprint").GetString()!);
        Assert.False(realizes.TryGetProperty("scope", out _));

        Assert.NotEmpty(root.GetProperty("check_implementations").EnumerateArray());
        Assert.NotEmpty(root.GetProperty("mechanism_implementations").EnumerateArray());
        Assert.NotEmpty(root.GetProperty("artifacts").EnumerateArray());
        Assert.False(root.TryGetProperty("covers", out _));
        Assert.False(root.TryGetProperty("mechanism_covers", out _));
        Assert.False(root.TryGetProperty("untraced_tests", out _));

        foreach (var collection in new[]
                 {
                     "realizes",
                     "check_implementations",
                     "mechanism_implementations",
                 })
        {
            Assert.All(
                root.GetProperty(collection).EnumerateArray(),
                entry => Assert.Matches(
                    "^sha256:[0-9a-f]{64}$",
                    entry.GetProperty("source_fingerprint").GetString()!));
        }
        Assert.False(root.TryGetProperty("observations", out _));
    }
}
