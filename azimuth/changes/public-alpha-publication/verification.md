# Verification: public-alpha-publication

## Registry-state adapters

Provider fixtures cover absent and exact package responses, malformed Cargo, npm and GHCR
responses, unauthorized, rate-limited and server-error package responses, GHCR command failures
and a conflicting GitHub Release support asset. Mutations range over the complete catalog-derived
target set. Tests require an adapter failure to prevent planner output and require exact digest or
content identity rather than version-string equality alone.

## No-write hosted preflight

A pull-request or manual dry run downloads retained candidates, verifies the tag/revision/account
relation, reads live public state and evaluates credential readiness without a publish command.
The workflow log and retained preflight receipt must show the complete selected population and zero
writes. The run must originate from `azimuth-sh/azimuth` after the artifact identity revision;
pre-transfer candidates cannot satisfy the revised catalog. Secret values never enter artifacts or
logs.

## Interrupted publication

The real operation supplies evidence unavailable to synthetic rehearsal. If it stops after any
successful immutable publication, the next run must retrieve that target as exact, preserve it and
select only the missing set. An uninterrupted first run cannot demonstrate this branch; in that
case the synthetic exhaustive mutation evidence remains the detector qualification and the public
receipt establishes only complete publication.

## Public completion

Fresh provider reads after the final write cover all five package identities, three native assets,
both image indexes, each selected image platform, checksums and required provenance. The completion
receipt is retained as operational evidence and its exact run and artifact URLs are recorded in the
change outcome. A failed, partial or unverifiable read is a release failure, not a residual accepted
by elapsed time.

## Residual: provider authorization probes vary

Some registries may not expose a non-mutating request that proves permission to create an unused
name. Presence and authenticated identity are weaker than publish authorization. The outcome must
name which adapters proved authorization and which first learned it from an attempted write.
