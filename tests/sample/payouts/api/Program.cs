using System.Reflection;
using System.Text.Json;
using Azimuth.Context;

// `api` of the invented payouts product, written the way the design's example shows. The tests drive
// it with FIXTURE_* variables and read it through the endpoints under /fixture, which belong to this
// fixture and not to the library.

var mode = Fixture.Env("FIXTURE_MODE");
if (mode == "inspect")
{
    Fixture.Inspect();
    return;
}

var builder = WebApplication.CreateBuilder(args);
var config = builder.Configuration;

var instantPath = Fixture.Env("FIXTURE_INSTANT_PATH") ?? "Features:Payouts:Instant";
var component = Fixture.Env("FIXTURE_COMPONENT") ?? "api";
var conditionVariable = Fixture.Env("FIXTURE_CONDITION_VARIABLE") ?? "PAYOUTS_TEST_STACK";
Fixture.Stack = Fixture.Env(conditionVariable) == "1";
Fixture.SideStore["Features:Payouts:Side"] = Fixture.Env("FIXTURE_SIDE");

Fixture.AddInMemory(config, Fixture.Env("FIXTURE_EARLIER"));
Fixture.SourcesBefore = Fixture.SourceTypes(config);

bool Condition()
{
    Fixture.ConditionCalls++;
    Fixture.SeenAtCondition = config[instantPath];
    return Fixture.Stack;
}

var context = (Fixture.Env("FIXTURE_KEYS") ?? "default") switch
{
    "instant" => config.AddAzimuthContext(
        component,
        applyingWhen: Condition,
        map => map.Key("payouts.instant", instantPath)),
    "region" => config.AddAzimuthContext(
        component,
        applyingWhen: Condition,
        map => map.Key("payouts.region", Fixture.Region)),
    "wide" => config.AddAzimuthContext(
        component,
        applyingWhen: Condition,
        map => map
            .Key("payouts.instant", instantPath)
            .Key("payouts.second", "Features:Payouts:Second")
            .Key("payouts.region", Fixture.Region)
            .Key("payouts.side", "Features:Payouts:Side")
            .Key("payouts.sideteam", Fixture.SideTeam)),
    _ => config.AddAzimuthContext(
        component,
        applyingWhen: Condition,
        map => map
            .Key("payouts.instant", instantPath)
            .Key("payouts.region", Fixture.Region)),
};

Fixture.SourcesAfterRegistration = Fixture.SourceTypes(config);
Fixture.AddInMemory(config, Fixture.Env("FIXTURE_LATER"));

if (mode == "console")
{
    Console.WriteLine(context.Report());
    Console.Out.Flush();
    Console.In.ReadToEnd();
    return;
}

builder.WebHost.UseUrls("http://127.0.0.1:0");
var app = builder.Build();

if (Fixture.Env("FIXTURE_NO_REPORT_PATH") != "1")
{
    app.MapGet("/internal/azimuth", () => Results.Text(context.Report(), "application/json"));
}

app.MapGet("/fixture/report", () => Results.Text(context.Report(), "application/json"));
app.MapGet("/fixture/setting", (string path) => Fixture.Json(new { value = config[path] }));
app.MapGet("/fixture/set", (string path, string value) =>
{
    config[path] = value;
    return Fixture.Json(new { });
});
app.MapGet("/fixture/condition", (string value) =>
{
    Fixture.Stack = value == "1";
    return Fixture.Json(new { });
});
app.MapGet("/fixture/env", (string name, string? value) =>
{
    Environment.SetEnvironmentVariable(name, value);
    return Fixture.Json(new { });
});
app.MapGet("/fixture/envvar", (string name) => Fixture.Json(new { value = Environment.GetEnvironmentVariable(name) }));
app.MapGet("/fixture/region", (string? value) =>
{
    if (value is not null)
    {
        Fixture.Region.Value = value;
    }
    return Fixture.Json(new
    {
        reads = Fixture.Region.Reads,
        applies = Fixture.Region.Applies,
        lastApplied = Fixture.Region.LastApplied,
    });
});
app.MapGet("/fixture/state", () => Fixture.Json(new
{
    conditionCalls = Fixture.ConditionCalls,
    seenAtCondition = Fixture.SeenAtCondition,
    sourcesBefore = Fixture.SourcesBefore,
    sourcesAfterRegistration = Fixture.SourcesAfterRegistration,
    sideTeamApplied = Fixture.SideTeam.LastApplied,
    providers = ((IConfigurationRoot)config).Providers.Count(),
}));

