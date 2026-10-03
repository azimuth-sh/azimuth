# Intent delta: framework/change-lifecycle

## Add claim: proposed-intent-has-an-inspectable-target
Criticality: routine

A proposed intent delta has a reviewable target spec and before/after account without changing accepted intent.

### Add case: new-spec-target-is-visible-before-implementation
When a change adds a Claim to a previously absent spec, preview shows the complete target spec and leaves the accepted model unchanged.

### Add case: existing-spec-surroundings-remain-visible
When a change updates an existing spec, preview shows the resulting spec with unaffected Claims and non-normative text preserved.

## Add claim: accepted-intent-changes-only-through-reviewed-application
Criticality: routine

Accepted intent changes only when the explicit application command installs the target approved for that change; finalization and archival never perform that edit.

### Add case: reviewed-target-is-applied-at-acceptance
When implementation supports an approved delta and application receives its current reviewed preview, the current spec becomes that target.

### Add case: unfinished-change-does-not-publish-intent
When a change is proposed or implementation remains unfinished, preview and change inspection leave the accepted spec unchanged.

## Add claim: intent-removal-is-explicit-and-reference-safe
Criticality: routine

Removing a Claim or Case requires a named decision and does not leave current references to the removed identity.

### Add case: replacement-cannot-silently-drop-a-case
When a replacement omits an accepted Case without naming it in `Remove cases:`, projection rejects the target.

### Add case: referenced-deletion-is-rejected
When a current realization, design, verification or challenge-plan declaration still refers to a Claim or Case proposed for removal, projection refuses the removal.

### Add case: last-claim-removal-retires-the-spec
When an explicitly removed Claim is the spec's last Claim and has no current references, the target retires that spec file.

## Add claim: stale-intent-decisions-are-rejected
Criticality: routine

Application refuses a reviewed intent decision when its accepted source or target preview has changed since review.

### Add case: accepted-block-changes-after-review
When an accepted Claim or Case block differs from the replacement or removal's captured `From` value, application refuses to change it.

### Add case: target-preview-changes-after-review
When a delta or surrounding accepted spec changes after a preview is saved, application refuses that saved preview and requires renewed review.
