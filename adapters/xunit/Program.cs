// The adapter contract: no arguments, one adapter-request-1 on standard input, one
// adapter-report-1 on standard output, and the runner's own output on standard error. Status 0
// means a report was written, also when tests failed; anything else is a crashed launch.

using System.Reflection;
using System.Runtime.InteropServices;
using System.Text.Json.Nodes;
using Azimuth.Xunit;

var reportState = 0;
using var interrupt = OperatingSystem.IsWindows() ? null : PosixSignalRegistration.Create(PosixSignal.SIGINT, signal =>
{
    signal.Cancel = true;
    Interlocked.Exchange(ref reportState, 1);
    Environment.Exit(1);
});

try
{
    var request = JsonNode.Parse(Console.In.ReadToEnd()) as JsonObject
        ?? throw new AdapterException("the request is not one JSON object");
    if ((string?)request["format"] != "adapter-request-1")
    {
        throw new AdapterException("the request is not an adapter-request-1");
    }

    var report = (string?)request["verb"] switch
    {
        "discover" => Discovery.Report(Text(request, "folder")),
        "run" => Suite.Run(Text(request, "marker_key"), Checks(request), request["components"] as JsonArray),
        var verb => throw new AdapterException($"the verb `{verb}` is not known; this adapter knows `discover` and `run`"),
    };
    report.Insert(0, "format", "adapter-report-1");
    report.Insert(1, "adapter", new JsonObject { ["name"] = "azimuth-xunit", ["version"] = Version() });
    var text = report.ToJsonString();
    if (Interlocked.CompareExchange(ref reportState, 2, 0) != 0)
    {
        return 1;
    }
    Console.Out.Write(text);
    return 0;
}
catch (AdapterException error)
{
    Console.Error.WriteLine($"azimuth-xunit: {error.Message}");
    return 1;
}

static string Text(JsonObject request, string field) =>
    (string?)request[field] ?? throw new AdapterException($"the request has no `{field}`");

static IReadOnlySet<string> Checks(JsonObject request) =>
    (request["checks"] as JsonArray ?? throw new AdapterException("the request has no `checks`"))
        .Select(check => (string?)check ?? throw new AdapterException("a Check id in `checks` is not text"))
        .ToHashSet();

static string Version() =>
    typeof(AdapterException).Assembly.GetCustomAttribute<AssemblyInformationalVersionAttribute>()?.InformationalVersion
        ?? typeof(AdapterException).Assembly.GetName().Version?.ToString()
        ?? "unknown";
