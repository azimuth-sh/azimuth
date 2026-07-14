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

    public static Manifest Collect(Assembly assembly, string? root = null) =>
        Collect(new[] { assembly }, root);

    public static Manifest Collect(IEnumerable<Assembly> assemblies, string? root = null)
    {
        var realizes = new List<RealizesEntry>();
        var covers = new List<CoversEntry>();

        foreach (var assembly in assemblies)
        {
            using var files = SourceFileResolver.ForAssembly(assembly);
            CollectFrom(assembly, files, root, realizes, covers);
        }

        return new Manifest { Realizes = realizes, Covers = covers };
    }

    private static void CollectFrom(
        Assembly assembly,
        SourceFileResolver files,
        string? root,
        List<RealizesEntry> realizes,
        List<CoversEntry> covers)
    {
        foreach (var type in Types(assembly))
        {
            var typeSite = type.Name;
            var typeFile = files.FileFor(type, root);

            foreach (var data in type.GetCustomAttributesData())
            {
                if (IsAttr(data, RealizesName))
                {
                    realizes.Add(RealizesOf(data, typeSite, typeFile));
                }
            }

            foreach (var method in Methods(type))
            {
                var methodSite = $"{method.DeclaringType!.Name}.{method.Name}";
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
            }
        }
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
