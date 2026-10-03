using System;

namespace Azimuth.Annotations
{
    /// <summary>
    /// Declares that a production-code site is on a claim's path, by its stable project-wide Claim ID.
    /// </summary>
    /// <remarks>
    /// Cases remain repository-owned evidence addresses. They do not enter source markers, so
    /// refining a Claim's Cases does not duplicate or churn production linkage.
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
        /// <summary>Tags a site as being on the path of <paramref name="claim"/>.</summary>
        public RealizesAttribute(string claim)
        {
            Claim = claim;
        }

        /// <summary>Stable project-wide Claim ID.</summary>
        public string Claim { get; }
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
        public ImplementsMechanismAttribute(string mechanism)
        {
            Mechanism = mechanism;
        }

        /// <summary>Stable project-wide Mechanism ID.</summary>
        public string Mechanism { get; }
    }

    /// <summary>Identifies a source method that implements a stable project-wide Check.</summary>
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

        /// <summary>Stable project-wide Check ID.</summary>
        public string Check { get; }
    }

    /// <summary>Identifies source or configuration that supports a verification Element.</summary>
    [AttributeUsage(
        AttributeTargets.Class
        | AttributeTargets.Struct
        | AttributeTargets.Interface
        | AttributeTargets.Enum
        | AttributeTargets.Method,
        AllowMultiple = true)]
    public sealed class SupportsVerificationElementAttribute : Attribute
    {
        /// <summary>Links a source site to a Claim's verification Element.</summary>
        public SupportsVerificationElementAttribute(string claim, string element)
        {
            Claim = claim;
            Element = element;
        }

        /// <summary>Stable project-wide Claim identity.</summary>
        public string Claim { get; }
        /// <summary>Qualified verification Element identity.</summary>
        public string Element { get; }
        /// <summary>Optional workspace-relative configuration artifact used by this Element.</summary>
        public string? ConfigurationFile { get; set; }
    }

    /// <summary>Defines one independently decidable verification proposition.</summary>
    [AttributeUsage(AttributeTargets.Method, AllowMultiple = true)]
    public sealed class DefinesVerificationCheckAttribute : Attribute
    {
        /// <summary>Declares a Check's terminal proposition and method Element.</summary>
        public DefinesVerificationCheckAttribute(string check, string terminal, string element)
        {
            Check = check;
            Terminal = terminal;
            Element = element;
        }

        /// <summary>Stable project-wide Check identity.</summary>
        public string Check { get; }
        /// <summary>One independently decidable terminal proposition.</summary>
        public string Terminal { get; }
        /// <summary>Qualified method Element identity.</summary>
        public string Element { get; }
    }

    /// <summary>States how a source-authored Check bears on an existing Case.</summary>
    [AttributeUsage(AttributeTargets.Method, AllowMultiple = true)]
    public sealed class ContributesCheckToCaseAttribute : Attribute
    {
        /// <summary>Connects a method's Check to an existing Case.</summary>
        public ContributesCheckToCaseAttribute(
            string check,
            string caseId,
            string element,
            string proposition)
        {
            Check = check;
            Case = caseId;
            Element = element;
            Proposition = proposition;
        }

        /// <summary>Stable project-wide Check identity.</summary>
        public string Check { get; }
        /// <summary>Stable project-wide Case identity.</summary>
        public string Case { get; }
        /// <summary>Qualified method Element identity.</summary>
        public string Element { get; }
        /// <summary>How this Check bears on the Case.</summary>
        public string Proposition { get; }
    }
}
