using System.Text.Json.Nodes;
using System.Xml.Linq;

namespace Azimuth.Xunit;

/// <summary>The verb <c>discover</c>: the xUnit test projects under a folder, for <c>azimuth init</c>.</summary>
public static class Discovery
{
    // Folders that hold build output or other repositories' content, never a project of this one.
    private static readonly HashSet<string> SkippedFolders = new(StringComparer.OrdinalIgnoreCase) { "bin", "obj", "node_modules" };

    public static JsonObject Report(string folder)
    {
        var start = Path.GetFullPath(folder);
        var root = Directory.GetCurrentDirectory();
        var projects = new JsonArray();
        foreach (var project in ProjectFiles(start).Where(ProjectFile.ReferencesXunit).Order(StringComparer.Ordinal))
        {
            var path = Path.GetRelativePath(root, Path.GetDirectoryName(project)!).Replace(Path.DirectorySeparatorChar, '/');
            projects.Add(new JsonObject { ["path"] = path });
        }
        return new JsonObject { ["projects"] = projects };
    }

    private static IEnumerable<string> ProjectFiles(string folder)
    {
        foreach (var file in Directory.EnumerateFiles(folder).Where(ProjectFile.IsProjectFile))
        {
            yield return file;
        }
        foreach (var child in Directory.EnumerateDirectories(folder))
        {
            var name = Path.GetFileName(child);
            if (name.StartsWith('.') || SkippedFolders.Contains(name))
            {
                continue;
            }
            foreach (var file in ProjectFiles(child))
            {
                yield return file;
            }
        }
    }
}

/// <summary>What the adapter reads from a project file without building it.</summary>
public static class ProjectFile
{
    private static readonly string[] Endings = [".csproj", ".fsproj", ".vbproj"];
    private static readonly HashSet<string> XunitPackages = new(StringComparer.OrdinalIgnoreCase) { "xunit", "xunit.v3" };

    public static bool IsProjectFile(string path) => Endings.Any(ending => path.EndsWith(ending, StringComparison.OrdinalIgnoreCase));

    public static bool ReferencesXunit(string path)
    {
        try
        {
            return XDocument.Load(path).Descendants()
                .Where(element => element.Name.LocalName == "PackageReference")
                .Any(element => XunitPackages.Contains((string?)element.Attribute("Include") ?? ""));
        }
        catch (System.Xml.XmlException)
        {
            return false;
        }
    }

    /// <summary>The one project file in the suite's folder.</summary>
    public static string InFolder(string folder)
    {
        var files = Directory.EnumerateFiles(folder).Where(IsProjectFile).ToList();
        return files.Count switch
        {
            1 => files[0],
            0 => throw new AdapterException($"the suite folder {folder} holds no project file"),
            _ => throw new AdapterException($"the suite folder {folder} holds {files.Count} project files; a suite is one test project"),
        };
    }
}
