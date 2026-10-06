using System.Reflection;
using System.Text.Json.Nodes;

namespace Azimuth.Emit;

internal static class PackageProducers
{
    public static JsonObject Schema(Type type, HashSet<Type>? ancestors = null)
    {
        ancestors ??= [];
        if (type == typeof(string)) return new() { ["kind"] = "string" };
        if (type == typeof(bool)) return new() { ["kind"] = "boolean" };
        if (type == typeof(char) || type == typeof(IntPtr) || type == typeof(UIntPtr))
            throw new InvalidOperationException($"Unsupported producer scalar {type.FullName}");
        if (type.IsPrimitive || type == typeof(decimal)) return new() { ["kind"] = "number" };
        if (Nullable.GetUnderlyingType(type) is { } underlying)
            return new() { ["kind"] = "nullable", ["value"] = Schema(underlying, ancestors) };
        if (type.IsEnum)
        {
            var values = new JsonArray();
            foreach (var field in type.GetFields(BindingFlags.Public | BindingFlags.Static))
            {
                var name = field.GetCustomAttributesData().FirstOrDefault(attribute =>
                    attribute.AttributeType.FullName == "System.Text.Json.Serialization.JsonStringEnumMemberNameAttribute")
                    ?.ConstructorArguments[0].Value as string;
                values.Add(name ?? Kebab(field.Name));
            }
            if (values.Select(value => value!.GetValue<string>()).Distinct().Count() != values.Count)
                throw new InvalidOperationException($"Ambiguous serialized values for {type.FullName}");
            return new() { ["kind"] = "enum", ["values"] = values };
        }
        var interfaces = type.GetInterfaces().Append(type).ToArray();
        var map = interfaces.FirstOrDefault(candidate => candidate.IsGenericType &&
            candidate.GetGenericTypeDefinition().FullName is "System.Collections.Generic.IDictionary`2" or "System.Collections.Generic.IReadOnlyDictionary`2");
        if (map is not null)
        {
            var arguments = map.GetGenericArguments();
            return new() { ["kind"] = "map", ["keys"] = Schema(arguments[0], ancestors), ["values"] = Schema(arguments[1], ancestors) };
        }
        var sequence = interfaces.FirstOrDefault(candidate => candidate.IsGenericType &&
            candidate.GetGenericTypeDefinition() == typeof(IEnumerable<>));
        if (sequence is not null)
            return new() { ["kind"] = "list", ["items"] = Schema(sequence.GetGenericArguments()[0], ancestors) };
        if (!ancestors.Add(type)) throw new InvalidOperationException($"Recursive producer schema {type.FullName}");
        var fields = new JsonArray();
        foreach (var property in type.GetProperties(BindingFlags.Public | BindingFlags.Instance).OrderBy(property => property.Name, StringComparer.Ordinal))
        {
            if (!property.CanRead || property.GetIndexParameters().Length != 0) continue;
            var name = property.GetCustomAttributesData().FirstOrDefault(attribute =>
                attribute.AttributeType.FullName == "System.Text.Json.Serialization.JsonPropertyNameAttribute")?.ConstructorArguments[0].Value as string;
            var schema = Schema(property.PropertyType, ancestors);
            if (!property.PropertyType.IsValueType && new NullabilityInfoContext().Create(property).ReadState == NullabilityState.Nullable)
                schema = new JsonObject { ["kind"] = "nullable", ["value"] = schema };
            fields.Add(new JsonObject { ["name"] = name ?? property.Name, ["schema"] = schema });
        }
        ancestors.Remove(type);
        if (fields.Count == 0) throw new InvalidOperationException($"Producer output {type.FullName} has no public record fields");
        return new() { ["kind"] = "record", ["fields"] = fields };
    }

    public static Type MemberType(Type output)
    {
        var sequences = output.GetInterfaces().Append(output).Where(type => type.IsGenericType &&
            type.GetGenericTypeDefinition() == typeof(IEnumerable<>)).Select(type => type.GetGenericArguments()[0]).Distinct().ToArray();
        return sequences.Length == 1 ? sequences[0] : throw new InvalidOperationException("Producer must return one typed IEnumerable<T> member set");
    }

    private static string Kebab(string value) => string.Concat(value.Select((letter, index) =>
        char.IsUpper(letter) && index > 0 ? "-" + char.ToLowerInvariant(letter) : char.ToLowerInvariant(letter).ToString()));
}
