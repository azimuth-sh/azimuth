using System;

namespace Azimuth.Annotations;

/// <summary>
/// Opts a test method out of tracing: it legitimately maps to no spec scenario (setup, infra, or
/// smoke tests). Meaningful only under a traced root — an opt-in area (a namespace prefix) the
/// emitter is told to hold to tracing. There, the untraced-test check (the dual of an uncovered
/// scenario) demands every test method carry either a <c>[Covers]</c> or this opt-out; a bare
/// test declares behavior the spec never named. The <c>reason</c> is recorded so the opt-out is a
/// deliberate, reviewable choice rather than a silent gap.
/// </summary>
[AttributeUsage(AttributeTargets.Method)]
public sealed class UntracedAttribute : Attribute
{
    public UntracedAttribute(string reason)
    {
        Reason = reason;
    }

    public string Reason { get; }
}