app.Lifetime.ApplicationStarted.Register(() =>
{
    Console.WriteLine("listening " + app.Urls.First());
    Console.Out.Flush();
});
app.Run();

static class Fixture
{
    public static bool Stack;
    public static int ConditionCalls;
    public static string? SeenAtCondition;
    public static string[] SourcesBefore = [];
    public static string[] SourcesAfterRegistration = [];
    public static RegionSource Region = new();
    public static SideTeamSource SideTeam = new();
    public static Dictionary<string, string?> SideStore = new();

    public static string? Env(string name) => Environment.GetEnvironmentVariable(name);

    public static IResult Json(object value) => Results.Text(JsonSerializer.Serialize(value), "application/json");

    public static string[] SourceTypes(IConfigurationBuilder config) =>
        config.Sources.Select(source => source.GetType().FullName ?? "").ToArray();

    public static void AddInMemory(IConfigurationBuilder config, string? pairs)
    {
        if (string.IsNullOrEmpty(pairs))
        {
            return;
        }
        var values = new Dictionary<string, string?>();
        foreach (var pair in pairs.Split(';'))
        {
            var at = pair.IndexOf('=');
            values[pair[..at]] = pair[(at + 1)..];
        }
        config.AddInMemoryCollection(values);
    }

    public static void Inspect()
    {
        var assembly = typeof(IContextSource).Assembly;
        var types = assembly.GetExportedTypes();
        var names = new List<string>();
        var genericMembers = new List<string>();
        foreach (var type in types)
        {
            names.Add(type.FullName ?? type.Name);
            foreach (var member in type.GetMembers(BindingFlags.Public | BindingFlags.Instance | BindingFlags.Static | BindingFlags.DeclaredOnly))
            {
                names.Add(member.Name);
                if (member is MethodInfo { IsGenericMethod: true })
                {
                    genericMembers.Add(type.Name + "." + member.Name);
                }
            }
        }
        Console.WriteLine(JsonSerializer.Serialize(new
        {
            assembly = assembly.GetName().Name,
            references = assembly.GetReferencedAssemblies().Select(reference => reference.Name).ToArray(),
            names,
            genericMembers,
        }));
    }
}

class RegionSource : IContextSource
{
    public string? Value = Fixture.Env("FIXTURE_REGION_VALUE");
    public string? LastApplied;
    public int Reads;
    public int Applies;

    public ContextRead Read()
    {
        Reads++;
        ContextSource? source = null;
        if (Fixture.Env("FIXTURE_REGION_SOURCE") != "none")
        {
            source = new ContextSource(
                Fixture.Env("FIXTURE_REGION_NAME") ?? "region-flags",
                Fixture.Env("FIXTURE_REGION_PRECEDENCE") ?? (LastApplied is null ? "before azimuth" : "azimuth"));
        }
        return new ContextRead(Value, source);
    }

    public void Apply(string text)
    {
        Applies++;
        LastApplied = text;
        Value = text;
    }
}

class SideTeamSource : IContextSource
{
    public string? LastApplied;

    public ContextRead Read()
    {
        var value = Fixture.SideStore["Features:Payouts:Side"];
        return new ContextRead(value, value is null ? null : new ContextSource("side-store", "before azimuth"));
    }

    public void Apply(string text)
    {
        LastApplied = text;
        Fixture.SideStore["Features:Payouts:Side"] = text;
    }
}
