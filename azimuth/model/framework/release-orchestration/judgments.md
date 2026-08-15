# Judgments: framework/release-orchestration

## Claim: ordinary-ci-excludes-release-only-matrix
Verdict: sound
Fingerprint: fabf9beab468ce97
Judged: 2026-08-15
Judge: Codex

I inspected the sole ordinary workflow command, the root gate's explicit release-image branch, the
workflow account guard and the revision-bound receipt for GitHub run 31860141749. The canonical
`./scripts/check.sh` job completed in 456 seconds without selecting that branch. A wrong workflow
with an extra command, an implicit image qualification or a duration at the 45-minute limit would
fail receipt validation or the static account. The e2e and universal declaration therefore covers
the complete bounded workflow rather than inferring ordinary cost from a local run.

## Claim: selected-lanes-are-independent
Verdict: sound
Fingerprint: a0f4756878d1845c
Judged: 2026-08-15
Judge: Codex

I inspected the four-job DAG, both non-fail-fast matrices, the per-lane artifact uploads and the
account's `always()` dependency over all producer jobs. The mutation loop removes each lane and the
static account rejects it. Earlier GitHub run 31859874354 supplied the relevant failure case: its
package lane failed while all three native lanes and both image lanes completed and retained their
artifacts, after which the account failed closed. A wrong serial dependency or fail-fast matrix
would contradict both the source account and that observed failure boundary.

## Claim: complete-account-needs-every-lane
Verdict: sound
Fingerprint: f10b03377f445d27
Judged: 2026-08-15
Judge: Codex

I inspected catalog-derived subject enumeration, recursive file indexing and the account assembly
loop. The evidence removes and duplicates every one of the ten selected filenames independently
and adds an unexpected file; every mutation fails before an account is returned. GitHub run
31860141686 then downloaded all six lane artifacts and assembled exactly ten subjects. A wrong
assembler accepting one absent, duplicated or extra output is discriminated across the full
catalog population, so the component-universal tag is honest.

## Claim: tag-catalog-and-revision-agree
Verdict: sound
Fingerprint: 52728289641e327c
Judged: 2026-08-15
Judge: Codex

I inspected the catalog tag/version checks, full-commit validation and `git rev-list` comparison
against the annotated tag. The test constructs a real repository and tag, then rejects a different
full revision; independent invalid tag and short-revision cases exercise the other inputs. The
hosted candidate account records tag `v0.1.0-alpha.1` and execution revision
`7d55069e8c9400a15001c23d6f177680d9495a17`, which the workflow tagged before assembly. A tag
pointing anywhere else cannot pass this choke point.

## Claim: retained-downloads-have-checksums
Verdict: sound
Fingerprint: 8bd85f3427119c5d
Judged: 2026-08-15
Judge: Codex

I inspected the exact filename index, byte-size and streaming SHA-256 account, and the verifier's
comparison with retained files. The mutation ranges over every selected subject and changes its
bytes; each must fail size or digest comparison. I also hashed all ten artifacts downloaded from
GitHub run 31860141686 and matched every digest to `candidates.json`, whose own digest is
`8b3e293debb617f9dc4695b4da0fd2107758c8c375afd98e46e7d21631c09059`. A substituted retained
download therefore cannot retain a passing account.

## Claim: executable-subjects-have-provenance
Verdict: sound
Fingerprint: aa37ed73c4e4aad8
Judged: 2026-08-15
Judge: Codex

I inspected each provenance step and queried GitHub's attestation API using the ten retained
SHA-256 digests from run 31860141686. The signed bundle population contains all five package
subjects plus each of the three native archives and two OCI archives; every bundle names workflow
`.github/workflows/release.yml` and execution revision
`7d55069e8c9400a15001c23d6f177680d9495a17`. A missing or substituted subject would fail the
receipt population or digest lookup. The judgment does not extend this to future GHCR image
digests; that rollout-dependent limitation is the claim's explicit accepted residual.

