# Verification: framework/release-artifacts

## Claim: one-source-version
Scope: component
Quantification: universal
Oracle: direct

The claim ranges over every selected package, image and native distribution entry and depends on
the real native manifest formats.

## Claim: registry-identities-match-contract
Scope: component
Quantification: universal
Oracle: direct

Every selected public registry identity is read from its native package candidate or Dockerfile and
compared with the approved release account.

## Claim: packed-contents-are-bounded-and-licensed
Scope: component
Quantification: universal
Oracle: direct

The complete file list and native license metadata of every package candidate are inspected after
the real ecosystem packer runs.

## Claim: support-and-platforms-are-explicit
Quantification: universal
Oracle: direct

The evidence ranges over the complete supported surface, native target and image platform sets in
the approved first-alpha account.

## Claim: experimental-source-is-not-published
Quantification: universal
Oracle: direct

The evidence ranges over every declared experimental source root and every selected public package
manifest, and rejects overlap between those sets.
