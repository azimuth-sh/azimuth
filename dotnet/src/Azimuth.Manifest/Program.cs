using System.Reflection;
using Azimuth.Manifest;

// Usage: azimuth-manifest --output <path> [--root <repo-root>] [--traced-root <ns-prefix>]... <assembly.dll> [more.dll ...]
//   --output        where the manifest JSON is written (required)
//   --root          repo root the emitted `file` paths are made relative to (defaults to CWD)
//   --traced-root   namespace prefix of an opt-in traced area; every test method under it must carry
//                   [Covers] or [Untraced] or it is emitted as untraced. Repeatable; none by default.

var arguments = ParseArguments(args);
if (arguments is null)
{
    Console.Error.WriteLine(
        "usage: azimuth-manifest --output <path> [--root <repo-root>] [--traced-root <ns-prefix>]... <assembly.dll> [<assembly.dll> ...]");
    return 1;
}

var (outputPath, root, tracedRoots, assemblyPaths) = arguments.Value;

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

var manifest = ManifestEmitter.Emit(assemblies, outputPath, root, tracedRoots);
Console.WriteLine(
    $"wrote {manifest.Realizes.Count} realizes + {manifest.Covers.Count} covers + {manifest.UntracedTests.Count} untraced to {outputPath}");
return 0;

static (string Output, string Root, IReadOnlyList<string> TracedRoots, IReadOnlyList<string> Assemblies)? ParseArguments(string[] args)
{
    string? output = null;
    var root = Directory.GetCurrentDirectory();
    var tracedRoots = new List<string>();
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
            case "--traced-root":
                if (++i >= args.Length) return null;
                tracedRoots.Add(args[i]);
                break;
            default:
                assemblies.Add(args[i]);
                break;
        }
    }

    return output is null || assemblies.Count == 0 ? null : (output, root, tracedRoots, assemblies);
}
