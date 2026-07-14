using System;

namespace Azimuth.Annotations;

/// <summary>
/// Declares that a production-code site is on the path of a spec scenario, by the stable
/// (spec-id, req-id, scenario-id) triple the tests share. It carries no form — form is how a
/// <em>test</em> checks a behavior, not a property of code. A site may realize several scenarios
/// (branches of one method); several sites may realize one scenario (its component fan-out).
/// The RTM reads these to build the matrix's code column.
/// </summary>
[AttributeUsage(AttributeTargets.Method | AttributeTargets.Class, AllowMultiple = true)]
public sealed class RealizesAttribute : Attribute
{
    public RealizesAttribute(string spec, string req, string scenario)
    {
        Spec = spec;
        Req = req;
        Scenario = scenario;
    }

    public string Spec { get; }
    public string Req { get; }
    public string Scenario { get; }
}
