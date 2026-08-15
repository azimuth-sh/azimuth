# Judgments: framework/release-orchestration

## Claim: ordinary-ci-excludes-release-only-matrix
Verdict: sound
Fingerprint: afeacdd98483c953
Judged: 2026-08-15
Judge: Codex

I inspected the sole ordinary workflow command, the root gate's explicit release-image branch, the
workflow account guard and the revision-bound receipt for GitHub run 31860141749. The canonical
`./scripts/check.sh` job completed in 456 seconds without selecting that branch. The transfer change
accepts the historical URL only as one of two exact repository identities; it does not weaken the
workflow, root-gate digest or duration checks. Diagnostic run 31874120337 repeated the current root
path in 429 seconds and failed only after it reached the deliberately stale release receipt. A wrong
workflow with an extra command, an implicit image qualification or a duration at the 45-minute limit
would fail receipt validation or the static account. The successful receipt remains the evidence;
the diagnostic run only confirms that the identity revision did not introduce the release matrix.

## Claim: selected-lanes-are-independent
Verdict: sound
Fingerprint: 3f602e5ff9e0e835
Judged: 2026-08-15
Judge: Codex

I inspected the four-job DAG, both non-fail-fast matrices, the per-lane artifact uploads and the
account's `always()` dependency over all producer jobs. The mutation loop removes each lane and the
static account rejects it. Earlier GitHub run 31859874354 supplied the relevant failure case: its
package lane failed while all three native lanes and both image lanes completed and retained their
artifacts, after which the account failed closed. GitHub run 31883286632 passed the revised
complete DAG. A wrong serial dependency or fail-fast matrix would contradict both the source
account and that observed failure boundary.

## Claim: complete-account-needs-every-lane
Verdict: sound
Fingerprint: 6e3979c8b3d299e4
Judged: 2026-08-15
Judge: Codex

I inspected catalog-derived subject enumeration, recursive file indexing and the account assembly
loop. The evidence removes and duplicates every one of the ten selected filenames independently
and adds an unexpected file; every mutation fails before an account is returned. GitHub run
31883286632 then downloaded all six lane artifacts and assembled exactly ten subjects. A wrong
assembler accepting one absent, duplicated or extra output is discriminated across the full
catalog population, so the component-universal tag is honest.

## Claim: tag-catalog-and-revision-agree
Verdict: sound
Fingerprint: 089c602b0715038b
Judged: 2026-08-15
Judge: Codex

I inspected the catalog tag/version checks, full-commit validation and `git rev-list` comparison
against the annotated tag. The test constructs a real repository and tag, then rejects a different
full revision; independent invalid tag and short-revision cases exercise the other inputs. The
hosted candidate account records tag `v0.1.0-alpha.1` and execution revision
`7f2d78ed982e05c9a316fa18ade8e3592fdaa86c`, which the workflow tagged before assembly. A tag
pointing anywhere else cannot pass this choke point.

Run 31882982629 supplied the adverse case: all producer lanes passed, but the account rejected the
fetched public tag because it named the preceding candidate. The repaired pull-request path forces
only its checkout-local synthetic tag, and run 31883286632 then passed at its merge revision. The
workflow account rejects removal of that PR-only force while owner dispatch retains an existing
fetched tag, so review isolation does not weaken the publication tag boundary.

I also inspected the publication preflight. It requires an annotated tag, reuses the retained
account verifier and independently compares the rehearsal run revision with the account and peeled
tag revision before registry state or credentials can authorize a write. This realization
establishes the same predicate at the public-operation boundary rather than merely transporting the
tag value. The focused wrong-checkout test observes both tag commands and requires the requested
root as their working directory; resolving either command against the agent's current checkout no
longer passes.

## Claim: retained-downloads-have-checksums
Verdict: sound
Fingerprint: 1d6bd5fad596f352
Judged: 2026-08-15
Judge: Codex

I inspected the exact filename index, byte-size and streaming SHA-256 account, and the verifier's
comparison with retained files. The mutation ranges over every selected subject and changes its
bytes; each must fail size or digest comparison. I also hashed all ten artifacts downloaded from
GitHub run 31883286632 and matched every digest to `candidates.json`, whose own digest is
`0d58e655f9801b8f8ee90886a2e163c7910215d3c64986d412edf851cfdb5500`. A substituted retained
download therefore cannot retain a passing account.

The publication preflight downloads the cross-run candidates, invokes that verifier and derives an
image registry digest only after the retained OCI archive checksum passes. The public account keeps
the archive checksum separately, so this realization does not substitute a registry manifest
digest for retained-byte identity. The image-provenance job now downloads the complete candidate
population and runs the same verifier before deriving its registry-attestation subject; a partial
or substituted cross-run download therefore fails at that second consumption boundary too.

## Claim: executable-subjects-have-provenance
Verdict: sound
Fingerprint: fc58cedd783a0bb8
Judged: 2026-08-15
Judge: Codex

I inspected each provenance step and queried GitHub's attestation API using the ten retained
SHA-256 digests from run 31883286632. The signed bundle population contains all five package
subjects plus each of the three native archives and two OCI archives; every bundle names workflow
`.github/workflows/release.yml` and execution revision
`7f2d78ed982e05c9a316fa18ade8e3592fdaa86c`. A missing or substituted subject would fail the
receipt population or digest lookup. The judgment does not extend this to future GHCR image
digests. The owner workflow now names each published index digest and requests registry-attached
GitHub provenance after publication, but that path has not executed. The rollout-dependent
limitation therefore remains the claim's explicit accepted residual rather than becoming evidence
from workflow text.

