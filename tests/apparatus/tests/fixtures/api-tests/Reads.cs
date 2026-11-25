using System.Text.Json;
using System.Text.Json.Nodes;

namespace Payouts.Api.Tests;

// The code a team's tests carry to give the adapter the reads of `api` and `website`. It stands for a
// team's fixture and is test support: it uses the folder the adapter names in AZIMUTH_READS, finds the
// components in `components.json`, and writes one JSON file per read.
public static class Reads
{
    public static string Folder =>
        Environment.GetEnvironmentVariable("AZIMUTH_READS") ?? throw new InvalidOperationException("AZIMUTH_READS is not set");

    public static string ComponentsFile => Path.Combine(Folder, "components.json");

    public static string[] ComponentNames()
    {
        var names = new List<string>();
        Collect(JsonNode.Parse(File.ReadAllText(ComponentsFile)), names);
        return names.ToArray();
    }

    private static void Collect(JsonNode? node, List<string> names)
    {
        switch (node)
        {
            case JsonArray array:
                foreach (var item in array) Collect(item, names);
                break;
            case JsonObject { } obj when obj["name"] is JsonValue name:
                names.Add(name.GetValue<string>());
                break;
        }
    }

    public static JsonObject Document(string component, string note) => new()
    {
        ["format"] = 1,
        ["component"] = component,
        ["applying"] = "possible",
        ["note"] = note,
    };

    public static void Report(string file, string name, string phase, JsonNode report) =>
        Write(file, new JsonObject { ["name"] = name, ["phase"] = phase, ["report"] = report });

    public static void Error(string file, string name, string phase, string error) =>
        Write(file, new JsonObject { ["name"] = name, ["phase"] = phase, ["error"] = error });

    public static void Write(string file, JsonNode read) =>
        File.WriteAllText(Path.Combine(Folder, file), read.ToJsonString(new JsonSerializerOptions { WriteIndented = false }));

    public static void WriteText(string file, string text) => File.WriteAllText(Path.Combine(Folder, file), text);
}
