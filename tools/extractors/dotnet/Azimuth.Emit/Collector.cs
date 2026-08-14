using System.Reflection;
using System.Text;
using System.Text.Json;

namespace Azimuth.Emit;

/// <summary>
/// Reflects over assemblies and collects their linkage tags into the language-neutral manifest the
/// core reads.
/// </summary>
/// <remarks>
/// Each ecosystem emits the manifest natively; the core only ever reads manifests. That seam is why
/// adding a language is a day's work rather than a fork of the core, and why the core can stay
/// dependency-free while this side does metadata work.
/// <para>
/// Attributes are matched by full <em>name</em> rather than CLR identity, so the emitter works when
/// the target assembly references a differently-located copy of Azimuth.Annotations.
/// </para>
/// </remarks>
internal static class Collector
{
    private const string Lang = "csharp";
    private const string RealizesName = "Azimuth.Annotations.RealizesAttribute";
    private const string CoversName = "Azimuth.Annotations.CoversAttribute";
    private const string ImplementsMechanismName =
        "Azimuth.Annotations.ImplementsMechanismAttribute";
    private const string CoversMechanismName = "Azimuth.Annotations.CoversMechanismAttribute";
    private const BindingFlags Members = BindingFlags.Public
        | BindingFlags.NonPublic
        | BindingFlags.Instance
        | BindingFlags.Static
        | BindingFlags.DeclaredOnly;

    public sealed record Entry(
        string Spec,
        string Scenario,
        string Site,
        string File,
        string SourceFingerprint,
        string? Scope,
        string? Quantification,
        string? Oracle);

    public sealed record Artifact(
        string Id,
        string Kind,
        string File,
        bool? Unique = null,
        IReadOnlyList<string>? Columns = null,
        string? Predicate = null);

    public sealed record MechanismImplementationEntry(
        string Spec,
        string Mechanism,
        string Binding,
        string File,
        string SourceFingerprint);

    public sealed record MechanismCoverEntry(
        string Spec,
        string Mechanism,
        string Site,
        string File,
        string SourceFingerprint,
        string? Scope,
        string? Quantification = null,
        string? Oracle = null);

    public sealed record Result(
        List<Entry> Realizes,
        List<Entry> Covers,
        List<MechanismImplementationEntry> MechanismImplementations,
        List<MechanismCoverEntry> MechanismCovers,
        List<Artifact> Artifacts,
        List<string> Warnings);

    public static Result Collect(
        IEnumerable<Assembly> assemblies,
        string root)
    {
        var result = new Result([], [], [], [], [], []);

        foreach (var assembly in assemblies)
        {
            using var files = SourceFiles.ForAssembly(assembly, root);
            if (!files.HasSymbols)
            {
                result.Warnings.Add(
                    $"{assembly.GetName().Name}: no portable PDB beside the assembly, so tags "
                    + "carry no file path and findings will not be navigable");
            }

            foreach (var type in Types(assembly, result.Warnings))
            {
                CollectType(type, files, result);
            }
        }

        result.Realizes.Sort(Compare);
        result.Covers.Sort(Compare);
        result.MechanismImplementations.Sort(CompareMechanismImplementation);
        result.MechanismCovers.Sort(CompareMechanismCover);
        result.Artifacts.Sort((a, b) => string.CompareOrdinal(a.Id, b.Id));
        for (var index = result.Artifacts.Count - 1; index > 0; index--)
        {
            if (result.Artifacts[index].Id == result.Artifacts[index - 1].Id)
            {
                result.Artifacts.RemoveAt(index);
            }
        }
        return result;
    }

    private static IEnumerable<Type> Types(Assembly assembly, List<string> warnings)
    {
        try
        {
            return assembly.GetTypes();
        }
        catch (ReflectionTypeLoadException e)
        {
            warnings.Add(
                $"{assembly.GetName().Name}: {e.LoaderExceptions.Length} type(s) failed to load "
                + "and were skipped; tags on them are missing from this manifest");
            return e.Types.Where(t => t is not null)!;
        }
    }

