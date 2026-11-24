using Microsoft.Extensions.Configuration;

namespace Azimuth.Context;

/// <summary>The explicit mapping of every product key a component uses to its own setting.</summary>
public sealed class ContextMap
{
    private readonly Func<string, IContextSource> configurationSource;
    private readonly List<(string Key, IContextSource Source)> entries = new();

    internal ContextMap(Func<string, IContextSource> configurationSource)
    {
        this.configurationSource = configurationSource;
    }

    internal IReadOnlyList<(string Key, IContextSource Source)> Entries => entries;

    /// <summary>Maps <paramref name="productKey"/> to a configuration path, read and written through the shipped source.</summary>
    public ContextMap Key(string productKey, string configurationPath)
    {
        ArgumentException.ThrowIfNullOrEmpty(configurationPath);
        return Key(productKey, configurationSource(configurationPath));
    }

    /// <summary>Maps <paramref name="productKey"/> to a team's own source.</summary>
    public ContextMap Key(string productKey, IContextSource source)
    {
        ArgumentException.ThrowIfNullOrEmpty(productKey);
        ArgumentNullException.ThrowIfNull(source);
        entries.Add((productKey, source));
        return this;
    }
}