## Claim: packed-packages-install
Verdict: sound
Fingerprint: a2c1d76712ab657a
Judged: 2026-08-15
Judge: Codex

I inspected all three disposable consumer implementations and the hosted package lane. Cargo
installs the retained crate with `--locked`; NuGet restores both retained packages from an isolated
source and invokes their entry points; npm installs both tarballs into a temporary consumer and
exercises annotation and emitter behavior. Run 31883286632 completed that exact five-package path
using the two `@azimuth-sh` tarballs.
A package that only works through workspace source, omits its executable or exports the wrong API
would fail before the lane artifact is accepted.

## Claim: native-binaries-run
Verdict: sound
Fingerprint: b5a27e8d414fdd11
Judged: 2026-08-15
Judge: Codex

I inspected the catalog-derived native matrix, runner mapping, archive construction and extraction
path. Each lane executes the binary from its retained archive and compares `--version` with the
catalog. GitHub run 31883286632 passed on Linux x86-64, macOS ARM64 and Windows x86-64. A wrong
archive name, missing executable bit, incompatible binary or version drift would fail on its
selected runner, so the universal population is the complete three-target catalog set.

## Claim: selected-image-platforms-start
Verdict: sound
Fingerprint: b52beec44782c9d2
Judged: 2026-08-15
Judge: Codex

I inspected catalog-derived image matrices, recursive OCI platform inspection, per-platform
`skopeo` import and both startup oracles. The API uses a real PostgreSQL container and `/health`;
the web candidate must answer HTTP through its declared port. Run 31883286632 exercised AMD64 and
ARM64 for both renamed images. Dropping an index manifest, producing a non-starting architecture or
bypassing the declared entry point fails before attestation and upload, so all four selected pairs
are covered.

## Claim: exact-existing-target-is-preserved
Verdict: sound
Fingerprint: 152a1808d362cab5
Judged: 2026-08-15
Judge: Codex

I inspected the planner's complete target map and its identity, digest and image-platform equality
checks. The exact-state evidence supplies all ten subjects and requires an empty publish set plus
ten preserved keys. The same test changes the kind-specific comparison fields and requires a
conflict, so an implementation that simply preserved every existing identity would not pass. The
loop contains no subject-specific exception; the component-universal tag therefore follows the
catalog-derived population.

I also inspected the public adapters and write boundary. They retrieve all selected providers into
one state, preserve omitted keys as absent, keep provider errors distinct from absence and rederive
the supplied plan immediately before writes. No provider adapter can independently bypass the
planner for a target classified exact. The npm adapter also rejects non-HTTPS and non-registry
tarball locations before reading bytes, so remote metadata cannot substitute an unrelated local or
network source while retaining an exact classification.

## Claim: absent-target-is-selected
Verdict: sound
Fingerprint: 082c53f0baab7398
Judged: 2026-08-15
Judge: Codex

I inspected the absent branch in the planner and the mutation loop over the complete ten-subject
account. Each mutation removes exactly one target and requires that target alone in `publish` and
absent from `preserve`. A wrong planner that republishes an exact neighbor, skips one ecosystem or
selects the whole release after a partial failure would fail on the affected iteration. The direct
component evidence ranges over every selected target rather than one representative absence.

The new write path consumes only the planner's `publish` keys and recomputes that plan from the
retained account and observed state before dispatch. The adapter tests deliberately remain ordinary
detector evidence: one selected-write example does not replace the existing universal
catalog-derived absence mutation.

## Claim: conflicting-target-fails
Verdict: sound
Fingerprint: ab84db116ba17b36
Judged: 2026-08-15
Judge: Codex

I inspected the single pre-plan conflict accumulator and its uniform loop over every selected key.
The evidence changes the checksum for a package, identity for a native archive and platform set for
an image, requiring each registry kind to raise before returning any plan. These mutations cover
the three distinct comparison shapes; the implementation contains no identity-specific branch
that could exempt another subject. A wrong comparison that ignored content or image platforms
would therefore accept one of the constructed conflicts and fail the oracle.

The provider boundary maps authorization failures, rate limits and malformed responses to errors,
not absence, and separately rejects a conflicting GitHub Release support asset before planning.
Focused fixtures now exercise malformed Cargo, npm, GitHub Release and GHCR responses plus a GHCR
command failure. This realizes fail-closed public observation without overstating those provider
fixtures as new universal Covers evidence.

## Claim: completion-needs-public-retrieval
Verdict: sound
Fingerprint: be20f06edea7c1e3
Judged: 2026-08-15
Judge: Codex

I inspected completion as the planner's only success boundary. The test removes every target and
removes provenance from every target independently; both populations must fail, while platform
drift is also rejected for each image. A constant-complete result or a check of only one registry
kind cannot pass. This evidence establishes the component guard over supplied registry state, not
fresh public retrieval. The claim and design now bind concrete package, GitHub Release and GHCR
adapters plus a post-provenance completion job. No public completion receipt exists, so the
accepted rollout-dependent residual remains. This verdict does not authorize calling the alpha
publication complete before real public reads discharge it.
