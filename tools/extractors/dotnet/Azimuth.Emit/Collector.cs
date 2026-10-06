using System.Reflection;
using System.Security.Cryptography;
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
    private const string ImplementsCheckName = "Azimuth.Annotations.ImplementsCheckAttribute";
    private const string ImplementsMechanismName =
        "Azimuth.Annotations.ImplementsMechanismAttribute";
    private const string SupportsVerificationElementName =
        "Azimuth.Annotations.SupportsVerificationElementAttribute";
    private const string DefinesVerificationCheckName =
        "Azimuth.Annotations.DefinesVerificationCheckAttribute";
    private const string ContributesCheckToCaseName =
        "Azimuth.Annotations.ContributesCheckToCaseAttribute";
    private const BindingFlags Members = BindingFlags.Public
        | BindingFlags.NonPublic
        | BindingFlags.Instance
        | BindingFlags.Static
        | BindingFlags.DeclaredOnly;

    public sealed record Entry(
        string Claim,
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
        string Mechanism,
        string Site,
        string Binding,
        string File,
        string SourceFingerprint);

    public sealed record CheckImplementationEntry(
        string Check,
        string Site,
        string File,
        string SourceFingerprint);

    public sealed record Result(
        List<Entry> Realizes,
        List<CheckImplementationEntry> CheckImplementations,
        List<MechanismImplementationEntry> MechanismImplementations,
        List<Artifact> Artifacts,
        List<string> Warnings)
    {
        public AccountSupportResult AccountSupport { get; } = new();
        public List<PackageProducer> Extensions { get; } = [];
    }

    public sealed record SupportArtifact(
        string Id,
        string Kind,
        string File,
        string Fingerprint,
        string Site);

    public sealed record ElementSupport(
        string Claim,
        string Element,
        IReadOnlyList<string> Artifacts,
        IReadOnlyList<string> DependsOn);

    public sealed record VerificationCheck(
        string Id,
        string Terminal,
        IReadOnlyList<string> Artifacts,
        IReadOnlyList<string> Elements);

    public sealed record CaseContribution(
        string Check,
        string Case,
        string Element,
        string Proposition);

    public sealed class AccountSupportResult
    {
        public List<SupportArtifact> Artifacts { get; } = [];
        public List<ElementSupport> Elements { get; } = [];
        public List<VerificationCheck> Checks { get; } = [];
        public List<CaseContribution> Contributions { get; } = [];
    }

    public sealed record PackageProducer(string Package, string Contract, string Entity, string Site, string File, string Lang, string SourceFingerprint, System.Text.Json.Nodes.JsonObject? Schema, IReadOnlyList<ProducerInput> Inputs);
    public sealed record ProducerInput(string File, string Fingerprint);

    private static void CollectPackage(MemberInfo member, IList<CustomAttributeData> attributes, string site, string file, string fingerprint, Result result, string root)
    {
        foreach (var attribute in attributes)
        {
            var name = FullName(attribute);
            var contract = name switch
            {
                "Azimuth.Surface.Annotations.SurfaceEnumeratorAttribute" => "surface-enumerator",
                "Azimuth.Surface.Annotations.SurfaceExpectationsAttribute" => "surface-expectations",
                "Azimuth.Network.Annotations.ImplementsProbeAttribute" => "probe-implementation",
                _ => null
            };
            if (contract is null) continue;
            var entity = EntityArgument(attribute, file, site);
            EnsureMechanismFingerprint(site, fingerprint, file);
            System.Text.Json.Nodes.JsonObject? schema = null;
            if (contract != "probe-implementation")
            {
                var method = member as MethodInfo;
                if (method is null && member is Type type)
                {
                    var candidates = type.GetMethods(BindingFlags.Public | BindingFlags.Instance | BindingFlags.Static | BindingFlags.DeclaredOnly)
                        .Where(candidate => candidate.Name == "Enumerate" && !candidate.IsGenericMethod).ToArray();
                    if (candidates.Length != 1) throw new InvalidOperationException($"{site}: producer class requires exactly one public Enumerate method");
                    method = candidates[0];
                }
                if (method is null) throw new InvalidOperationException($"{site}: unsupported producer site");
                var descriptor = PackageProducers.Schema(PackageProducers.MemberType(method.ReturnType));
                schema = new System.Text.Json.Nodes.JsonObject { ["fields"] = descriptor["fields"]!.DeepClone() };
            }
            var inputs = new List<ProducerInput>();
            var configuration = attribute.NamedArguments.FirstOrDefault(argument => argument.MemberName == "ConfigurationFile").TypedValue.Value as string;
            if (configuration is not null)
            {
                if (contract != "surface-expectations" || configuration.Length == 0 || Path.IsPathRooted(configuration) || configuration.Contains("\\") || configuration.Split('/').Any(segment => segment is "" or "." or ".."))
                    throw new InvalidOperationException($"{site}: ConfigurationFile requires a normalized repository-relative file");
                var path = Path.GetFullPath(configuration, Path.GetFullPath(root));
                if (!File.Exists(path)) throw new InvalidOperationException($"{site}: missing configuration input {configuration}");
                FileSystemInfo current = new DirectoryInfo(Path.GetFullPath(root));
                foreach (var segment in configuration.Split('/'))
                {
                    var next = Path.Combine(current.FullName, segment);
                    current = Directory.Exists(next) ? new DirectoryInfo(next) : new FileInfo(next);
                    if (current.LinkTarget is not null) throw new InvalidOperationException($"{site}: symlink configuration inputs are not supported");
                }
                inputs.Add(new ProducerInput(configuration, "sha256:" + Convert.ToHexString(SHA256.HashData(File.ReadAllBytes(path))).ToLowerInvariant()));
            }
            result.Extensions.Add(new PackageProducer(contract == "probe-implementation" ? "azimuth.network" : "azimuth.surface", contract, entity, site, file, Lang, fingerprint, schema, inputs));
        }
    }

    public static Result Collect(
        IEnumerable<Assembly> assemblies,
        string root,
        bool includeAccountSupport = false)
    {
        var result = new Result([], [], [], [], []);

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
                CollectType(type, files, result, root, includeAccountSupport);
            }
        }

        result.Extensions.Sort((left, right) => StringComparer.Ordinal.Compare($"{left.Package}|{left.Contract}|{left.Entity}|{left.Site}", $"{right.Package}|{right.Contract}|{right.Entity}|{right.Site}"));
        result.Realizes.Sort(Compare);
        result.CheckImplementations.Sort(CompareCheckImplementation);
        result.MechanismImplementations.Sort(CompareMechanismImplementation);
        result.Artifacts.Sort(CompareArtifact);
        for (var index = result.Artifacts.Count - 1; index > 0; index--)
        {
            if (result.Artifacts[index] == result.Artifacts[index - 1])
            {
                result.Artifacts.RemoveAt(index);
            }
        }
        NormalizeAccountSupport(result.AccountSupport);
        return result;
    }

    private static bool AvailableWorkspaceSource(string file, string root)
    {
        if (!WorkspaceRelative(file)) return false;
        var rootPath = Path.GetFullPath(root);
        var current = rootPath;
        var segments = file.Split('/');
        for (var index = 0; index < segments.Length; index++)
        {
            current = Path.Combine(current, segments[index]);
            FileSystemInfo entry = index == segments.Length - 1
                ? new FileInfo(current) : new DirectoryInfo(current);
            if (!entry.Exists) return false;
            if (entry.LinkTarget is not null)
            {
                current = entry.ResolveLinkTarget(true)?.FullName ?? string.Empty;
                if (current.Length == 0 || !WorkspaceRelative(Path.GetRelativePath(rootPath, current)))
                    return false;
            }
        }
        return File.Exists(current);
    }

    private static void EnsureSourceRoot(IList<CustomAttributeData> attributes, string file, string site, string root)
    {
        if (attributes.Any(attribute => FullName(attribute).StartsWith("Azimuth.", StringComparison.Ordinal))
            && !AvailableWorkspaceSource(file, root))
        {
            throw new InvalidOperationException($"{site}: annotated source requires an existing regular file inside the workspace root");
        }
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
        Result result,
        string root,
        bool includeAccountSupport)
    {
        if (type.GetCustomAttributesData().Any(attribute =>
                attribute.AttributeType.FullName ==
                "System.Runtime.CompilerServices.CompilerGeneratedAttribute"))
        {
            return;
        }

        var typeName = SiteName(type);
        var mechanismTypeSite = MetadataTypeName(type);
        var typeFile = files.PathOf(type);
        var typeFingerprint = ManifestFingerprint(files.FingerprintOf(type));
        EnsureSourceRoot(type.GetCustomAttributesData(), typeFile, mechanismTypeSite, root);
        CollectPackage(type, type.GetCustomAttributesData(), mechanismTypeSite, typeFile, typeFingerprint, result, root);
        var typeMechanisms = type.GetCustomAttributesData()
            .Where(attribute => FullName(attribute) == ImplementsMechanismName)
            .ToArray();
        EnsureOneMechanismTarget(mechanismTypeSite, typeMechanisms.Length);
        if (typeMechanisms.Length == 0 && AvailableWorkspaceSource(typeFile, root))
        {
            result.Artifacts.Add(
                new Artifact(
                    $"dotnet-symbol:{type.Assembly.GetName().Name}!{mechanismTypeSite}",
                    "dotnet-type",
                    typeFile));
        }

        foreach (var attribute in type.GetCustomAttributesData())
        {
            var name = FullName(attribute);
            if (name == RealizesName)
            {
                var claim = EntityArgument(attribute, file: typeFile, site: typeName);
                result.Realizes.Add(
                    new Entry(
                        claim,
                        type.FullName ?? type.Name,
                        typeFile,
                        typeFingerprint,
                        null,
                        null,
                        null));
            }
            else if (name == ImplementsMechanismName)
            {
                EnsureMechanismFingerprint(mechanismTypeSite, typeFingerprint, typeFile);
                var mechanism = EntityArgument(attribute, file: typeFile, site: mechanismTypeSite);
                var binding = $"dotnet-symbol:{mechanismTypeSite}";
                result.MechanismImplementations.Add(
                    new MechanismImplementationEntry(
                        mechanism,
                        mechanismTypeSite,
                        binding,
                        typeFile,
                        typeFingerprint));
                result.Artifacts.Add(new Artifact(binding, "dotnet-symbol", typeFile));
            }
            else if (includeAccountSupport && name == SupportsVerificationElementName)
            {
                AddElementSupport(
                    result.AccountSupport, attribute,
                    $"{type.Assembly.GetName().Name}!{mechanismTypeSite}",
                    typeFile, typeFingerprint,
                    root);
            }
        }

        foreach (var method in type.GetMethods(Members))
        {
            if (method.IsSpecialName)
            {
                continue;
            }
            var site = $"{typeName}.{method.Name}";
            var mechanismSite = MethodSite(method);
            var file = files.PathOf(method);
            var sourceFingerprint = ManifestFingerprint(files.FingerprintOf(method));
            var data = method.GetCustomAttributesData();
            EnsureSourceRoot(data, file, mechanismSite, root);
            CollectPackage(method, data, MethodSite(method), files.PathOf(method), ManifestFingerprint(files.FingerprintOf(method)), result, root);
            var mechanismAttributes = data
                .Where(attribute => FullName(attribute) == ImplementsMechanismName)
                .ToArray();
            EnsureOneMechanismTarget(mechanismSite, mechanismAttributes.Length);
            if (mechanismAttributes.Length == 0 && AvailableWorkspaceSource(file, root))
            {
                result.Artifacts.Add(
                    new Artifact(
                        $"dotnet-symbol:{method.Module.Assembly.GetName().Name}!{mechanismSite}",
                        "dotnet-method",
                        file));
            }
            foreach (var attribute in data)
            {
                var name = FullName(attribute);
                if (name == RealizesName)
                {
                    var claim = EntityArgument(attribute, file: file, site: site);
                    result.Realizes.Add(
                        new Entry(claim, site, file, sourceFingerprint, null, null, null));
                }
                else if (name == ImplementsCheckName)
                {
                    if (sourceFingerprint.Length == 0)
                    {
                        throw new InvalidOperationException(
                            $"{site}: ImplementsCheck requires an exact source fingerprint");
                    }
                    var check = EntityArgument(attribute, file: file, site: site);
                    EnsureEntityId(check, $"{file}: {site}");
                    result.CheckImplementations.Add(
                        new CheckImplementationEntry(
                            check,
                            site,
                            file,
                            sourceFingerprint));
                }
                else if (name == ImplementsMechanismName)
                {
                    EnsureMechanismFingerprint(mechanismSite, sourceFingerprint, file);
                    var mechanism = EntityArgument(attribute, file: file, site: mechanismSite);
                    var binding = $"dotnet-symbol:{mechanismSite}";
                    result.MechanismImplementations.Add(
                        new MechanismImplementationEntry(
                                mechanism,
                            mechanismSite,
                            binding,
                            file,
                            sourceFingerprint));
                    result.Artifacts.Add(new Artifact(binding, "dotnet-symbol", file));
                }
                else if (includeAccountSupport && name == SupportsVerificationElementName)
                {
                    AddElementSupport(
                        result.AccountSupport, attribute,
                        $"{method.Module.Assembly.GetName().Name}!{mechanismSite}",
                        file, sourceFingerprint,
                        root);
                }
                else if (includeAccountSupport && name == DefinesVerificationCheckName)
                {
                    AddVerificationCheck(
                        result.AccountSupport, attribute,
                        $"{method.Module.Assembly.GetName().Name}!{mechanismSite}",
                        file, sourceFingerprint);
                }
                else if (includeAccountSupport && name == ContributesCheckToCaseName)
                {
                    AddCaseContribution(
                        result.AccountSupport, attribute,
                        $"{method.Module.Assembly.GetName().Name}!{mechanismSite}", data);
                }
            }
        }

        CollectIndexes(type, files, result);
    }

    private static string SiteName(Type type) =>
        (type.FullName ?? type.Name).Replace('+', '.');

    private static string MethodSite(MethodInfo method)
    {
        var genericArity = method.IsGenericMethodDefinition
            ? $"``{method.GetGenericArguments().Length}"
            : string.Empty;
        var parameters = string.Join(
            ",",
            method.GetParameters().Select(parameter => MetadataTypeName(parameter.ParameterType)));
        return $"{MetadataTypeName(method.DeclaringType!)}.{method.Name}{genericArity}({parameters})";
    }

    private static string MetadataTypeName(Type type)
    {
        if (type.IsByRef)
        {
            return $"{MetadataTypeName(type.GetElementType()!)}&";
        }
        if (type.IsPointer)
        {
            return $"{MetadataTypeName(type.GetElementType()!)}*";
        }
        if (type.IsArray)
        {
            return MetadataTypeName(type.GetElementType()!)
                + "["
                + new string(',', type.GetArrayRank() - 1)
                + "]";
        }
        if (type.IsGenericParameter)
        {
            return $"{(type.DeclaringMethod is null ? "!" : "!!")}{type.GenericParameterPosition}";
        }
        if (type.IsConstructedGenericType)
        {
            var definition = MetadataTypeName(type.GetGenericTypeDefinition());
            var arguments = string.Join(
                ",",
                type.GetGenericArguments().Select(MetadataTypeName));
            return $"{definition}[{arguments}]";
        }
        return type.FullName ?? type.Name;
    }

    private static void EnsureOneMechanismTarget(string site, int targetCount)
    {
        if (targetCount > 1)
        {
            throw new InvalidOperationException(
                $"{site}: one qualified site cannot implement several mechanisms");
        }
    }

    private static void EnsureMechanismFingerprint(string site, string fingerprint, string file)
    {
        if (fingerprint.Length == 0 || file.Length == 0)
        {
            throw new InvalidOperationException(
                $"{site}: ImplementsMechanism requires an exact source locator and fingerprint");
        }
    }

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
                var source = files.PathOf(type);
                if (source.Length == 0)
                {
                    result.Warnings.Add(
                        $"{type.FullName}: derived database index `{table}.{name}` has no source locator");
                    continue;
                }
                result.Artifacts.Add(new Artifact(
                    $"postgres-index:{table}.{name}",
                    "database-index",
                    source,
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

    private static string EntityArgument(CustomAttributeData attribute, string file, string site)
    {
        var values = Arguments(attribute, 1, $"{file}: {site}");
        EnsureEntityId(values[0], $"{file}: {site}");
        return values[0];
    }

    private static string FirstArgument(CustomAttributeData attribute) =>
        attribute.ConstructorArguments.Count > 0
            ? attribute.ConstructorArguments[0].Value as string ?? string.Empty
            : string.Empty;

    private static string ManifestFingerprint(string fingerprint) =>
        fingerprint.Length == 0 ? string.Empty : $"sha256:{fingerprint}";

    private static int Compare(Entry a, Entry b)
    {
        var byClaim = string.CompareOrdinal(a.Claim, b.Claim);
        return byClaim != 0 ? byClaim : string.CompareOrdinal(a.Site, b.Site);
    }

    private static int CompareMechanismImplementation(
        MechanismImplementationEntry a,
        MechanismImplementationEntry b)
    {
        var byMechanism = string.CompareOrdinal(a.Mechanism, b.Mechanism);
        return byMechanism != 0 ? byMechanism : string.CompareOrdinal(a.Binding, b.Binding);
    }

    private static int CompareArtifact(Artifact a, Artifact b)
    {
        var byId = string.CompareOrdinal(a.Id, b.Id);
        if (byId != 0)
        {
            return byId;
        }
        var byKind = string.CompareOrdinal(a.Kind, b.Kind);
        return byKind != 0 ? byKind : string.CompareOrdinal(a.File, b.File);
    }

    private static int CompareCheckImplementation(
        CheckImplementationEntry a,
        CheckImplementationEntry b)
    {
        var byCheck = string.CompareOrdinal(a.Check, b.Check);
        if (byCheck != 0)
        {
            return byCheck;
        }

        return string.CompareOrdinal(a.Site, b.Site);
    }

    private static void AddElementSupport(
        AccountSupportResult support,
        CustomAttributeData attribute,
        string site,
        string file,
        string fingerprint,
        string root)
    {
        var values = Arguments(attribute, 2, site);
        EnsureEntityId(values[0], $"{file}: {site}");
        var artifact = SourceArtifact(site, file, fingerprint);
        support.Artifacts.Add(artifact);
        var artifacts = new List<string> { artifact.Id };
        var configuration = attribute.NamedArguments
            .FirstOrDefault(argument => argument.MemberName == "ConfigurationFile")
            .TypedValue.Value as string;
        if (configuration is not null)
        {
            var path = ConfigurationPath(root, configuration, site);
            var configurationArtifact = new SupportArtifact(
                $"configuration/{Digest(path)}",
                "configuration",
                path,
                $"sha256:{Convert.ToHexString(SHA256.HashData(File.ReadAllBytes(
                    Path.Combine(root, path)))).ToLowerInvariant()}",
                $"configuration:{path}");
            support.Artifacts.Add(configurationArtifact);
            artifacts.Add(configurationArtifact.Id);
        }
        support.Elements.Add(new ElementSupport(values[0], values[1], artifacts, []));
    }

    private static void AddVerificationCheck(
        AccountSupportResult support,
        CustomAttributeData attribute,
        string site,
        string file,
        string fingerprint)
    {
        var values = Arguments(attribute, 3, site);
        EnsureEntityId(values[0], $"{file}: {site}");
        var artifact = SourceArtifact(site, file, fingerprint);
        support.Artifacts.Add(artifact);
        support.Checks.Add(new VerificationCheck(values[0], values[1], [artifact.Id], [values[2]]));
    }

    private static void AddCaseContribution(
        AccountSupportResult support,
        CustomAttributeData attribute,
        string site,
        IList<CustomAttributeData> methodAttributes)
    {
        var values = Arguments(attribute, 4, site);
        EnsureEntityId(values[0], site);
        EnsureEntityId(values[1], site);
        var localCheck = methodAttributes.Any(candidate =>
            FullName(candidate) == DefinesVerificationCheckName
            && FirstArgument(candidate) == values[0]
            && candidate.ConstructorArguments.Count > 2
            && candidate.ConstructorArguments[2].Value as string == values[2]);
        if (!localCheck)
        {
            throw new InvalidOperationException(
                $"{site}: Case contribution requires a Check definition on this method "
                + $"for {values[0]} and {values[2]}");
        }
        support.Contributions.Add(
            new CaseContribution(values[0], values[1], values[2], values[3]));
    }

    private static void EnsureEntityId(string check, string site)
    {
        if (!System.Text.RegularExpressions.Regex.IsMatch(check, @"\A[a-z0-9](?:[a-z0-9-]*[a-z0-9])?\z"))
        {
            throw new InvalidOperationException($"{site}: entity ID must be one lowercase kebab segment");
        }
    }

    private static string[] Arguments(CustomAttributeData attribute, int count, string site)
    {
        var values = attribute.ConstructorArguments
            .Select(argument => argument.Value as string ?? string.Empty)
            .ToArray();
        if (values.Length != count || values.Any(string.IsNullOrWhiteSpace))
        {
            throw new InvalidOperationException(
                $"{site}: {attribute.AttributeType.Name} requires {count} nonempty strings");
        }
        return values;
    }

    private static SupportArtifact SourceArtifact(
        string site,
        string file,
        string fingerprint)
    {
        if (fingerprint.Length == 0 || !WorkspaceRelative(file))
        {
            throw new InvalidOperationException(
                $"{site}: account support requires a PDB-resolved workspace source path "
                + "and an exact source fingerprint");
        }
        return new SupportArtifact($"dotnet/{Digest(site)}", "source", file, fingerprint, site);
    }

    private static string ConfigurationPath(string root, string input, string site)
    {
        if (Path.IsPathRooted(input))
        {
            throw new InvalidOperationException(
                $"{site}: configuration file must be workspace-relative");
        }
        var full = Path.GetFullPath(Path.Combine(root, input));
        var relative = Path.GetRelativePath(root, full).Replace('\\', '/');
        if (!WorkspaceRelative(relative) || !File.Exists(full))
        {
            throw new InvalidOperationException(
                $"{site}: configuration file is missing or outside the workspace: {input}");
        }
        return relative;
    }

    private static bool WorkspaceRelative(string path) =>
        path.Length > 0
        && !Path.IsPathRooted(path)
        && !path.Contains('\\')
        && !path.Split('/').Any(part => part is "" or "." or "..");

    private static string Digest(string value) =>
        Convert.ToHexString(SHA256.HashData(Encoding.UTF8.GetBytes(value))).ToLowerInvariant();

    private static void NormalizeAccountSupport(AccountSupportResult support)
    {
        var artifacts = support.Artifacts
            .GroupBy(item => item.Id, StringComparer.Ordinal)
            .OrderBy(group => group.Key, StringComparer.Ordinal)
            .Select(group =>
            {
                if (group.Distinct().Count() != 1)
                {
                    throw new InvalidOperationException(
                        $"account Artifact {group.Key} has conflicting site or fingerprint");
                }
                return group.First();
            }).ToArray();
        support.Artifacts.Clear();
        support.Artifacts.AddRange(artifacts);

        var elements = support.Elements
            .GroupBy(item => (item.Claim, item.Element))
            .OrderBy(group => group.Key.Claim, StringComparer.Ordinal)
            .ThenBy(group => group.Key.Element, StringComparer.Ordinal)
            .Select(group => new ElementSupport(
                group.Key.Claim,
                group.Key.Element,
                group.SelectMany(item => item.Artifacts)
                    .Distinct(StringComparer.Ordinal)
                    .Order(StringComparer.Ordinal).ToArray(),
                group.SelectMany(item => item.DependsOn)
                    .Distinct(StringComparer.Ordinal)
                    .Order(StringComparer.Ordinal).ToArray()))
            .ToArray();
        support.Elements.Clear();
        support.Elements.AddRange(elements);

        var checks = support.Checks
            .GroupBy(item => item.Id, StringComparer.Ordinal)
            .OrderBy(group => group.Key, StringComparer.Ordinal)
            .Select(group =>
            {
                var terminals = group.Select(item => item.Terminal)
                    .Distinct(StringComparer.Ordinal).ToArray();
                if (terminals.Length != 1)
                {
                    throw new InvalidOperationException(
                        $"Check {group.Key} has conflicting terminal propositions");
                }
                return new VerificationCheck(
                    group.Key,
                    terminals[0],
                    group.SelectMany(item => item.Artifacts)
                        .Distinct(StringComparer.Ordinal)
                        .Order(StringComparer.Ordinal).ToArray(),
                    group.SelectMany(item => item.Elements)
                        .Distinct(StringComparer.Ordinal)
                        .Order(StringComparer.Ordinal).ToArray());
            }).ToArray();
        support.Checks.Clear();
        support.Checks.AddRange(checks);

        var contributions = support.Contributions
            .GroupBy(item => (item.Check, item.Case))
            .OrderBy(group => group.Key.Check, StringComparer.Ordinal)
            .ThenBy(group => group.Key.Case, StringComparer.Ordinal)
            .Select(group =>
            {
                if (group.Distinct().Count() != 1)
                {
                    throw new InvalidOperationException(
                        $"Check {group.Key.Check} has conflicting contribution to {group.Key.Case}");
                }
                return group.First();
            }).ToArray();
        support.Contributions.Clear();
        support.Contributions.AddRange(contributions);
    }

    public static string ToAccountSupportJson(Result result)
    {
        var body = new
        {
            format = "azimuth-account-support",
            version = 1,
            artifacts = result.AccountSupport.Artifacts,
            elements = result.AccountSupport.Elements,
            checks = result.AccountSupport.Checks,
            contributions = result.AccountSupport.Contributions
        };
        var options = new JsonSerializerOptions
        {
            WriteIndented = true,
            PropertyNamingPolicy = JsonNamingPolicy.SnakeCaseLower
        };
        return JsonSerializer.Serialize(body, options) + "\n";
    }

    public static string ToJson(Result result)
    {
        var options = new JsonWriterOptions { Indented = true };
        using var stream = new MemoryStream();
        using (var writer = new Utf8JsonWriter(stream, options))
        {
            writer.WriteStartObject();
            WriteEntries(writer, "realizes", result.Realizes, form: false);
            WriteCheckImplementations(writer, result.CheckImplementations);
            WriteMechanismImplementations(writer, result.MechanismImplementations);
            WriteArtifacts(writer, result.Artifacts);
            writer.WritePropertyName("extensions");
            JsonSerializer.Serialize(writer, result.Extensions, new JsonSerializerOptions { PropertyNamingPolicy = JsonNamingPolicy.SnakeCaseLower, DefaultIgnoreCondition = System.Text.Json.Serialization.JsonIgnoreCondition.WhenWritingNull });
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
            writer.WriteString("claim", entry.Claim);
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
            writer.WriteString("mechanism", entry.Mechanism);
            writer.WriteString("site", entry.Site);
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

    private static void WriteCheckImplementations(
        Utf8JsonWriter writer,
        List<CheckImplementationEntry> entries)
    {
        writer.WriteStartArray("check_implementations");
        foreach (var entry in entries)
        {
            writer.WriteStartObject();
            writer.WriteString("check", entry.Check);
            writer.WriteString("site", entry.Site);
            writer.WriteString("file", entry.File);
            writer.WriteString("lang", Lang);
            writer.WriteString("source_fingerprint", entry.SourceFingerprint);
            writer.WriteEndObject();
        }
        writer.WriteEndArray();
    }
}
