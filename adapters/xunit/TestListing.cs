using System.Reflection;
using System.Runtime.Loader;

namespace Azimuth.Xunit;

/// <summary>A test that carries the marker: one entry, also for a test with many data rows.</summary>
public sealed record MarkedTest(string Name, string ClassName, string MethodName, IReadOnlyList<string> Checks);

/// <summary>
/// Lists the marked tests of a built test assembly. The assembly is loaded to read its
/// attributes and nothing in it is run, so listing starts no test.
/// </summary>
public static class TestListing
{
    private const string FactAttribute = "Xunit.FactAttribute";
    private const string TraitAttribute = "Xunit.TraitAttribute";

    public static IReadOnlyList<MarkedTest> Marked(string assemblyPath, string markerKey)
    {
        var context = new TestAssemblyContext(assemblyPath);
        try
        {
            var assembly = context.LoadFromAssemblyPath(assemblyPath);
            var tests = new List<MarkedTest>();
            foreach (var type in Types(assembly).Where(type => type.IsClass && !type.IsAbstract && !type.ContainsGenericParameters))
            {
                var classChecks = Traits(ClassAndBases(type).SelectMany(CustomAttributeData.GetCustomAttributes), markerKey);
                const BindingFlags methods = BindingFlags.Public | BindingFlags.Instance | BindingFlags.Static;
                foreach (var method in type.GetMethods(methods))
                {
                    var attributes = CustomAttributeData.GetCustomAttributes(method);
                    if (!attributes.Any(IsFact))
                    {
                        continue;
                    }
                    var checks = classChecks.Concat(Traits(attributes, markerKey)).Distinct().ToList();
                    if (checks.Count > 0)
                    {
                        var className = type.FullName ?? type.Name;
                        tests.Add(new MarkedTest($"{className}.{method.Name}", className, method.Name, checks));
                    }
                }
            }
            return tests;
        }
        finally
        {
            context.Unload();
        }
    }

    private static IEnumerable<Type> Types(Assembly assembly)
    {
        try
        {
            return assembly.GetTypes();
        }
        catch (ReflectionTypeLoadException error)
        {
            return error.Types.OfType<Type>();
        }
    }

    private static IEnumerable<Type> ClassAndBases(Type type)
    {
        for (var current = type; current is not null && current != typeof(object); current = current.BaseType)
        {
            yield return current;
        }
    }

    /// <summary><c>[Theory]</c> derives from <c>[Fact]</c>, so both are tests.</summary>
    private static bool IsFact(CustomAttributeData attribute)
    {
        for (var type = attribute.AttributeType; type is not null; type = type.BaseType)
        {
            if (type.FullName == FactAttribute)
            {
                return true;
            }
        }
        return false;
    }

    private static IEnumerable<string> Traits(IEnumerable<CustomAttributeData> attributes, string markerKey) =>
        attributes
            .Where(attribute => attribute.AttributeType.FullName == TraitAttribute && attribute.ConstructorArguments.Count == 2)
            .Where(attribute => (string?)attribute.ConstructorArguments[0].Value == markerKey)
            .Select(attribute => (string?)attribute.ConstructorArguments[1].Value)
            .OfType<string>();

    /// <summary>Resolves the test assembly's own dependencies from its build output.</summary>
    private sealed class TestAssemblyContext(string assemblyPath) : AssemblyLoadContext(isCollectible: true)
    {
        private readonly AssemblyDependencyResolver resolver = new(assemblyPath);

        protected override Assembly? Load(AssemblyName name)
        {
            var path = resolver.ResolveAssemblyToPath(name);
            return path is null ? null : LoadFromAssemblyPath(path);
        }
    }
}
