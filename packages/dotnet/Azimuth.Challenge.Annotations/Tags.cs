using System;

namespace Azimuth.Challenge.Annotations;

[AttributeUsage(AttributeTargets.Class | AttributeTargets.Method, AllowMultiple = true)]
public sealed class ImplementsChallengerAttribute(string challenger) : Attribute
{
    public string Challenger { get; } = challenger;
    public string? ConfigurationFile { get; set; }
}
