using System.Text;
using System.Text.Json;
using Microsoft.Extensions.Configuration;

namespace Azimuth.Context;

/// <summary>A registered component: it applies its launch's values and gives its report.</summary>
public sealed class AzimuthContext
{
    private const string Variable = "AZIMUTH_CONTEXT";

    private readonly string component;
    private readonly bool applying;
    private readonly IReadOnlyList<(string Key, IContextSource Source)> entries;

    internal AzimuthContext(IConfigurationBuilder configuration, string component, Func<bool> applyingWhen, Action<ContextMap> map)
    {
        ArgumentException.ThrowIfNullOrEmpty(component);
        ArgumentNullException.ThrowIfNull(applyingWhen);
        ArgumentNullException.ThrowIfNull(map);

        this.component = component;
        applying = applyingWhen();
        var launch = applying ? ReadLaunch() : new Dictionary<string, string>();

        var source = new AppliedValuesSource();
        configuration.Add(source);
        var root = configuration as IConfigurationRoot;
        var mapping = new ContextMap(path => new ConfigurationValueSource(
            () => root ?? configuration.Build(), source.Provider, path));
        map(mapping);
        entries = mapping.Entries.ToList();

        foreach (var (key, keySource) in entries)
        {
            if (launch.TryGetValue(key, out var text))
            {
                keySource.Apply(text);
            }
        }
    }

    /// <summary>The report of the component, as JSON text, format <c>1</c>. It reads every mapped key again.</summary>
    public string Report()
    {
        var stream = new MemoryStream();
        using (var writer = new Utf8JsonWriter(stream))
        {
            writer.WriteStartObject();
            writer.WriteNumber("format", 1);
            writer.WriteString("component", component);
            writer.WriteString("applying", applying ? "possible" : "not possible");
            writer.WriteStartObject("context");
            foreach (var (key, source) in entries)
            {
                var read = source.Read();
                var readAt = DateTime.UtcNow;
                writer.WriteStartObject(key);
                writer.WriteString("value", read.Value);
                if (read.Source is null)
                {
                    writer.WriteNull("source");
                }
                else
                {
                    writer.WriteStartObject("source");
                    writer.WriteString("name", read.Source.Name);
                    writer.WriteString("precedence", read.Source.Precedence);
                    writer.WriteEndObject();
                }
                writer.WriteString("read", readAt.ToString("yyyy-MM-dd'T'HH:mm:ss.fffffff'Z'", System.Globalization.CultureInfo.InvariantCulture));
                writer.WriteEndObject();
            }
            writer.WriteEndObject();
            writer.WriteEndObject();
        }
        return Encoding.UTF8.GetString(stream.ToArray());
    }

    private static Dictionary<string, string> ReadLaunch()
    {
        var text = Environment.GetEnvironmentVariable(Variable);
        var values = new Dictionary<string, string>();
        if (text is null)
        {
            return values;
        }
        try
        {
            using var document = JsonDocument.Parse(text);
            if (document.RootElement.ValueKind != JsonValueKind.Object)
            {
                throw new InvalidOperationException($"{Variable} must be one JSON object of text values.");
            }
            foreach (var property in document.RootElement.EnumerateObject())
            {
                if (property.Value.ValueKind != JsonValueKind.String)
                {
                    throw new InvalidOperationException($"{Variable} must be one JSON object of text values; \"{property.Name}\" is not text.");
                }
                values[property.Name] = property.Value.GetString()!;
            }
        }
        catch (JsonException error)
        {
            throw new InvalidOperationException($"{Variable} must be one JSON object of text values: {error.Message}", error);
        }
        return values;
    }
}

public static class AzimuthContextRegistration
{
    /// <summary>
    /// Registers the component, evaluates <paramref name="applyingWhen"/> once, adds the source that
    /// applies Azimuth's values last among the configuration's sources and, when the condition held,
    /// applies the values <c>AZIMUTH_CONTEXT</c> names for the mapped keys.
    /// </summary>
    public static AzimuthContext AddAzimuthContext(
        this IConfigurationBuilder configuration,
        string name,
        Func<bool> applyingWhen,
        Action<ContextMap> map) => new(configuration, name, applyingWhen, map);
}
