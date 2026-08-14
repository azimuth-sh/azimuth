using System.Collections.Immutable;
using System.Reflection;
using System.Runtime.CompilerServices;
using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;
using System.Security.Cryptography;
using System.Text;

namespace Azimuth.Emit;

/// <summary>
/// Maps a method back to the source file it was compiled from, by reading the assembly's portable
/// PDB.
/// </summary>
/// <remarks>
/// The <c>file</c> field is what makes a finding navigable — a dangling tag reported against
/// "some assembly" is a finding nobody acts on. Best-effort: an assembly built without a portable
/// PDB yields no path, and the emitter says so rather than inventing one.
/// </remarks>
internal sealed class SourceFiles : IDisposable
{
    private const BindingFlags DeclaredMembers = BindingFlags.Public
        | BindingFlags.NonPublic
        | BindingFlags.Instance
        | BindingFlags.Static
        | BindingFlags.DeclaredOnly;
    private readonly MetadataReaderProvider? _provider;
    private readonly MetadataReader? _reader;
    private readonly string _root;

    private SourceFiles(MetadataReaderProvider? provider, MetadataReader? reader, string root)
    {
        _provider = provider;
        _reader = reader;
        _root = root;
    }

    public bool HasSymbols => _reader is not null;

    public static SourceFiles ForAssembly(Assembly assembly, string root)
    {
        var location = assembly.Location;
        if (string.IsNullOrEmpty(location))
        {
            return new SourceFiles(null, null, root);
        }

        var pdbPath = Path.ChangeExtension(location, ".pdb");
        if (!File.Exists(pdbPath))
        {
            return new SourceFiles(null, null, root);
        }

        try
        {
            var stream = File.OpenRead(pdbPath);
            var provider = MetadataReaderProvider.FromPortablePdbStream(stream);
            return new SourceFiles(provider, provider.GetMetadataReader(), root);
        }
        catch (Exception)
        {
            // A native or Windows PDB, or a truncated file. Degrade to no paths rather than
            // failing the emit: a manifest without file paths is still a correct manifest.
            return new SourceFiles(null, null, root);
        }
    }

    public string PathOf(MethodBase method)
    {
        var direct = Lookup(method);
        if (direct.Length > 0)
        {
            return direct;
        }

        // An async or iterator method's sequence points live on the compiler-generated state
        // machine's MoveNext, not on the method the tag sits on. Most tagged methods in a service
        // are async, so without this the manifest carries almost no paths — which is invisible
        // until something depends on them.
        var stateMachine = StateMachineOf(method);
        if (stateMachine is null)
        {
            return string.Empty;
        }

        var moveNext = stateMachine.GetMethod(
            "MoveNext",
            BindingFlags.Instance | BindingFlags.NonPublic | BindingFlags.Public);
        return moveNext is null ? string.Empty : Lookup(moveNext);
    }

    public string FingerprintOf(MethodBase method)
    {
        var direct = Fingerprint(method);
        if (direct.Length > 0)
        {
            return direct;
        }

        var stateMachine = StateMachineOf(method);
        var moveNext = stateMachine?.GetMethod(
            "MoveNext",
            BindingFlags.Instance | BindingFlags.NonPublic | BindingFlags.Public);
        return moveNext is null ? string.Empty : Fingerprint(moveNext);
    }

    public string FingerprintOf(Type type)
    {
        var fingerprints = SourceMembersOf(type)
            .Select(FingerprintOf)
            .Where(value => value.Length > 0)
            .OrderBy(value => value, StringComparer.Ordinal)
            .ToArray();
        return fingerprints.Length == 0 ? string.Empty : Hash(string.Join("\n", fingerprints));
    }

