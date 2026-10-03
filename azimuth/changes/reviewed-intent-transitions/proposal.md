# Change: Project and apply reviewed intent transitions

Status: proposed

## Problem

Change deltas can add whole Claims or change criticality, but ordinary revisions and deletions of accepted intent are not expressible. An agent manually copies additions into current specs. Reviewers cannot inspect the exact target spec or detect accepted-source drift before the copy. The archive gate only sees whether the copy appears applied.

## Outcome

An intent owner can review the target spec and its diff while the delta is proposed. At acceptance, Azimuth applies that exact reviewed target mechanically or refuses stale input. Replacements and removals are explicit decisions with guards and reference checks. Finalize and archive remain acceptance checks, never implicit application commands.

Target-spec preview: [target-spec.preview](target-spec.preview).

## Scope

In scope: intent-delta parsing and projection, preview and guarded application commands, machine capture of accepted block fingerprints, exact applied-state checks, reference and bundled-skill guidance, and the current change lifecycle Claim additions. The parser remains zero-dependency. Multi-file filesystem replacement is staged but is not transactional; a failed replacement requires repair and a fresh preview.

Excluded: automatic application during implementation or archive, rename inference, silent Case removal, assurance judgments for routine Claims, and changes to old archived records.

## Affected claims

- Add `framework/change-lifecycle#proposed-intent-has-an-inspectable-target`.
- Add `framework/change-lifecycle#accepted-intent-changes-only-through-reviewed-application`.
- Add `framework/change-lifecycle#intent-removal-is-explicit-and-reference-safe`.
- Add `framework/change-lifecycle#stale-intent-decisions-are-rejected`.

## Completion conditions

- The four proposed Claims and their Cases describe the implemented lifecycle without requiring a natural-language template.
- Preview shows the complete target spec and before/after text for new and existing specs while leaving accepted intent untouched.
- Application accepts only a current reviewed preview and refuses stale `From` guards, undeclared removed Cases, conflicting Claim content, and referenced deletions.
- Finalize and archive require applied intent but do not apply it; current unrelated accepted blocks survive projection.
- Bundled reference, skills, format contract and process guidance describe the same commands and acceptance boundary.
- Run repository-permitted CLI inspection and compilation checks; record pre-existing validation findings without treating them as new regressions. Do not run tests under this repository's current instruction.
