# Exploration: Canonical Azimuth alpha release

Id: canonical-alpha-release Created: 2026-08-14 Status: archived Archived: 2026-08-14

## Objective

Replace the frozen contents of the canonical `drim-dev/azimuth` repository, whose local checkout is `~/drim/azimuth`, with the generic framework developed in this repository. Publish the resulting framework as `v0.1.0-alpha.1` for public consumption and subsequent Drim dogfooding.

The release is complete when every selected public artifact has been independently qualified and published. Drim consumption and the referral slice follow as a separate activity; findings from that activity may require `alpha.2`.

## Boundaries

The release includes:

- the Rust CLI and core, including federation as a supported alpha feature;
- .NET and TypeScript annotations and extractors;
- formats, standards, framework documentation and agent skills;
- the optional assurance API and diagnostic web application;
- generic experiments and experimental polyglot implementations, with CI coverage but without published artifacts or a support promise for those languages; and
- Apache-2.0 licensing throughout.

The release excludes:

- ride-hailing application code, model packages and other domain-owned artifacts;
- the real-domain multi-repository laboratory, which remains immutable provenance here;
- referral functionality, which is a separate post-release Drim activity;
- internet-facing assurance-service hardening and built-in authentication; and
- general compatibility guarantees between alpha releases. Forward-only assurance database migrations are the exception because ledger history is durable product data.

Azimuth must build, test, release and accept its transferred changes without a Drim checkout or any other domain. An immutable citation to external evidence records provenance; it is not a build, test, release or acceptance dependency.

## Existing context

Repository inspection during deliberation established the following facts:

- The old Azimuth repository has 21 commits. Its implementation and release configuration predate the framework developed here.
- `azimuth-demo` currently holds the generic framework and the ride-hailing fixture together.
- `tools/azimuth/src/federation.rs` and `tools/azimuth/tests/federation.rs` provide a generic federation implementation and synthetic evidence without reading the real-domain laboratory.
- `.github/workflows/azimuth-release.yml` publishes only the Rust crate and three CLI binaries. It uses `azimuth-v*` tags, declares no release-wide provenance contract and does not cover the other selected artifacts.
- `services/assurance/docker-compose.yml` uses fixed credentials and publishes PostgreSQL and both application ports. `services/assurance/README.md` states that the service has no authentication, tenant isolation, backup policy or availability objective.
- The assurance server already uses SQLx migrations and PostgreSQL. Its API and web application already have production Dockerfiles.
- Five generic changes are implemented and pending acceptance: `assurance-project-snapshots`, `assurance-service-reference`, `declared-surfaces-and-obligations`, `generic-assurance-observations` and `reusable-evidence-qualification`.
- Exact-name registry checks on 2026-08-14 found no package records for the selected crates.io, NuGet and npm identifiers. Search results do not reserve names. The configured npm identity does not control the `@azimuth` scope.

## Findings

### F1 — The transition changes authority

The work is an authority transfer rather than a directory copy, because the destination becomes the only repository allowed to accept generic framework changes. Retaining obsolete files as a second current tree would leave two incompatible accounts of the framework.

### F2 — Federation is separable from its real-domain trial

Federation can ship without the ride-hailing laboratory, because the implementation and its synthetic tests are self-contained. The laboratory remains useful provenance, but moving it would violate the domain-independence boundary.

### F3 — Artifact lockstep is temporary risk control

One version across the first artifact set avoids an untested compatibility matrix. Independent versions become justified only after protocols have explicit versions and cross-version conformance evidence.

### F4 — Public publication is the dogfood input

Drim must consume public packages and immutable image digests to test the actual distribution path. Local pre-publication consumption cannot establish that registry metadata, packed contents and published images compose.

### F5 — Multi-registry publication cannot be atomic

crates.io, NuGet, npm, GitHub Releases and GHCR have independent publication boundaries. A global Git tag identifies one source state, but it cannot make those boundaries transactional. Release recovery must therefore preserve successful artifacts and resume only missing targets.

### F6 — Private deployment is narrower than production hardening

The assurance service can honestly support one team behind a reverse proxy or VPN without adding application authentication. That claim requires restricted port bindings, supplied secrets and a documented network boundary; the current Compose file does not satisfy it.

### F7 — Release breadth is the dominant accepted risk

The complete artifact set creates more qualification work than a CLI-only alpha. Independent qualification lanes localize failures but do not reduce the total work. This risk was retained because deferring assurance or federation would remove parts of the agreed dogfood slice.

## Decisions

### CAR1 — Canonical authority and history

Create an annotated `frozen-alpha` tag at the old repository tip, then replace the obsolete tree in one explicit commit on `main`. `drim-dev/azimuth` becomes canonical; `azimuth-demo` becomes a pinned consumer and domain fixture. This preserves 21 commits while making the replacement visible.

### CAR2 — License

Apply Apache-2.0 throughout. There is no current consumer whose use requires preserving the old component license declarations.

### CAR3 — First artifact set

Publish the Rust CLI and core, .NET and TypeScript integration packages, native binaries, and the assurance API and web images. Formats, standards, skills and documentation ship in the repository. Polyglot implementations remain source-level experiments.

### CAR4 — Federation support

Federation is a supported alpha CLI feature, including `project check`, `export`, `finalize`, `accept-change`, `observe` and `locate`. Its contracts may still change during alpha.

### CAR5 — Unified first version