    private string Lookup(MethodBase method)
    {
        if (_reader is null)
        {
            return string.Empty;
        }

        try
        {
            // A MethodDebugInformation row corresponds 1:1 with a MethodDef row, and the handle is
            // built from the *row number* — not the full token. Passing the token silently yielded
            // no path for most methods, which the fingerprint work surfaced: judgments would have
            // been fingerprinted over an empty evidence set and never gone stale.
            var row = method.MetadataToken & 0x00FFFFFF;
            var handle = MetadataTokens.MethodDebugInformationHandle(row);
            var info = _reader.GetMethodDebugInformation(handle);
            if (info.Document.IsNil)
            {
                return string.Empty;
            }

            var document = _reader.GetDocument(info.Document);
            var name = _reader.GetString(document.Name);
            return Relative(name);
        }
        catch (Exception)
        {
            return string.Empty;
        }
    }

    private string Fingerprint(MethodBase method)
    {
        if (_reader is null)
        {
            return string.Empty;
        }

        try
        {
            var row = method.MetadataToken & 0x00FFFFFF;
            var info = _reader.GetMethodDebugInformation(
                MetadataTokens.MethodDebugInformationHandle(row));
            var points = info.GetSequencePoints()
                .Where(point => !point.IsHidden)
                .ToArray();
            if (points.Length == 0)
            {
                return string.Empty;
            }

            var documentHandle = points.First(point => !point.Document.IsNil).Document;
            var relevant = points.Where(point => point.Document == documentHandle).ToArray();
            var startLine = relevant.Min(point => point.StartLine);
            var endLine = relevant.Max(point => point.EndLine);
            var document = _reader.GetDocument(documentHandle);
            var sourcePath = _reader.GetString(document.Name);
            var readablePath = File.Exists(sourcePath)
                ? sourcePath
                : Path.Combine(_root, Relative(sourcePath));
            if (!File.Exists(readablePath))
            {
                return string.Empty;
            }

            var lines = File.ReadAllLines(readablePath);
            if (startLine < 1 || endLine < startLine || startLine > lines.Length)
            {
                return string.Empty;
            }
            var count = Math.Min(endLine, lines.Length) - startLine + 1;
            return Hash(string.Join("\n", lines.Skip(startLine - 1).Take(count)));
        }
        catch (Exception)
        {
            return string.Empty;
        }
    }

    private static string Hash(string source) =>
        Convert.ToHexString(SHA256.HashData(Encoding.UTF8.GetBytes(source))).ToLowerInvariant();

    private static Type? StateMachineOf(MethodBase method)
    {
        try
        {
            return method.GetCustomAttribute<AsyncStateMachineAttribute>()?.StateMachineType
                ?? method.GetCustomAttribute<IteratorStateMachineAttribute>()?.StateMachineType;
        }
        catch (Exception error) when (error is TypeLoadException
            or FileNotFoundException
            or FileLoadException)
        {
            // Source navigation is optional. A package type in the generated machine must not turn
            // an otherwise readable assembly into a missing manifest.
            try
            {
                return method.DeclaringType?.GetNestedTypes(BindingFlags.Public | BindingFlags.NonPublic)
                    .FirstOrDefault(type => type.Name.StartsWith(
                        $"<{method.Name}>d__",
                        StringComparison.Ordinal));
            }
            catch (Exception nestedError) when (nestedError is TypeLoadException
                or FileNotFoundException
                or FileLoadException)
            {
                return null;
            }
        }
    }

    /// <summary>A type's path is taken from its first declared source member that has one.</summary>
    public string PathOf(Type type)
    {
        foreach (var method in SourceMembersOf(type))
        {
            var path = PathOf(method);
            if (path.Length > 0)
            {
                return path;
            }
        }

        return string.Empty;
    }

    private static IEnumerable<MethodBase> SourceMembersOf(Type type) =>
        type.GetMethods(DeclaredMembers).Cast<MethodBase>()
            .Concat(type.GetConstructors(DeclaredMembers));

    private string Relative(string absolute)
    {
        var normalized = absolute.Replace('\\', '/');
        var root = _root.Replace('\\', '/').TrimEnd('/');
        if (root.Length > 0 && normalized.StartsWith(root + "/", StringComparison.Ordinal))
        {
            return normalized[(root.Length + 1)..];
        }

        return normalized;
    }

    public void Dispose() => _provider?.Dispose();

    public static ImmutableArray<string> Empty => ImmutableArray<string>.Empty;
}
