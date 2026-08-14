# Design: framework-only-change-completion

## Decision: explicit proposal metadata

The declaration is two metadata lines before the first level-two heading:

```text
Intent delta: none
Because: <why accepted intent does not change>
```

The parser accepts exactly one declaration and one non-empty rationale. It does not derive the mode
from the absence of files, the phrase "affected claims: none" or repository paths, because each can
also describe an incomplete proposal.

## Completion boundary

The declaration replaces only the requirement for a supported parsed intent delta. It does not
replace completed plan items, accepted proposal and outcome statuses, required outcome headings, a
hole-free accepted model, finalization freshness or the content-preserving archive move.

The declaration and a parsed addition or criticality transition are contradictory. Failing that
combination prevents the metadata from becoming a general bypass when a real intent transition is
present.

## Rejected alternatives

- Treating every empty `specs/` directory as framework-only was rejected because accidental
  omission would become indistinguishable from a deliberate mechanism change.
- Inferring the mode from `## Affected claims` prose was rejected because prose is not a closed
  parser contract and the phrase also appears in historical records.
- Inventing a product or fixture delta was rejected because archive history would then claim a
  semantic transition that did not occur.
- Removing the delta gate was rejected because undeclared or incomplete changes must continue to
  fail closed.

## Falsifier

The design fails if a proposal can complete with no parsed delta and no explicit rationale, if the
declaration suppresses any existing completion issue, or if a proposal can declare both completion
modes without an error.
