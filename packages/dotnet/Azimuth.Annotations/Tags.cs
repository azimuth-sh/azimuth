using System;

namespace Azimuth.Annotations
{
    /// <summary>
    /// Declares that a production-code site is on a claim's path, by the stable
    /// <c>(spec-id, scenario-id)</c> pair.
    /// </summary>
    /// <remarks>
    /// The pair, not a triple: the requirement id is redundant because scenario ids are unique per
    /// spec, and redundancy that can go stale is a liability. Dropping it is what makes splitting or
    /// merging a requirement free — scenarios move between parents without a tag being touched.
    /// <para>
    /// Carries no form. Form is how a <em>test</em> checks a behaviour, not a property of code.
    /// </para>
    /// </remarks>
    /// <remarks>
    /// The targets match exactly what the extractor walks — types and methods. Permitting a target
    /// the emitter does not read would let a tag vanish silently, which is the one failure a
    /// linkage tag must not have.
    /// </remarks>
    [AttributeUsage(
        AttributeTargets.Class
        | AttributeTargets.Struct
        | AttributeTargets.Interface
        | AttributeTargets.Enum
        | AttributeTargets.Method,
        AllowMultiple = true)]
    public sealed class RealizesAttribute : Attribute
    {
        /// <summary>Tags a site as being on the path of <paramref name="scenario"/>.</summary>
        public RealizesAttribute(string spec, string scenario)
        {
            Spec = spec;
            Scenario = scenario;
        }

        /// <summary>Stable spec id.</summary>
        public string Spec { get; }

        /// <summary>Stable scenario id, unique within the spec.</summary>
        public string Scenario { get; }
    }

    /// <summary>Declares that a production artifact implements a named design mechanism.</summary>
    /// <remarks>
    /// The design owns the mechanism's identity, enforcement kind and rationale. The compiler
    /// extractor derives the concrete symbol binding from the attributed type or method, so a code
    /// rename cannot leave a hand-written symbol path behind. Removing the attribute leaves the
    /// independent design declaration unresolved.
    /// </remarks>
    [AttributeUsage(
        AttributeTargets.Class
        | AttributeTargets.Struct
        | AttributeTargets.Interface
        | AttributeTargets.Enum
        | AttributeTargets.Method,
        AllowMultiple = true)]
    public sealed class ImplementsMechanismAttribute : Attribute
    {
        /// <summary>Links the attributed symbol to a design mechanism.</summary>
        public ImplementsMechanismAttribute(string spec, string mechanism)
        {
            Spec = spec;
            Mechanism = mechanism;
        }

        /// <summary>Stable id of the design's spec.</summary>
        public string Spec { get; }

        /// <summary>Stable mechanism id within the design.</summary>
        public string Mechanism { get; }
    }

    /// <summary>Identifies a source method that implements a project-global Check.</summary>
    /// <remarks>
    /// The marker declares implementation identity only. The repository-owned verification file
    /// declares the Check's Claim bindings, evidence form, context and Qualification.
    /// </remarks>
    [AttributeUsage(AttributeTargets.Method, AllowMultiple = true)]
    public sealed class ImplementsCheckAttribute : Attribute
    {
        /// <summary>Marks this method as one implementation site of <paramref name="check"/>.</summary>
        public ImplementsCheckAttribute(string check)
        {
            Check = check;
        }

        /// <summary>Stable project-global Check id.</summary>
        public string Check { get; }
    }
}
