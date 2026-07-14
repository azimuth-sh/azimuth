using System;
using Azimuth.Annotations;
using Azimuth.Manifest.Tests.Fixtures;

namespace Azimuth.Manifest.Tests.Fixtures
{
    /// <summary>
    /// A stand-in test attribute matched by simple name (<c>FactAttribute</c>), letting the fixtures
    /// exercise the emitter's framework-agnostic test discovery without xUnit picking these no-op
    /// methods up as real tests (it derives from <see cref="Attribute"/>, not <c>Xunit.FactAttribute</c>).
    /// </summary>
    [AttributeUsage(AttributeTargets.Method)]
    public sealed class FactAttribute : Attribute
    {
    }
}

namespace Azimuth.Manifest.Tests.Fixtures.Traced
{
    /// <summary>
    /// A class with zero <c>[Covers]</c> that sits inside a traced root — the case the old
    /// class-scoped rule missed. Its bare test must be flagged: the whole file is untagged inside a
    /// declared traced area.
    /// </summary>
    public sealed class EntirelyUntaggedRevokeTests
    {
        [Fact]
        public void SeedsFixtures()
        {
        }
    }

    /// <summary>A partially tagged class inside the same traced root.</summary>
    public sealed class PartiallyTaggedRevokeTests
    {
        [Fact]
        [Covers("public-certificates", "revoke", "revoke-hides-detail",
            RtmScope.Component, RtmQuantification.Invariant)]
        public void RevokedCertificateIsHidden()
        {
        }

        [Fact]
        public void UntaggedCase()
        {
        }

        [Fact]
        [Untraced("shared harness setup — maps to no scenario")]
        public void ResetsDatabase()
        {
        }
    }
}

namespace Azimuth.Manifest.Tests.Fixtures.Outside
{
    /// <summary>A class outside every traced root — its untagged tests are never flagged.</summary>
    public sealed class NeighbourTests
    {
        [Fact]
        public void SomeUnrelatedTest()
        {
        }
    }
}
