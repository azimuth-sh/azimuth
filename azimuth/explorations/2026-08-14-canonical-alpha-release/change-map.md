# Candidate change map: Canonical Azimuth alpha release

Status: archived with approved exploration

This map identifies semantic boundaries and dependencies. Its names are candidates, not created
change ids. A downstream proposal owns its own scope and cites the exploration decisions it carries.

## Invariants

- One change id has one repository authority in a complete project account.
- Existing active change ids transfer; no work package recreates them locally.
- Shared artifact identities, versions and release contracts remain coordinator-owned.
- Worker paths must be non-overlapping before any agent delegation occurs.
- Publication is an operation after accepted changes and evidence, not an Azimuth change itself.

## Dependency graph

```text
A. Separate domain-owned assurance observations          [azimuth-demo authority]
                              |
                              v
B. Establish canonical Azimuth authority                 [repository operation]
                              |
              +---------------+----------------+
              |               |                |
              v               v                v
C1. Accept transferred    C2. Release       C3. Experimental-source
    generic changes           artifact          isolation
                              contract
              |               |                |
              +---------------+----------------+
                              |
                              v
D. Private assurance deployment                           [drim-dev/azimuth]
                              |
                              v
E. Multi-registry release orchestration                   [drim-dev/azimuth]
                              |
                              v
F. Publish v0.1.0-alpha.1                                 [release operation]
                              |
                              v
G. Drim referrals dogfood                                  [separate activity]
```

C1, C2 and C3 may proceed independently after B. D depends on the accepted assurance semantics.
E depends on every retained release lane; F depends on E's qualification account. G is deliberately
outside the release completion boundary.

## A — Separate domain-owned assurance observations

Authority: `azimuth-demo`.

Revise the mixed `generic-assurance-observations` account so that only its generic protocol and
framework mechanism remain transferable. Create one demo-local change for the two ride-hailing
alert claims and Prometheus adoption. This boundary must exist before repository observations can
name singular authorities for the transferred and retained work.

Carries: CAR16, CAR17.

## B — Establish canonical Azimuth authority

This is a controlled repository transition rather than a duplicate local proposal. Tag the old tip,
replace the obsolete tree in one explicit commit, transfer generic source and active change records,
and establish standalone project identity. The transition must preserve history and leave no
current dependency on this checkout.

Carries: CAR1, CAR2, CAR16, CAR17.

## C1 — Accept transferred generic changes

Authority: `drim-dev/azimuth` after B.

Continue the existing identifiers rather than creating replacements. Their semantic dependency is:

```text
reusable-evidence-qualification
              |
              v
assurance-service-reference
              |
              v
assurance-project-snapshots

declared-surfaces-and-obligations       generic-assurance-observations
              |                                      |
              +------------------+-------------------+
                                 v
                         canonical acceptance
```

Acceptance requires fresh standalone evidence under the destination authority. Evidence executed
only against the mixed demo checkout cannot establish destination independence.

Carries: CAR4, CAR16.

## C2 — Release artifact contract

Authority: `drim-dev/azimuth`.

Define the synchronized version, registry identities, license metadata, packed contents, supported
versus experimental surfaces and qualified platform matrix. It owns shared artifact contracts; no
language-specific work package may redefine them.

Carries: CAR2–CAR10.

## C3 — Experimental-source isolation

Authority: `drim-dev/azimuth`.

Move polyglot implementations and generic experiments as explicitly experimental source with
self-contained CI. Replace references to the retained real-domain laboratory with immutable
provenance citations. No test in this lane may mount or read the demo fixture.

Carries: CAR3, CAR17.

## D — Private assurance deployment

Authority: `drim-dev/azimuth`.

Turn the current evaluation Compose shape into the agreed private single-team deployment example.
Its evidence must distinguish private network containment, application authentication and
internet-facing hardening; only the first is claimed. Establish forward-only persistence and both
selected image architectures.

Depends on: B and the assurance path in C1.

Carries: CAR9, CAR14, CAR15.

## E — Multi-registry release orchestration

Authority: `drim-dev/azimuth`.

Coordinate independent qualification lanes from one tag, retain immutable outputs, attach
checksums and provenance, and resume only missing targets after a partial failure. Clean-room and
packed-consumer evidence converge here without collapsing their separate failure boundaries.

Depends on: C1, C2, C3 and D.

Carries: CAR5–CAR13.

## F — Publish `v0.1.0-alpha.1`

This is a release operation after E is accepted. It is incomplete until every selected registry
target can be retrieved and verified against the tag, version, checksum, provenance and platform
smoke account. A partial publication retains successful targets and resumes missing ones.

Depends on: E and successful reservation of the `@azimuth` npm scope.

Carries: CAR5, CAR8, CAR11–CAR13.

## G — Drim referrals dogfood

This is a separate activity in `drim-dev/drim-dev`, not a release dependency. It removes the old
product-side Azimuth/OpenSpec state, pins the public alpha packages and image digests, and authors a
new referral slice through the released workflow. Its findings may motivate `alpha.2`; they do not
change whether `alpha.1` was completely published.

Depends on: F.

Carries: CAR7, CAR11.
