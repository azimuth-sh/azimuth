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

## Claim: all-experimental-source-is-gated
Scope: component
Quantification: universal
Oracle: direct

The population is derived from every experimental root in the release catalog. The gate resolves
each root through tracked check scripts, command invocations and relative manifest dependencies,
and validates that the hosted workflow invokes the root repository check.

## Claim: experiment-gates-need-no-domain-checkout
Scope: component
Quantification: universal
Oracle: direct

Qualification ranges over every tracked executable input beneath the derived experimental roots,
plus their root gates and hosted workflow. The result records the complete experiment-gate set only
after the canonical root sequence reaches release qualification. Clean-checkout execution remains
a completion condition because the static account alone cannot detect an ambient filesystem
dependency.

## Claim: external-domain-evidence-is-citation-only
Quantification: universal
Oracle: direct

The evidence ranges over every tracked URL and local locator naming retained domain evidence. Only
HTTP links whose path contains a full commit identity are accepted, and executable inputs reject
those links even when pinned.
