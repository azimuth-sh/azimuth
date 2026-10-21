using System;

namespace Azimuth.Challenge.Annotations;

[AttributeUsage(AttributeTargets.Class | AttributeTargets.Method, AllowMultiple = true)]
public sealed class ImplementsVerificationChallengeAttribute(string verificationChallenge) : Attribute
{
    public string VerificationChallenge { get; } = verificationChallenge;
    public string? ConfigurationFile { get; set; }
}
