namespace Azimuth.Annotations;

/// <summary>
/// Where a test's expected result came from. Descriptive only — it records the strategy, it is
/// never gated: an oracle strategy is forced by the problem (you reach for metamorphic when no
/// direct oracle exists), not chosen as a strength rung, and whenever strength matters it reduces
/// to (<see cref="RtmScope"/> × <see cref="RtmQuantification"/>). Useful for the code-map and as a
/// vocabulary; it never opens a matrix hole. <see cref="Contract"/> (Pact-style) is the
/// cross-service oracle, not a scope or quantification value.
/// </summary>
public enum RtmOracle
{
    /// <summary>The expected value is stated directly against a compact spec.</summary>
    Direct,

    /// <summary>A recorded reference output (snapshot); pins current behavior.</summary>
    Golden,

    /// <summary>A relational oracle over transformed inputs (f(permuted) is stable).</summary>
    Metamorphic,

    /// <summary>A reference model the system-under-test is checked against over sequences.</summary>
    ModelBased,

    /// <summary>A shared cross-service agreement (Pact file/schema) both sides check against.</summary>
    Contract,
}
