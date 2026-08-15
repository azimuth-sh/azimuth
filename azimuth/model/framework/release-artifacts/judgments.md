# Judgments: framework/release-artifacts

## Claim: one-source-version
Verdict: sound
Fingerprint: 0e427e0f1f7cb92e
Judged: 2026-08-15
Judge: Codex

I inspected the release catalog, the independent approved account, the native metadata readers and
the five real package candidates produced by Cargo, NuGet and npm. The qualifier also reads both
image version manifests and the version argument in each Dockerfile before it emits the linked
result. A wrong implementation that changed one ecosystem version, one image manifest version or
the release tag would fail either the independent account comparison or the native metadata
comparison. The evidence therefore distinguishes the stated one-version predicate over the full
selected set.

## Claim: registry-identities-match-contract
Verdict: sound
Fingerprint: 8810bcfae2bfbe3d
Judged: 2026-08-15
Judge: Codex

I inspected the seven identities in the independent approved account, their catalog entries and
the identity, repository and homepage readers for Cargo, NuGet and npm candidates. The same source
and homepage values are required in both image Dockerfiles. The catalog validator rejects duplicate
identities, and the image identities are compared as a complete keyed platform map. The mutation
test changes the repository, homepage and identity dimensions independently. A wrong package or
image that retained a Drim-owned coordinate, omitted one selected identity or substituted either
metadata URL would fail before qualification is emitted. The evidence discriminates the complete
revised registry set rather than merely observing the new names in the catalog.

## Claim: packed-contents-are-bounded-and-licensed
Verdict: sound
Fingerprint: 159452d73a3c82ad
Judged: 2026-08-15
Judge: Codex

I inspected the real packer invocations, archive readers, complete-file allowlists, required-file
checks and native license readers for all five package candidates. The emitted account records the
number of files matched by each allowed pattern, avoiding generated archive names without hiding a
count change. The unit evidence separately shows that both a missing required file and an
unexpected file fail qualification. A wrong package that omitted its executable or annotation
assembly, included an undeclared test or secret file, or declared a non-Apache license would fail
before the result and linkage manifest were written. The component evidence therefore establishes
both halves of the predicate over every selected package.

## Claim: support-and-platforms-are-explicit
Verdict: sound
Fingerprint: d8d82c883e4e9c68
Judged: 2026-08-15
Judge: Codex

I inspected the independently declared three native targets, both complete image platform sets and
the five supported framework surfaces, plus the README checks that require their human-readable
platform names. The mutation test removes each set dimension and proves that the direct approved
account oracle rejects the drift. A wrong implementation that silently dropped or substituted a
target, platform or supported surface could not emit a qualification result. This directly tests
the complete approved support account rather than inferring support from repository contents.

## Claim: experimental-source-is-not-published
Verdict: sound
Fingerprint: 532c3a497405136c
Judged: 2026-08-15
Judge: Codex

I inspected all eleven approved experimental roots, every selected public manifest and the
catalog rule that rejects a public package beneath an experimental root. The tests remove an
experimental root and move a public manifest under one, and both mutations fail. A wrong
implementation that omitted an approved experimental family from the account or assigned a public
package manifest inside it would therefore be rejected. The evidence ranges over both complete
sets named by the predicate and does not treat the absence of current manifests as proof.

## Claim: external-domain-evidence-is-citation-only
Verdict: sound
Fingerprint: 96012f5761c99523
Judged: 2026-08-15
Judge: Codex

I inspected the tracked-reference enumerator, the executable-input boundary, all three current
domain citations and the test that derives that complete set. The test replaces each derived commit
identity with `main` and requires the direct oracle to reject every mutation; it also injects a
local demo locator. The component qualification separately rejects even a pinned domain URL when
it occurs in an executable input. A wrong account using a branch URL, local checkout path or pinned
link as a build input therefore fails. The exact synthetic fixture is excluded from the production
population because it is the constructed violation rather than a provenance assertion.

## Claim: all-experimental-source-is-gated
Verdict: sound
Fingerprint: 5475d44b447209e3
Judged: 2026-08-15
Judge: Codex

I inspected the catalog-derived population, root-to-command resolver, canonical root sequence,
workflow validator, mutation cases and exact-revision workflow receipt. The account covers all 11
declared roots and rejects a newly unaccounted root, a removed gate and a no-op textual mention.
GitHub run 31811542129 checked out revision
`64992346726bb75af4ea7997f85cf1db33661262` and completed the sole
`./scripts/check.sh` step successfully. The repository transfer validator accepts that historical
receipt only as the exact `drim-dev/azimuth` execution and rejects a repository/run-URL mismatch.
The current isolation account has the same derived fingerprint, and diagnostic run 31874120337
executed the revised experiment gates before failing later at a deliberately stale release receipt.
A dropped relation, bypassed root gate or receipt floating to a different account therefore fails
before emitting this evidence; the diagnostic run is corroboration, not a replacement receipt.

## Claim: experiment-gates-need-no-domain-checkout
Verdict: sound
Fingerprint: ea2f2ad3aec73e51
Judged: 2026-08-15
Judge: Codex

I inspected all 49 derived executable inputs, the local and mounted locator detectors, the clean
checkout workflow and its exact successful receipt. The mutation tests inject both local and
mounted domain paths, and the executable-input scan also rejects domain repository URLs even when
commit-pinned. The accepted hosted runner checked out only the release repository and completed
every experiment gate at the recorded revision. The repository-transfer change preserves that
receipt only under its exact former owner and matching run URL; current diagnostic run 31874120337
repeated the gates in `azimuth-sh/azimuth` before its later stale-receipt failure. A gate reading
the demo checkout through a path, mount or repository URL would be rejected, while an undeclared
ambient dependency would fail on either clean runner.
