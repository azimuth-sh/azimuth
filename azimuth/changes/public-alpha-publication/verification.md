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

The real operation supplies evidence unavailable to synthetic rehearsal. Run 31902402263 stopped
at npm after six targets became publicly retrievable. Read-only run 31902521510 preserved those six
and selected four package targets. The two NuGet absences were observed only two minutes after the
write attempt and may reflect indexing delay, so another observation is required before resumption.
The final rerun must retrieve every settled exact target, preserve it and select only the remaining
missing set.

Read-only run 31902967972 later retrieved both NuGet packages but rejected their raw checksums.
Direct inspection found that every retained path and payload remained equal and NuGet.org had added
only `.signature.p7s`; `dotnet nuget verify --all` accepted both repository signatures and reported
the same content hashes for the retained and published forms. Regression evidence must vary archive
order and signature bytes without changing payload identity, and must reject an invalid signature
or changed payload.

Targeted rerun attempt 2 of run 31902402263 preserved the tagged SHA but GitHub skipped the
image-provenance job again because its failed dependency remained failed. This establishes that job
rerun is not a recovery mechanism for this dependency shape. The repair path instead requires two
independent attestation lookups around the deterministic OCI index mapping: retained archive at the
candidate revision and published digest at the recorded publication revision. Removing either
lookup must fail completion.

## Public completion

Fresh provider reads after the final write cover all five package identities, three native assets,
both image indexes, each selected image platform, checksums and required provenance. The completion
receipt is retained as operational evidence and its exact run and artifact URLs are recorded in the
change outcome. A failed, partial or unverifiable read is a release failure, not a residual accepted
by elapsed time.

## Residual: provider authorization probes vary

The write-enabled run on 2026-08-15 demonstrated that a crates.io `publish-new` token cannot use the
legacy `/me` identity endpoint and that a repository read does not prove a job-scoped GitHub token's
declared write permissions. NuGet and GHCR likewise expose no non-mutating request that proves
permission to create an unused identity. Presence and authenticated npm administration are weaker
than complete publish authorization. The outcome must name which adapters proved identity and
which first learned authorization from an attempted write.
