using System;

namespace Azimuth.Annotations;

/// <summary>
/// Declares that a test verifies a spec scenario, by the stable (spec-id, req-id, scenario-id)
/// triple, at a declared form. Form is the pair (<see cref="RtmScope"/>, <see cref="RtmQuantification"/>)
/// — the two gated sub-axes the matrix reddens on when a scenario is under-proven. The
/// <see cref="RtmOracle"/> is an optional, descriptive label (how the expected result was obtained),
/// never gated. A test may cover several scenarios; a whole-stack test covers one scenario at
/// several component sites.
/// </summary>
/// <remarks>
/// This is a code tag, so it only <em>realizes</em>/<em>covers</em> scenarios. Spec-side
/// scenario attributes (<c>exposes</c>/<c>upholds</c>) live in the spec, not in code.
/// </remarks>
[AttributeUsage(AttributeTargets.Method, AllowMultiple = true)]
public sealed class CoversAttribute : Attribute
{
    public CoversAttribute(
        string spec,
        string req,
        string scenario,
        RtmScope scope,
        RtmQuantification quant,
        RtmOracle oracle = RtmOracle.Direct)
    {
        Spec = spec;
        Req = req;
        Scenario = scenario;
        Scope = scope;
        Quant = quant;
        Oracle = oracle;
    }

    public string Spec { get; }
    public string Req { get; }
    public string Scenario { get; }
    public RtmScope Scope { get; }
    public RtmQuantification Quant { get; }
    public RtmOracle Oracle { get; }
}