## Claim: packed-packages-install
Verdict: sound
Fingerprint: e939676af6dace95
Judged: 2026-08-15
Judge: Codex

I inspected all three disposable consumer implementations and the hosted package lane. Cargo
installs the retained crate with `--locked`; NuGet restores both retained packages from an isolated
source and invokes their entry points; npm installs both tarballs into a temporary consumer and
exercises annotation and emitter behavior. Run 31860141686 completed that exact five-package path.
A package that only works through workspace source, omits its executable or exports the wrong API
would fail before the lane artifact is accepted.

## Claim: native-binaries-run
Verdict: sound
Fingerprint: 5b3d67ef9d095452
Judged: 2026-08-15
Judge: Codex

I inspected the catalog-derived native matrix, runner mapping, archive construction and extraction
path. Each lane executes the binary from its retained archive and compares `--version` with the
catalog. GitHub run 31860141686 passed on Linux x86-64, macOS ARM64 and Windows x86-64. A wrong
archive name, missing executable bit, incompatible binary or version drift would fail on its
selected runner, so the universal population is the complete three-target catalog set.

## Claim: selected-image-platforms-start
Verdict: sound
Fingerprint: 68f2ea2e57aa8ded
Judged: 2026-08-15
Judge: Codex

I inspected catalog-derived image matrices, recursive OCI platform inspection, per-platform
`skopeo` import and both startup oracles. The API uses a real PostgreSQL container and `/health`;
the web candidate must answer HTTP through its declared port. Run 31860141686 exercised AMD64 and
ARM64 for both images. Dropping an index manifest, producing a non-starting architecture or
bypassing the declared entry point fails before attestation and upload, so all four selected pairs
are covered.

## Claim: exact-existing-target-is-preserved
Verdict: sound
Fingerprint: 440302f933fb5c36
Judged: 2026-08-15
Judge: Codex

I inspected the planner's complete target map and its identity, digest and image-platform equality
checks. The exact-state evidence supplies all ten subjects and requires an empty publish set plus
ten preserved keys. The same test changes the kind-specific comparison fields and requires a
conflict, so an implementation that simply preserved every existing identity would not pass. The
loop contains no subject-specific exception; the component-universal tag therefore follows the
catalog-derived population.

## Claim: absent-target-is-selected
Verdict: sound
Fingerprint: ad681196e593b10b
Judged: 2026-08-15
Judge: Codex

I inspected the absent branch in the planner and the mutation loop over the complete ten-subject
account. Each mutation removes exactly one target and requires that target alone in `publish` and
absent from `preserve`. A wrong planner that republishes an exact neighbor, skips one ecosystem or
selects the whole release after a partial failure would fail on the affected iteration. The direct
component evidence ranges over every selected target rather than one representative absence.

## Claim: conflicting-target-fails
Verdict: sound
Fingerprint: f611a40307182dd3
Judged: 2026-08-15
Judge: Codex

I inspected the single pre-plan conflict accumulator and its uniform loop over every selected key.
The evidence changes the checksum for a package, identity for a native archive and platform set for
an image, requiring each registry kind to raise before returning any plan. These mutations cover
the three distinct comparison shapes; the implementation contains no identity-specific branch
that could exempt another subject. A wrong comparison that ignored content or image platforms
would therefore accept one of the constructed conflicts and fail the oracle.

## Claim: completion-needs-public-retrieval
Verdict: sound
Fingerprint: 21df9acf96e84c8c
Judged: 2026-08-15
Judge: Codex

I inspected completion as the planner's only success boundary. The test removes every target and
removes provenance from every target independently; both populations must fail, while platform
drift is also rejected for each image. A constant-complete result or a check of only one registry
kind cannot pass. This evidence establishes the component guard over supplied registry state, not
fresh public retrieval. The claim and design state that missing adapter explicitly as an accepted
rollout-dependent residual, so this verdict does not authorize calling the alpha publication
complete before real public reads discharge it.
