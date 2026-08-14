# Judgments: framework/release-artifacts

## Claim: one-source-version
Verdict: sound
Fingerprint: 348309ebeec8fb00
Judged: 2026-08-14
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
Fingerprint: 6265ac892beeb8e9
Judged: 2026-08-14
Judge: Codex

I inspected the seven identities in the independent approved account, their catalog entries and
the identity readers for Cargo, NuGet and npm candidates. The catalog validator rejects duplicate
identities, and the image identities are compared as a complete keyed platform map. A wrong
implementation that renamed, omitted, duplicated or added a selected identity would differ from
the approved account; a package whose packed native identity differed from its catalog identity
would fail archive-metadata validation. The evidence is discriminating for the complete selected
registry set.

## Claim: packed-contents-are-bounded-and-licensed
Verdict: sound
Fingerprint: b31d51025fd5e5fb
Judged: 2026-08-14
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
Fingerprint: 65c7d7b1d7873c5f
Judged: 2026-08-14
Judge: Codex

I inspected the independently declared three native targets, both complete image platform sets and
the five supported framework surfaces, plus the README checks that require their human-readable
platform names. The mutation test removes each set dimension and proves that the direct approved
account oracle rejects the drift. A wrong implementation that silently dropped or substituted a
target, platform or supported surface could not emit a qualification result. This directly tests
the complete approved support account rather than inferring support from repository contents.

## Claim: experimental-source-is-not-published
Verdict: sound
Fingerprint: 66a68e0b4fd8d15b
Judged: 2026-08-14
Judge: Codex

I inspected all eleven approved experimental roots, every selected public manifest and the
catalog rule that rejects a public package beneath an experimental root. The tests remove an
experimental root and move a public manifest under one, and both mutations fail. A wrong
implementation that omitted an approved experimental family from the account or assigned a public
package manifest inside it would therefore be rejected. The evidence ranges over both complete
sets named by the predicate and does not treat the absence of current manifests as proof.
