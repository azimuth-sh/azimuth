using System.Reflection;
using Azimuth.Manifest;

// Usage: azimuth-manifest --output <path> [--root <repo-root>] <assembly.dll> [more.dll ...]
//   --output  where the manifest JSON is written (required)
//   --root    repo root the emitted `file` paths are made relative to (defaults to CWD)

var arguments = ParseArguments(args);
if (arguments is null)
{
    Console.Error.WriteLine(
        "usage: azimuth-manifest --output <path> [--root <repo-root>] <assembly.dll> [<assembly.dll> ...]");
    return 1;
}

var (outputPath, root, assemblyPaths) = arguments.Value;

var assemblies = new List<Assembly>();
foreach (var path in assemblyPaths)
{
    var full = Path.GetFullPath(path);
    if (!File.Exists(full))
    {
        Console.Error.WriteLine($"assembly not found: {path}");
        return 1;
    }

    assemblies.Add(Assembly.LoadFrom(full));
}

var manifest = ManifestEmitter.Emit(assemblies, outputPath, root);
Console.WriteLine(
    $"wrote {manifest.Realizes.Count} realizes + {manifest.Covers.Count} covers to {outputPath}");
return 0;

static (string Output, string Root, IReadOnlyList<string> Assemblies)? ParseArguments(string[] args)
{
    string? output = null;
    var root = Directory.GetCurrentDirectory();
    var assemblies = new List<string>();

    for (var i = 0; i < args.Length; i++)
    {
        switch (args[i])
        {
            case "--output" or "-o":
                if (++i >= args.Length) return null;
                output = args[i];
                break;
            case "--root" or "-r":
                if (++i >= args.Length) return null;
                root = Path.GetFullPath(args[i]);
                break;
            default:
                assemblies.Add(args[i]);
                break;
        }
    }

    return output is null || assemblies.Count == 0 ? null : (output, root, assemblies);
}
