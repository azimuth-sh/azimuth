using System;
using Azimuth.Annotations;

namespace Azimuth.Manifest.Tests.Fixtures;

/// <summary>
/// A stand-in test attribute matched by simple name (<c>FactAttribute</c>), letting the fixtures
/// exercise the emitter's framework-agnostic test discovery without xUnit picking these no-op
/// methods up as real tests (it derives from <see cref="Attribute"/>, not <c>Xunit.FactAttribute</c>).
/// </summary>
[AttributeUsage(AttributeTargets.Method)]
public sealed class FactAttribute : Attribute
{
}

/// <summary>
/// A class that participates in tracing — one method carries <c>[Covers]</c> — so every test method
/// here is held to the untraced-test check.
/// </summary>
public sealed class SampleTracedRevokeTests
{
    [Fact]
    [Covers("public-certificates", "revoke", "revoke-hides-detail",
        RtmScope.Component, RtmQuantification.Invariant)]
    public void RevokedCertificateIsHidden()
    {
    }

    [Fact]
    public void SeedsFixtures()
    {
    }

    [Fact]
    [Untraced("shared harness setup — maps to no scenario")]
    public void ResetsDatabase()
    {
    }
}

/// <summary>
/// A class with zero <c>[Covers]</c>: it does not participate in tracing, so the scope rule leaves
/// its untagged tests untouched.
/// </summary>
public sealed class SampleUntracedNeighbourTests
{
    [Fact]
    public void SomeUnrelatedTest()
    {
    }
}
