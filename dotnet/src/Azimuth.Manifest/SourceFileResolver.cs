using System.Collections.Immutable;
using System.Reflection;
using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;
using System.Reflection.PortableExecutable;

namespace Azimuth.Manifest;

/// <summary>
/// Best-effort source-file resolution for a tagged site, read from the assembly's portable PDB
/// (standalone <c>.pdb</c> next to the dll, or one embedded in the PE). Maps each method's metadata
/// row to the document its first sequence point points at. When no PDB is present the file is
/// simply unknown — the emitter falls back to an empty path rather than failing.
/// </summary>
public sealed class SourceFileResolver : IDisposable
{
    private readonly MetadataReaderProvider? _pdbProvider;
    private readonly PEReader? _peReader;
    private readonly Dictionary<int, string> _fileByMethodRow = new();

    private SourceFileResolver(MetadataReaderProvider? pdbProvider, PEReader? peReader)
    {
        _pdbProvider = pdbProvider;
        _peReader = peReader;
        if (pdbProvider is not null)
        {
            Index(pdbProvider.GetMetadataReader());
        }
    }

    public static SourceFileResolver ForAssembly(Assembly assembly)
    {
        var location = TryGetLocation(assembly);
        return location is null ? Empty() : ForAssemblyFile(location);
    }

    public static SourceFileResolver ForAssemblyFile(string assemblyPath)
    {
        var standalonePdb = Path.ChangeExtension(assemblyPath, ".pdb");
        if (File.Exists(standalonePdb))
        {
            try
            {
                var stream = File.OpenRead(standalonePdb);
                return new SourceFileResolver(MetadataReaderProvider.FromPortablePdbStream(stream), null);
            }
            catch
            {
                // fall through to embedded / empty
            }
        }

        return FromEmbedded(assemblyPath) ?? Empty();
    }

    /// <summary>The source path of a method, relative to <paramref name="root"/> and slash-normalized.</summary>
    public string FileFor(MethodBase method, string? root)
    {
        return _fileByMethodRow.TryGetValue(MetadataTokens.GetRowNumber(MethodHandle(method)), out var file)
            ? Relativize(file, root)
            : string.Empty;
    }

    /// <summary>A type's file, taken from the first of its methods that carries debug info.</summary>
    public string FileFor(Type type, string? root)
    {
        foreach (var method in type.GetMethods(
                     BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.Instance | BindingFlags.Static |
                     BindingFlags.DeclaredOnly))
        {
            var file = FileFor(method, root);
            if (file.Length > 0)
            {
                return file;
            }
        }

        return string.Empty;
    }

    private void Index(MetadataReader reader)
    {
        foreach (var handle in reader.MethodDebugInformation)
        {
            var debugInfo = reader.GetMethodDebugInformation(handle);
            var file = FirstDocumentName(reader, debugInfo);
            if (file is not null)
            {
                _fileByMethodRow[MetadataTokens.GetRowNumber(handle.ToDefinitionHandle())] = file;
            }
        }
    }

    private static string? FirstDocumentName(MetadataReader reader, MethodDebugInformation debugInfo)
    {
        if (debugInfo.SequencePointsBlob.IsNil)
        {
            return null;
        }

        foreach (var point in debugInfo.GetSequencePoints())
        {
            if (point.IsHidden)
            {
                continue;
            }

            var document = reader.GetDocument(point.Document);
            return reader.GetString(document.Name);
        }

        return null;
    }

    private static MethodDefinitionHandle MethodHandle(MethodBase method) =>
        (MethodDefinitionHandle)MetadataTokens.Handle(method.MetadataToken);

    private static SourceFileResolver? FromEmbedded(string assemblyPath)
    {
        try
        {
            var peStream = File.OpenRead(assemblyPath);
            var peReader = new PEReader(peStream);
            foreach (var entry in peReader.ReadDebugDirectory())
            {
                if (entry.Type == DebugDirectoryEntryType.EmbeddedPortablePdb)
                {
                    return new SourceFileResolver(peReader.ReadEmbeddedPortablePdbDebugDirectoryData(entry), peReader);
                }
            }

            peReader.Dispose();
            peStream.Dispose();
        }
        catch
        {
            // no readable embedded PDB
        }

        return null;
    }

    private static string Relativize(string file, string? root)
    {
        var normalized = file.Replace('\\', '/');
        if (string.IsNullOrEmpty(root))
        {
            return normalized;
        }

        var normalizedRoot = root!.Replace('\\', '/').TrimEnd('/') + "/";
        return normalized.StartsWith(normalizedRoot, StringComparison.OrdinalIgnoreCase)
            ? normalized[normalizedRoot.Length..]
            : normalized;
    }

    private static string? TryGetLocation(Assembly assembly)
    {
        try
        {
            return string.IsNullOrEmpty(assembly.Location) ? null : assembly.Location;
        }
        catch
        {
            return null;
        }
    }

    private static SourceFileResolver Empty() => new(null, null);

    public void Dispose()
    {
        _pdbProvider?.Dispose();
        _peReader?.Dispose();
    }
}
