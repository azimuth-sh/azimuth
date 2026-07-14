using System.Text.Json;
using FluentAssertions;
using Xunit;

namespace Azimuth.Manifest.Tests;

/// <summary>
/// Emits a manifest from this assembly's own tagged sample surface and asserts the produced JSON
/// matches the schema's field names and the tags' (scope, quantification, oracle) forms — the
/// end-to-end path a consumer runs.
/// </summary>
public sealed class ManifestEmitterTests
{
    private static readonly string Root =
        Path.GetFullPath(Path.Combine(AppContext.BaseDirectory, "..", "..", ".."));

    [Fact]
    public void Collects_the_realizes_tags_on_types_and_methods()
    {
        var manifest = ManifestCollector.Collect(typeof(GetPublicCertificate).Assembly, Root);

        manifest.Realizes.Should().Contain(entry =>
            entry.Spec == "public-certificates" &&
            entry.Req == "detail" &&
            entry.Scenario == "detail-valid" &&
            entry.Site == "GetPublicCertificate" &&
            entry.Lang == "csharp");

        manifest.Realizes.Should().Contain(entry =>
            entry.Scenario == "detail-revoked-void" &&
            entry.Site == "GetPublicCertificate.Handle");
    }

    [Fact]
    public void Collects_the_covers_tags_with_scope_quantification_and_oracle()
    {
        var manifest = ManifestCollector.Collect(typeof(GetPublicCertificate).Assembly, Root);

        manifest.Covers.Should().Contain(entry =>
            entry.Scenario == "detail-valid" &&
            entry.Site == "GetPublicCertificateTests.ValidCertificateIsReturned" &&
            entry.Scope == "component" &&
            entry.Quantification == "example" &&
            entry.Oracle == "direct");

        manifest.Covers.Should().Contain(entry =>
            entry.Scenario == "detail-revoked-void" &&
            entry.Scope == "component" &&
            entry.Quantification == "invariant" &&
            entry.Oracle == "direct");

        manifest.Covers.Should().Contain(entry =>
            entry.Scenario == "completeness" &&
            entry.Scope == "e2e" &&
            entry.Quantification == "invariant" &&
            entry.Oracle == "model-based");
    }

    [Fact]
    public void Flags_a_test_in_a_tracing_class_that_declares_no_scenario()
    {
        var manifest = ManifestCollector.Collect(typeof(GetPublicCertificate).Assembly, Root);

        manifest.UntracedTests.Should().Contain(entry =>
            entry.Site == "SampleTracedRevokeTests.SeedsFixtures" &&
            entry.File == "SampleUntracedTests.cs");
    }

    [Fact]
    public void Does_not_flag_a_test_that_carries_covers_or_untraced()
    {
        var manifest = ManifestCollector.Collect(typeof(GetPublicCertificate).Assembly, Root);

        manifest.UntracedTests.Should().NotContain(entry =>
            entry.Site == "SampleTracedRevokeTests.RevokedCertificateIsHidden");
        manifest.UntracedTests.Should().NotContain(entry =>
            entry.Site == "SampleTracedRevokeTests.ResetsDatabase");
    }

    [Fact]
    public void Does_not_flag_untagged_tests_in_a_class_that_does_not_trace()
    {
        var manifest = ManifestCollector.Collect(typeof(GetPublicCertificate).Assembly, Root);

        manifest.UntracedTests.Should().NotContain(entry =>
            entry.Site == "SampleUntracedNeighbourTests.SomeUnrelatedTest");
    }

    [Fact]
    public void Resolves_the_source_file_relative_to_the_repo_root()
    {
        var manifest = ManifestCollector.Collect(typeof(GetPublicCertificate).Assembly, Root);

        var entry = manifest.Realizes.Single(e => e.Site == "GetPublicCertificate");
        entry.File.Should().Be("SampleTaggedCertificates.cs");
    }

    [Fact]
    public void Emits_json_with_the_schema_field_names()
    {
        var outputPath = Path.Combine(Path.GetTempPath(), $"azimuth-{Guid.NewGuid():N}.manifest.json");
        try
        {
            ManifestEmitter.Emit(new[] { typeof(GetPublicCertificate).Assembly }, outputPath, Root);

            using var document = JsonDocument.Parse(File.ReadAllText(outputPath));
            var covers = document.RootElement.GetProperty("covers");

            var revoked = covers.EnumerateArray()
                .Single(e => e.GetProperty("scenario").GetString() == "detail-revoked-void");

            revoked.GetProperty("spec").GetString().Should().Be("public-certificates");
            revoked.GetProperty("req").GetString().Should().Be("detail");
            revoked.GetProperty("site").GetString().Should().Be("GetPublicCertificateTests.RevokedCertificateReturns404");
            revoked.GetProperty("lang").GetString().Should().Be("csharp");
            revoked.GetProperty("scope").GetString().Should().Be("component");
            revoked.GetProperty("quantification").GetString().Should().Be("invariant");
            revoked.GetProperty("oracle").GetString().Should().Be("direct");

            document.RootElement.GetProperty("realizes").GetArrayLength().Should().BeGreaterThanOrEqualTo(2);
        }
        finally
        {
            File.Delete(outputPath);
        }
    }
}
