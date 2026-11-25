using System.Text;
using System.Text.Json;
using System.Text.Json.Nodes;

namespace Azimuth.Xunit;

/// <summary>
/// The folder one launch gives the code of the tests for its reads: empty at the start, described
/// by <c>components.json</c>, and emptied of its files into the report's <c>components</c> at the end.
/// </summary>
public sealed class ReadsFolder : IDisposable
{
    public const string EnvironmentVariable = "AZIMUTH_READS";
    private const string ComponentsFile = "components.json";

    private static readonly UTF8Encoding Strict = new(encoderShouldEmitUTF8Identifier: false, throwOnInvalidBytes: true);

    private readonly DirectoryInfo directory = Directory.CreateTempSubdirectory("azimuth-reads-");
    private readonly List<string> requested = [];

    public string Path => directory.FullName;

    public static ReadsFolder For(JsonArray? components)
    {
        var folder = new ReadsFolder();
        var listed = new JsonArray();
        foreach (var component in components ?? [])
        {
            listed.Add(new JsonObject { ["name"] = (string?)component?["name"], ["report"] = (string?)component?["report"] });
            if ((string?)component?["name"] is { } name)
            {
                folder.requested.Add(name);
            }
        }
        File.WriteAllText(System.IO.Path.Combine(folder.Path, ComponentsFile), listed.ToJsonString(), Strict);
        return folder;
    }

    /// <summary>
    /// Every read file in the folder, grouped by component name: the components of the request first
    /// in their order, then the names no component of the request has. A file that is not a read
    /// stops the launch.
    /// </summary>
    public JsonArray Collect()
    {
        var byName = new Dictionary<string, JsonArray>();
        var files = directory.EnumerateFiles().Where(file => file.Name != ComponentsFile).OrderBy(file => file.Name, StringComparer.Ordinal);
        foreach (var file in files)
        {
            var (name, read) = Read(file);
            if (!byName.TryGetValue(name, out var reads))
            {
                byName[name] = reads = [];
            }
            reads.Add(read);
        }

        var names = requested.Where(byName.ContainsKey).Distinct().Concat(byName.Keys.Except(requested).Order(StringComparer.Ordinal));
        var entries = new JsonArray();
        foreach (var name in names)
        {
            entries.Add(new JsonObject { ["name"] = name, ["reads"] = byName[name] });
        }
        return entries;
    }

    private static (string Name, JsonObject Read) Read(FileInfo file)
    {
        JsonObject document;
        try
        {
            document = JsonNode.Parse(Strict.GetString(File.ReadAllBytes(file.FullName))) as JsonObject
                ?? throw new AdapterException($"the file {file.Name} in the folder for reads is not a read: it is not a JSON object");
        }
        catch (Exception error) when (error is JsonException or DecoderFallbackException)
        {
            throw new AdapterException($"the file {file.Name} in the folder for reads is not a read: it is not JSON text");
        }

        if (document["name"] is not JsonValue nameValue || !nameValue.TryGetValue<string>(out var name))
        {
            throw new AdapterException($"the file {file.Name} in the folder for reads is not a read: it has no `name`");
        }
        if (document["phase"] is not JsonValue phaseValue || !phaseValue.TryGetValue<string>(out var phase))
        {
            throw new AdapterException($"the file {file.Name} in the folder for reads is not a read: it has no `phase`");
        }

        var read = new JsonObject { ["phase"] = phase };
        if (document["report"] is { } report)
        {
            read["report"] = report.DeepClone();
        }
        if (document["error"] is { } cause)
        {
            read["error"] = cause.DeepClone();
        }
        return (name, read);
    }

    public void Dispose() => directory.Delete(recursive: true);
}
