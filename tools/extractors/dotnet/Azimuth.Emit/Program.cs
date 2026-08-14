using System.Reflection;
using System.Runtime.Loader;
using Azimuth.Emit;

// azimuth-emit-dotnet --output <path> [--root <dir>] <assembly.dll>...
//
// Reflects over built assemblies and writes the language-neutral manifest the core reads. Running
// after a build rather than scanning source is deliberate: reflection resolves inheritance and
// generics that a text scan gets wrong, and a tag the compiler rejected is not a tag.

var parsed = Options.Parse(args);
if (parsed is null)
{
    Console.Error.WriteLine(Options.Usage);
    return 2;
}

var options = parsed.Value;
var assemblies = new List<Assembly>();

foreach (var path in options.Assemblies)
{
    var full = Path.GetFullPath(path);
    if (!File.Exists(full))
    {
        Console.Error.WriteLine($"azimuth-emit: assembly not found: {path}");
        return 2;
    }

    // Resolve dependencies from beside each assembly, so a target that references packages the
    // emitter does not have still loads.
    var context = new AssemblyLoadContext($"azimuth-{Path.GetFileNameWithoutExtension(full)}");
    var resolver = new AssemblyDependencyResolver(full);
    context.Resolving += (_, name) =>
    {
        var resolved = resolver.ResolveAssemblyToPath(name);
        var adjacent = Path.Combine(Path.GetDirectoryName(full)!, $"{name.Name}.dll");
        if (resolved is null && File.Exists(adjacent))
        {
            resolved = adjacent;
        }
        return resolved is null ? null : context.LoadFromAssemblyPath(resolved);
    };
    assemblies.Add(context.LoadFromAssemblyPath(full));
}

var result = Collector.Collect(assemblies, options.Root);

foreach (var warning in result.Warnings)
{
    Console.Error.WriteLine($"warning: {warning}");
}

var directory = Path.GetDirectoryName(Path.GetFullPath(options.Output));
if (!string.IsNullOrEmpty(directory))
{
    Directory.CreateDirectory(directory);
}

File.WriteAllText(options.Output, Collector.ToJson(result));
Console.Error.WriteLine(
    $"{result.Realizes.Count} realizes, {result.Covers.Count} covers, "
    + $"{result.MechanismImplementations.Count} mechanism implementations, "
    + $"{result.MechanismCovers.Count} mechanism covers → {options.Output}");
return 0;

internal readonly record struct Options(
    string Output,
    string Root,
    IReadOnlyList<string> Assemblies)
{
    public const string Usage =
        "usage: azimuth-emit-dotnet --output <path> [--root <dir>] <assembly.dll>...\n"
        + "  --output       where the manifest is written\n"
        + "  --root         paths in the manifest are made relative to this (default: cwd)";

    public static Options? Parse(string[] args)
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
                case "--root":
                    if (++i >= args.Length) return null;
                    root = Path.GetFullPath(args[i]);
                    break;
                default:
                    if (args[i].StartsWith('-')) return null;
                    assemblies.Add(args[i]);
                    break;
            }
        }

        if (output is null || assemblies.Count == 0)
        {
            return null;
        }

        return new Options(output, root, assemblies);
    }
}