Every first-release artifact uses `0.1.0-alpha.1` and one annotated Git tag, `v0.1.0-alpha.1`. Components remain in lockstep until explicit protocol versions and cross-version conformance tests make drift safe.

### CAR6 — Evidence-gated maturity

Movement from alpha to beta, release candidate and stable depends on evidence defined for those transitions, not elapsed time. This exploration does not claim that the later gates already exist.

### CAR7 — Consumer pins

Drim pins exact prerelease package versions and assurance image digests. A dogfood result therefore identifies the artifacts it exercised.

### CAR8 — Registry identities

Use the following public identities:

| Registry | Identity |
|---|---|
| crates.io | `azimuth` |
| NuGet | `Azimuth.Annotations`, `Azimuth.Emit` |
| npm | `@azimuth/annotations`, `@azimuth/emit` |
| GitHub Releases | native Azimuth CLI binaries |
| GHCR | versioned assurance API and web images under `drim-dev` |

Claiming `@azimuth` is a release prerequisite, not an inferred consequence of an empty package search.

### CAR9 — Qualified platforms

Qualify CLI binaries for Linux x64, macOS ARM64 and Windows x64. Publish assurance images for Linux AMD64 and ARM64. Other platforms may compile the Rust crate from source but are not advertised as qualified binary targets.

### CAR10 — Independent qualification

Retain the complete scope, but qualify artifacts in independent lanes before one release orchestration gate. This limits the diagnostic radius of a failure while accepting the total scope.

### CAR11 — Publication order and completion

Publish `alpha.1` after independent artifact qualification, then dogfood those exact public artifacts in Drim. The release is complete at verified public publication; referrals remain a separate activity and integration defects become later alpha work.

### CAR12 — Partial publication recovery

Retain verified outputs from the tag and resume only missing registry targets. Never overwrite a successful publication and do not declare the release complete until every selected target exists.

### CAR13 — Executable-artifact provenance

Publish SHA-256 checksums and GitHub build-provenance attestations for downloadable binaries and container images. Complete SBOM and cross-ecosystem signing coverage are not first-alpha claims.

### CAR14 — Private assurance deployment

Publish versioned API and web images plus a secure-by-default private Compose example. Require supplied secrets, persistent PostgreSQL, restricted port bindings and an explicit reverse-proxy or VPN boundary. Do not add application authentication or claim internet-facing readiness.

### CAR15 — Assurance database upgrades

Preserve ledger data with forward-only database migrations. Downgrades are unsupported during alpha. The durability promise requires upgrade evidence whenever a later release changes schema.

### CAR16 — Active-change authority

Transfer the five generic active change identifiers without recreating them. Accept their generic outcomes only in the canonical repository. Split `generic-assurance-observations` before transfer: its protocol moves, while two ride-hailing alert claims and Prometheus adoption remain under a new demo-local change.

### CAR17 — Experiment boundary

Move self-contained generic experiments and all polyglot implementations with CI coverage. Leave the real-domain multi-repository laboratory in `azimuth-demo` as immutable provenance.

## Rejected alternatives and residual risks

- **Repository.** Preserving selected old files was rejected. Reopen for a specific provenance or legal need not met by history and `frozen-alpha`.
- **Scope.** Deferring assurance or experiments was rejected. Reopen if a lane cannot qualify independently or creates a domain dependency.
- **Federation.** Marking it experimental was rejected. Reopen if synthetic evidence fails to establish domain-independent behavior.
- **Versions.** Immediate artifact drift was rejected. Reopen after protocol identities and cross-version conformance tests exist.
- **Dogfood.** Testing unpublished artifacts first was rejected. Reopen before beta if external adoption makes an integration defect too costly.
- **Assurance access.** Adding authentication now was rejected. Reopen before internet exposure, multiple teams or tenant separation.
- **Database.** Permitting alpha resets was rejected. Reopen only with an explicit export-and-recovery account for an unavoidable redesign.
- **Publication.** Abandoning a partial version was rejected. Reopen if a registry prevents safe resumption from retained tagged outputs.
- **Provenance.** Complete SBOM and signing coverage was rejected. Reopen before stable or when a consumer requires stronger evidence.
- **Platforms.** Publishing five CLI targets was rejected. Expand when demand or observed portability failures justify another lane.
- **Federation laboratory.** Moving the real-domain laboratory was rejected. Reopen if immutable citation cannot support a required claim without a domain dependency.

## Open questions

No user-owned release decision remains open. Publication is blocked until these empirical conditions are established:

- the `@azimuth` npm scope can be claimed by the release identity;
- registry credentials and `drim-dev` organization permissions exist;
- a clean canonical checkout passes all generic checks without domain mounts;
- packed Rust, NuGet and npm artifacts install and expose their public entry points;
- private assurance deployment preserves data across restart and upgrade;
- every advertised executable has matching checksums and provenance; and
- an interrupted release can resume a missing target from retained immutable outputs.

The smallest proposed evidence is a clean-room repository rehearsal, disposable packed-artifact consumers, a private assurance deployment lifecycle, a retained-output release-resume rehearsal and post-publication verification of every selected target. These obligations must be made concrete in their downstream changes; this exploration does not manufacture their results.

## Result

The direction and candidate change graph in `change-map.md` were approved conversationally and in file review on 2026-08-14. This account is non-normative and does not authorize a proposal, implementation, repository replacement, publication or Drim modification.