    private static void CollectType(
        Type type,
        SourceFiles files,
        Result result)
    {
        if (type.GetCustomAttributesData().Any(attribute =>
                attribute.AttributeType.FullName ==
                "System.Runtime.CompilerServices.CompilerGeneratedAttribute"))
        {
            return;
        }

        var typeName = SiteName(type);
        result.Artifacts.Add(
            new Artifact($"dotnet-symbol:{typeName}", "dotnet-type", files.PathOf(type)));

        foreach (var attribute in type.GetCustomAttributesData())
        {
            var name = FullName(attribute);
            if (name == RealizesName)
            {
                var (spec, scenario) = Pair(attribute);
                result.Realizes.Add(
                    new Entry(
                        spec,
                        scenario,
                        type.FullName ?? type.Name,
                        files.PathOf(type),
                        files.FingerprintOf(type),
                        null,
                        null,
                        null));
            }
            else if (name == ImplementsMechanismName)
            {
                var (spec, mechanism) = Pair(attribute);
                result.MechanismImplementations.Add(
                    new MechanismImplementationEntry(
                        spec,
                        mechanism,
                        $"dotnet-symbol:{typeName}",
                        files.PathOf(type),
                        files.FingerprintOf(type)));
            }
        }

        foreach (var method in type.GetMethods(Members))
        {
            if (method.IsSpecialName)
            {
                continue;
            }
            var site = $"{typeName}.{method.Name}";
            var file = files.PathOf(method);
            var sourceFingerprint = files.FingerprintOf(method);
            result.Artifacts.Add(
                new Artifact($"dotnet-symbol:{site}", "dotnet-method", file));
            var data = method.GetCustomAttributesData();
            foreach (var attribute in data)
            {
                var name = FullName(attribute);
                if (name == RealizesName)
                {
                    var (spec, scenario) = Pair(attribute);
                    result.Realizes.Add(
                        new Entry(spec, scenario, site, file, sourceFingerprint, null, null, null));
                }
                else if (name == CoversName)
                {
                    result.Covers.Add(Covers(attribute, site, file, sourceFingerprint));
                }
                else if (name == ImplementsMechanismName)
                {
                    var (spec, mechanism) = Pair(attribute);
                    result.MechanismImplementations.Add(
                        new MechanismImplementationEntry(
                            spec,
                            mechanism,
                            $"dotnet-symbol:{site}",
                            file,
                            sourceFingerprint));
                }
                else if (name == CoversMechanismName)
                {
                    result.MechanismCovers.Add(
                        CoversMechanism(attribute, site, file, sourceFingerprint));
                }
            }
        }

        CollectIndexes(type, files, result);
    }

    private static string SiteName(Type type) =>
        (type.FullName ?? type.Name).Replace('+', '.');

    private static void CollectIndexes(Type type, SourceFiles files, Result result)
    {
        if (!Inherits(type, "Microsoft.EntityFrameworkCore.Migrations.Migration"))
        {
            return;
        }

        try
        {
            var migration = Activator.CreateInstance(type);
            var builderType = type.BaseType!.Assembly.GetType(
                "Microsoft.EntityFrameworkCore.Migrations.MigrationBuilder",
                throwOnError: true)!;
            var builder = Activator.CreateInstance(
                builderType,
                ["Npgsql.EntityFrameworkCore.PostgreSQL"])!;
            type.GetMethod("Up", BindingFlags.Instance | BindingFlags.NonPublic)!
                .Invoke(migration, [builder]);
            var operations = (System.Collections.IEnumerable)builderType
                .GetProperty("Operations")!
                .GetValue(builder)!;

            foreach (var operation in operations)
            {
                var operationType = operation.GetType();
                if (operationType.FullName !=
                    "Microsoft.EntityFrameworkCore.Migrations.Operations.CreateIndexOperation")
                {
                    continue;
                }

                var table = (string)operationType.GetProperty("Table")!.GetValue(operation)!;
                var name = (string)operationType.GetProperty("Name")!.GetValue(operation)!;
                var columns = (string[])operationType.GetProperty("Columns")!.GetValue(operation)!;
                var unique = (bool)operationType.GetProperty("IsUnique")!.GetValue(operation)!;
                var predicate = operationType.GetProperty("Filter")!.GetValue(operation) as string;
                result.Artifacts.Add(new Artifact(
                    $"postgres-index:{table}.{name}",
                    "database-index",
                    files.PathOf(type),
                    unique,
                    columns,
                    predicate));
            }
        }
        catch (Exception error)
        {
            result.Warnings.Add(
                $"{type.FullName}: migration metadata could not be enumerated: "
                + $"{error.GetBaseException().Message}");
        }
    }

