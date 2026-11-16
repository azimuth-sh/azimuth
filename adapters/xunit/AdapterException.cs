namespace Azimuth.Xunit;

/// <summary>A reason the adapter could not write a report, which makes the launch crashed.</summary>
public sealed class AdapterException(string message) : Exception(message);
