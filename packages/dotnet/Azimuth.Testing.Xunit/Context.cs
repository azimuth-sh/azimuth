using System.Text.Json;
using System.Text.Json.Serialization;

namespace Azimuth.Testing.Xunit;

public sealed record ContextDefinition(string ContractRevision, IReadOnlyDictionary<string, JsonElement> Values);

public sealed record VerificationContextReport(
    string Format,
    int Version,
    IReadOnlyDictionary<string, ContextDefinition> Contexts)
{
    public static VerificationContextReport Create(IReadOnlyDictionary<string, ContextDefinition> contexts)
        => new("azimuth-verification-context", 1, contexts);
}

public sealed record ContextCapture(string Instance, VerificationContextReport Report);

public sealed record ContextSource(
    string Name,
    Func<CancellationToken, Task<ContextCapture>> Capture);

public sealed record ContextSnapshot(long CapturedAtMs, string Instance, VerificationContextReport Report);

public sealed record TestContext(string Name, ContextSnapshot Before, ContextSnapshot After);

public sealed record TestFact(
    string Id,
    int Attempt,
    string Outcome,
    long StartedAtMs,
    long FinishedAtMs,
    IReadOnlyList<TestContext> Contexts,
    IReadOnlyList<string> Diagnostics);

public sealed record TestReport(
    string Format,
    int Version,
    string Framework,
    string Execution,
    long StartedAtMs,
    long FinishedAtMs,
    IReadOnlyList<TestFact> Tests);

public static class ContextProtocol
{
    public static JsonSerializerOptions JsonOptions { get; } = new()
    {
        PropertyNamingPolicy = JsonNamingPolicy.SnakeCaseLower,
        UnmappedMemberHandling = JsonUnmappedMemberHandling.Disallow,
        WriteIndented = true
    };

    public static VerificationContextReport Parse(string json)
    {
        var report = JsonSerializer.Deserialize<VerificationContextReport>(json, JsonOptions)
            ?? throw new JsonException("Verification context report is absent.");
        Validate(report);
        return report;
    }

    public static void Validate(VerificationContextReport report)
    {
        if (report.Format != "azimuth-verification-context" || report.Version != 1 || report.Contexts is null)
            throw new JsonException("Unsupported verification context report.");
        foreach (var (name, context) in report.Contexts)
        {
            if (string.IsNullOrWhiteSpace(name) || context is null || context.Values is null ||
                context.ContractRevision is null || context.ContractRevision.Length != 71 ||
                !context.ContractRevision.StartsWith("sha256:", StringComparison.Ordinal) ||
                context.ContractRevision[7..].Any(c => !char.IsAsciiHexDigit(c) || char.IsUpper(c)))
                throw new JsonException("Context requires a name, values and canonical SHA-256 contract revision.");
            foreach (var (field, value) in context.Values)
            {
                if (string.IsNullOrWhiteSpace(field) || !ValidValue(value))
                    throw new JsonException("Context values must be strings, booleans or nonnegative safe integers.");
            }
        }
    }

    private static bool ValidValue(JsonElement value) => value.ValueKind switch
    {
        JsonValueKind.String or JsonValueKind.True or JsonValueKind.False => true,
        JsonValueKind.Number => value.TryGetInt64(out var integer) && integer >= 0 && integer <= 9007199254740991L,
        _ => false
    };

    internal static VerificationContextReport Freeze(VerificationContextReport report)
        => Parse(JsonSerializer.Serialize(report, JsonOptions));

    internal static bool Equal(ContextSnapshot before, ContextSnapshot after)
    {
        if (before.Instance != after.Instance || before.Report.Contexts.Count != after.Report.Contexts.Count)
            return false;
        foreach (var (name, expected) in before.Report.Contexts)
        {
            if (!after.Report.Contexts.TryGetValue(name, out var actual) ||
                expected.ContractRevision != actual.ContractRevision || expected.Values.Count != actual.Values.Count)
                return false;
            foreach (var (field, value) in expected.Values)
                if (!actual.Values.TryGetValue(field, out var other) || value.ValueKind != other.ValueKind ||
                    value.ToString() != other.ToString())
                    return false;
        }
        return true;
    }
}