    private static bool Inherits(Type type, string baseType)
    {
        for (var current = type.BaseType; current is not null; current = current.BaseType)
        {
            if (current.FullName == baseType)
            {
                return true;
            }
        }

        return false;
    }

    private static string FullName(CustomAttributeData attribute) =>
        attribute.AttributeType.FullName ?? string.Empty;

    private static (string Spec, string Scenario) Pair(CustomAttributeData attribute)
    {
        var args = attribute.ConstructorArguments;
        var spec = args.Count > 0 ? args[0].Value as string ?? string.Empty : string.Empty;
        var scenario = args.Count > 1 ? args[1].Value as string ?? string.Empty : string.Empty;
        return (spec, scenario);
    }

    private static Entry Covers(
        CustomAttributeData attribute,
        string site,
        string file,
        string sourceFingerprint)
    {
        var args = attribute.ConstructorArguments;
        var (spec, scenario) = Pair(attribute);
        return new Entry(
            spec,
            scenario,
            site,
            file,
            sourceFingerprint,
            args.Count > 2 ? EnumName(args[2]) : null,
            args.Count > 3 ? EnumName(args[3]) : null,
            args.Count > 4 ? EnumName(args[4]) : null);
    }

    private static MechanismCoverEntry CoversMechanism(
        CustomAttributeData attribute,
        string site,
        string file,
        string sourceFingerprint)
    {
        var args = attribute.ConstructorArguments;
        var (spec, mechanism) = Pair(attribute);
        return new MechanismCoverEntry(
            spec,
            mechanism,
            site,
            file,
            sourceFingerprint,
            args.Count > 2 ? EnumName(args[2]) : null,
            args.Count > 3 ? EnumName(args[3]) : null,
            args.Count > 4 ? EnumName(args[4]) : null);
    }

    /// <summary>
    /// Enum arguments arrive through <c>CustomAttributeData</c> as their boxed underlying integer,
    /// so the name is resolved against the declared argument type rather than read off the value.
    /// </summary>
    private static string? EnumName(CustomAttributeTypedArgument argument)
    {
        if (argument.Value is null)
        {
            return null;
        }

        var type = argument.ArgumentType;
        if (!type.IsEnum)
        {
            return Kebab(argument.Value.ToString());
        }

        var name = Enum.GetName(type, argument.Value);
        return name is null ? null : Kebab(name);
    }

    /// <summary>
    /// The manifest speaks kebab-case because the specs do. A format that spoke two casings would
    /// invite exactly one bug, at the boundary, in the field nobody re-reads.
    /// </summary>
    private static string? Kebab(string? value)
    {
        if (value is null)
        {
            return null;
        }

        var builder = new StringBuilder();
        foreach (var (c, index) in value.Select((c, i) => (c, i)))
        {
            if (char.IsUpper(c) && index > 0)
            {
                builder.Append('-');
            }

            builder.Append(char.ToLowerInvariant(c));
        }

        return builder.ToString();
    }

    private static int Compare(Entry a, Entry b)
    {
        var bySpec = string.CompareOrdinal(a.Spec, b.Spec);
        if (bySpec != 0)
        {
            return bySpec;
        }

        var byScenario = string.CompareOrdinal(a.Scenario, b.Scenario);
        return byScenario != 0 ? byScenario : string.CompareOrdinal(a.Site, b.Site);
    }

    private static int CompareMechanismImplementation(
        MechanismImplementationEntry a,
        MechanismImplementationEntry b)
    {
        var bySpec = string.CompareOrdinal(a.Spec, b.Spec);
        if (bySpec != 0)
        {
            return bySpec;
        }

        var byMechanism = string.CompareOrdinal(a.Mechanism, b.Mechanism);
        return byMechanism != 0 ? byMechanism : string.CompareOrdinal(a.Binding, b.Binding);
    }

    private static int CompareMechanismCover(MechanismCoverEntry a, MechanismCoverEntry b)
    {
        var bySpec = string.CompareOrdinal(a.Spec, b.Spec);
        if (bySpec != 0)
        {
            return bySpec;
        }

        var byMechanism = string.CompareOrdinal(a.Mechanism, b.Mechanism);
        return byMechanism != 0 ? byMechanism : string.CompareOrdinal(a.Site, b.Site);
    }

