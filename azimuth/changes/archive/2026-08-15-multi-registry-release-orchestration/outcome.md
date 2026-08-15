# Outcome: multi-registry-release-orchestration

Status: accepted

## Result

Ordinary CI retains one canonical `./scripts/check.sh` command and excludes the release-only image
matrix. GitHub run 31860141749 executed that command from a clean checkout in 7 minutes 36 seconds,
below the 45-minute job limit.

The separate release workflow derives five packages, three native archives and two image indexes
from `release/artifacts.json`. GitHub run 31862856073 produced all ten subjects, exercised every
declared entry point, attached build provenance, assembled one tag- and revision-bound account and
completed in 1 minute 26 seconds. The account records exact filenames, sizes and SHA-256 checksums
and has digest `5dc651dd73e26703f0784e51387bf353c938df5d9181fa270451733147572e71`.

The target-aware planner preserves exact existing subjects, selects only absent subjects and
rejects conflicting package, native and image states. It does not publish. Public registry reads,
credentials and the final completion decision remain part of the later alpha-publication change.

## Evidence

- The local release gate ran 35 tests, qualified all five package candidates, accounted for two
  image contracts and three native targets, and reported 29 claims with no holes, errors or
  warnings.
- The hosted package lane installed and exercised one Cargo, two NuGet and two npm archives through
  disposable consumers.
- Linux x86-64, macOS ARM64 and Windows x86-64 runners extracted and executed their retained CLI
  archives.
- Both assurance images started from retained OCI archives on Linux AMD64 and ARM64; the API used a
  real PostgreSQL container and the web client answered through its declared port.
- The GitHub attestation API returned signed provenance for 10 of 10 retained SHA-256 digests. Each
  bundle names `.github/workflows/release.yml`, run 31862856073 and execution revision
  `bbe909363bd13a855fa482696b34b19177eac0fe`.
- A failed rehearsal, run 31859874354, left all three native lanes and both image lanes successful
  while the package lane and complete account failed. This exercised the intended failure
  isolation without treating the partial result as release completion.

## Departures

- The ordinary root gate exposes release image qualification only through explicit
  `--release-images`; its canonical no-argument execution does not pay the multi-platform build
  cost.
- Clean runners exposed two npm lockfiles hidden by the repository-wide ignore rule. Both public
  TypeScript packages now track their exact dependency locks.
- Direct execution of `release/candidates.py` did not put the repository root on Python's import
  path. Image inspection now resolves the supplied root before loading the assurance qualifier.
- Clean Git checkouts have no committer identity. Both the annotated-tag test and the workflow's
  temporary rehearsal tag now supply a bounded non-publishing identity.
- Automated review exposed eight defects or maintenance hazards: PR-controlled shell expansion, a
  reversed publication predicate, implicit Python availability, unsafe tar defaults, a port race,
  syntax-coupled workflow inspection, undocumented receipt refresh and one lint violation. All
  eight were corrected before merge.
- Six failed hosted rehearsal attempts preceded the latest accepted run. The sixth showed that
  PowerShell does not read `$TARGET` as an environment variable; the native step now selects Bash
  explicitly on all three runners. No failed run was used as completion evidence.

## Residual decisions

- No public registry credential was used. The publication change must verify the bounded
  crates.io, NuGet, npm, GitHub Releases and GHCR permissions before its first write.
- Registry state is synthetic. The alpha is not complete until fresh public reads bind every
  selected target to the retained account and discharge the completion residual.
- Current image provenance names the retained OCI archives. Published GHCR image-index digests need
  their own provenance and public verification during publication.
- GitHub build provenance does not provide a complete SBOM or cross-ecosystem signing account.
  Those remain outside first-alpha scope.

## Measurements

- Retained candidate subjects: 10 of 10.
- Package consumers: 5 of 5 passed.
- Native runner targets: 3 of 3 passed.
- Image/platform startup pairs: 4 of 4 passed.
- Attested retained digests independently queried: 10 of 10.
- Release-orchestration test methods: 15; complete release test methods: 35.
- Current model: 29 claims in 4 specs; 0 holes, 0 errors and 0 warnings.
- Hosted release rehearsal: latest accepted run in 1 minute 26 seconds after 6 diagnostic failures.
- Hosted ordinary gate: 1 accepted run in 7 minutes 36 seconds.
