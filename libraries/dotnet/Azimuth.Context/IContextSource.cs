namespace Azimuth.Context;

/// <summary>Reads a key's current value and where it came from, and applies a value in this process.</summary>
public interface IContextSource
{
    ContextRead Read();

    void Apply(string text);
}

/// <summary>The current text of a key, or <c>null</c> when absent, and its source, or <c>null</c> when absent.</summary>
public record ContextRead(string? Value, ContextSource? Source);

/// <summary>Where a value came from. <see cref="Precedence"/> is the report's own text: <c>before azimuth</c>, <c>azimuth</c> or <c>after azimuth</c>.</summary>
public record ContextSource(string Name, string Precedence);