    public static string ToJson(Result result)
    {
        var options = new JsonWriterOptions { Indented = true };
        using var stream = new MemoryStream();
        using (var writer = new Utf8JsonWriter(stream, options))
        {
            writer.WriteStartObject();
            WriteEntries(writer, "realizes", result.Realizes, form: false);
            WriteEntries(writer, "covers", result.Covers, form: true);
            WriteMechanismImplementations(writer, result.MechanismImplementations);
            WriteMechanismCovers(writer, result.MechanismCovers);
            WriteArtifacts(writer, result.Artifacts);
            writer.WriteEndObject();
        }

        return Encoding.UTF8.GetString(stream.ToArray()) + "\n";
    }

    private static void WriteArtifacts(Utf8JsonWriter writer, IReadOnlyList<Artifact> artifacts)
    {
        writer.WriteStartArray("artifacts");
        foreach (var artifact in artifacts)
        {
            writer.WriteStartObject();
            writer.WriteString("id", artifact.Id);
            writer.WriteString("kind", artifact.Kind);
            writer.WriteString("file", artifact.File);
            if (artifact.Unique is bool unique)
            {
                writer.WriteBoolean("unique", unique);
            }
            if (artifact.Columns is { } columns)
            {
                writer.WriteStartArray("columns");
                foreach (var column in columns)
                {
                    writer.WriteStringValue(column);
                }
                writer.WriteEndArray();
            }
            if (artifact.Predicate is { } predicate)
            {
                writer.WriteString("predicate", predicate);
            }
            writer.WriteEndObject();
        }
        writer.WriteEndArray();
    }

    private static void WriteEntries(
        Utf8JsonWriter writer,
        string name,
        List<Entry> entries,
        bool form)
    {
        writer.WriteStartArray(name);
        foreach (var entry in entries)
        {
            writer.WriteStartObject();
            writer.WriteString("spec", entry.Spec);
            writer.WriteString("scenario", entry.Scenario);
            writer.WriteString("site", entry.Site);
            writer.WriteString("file", entry.File);
            writer.WriteString("lang", Lang);
            if (entry.SourceFingerprint.Length > 0)
            {
                writer.WriteString("source_fingerprint", entry.SourceFingerprint);
            }
            if (form)
            {
                if (entry.Scope is not null)
                {
                    writer.WriteString("scope", entry.Scope);
                }

                if (entry.Quantification is not null)
                {
                    writer.WriteString("quantification", entry.Quantification);
                }

                if (entry.Oracle is not null)
                {
                    writer.WriteString("oracle", entry.Oracle);
                }
            }

            writer.WriteEndObject();
        }

        writer.WriteEndArray();
    }

    private static void WriteMechanismImplementations(
        Utf8JsonWriter writer,
        List<MechanismImplementationEntry> entries)
    {
        writer.WriteStartArray("mechanism_implementations");
        foreach (var entry in entries)
        {
            writer.WriteStartObject();
            writer.WriteString("spec", entry.Spec);
            writer.WriteString("mechanism", entry.Mechanism);
            writer.WriteString("binding", entry.Binding);
            writer.WriteString("file", entry.File);
            writer.WriteString("lang", Lang);
            if (entry.SourceFingerprint.Length > 0)
            {
                writer.WriteString("source_fingerprint", entry.SourceFingerprint);
            }
            writer.WriteEndObject();
        }
        writer.WriteEndArray();
    }

    private static void WriteMechanismCovers(
        Utf8JsonWriter writer,
        List<MechanismCoverEntry> entries)
    {
        writer.WriteStartArray("mechanism_covers");
        foreach (var entry in entries)
        {
            writer.WriteStartObject();
            writer.WriteString("spec", entry.Spec);
            writer.WriteString("mechanism", entry.Mechanism);
            writer.WriteString("site", entry.Site);
            writer.WriteString("file", entry.File);
            writer.WriteString("lang", Lang);
            if (entry.SourceFingerprint.Length > 0)
            {
                writer.WriteString("source_fingerprint", entry.SourceFingerprint);
            }
            if (entry.Scope is not null)
            {
                writer.WriteString("scope", entry.Scope);
            }
            if (entry.Quantification is not null)
            {
                writer.WriteString("quantification", entry.Quantification);
            }
            if (entry.Oracle is not null)
            {
                writer.WriteString("oracle", entry.Oracle);
            }
            writer.WriteEndObject();
        }
        writer.WriteEndArray();
    }
}
