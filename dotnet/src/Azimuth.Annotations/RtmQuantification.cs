namespace Azimuth.Annotations;

/// <summary>
/// The logical shape of a check's claim — the second gated form sub-axis (with
/// <see cref="RtmScope"/>). <see cref="Example"/> is existential (∃: for THIS input, THAT output);
/// <see cref="Invariant"/> is universal (∀: a property over the whole space of inputs/states).
/// Completeness ("no realized case is lost") is a named invariant, not a separate value.
/// </summary>
public enum RtmQuantification
{
    /// <summary>One case — ∃: for this input, that output.</summary>
    Example,

    /// <summary>A property over all inputs/states — ∀: under no X does Y happen.</summary>
    Invariant,
}
