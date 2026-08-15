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

Run 31905024636 was rejected before checkout because the `release` environment admitted only `v*`
tags while reviewed repair code had to execute after the immutable candidate tag. Recovery used an
ephemeral branch pinned to the reviewed repair revision and one exact branch policy; it did not
broaden `main`. The branch and policy are operational scaffolding and must be removed after the
corrected completion run.

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

No-write run 31905158474 executed from reviewed publication revision
`f6da5064c5448f15fe64d63b8ff72bd567e3cd84` against unchanged candidate revision
`49d350b9d3cacc1cfddd8874b97ba67301090960`. It preserved eight exact targets, selected only the two
npm packages and recorded zero writes. Write run 31905266399 published exactly those two tarballs,
attached both image-digest attestations and retrieved all ten targets. Its completion receipt is not
final evidence: independent `npm view` calls found `alpha` and `latest` both at `0.1.0-alpha.1` for
both packages. That observation falsified the then-current completion oracle.

## npm distribution tags

Focused tests distinguish immutable tarball state from mutable distribution-tag state. Each absent
npm package must enter both the publication and normalization sets. An exact tarball with `latest`
at the prerelease must remain preserved, enter only the normalization set and fail completion. The
normalizer must retain the derived `alpha` tag, remove `latest` only when it points to the alpha and
verify the resulting public tag set. The final hosted receipt must contain the fresh distribution
tags for both npm targets; independent reads must agree.

## Public completion

Fresh provider reads after the final write cover all five package identities, three native assets,
both image indexes, each selected image platform, checksums, npm channel tags and required
provenance. The completion receipt is retained as operational evidence and its exact run and
artifact URLs are recorded in the change outcome. A failed, partial or unverifiable read is a
release failure, not a residual accepted by elapsed time.

## Residual: provider authorization probes vary

The write-enabled run on 2026-08-15 demonstrated that a crates.io `publish-new` token cannot use the
legacy `/me` identity endpoint and that a repository read does not prove a job-scoped GitHub token's
declared write permissions. NuGet and GHCR likewise expose no non-mutating request that proves
permission to create an unused identity. Presence and authenticated npm administration are weaker
than complete publish authorization. The outcome must name which adapters proved identity and
which first learned authorization from an attempted write.
