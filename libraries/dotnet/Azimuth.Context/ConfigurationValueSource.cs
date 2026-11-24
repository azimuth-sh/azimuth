using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.Primitives;

namespace Azimuth.Context;

// Azimuth's values live in a provider of their own, added last, so that a read can tell from the
// provider's position whether Azimuth's value would win, is the applied value, or is overridden.
internal sealed class ConfigurationValueSource : IContextSource
{
    private readonly Func<IConfigurationRoot> root;
    private readonly AppliedValuesProvider applied;
    private readonly string path;

    public ConfigurationValueSource(Func<IConfigurationRoot> root, AppliedValuesProvider applied, string path)
    {
        this.root = root;
        this.applied = applied;
        this.path = path;
    }

    public ContextRead Read()
    {
        var providers = root().Providers.ToList();
        var appliedAt = providers.IndexOf(applied);
        for (var at = providers.Count - 1; at >= 0; at--)
        {
            if (!providers[at].TryGet(path, out var value))
            {
                continue;
            }
            var precedence = at == appliedAt ? "azimuth" : at < appliedAt ? "before azimuth" : "after azimuth";
            var name = at == appliedAt ? AppliedValuesProvider.Name : providers[at].ToString() ?? providers[at].GetType().Name;
            return new ContextRead(value, new ContextSource(name, precedence));
        }
        return new ContextRead(null, null);
    }

    public void Apply(string text) => applied.Set(path, text);
}

internal sealed class AppliedValuesSource : IConfigurationSource
{
    public AppliedValuesProvider Provider { get; } = new();

    public IConfigurationProvider Build(IConfigurationBuilder builder) => Provider;
}

internal sealed class AppliedValuesProvider : IConfigurationProvider
{
    public const string Name = "azimuth";

    private readonly Dictionary<string, string?> values = new(StringComparer.OrdinalIgnoreCase);
    private CancellationTokenSource reload = new();

    public bool TryGet(string key, out string? value) => values.TryGetValue(key, out value);

    public void Set(string key, string? value)
    {
        values[key] = value;
        Interlocked.Exchange(ref reload, new CancellationTokenSource()).Cancel();
    }

    public void Load()
    {
    }

    public IChangeToken GetReloadToken() => new CancellationChangeToken(reload.Token);

    public IEnumerable<string> GetChildKeys(IEnumerable<string> earlierKeys, string? parentPath)
    {
        var prefix = parentPath is null ? string.Empty : parentPath + ConfigurationPath.KeyDelimiter;
        var children = new List<string>();
        foreach (var key in values.Keys)
        {
            if (!key.StartsWith(prefix, StringComparison.OrdinalIgnoreCase))
            {
                continue;
            }
            var rest = key.AsSpan(prefix.Length);
            var end = rest.IndexOf(ConfigurationPath.KeyDelimiter, StringComparison.OrdinalIgnoreCase);
            children.Add(end < 0 ? rest.ToString() : rest[..end].ToString());
        }
        children.AddRange(earlierKeys);
        children.Sort(StringComparer.OrdinalIgnoreCase);
        return children;
    }

    public override string ToString() => Name;
}
