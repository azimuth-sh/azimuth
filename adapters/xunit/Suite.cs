using System.Text.Json.Nodes;

namespace Azimuth.Xunit;

/// <summary>
/// The verb <c>run</c>: builds the suite in the current folder, lists its marked tests, runs the
/// ones of the asked Checks and reports every marked test, also the ones not asked for.
/// </summary>
public static class Suite
{
    public static JsonObject Run(string markerKey, IReadOnlySet<string> asked, JsonArray? components)
    {
        var folder = Directory.GetCurrentDirectory();
        var project = ProjectFile.InFolder(folder);
        if (Processes.Dotnet(folder, "build", project, "--nologo") != 0)
        {
            return new JsonObject { ["build"] = "failed", ["tests"] = new JsonArray() };
        }
        var (status, output) = Processes.DotnetOutput(folder, "msbuild", project, "-nologo", "-getProperty:TargetPath");
        var assembly = output.Trim();
        if (status != 0 || !File.Exists(assembly))
        {
            throw new AdapterException($"the build output of {project} was not found");
        }

        var tests = TestListing.Marked(assembly, markerKey);
        var askedChecks = tests.SelectMany(test => test.Checks).Where(asked.Contains).ToHashSet();
        using var reads = ReadsFolder.For(components);
        var outcomes = askedChecks.Count == 0
            ? new Dictionary<(string, string), TestOutcome>()
            : TestRun.Run(folder, project, markerKey, askedChecks, reads.Path);

        var entries = new JsonArray();
        foreach (var test in tests.OrderBy(test => test.Name, StringComparer.Ordinal))
        {
            var outcome = outcomes.GetValueOrDefault((test.ClassName, test.MethodName)) ?? new TestOutcome("not_run", 0);
            entries.Add(new JsonObject
            {
                ["name"] = test.Name,
                ["checks"] = new JsonArray(test.Checks.Select(check => (JsonNode?)check).ToArray()),
                ["outcome"] = outcome.Outcome,
                ["duration_ms"] = outcome.DurationMs,
            });
        }
        return new JsonObject { ["build"] = "ok", ["tests"] = entries, ["components"] = reads.Collect() };
    }
}
