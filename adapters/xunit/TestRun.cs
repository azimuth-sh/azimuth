using System.Globalization;
using System.Xml.Linq;

namespace Azimuth.Xunit;

/// <summary>One marked test as the report lists it.</summary>
public sealed record TestOutcome(string Outcome, long DurationMs);

/// <summary>
/// Starts the runner with its own trait filter for the asked Check ids, and reads its TRX report.
/// xUnit has no retries of its own, so the first attempt of each test is the only one.
/// </summary>
public static class TestRun
{
    private static readonly XNamespace Trx = "http://microsoft.com/schemas/VisualStudio/TeamTest/2010";

    /// <summary>The outcome of each test the runner reported, by class and method name.</summary>
    public static IReadOnlyDictionary<(string ClassName, string MethodName), TestOutcome> Run(
        string folder, string project, string markerKey, IReadOnlyCollection<string> checks)
    {
        var results = Directory.CreateTempSubdirectory("azimuth-xunit-");
        try
        {
            var trx = Path.Combine(results.FullName, "results.trx");
            var filter = string.Join("|", checks.Order(StringComparer.Ordinal).Select(check => $"{markerKey}={check}"));
            // The runner ends with a failure status when a test failed; the TRX file says what happened.
            Processes.Dotnet(folder, "test", project, "--no-build", "--nologo", "--filter", filter,
                "--logger", $"trx;LogFileName={trx}", "--results-directory", results.FullName);
            if (!File.Exists(trx))
            {
                throw new AdapterException("the runner wrote no TRX report");
            }
            return Read(XDocument.Load(trx));
        }
        finally
        {
            results.Delete(recursive: true);
        }
    }

    private static Dictionary<(string, string), TestOutcome> Read(XDocument trx)
    {
        var methods = new Dictionary<string, (string, string)>();
        foreach (var test in trx.Descendants(Trx + "UnitTest"))
        {
            var method = test.Element(Trx + "TestMethod");
            if ((string?)test.Attribute("id") is { } id && method is not null)
            {
                methods[id] = ((string?)method.Attribute("className") ?? "", (string?)method.Attribute("name") ?? "");
            }
        }

        var rows = new Dictionary<(string, string), List<(string Outcome, long DurationMs)>>();
        foreach (var result in trx.Descendants(Trx + "UnitTestResult"))
        {
            if ((string?)result.Attribute("testId") is not { } id || !methods.TryGetValue(id, out var method))
            {
                continue;
            }
            var duration = TimeSpan.TryParse((string?)result.Attribute("duration"), CultureInfo.InvariantCulture, out var span) ? span : TimeSpan.Zero;
            if (!rows.TryGetValue(method, out var list))
            {
                rows[method] = list = [];
            }
            list.Add((Outcome((string?)result.Attribute("outcome")), (long)duration.TotalMilliseconds));
        }

        // A test with many data rows is one entry: it failed if any row failed.
        return rows.ToDictionary(
            pair => pair.Key,
            pair => new TestOutcome(Combine(pair.Value.Select(row => row.Outcome)), pair.Value.Sum(row => row.DurationMs)));
    }

    private static string Outcome(string? trxOutcome) => trxOutcome switch
    {
        "Passed" => "passed",
        "NotExecuted" => "skipped",
        _ => "failed",
    };

    private static string Combine(IEnumerable<string> outcomes)
    {
        var all = outcomes.ToList();
        if (all.Contains("failed"))
        {
            return "failed";
        }
        return all.All(outcome => outcome == "passed") ? "passed" : "skipped";
    }
}
