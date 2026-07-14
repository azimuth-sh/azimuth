using System.Reflection;

namespace Azimuth.Manifest;

/// <summary>
/// Reflects over one or more assemblies and collects their <c>[Realizes]</c> and <c>[Covers]</c>
/// tags into a <see cref="Manifest"/>. Mirrors the harness collectors: a type-level tag names its
/// site by the type, a method-level tag by <c>Type.Method</c>. Attributes are matched by full name
/// (not CLR identity) so the emitter works even when a target assembly was loaded from a different
/// path than the emitter's own reference to <c>Azimuth.Annotations</c>.
/// </summary>
public static class ManifestCollector
{
    private const string Lang = "csharp";
    private const string RealizesName = "Azimuth.Annotations.RealizesAttribute";
    private const string CoversName = "Azimuth.Annotations.CoversAttribute";
    private const string UntracedName = "Azimuth.Annotations.UntracedAttribute";

    /// <summary>
    /// Attributes (matched by simple name, so no test-framework reference is needed) that mark a
    /// method as a test. Kept a small const list rather than hard-wired to one call so another
    /// xUnit-style framework can be added without touching the walk.
    /// </summary>
    private static readonly HashSet<string> TestAttributeNames = new(StringComparer.Ordinal)
    {
        "FactAttribute",
        "TheoryAttribute",
    };

    public static Manifest Collect(
        Assembly assembly,
        string? root = null,
        IReadOnlyList<string>? tracedRoots = null) =>
        Collect(new[] { assembly }, root, tracedRoots);

    public static Manifest Collect(
        IEnumerable<Assembly> assemblies,
        string? root = null,
        IReadOnlyList<string>? tracedRoots = null)
    {
        var roots = tracedRoots ?? Array.Empty<string>();
        var realizes = new List<RealizesEntry>();
        var covers = new List<CoversEntry>();
        var untraced = new List<UntracedTestEntry>();

        foreach (var assembly in assemblies)
        {
            using var files = SourceFileResolver.ForAssembly(assembly);
            CollectFrom(assembly, files, root, roots, realizes, covers, untraced);
        }

        return new Manifest { Realizes = realizes, Covers = covers, UntracedTests = untraced };
    }

    private static void CollectFrom(
        Assembly assembly,
        SourceFileResolver files,
        string? root,
        IReadOnlyList<string> tracedRoots,
        List<RealizesEntry> realizes,
        List<CoversEntry> covers,
        List<UntracedTestEntry> untraced)
    {
        foreach (var type in Types(assembly))
        {
            var typeSite = QualifiedTypeName(type);
            var typeFile = files.FileFor(type, root);

            foreach (var data in type.GetCustomAttributesData())
            {
                if (IsAttr(data, RealizesName))
                {
                    realizes.Add(RealizesOf(data, typeSite, typeFile));
                }
            }

            // Area scope: a test is held to the untraced-test check only when its type sits under an
            // opt-in traced root (a namespace prefix). This catches whole untagged test files inside
            // a traced area, and never red-walls anything outside the declared roots.
            var typeIsTraced = IsUnderTracedRoot(type, tracedRoots);

            foreach (var method in Methods(type))
            {
                var methodSite = $"{QualifiedTypeName(method.DeclaringType!)}.{method.Name}";
                var methodFile = files.FileFor(method, root);

                foreach (var data in method.GetCustomAttributesData())
                {
                    if (IsAttr(data, RealizesName))
                    {
                        realizes.Add(RealizesOf(data, methodSite, methodFile));
                    }
                    else if (IsAttr(data, CoversName))
                    {
                        covers.Add(CoversOf(data, methodSite, methodFile));
                    }
                }

                if (typeIsTraced && IsUntracedTest(method))
                {
                    untraced.Add(new UntracedTestEntry(methodSite, methodFile));
                }
            }
        }
    }

    /// <summary>A type is under a traced root when its namespace equals or is nested beneath one of
    /// the declared prefixes (matched on the dot boundary, so <c>Foo.Bar</c> never captures
    /// <c>Foo.BarBaz</c>).</summary>
    private static bool IsUnderTracedRoot(Type type, IReadOnlyList<string> tracedRoots)
    {
        var ns = type.Namespace ?? string.Empty;
        return tracedRoots.Any(prefix =>
            ns == prefix || ns.StartsWith(prefix + ".", StringComparison.Ordinal));
    }

    /// <summary>A test method under a traced root earns an untraced entry unless it carries a
    /// <c>[Covers]</c> (it traces a scenario) or an <c>[Untraced]</c> (a deliberate opt-out).</summary>
    private static bool IsUntracedTest(MethodInfo method)
    {
        var data = method.GetCustomAttributesData();
        var isTest = data.Any(d => TestAttributeNames.Contains(d.AttributeType.Name));
        if (!isTest)
        {
            return false;
        }

        return !data.Any(d => IsAttr(d, CoversName) || IsAttr(d, UntracedName));
    }

    /// A readable, collision-free site name: nested types are qualified by their declaring type(s)
    /// (e.g. `RevokeCertificate.Endpoint`), so the per-slice `Endpoint`/`RequestHandler` classes of
    /// vertical-slice architecture don't all collapse to one site and defeat per-site invariant checks.
    private static string QualifiedTypeName(Type type)
    {
        var name = type.Name;
        for (var declaring = type.DeclaringType; declaring is not null; declaring = declaring.DeclaringType)
        {
            name = $"{declaring.Name}.{name}";
        }
        return name;
    }

    private static RealizesEntry RealizesOf(CustomAttributeData data, string site, string file)
    {
        var args = data.ConstructorArguments;
        return new RealizesEntry(
            Spec: Str(args[0]),
            Req: Str(args[1]),
            Scenario: Str(args[2]),
            Site: site,
            File: file,
            Lang: Lang);
    }

    private static CoversEntry CoversOf(CustomAttributeData data, string site, string file)
    {
        var args = data.ConstructorArguments;
        return new CoversEntry(
            Spec: Str(args[0]),
            Req: Str(args[1]),
            Scenario: Str(args[2]),
            Site: site,
            File: file,
            Lang: Lang,
            Scope: EnumValue(args[3]),
            Quantification: EnumValue(args[4]),
            Oracle: EnumValue(args[5]));
    }

    private static bool IsAttr(CustomAttributeData data, string fullName) =>
        data.AttributeType.FullName == fullName;

    private static string Str(CustomAttributeTypedArgument arg) => (string)arg.Value!;

    /// <summary>The schema wire form of an enum value: PascalCase name → kebab-case (ModelBased → model-based).</summary>
    private static string EnumValue(CustomAttributeTypedArgument arg)
    {
        var name = Enum.GetName(arg.ArgumentType, arg.Value!) ?? arg.Value!.ToString()!;
        return Kebab(name);
    }

    private static string Kebab(string pascal)
    {
        var chars = new List<char>(pascal.Length + 2);
        for (var i = 0; i < pascal.Length; i++)
        {
            var c = pascal[i];
            if (char.IsUpper(c) && i > 0)
            {
                chars.Add('-');
            }

            chars.Add(char.ToLowerInvariant(c));
        }

        return new string(chars.ToArray());
    }

    private static IEnumerable<Type> Types(Assembly assembly)
    {
        try
        {
            return assembly.GetTypes();
        }
        catch (ReflectionTypeLoadException ex)
        {
            return ex.Types.Where(t => t is not null)!;
        }
    }

    private static IEnumerable<MethodInfo> Methods(Type type) =>
        type.GetMethods(
            BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.Instance | BindingFlags.Static |
            BindingFlags.DeclaredOnly);
}
