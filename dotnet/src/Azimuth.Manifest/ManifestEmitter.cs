using System.Reflection;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace Azimuth.Manifest;

/// <summary>
/// Collects tags from assemblies and serializes them to a manifest JSON matching
/// <c>schema/manifest.schema.json</c>. The static API is the reusable seam; <see cref="Program"/>
/// wraps it as a console tool.
/// </summary>
public static class ManifestEmitter
{
    private static readonly JsonSerializerOptions JsonOptions = new()
    {
        WriteIndented = true,
        DefaultIgnoreCondition = JsonIgnoreCondition.Never,
    };

    public static string ToJson(Manifest manifest) => JsonSerializer.Serialize(manifest, JsonOptions);

    public static Manifest Emit(
        IEnumerable<Assembly> assemblies,
        string outputPath,
        string? root = null,
        IReadOnlyList<string>? tracedRoots = null)
    {
        var manifest = ManifestCollector.Collect(assemblies, root, tracedRoots);
        var directory = Path.GetDirectoryName(Path.GetFullPath(outputPath));
        if (!string.IsNullOrEmpty(directory))
        {
            Directory.CreateDirectory(directory);
        }

        File.WriteAllText(outputPath, ToJson(manifest));
        return manifest;
    }
}
