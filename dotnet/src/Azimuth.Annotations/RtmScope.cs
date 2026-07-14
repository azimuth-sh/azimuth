namespace Azimuth.Annotations;

/// <summary>
/// How much of the real system a check runs against — the radius of what actually executes.
/// One of the two gated form sub-axes (with <see cref="RtmQuantification"/>). Azimuth's
/// <c>integration</c> level folds into <see cref="Component"/>: for a backend, the component test
/// (one service, real internal dependencies, exercised over HTTP against a real database) is the
/// integration level.
/// </summary>
public enum RtmScope
{
    /// <summary>One node in isolation.</summary>
    Unit,

    /// <summary>One service with its real internal dependencies (backend over HTTP + real DB).</summary>
    Component,

    /// <summary>The whole stack along a real user path.</summary>
    E2e,
}
