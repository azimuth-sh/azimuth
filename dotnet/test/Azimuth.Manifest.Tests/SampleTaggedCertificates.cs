using Azimuth.Annotations;

namespace Azimuth.Manifest.Tests;

/// <summary>
/// A stand-in production+test surface, tagged the way a real certificates slice would be, used to
/// pin the emitter end-to-end. The triples mirror the schema README's worked example.
/// </summary>
[Realizes("public-certificates", "detail", "detail-valid")]
public sealed class GetPublicCertificate
{
    [Realizes("public-certificates", "detail", "detail-revoked-void")]
    public string Handle(string slug) => slug;
}

public sealed class GetPublicCertificateTests
{
    [Covers("public-certificates", "detail", "detail-valid",
        RtmScope.Component, RtmQuantification.Example)]
    public void ValidCertificateIsReturned()
    {
    }

    [Covers("public-certificates", "detail", "detail-revoked-void",
        RtmScope.Component, RtmQuantification.Invariant, RtmOracle.Direct)]
    public void RevokedCertificateReturns404()
    {
    }

    [Covers("public-certificates", "list", "completeness",
        RtmScope.E2e, RtmQuantification.Invariant, RtmOracle.ModelBased)]
    public void EveryIssuedCertificateAppears()
    {
    }
}
