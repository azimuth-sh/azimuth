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
npm package must enter both the publication and channel-normalization sets. Retrieval records
stable versions as well as tags. An exact first-only prerelease at both `alpha` and `latest` remains
preserved and may complete because the npm registry package model requires `latest`. The same tags
with any stable version remain preserved but block completion; the normalizer must not guess a
stable target. The final hosted receipt must contain the fresh distribution tags and empty
stable-version populations for both npm targets; independent reads must agree.

Write-enabled run 31907022845 selected no immutable publication and attempted the two planned tag
normalizations. Its first bypass-2FA-token deletion returned HTTP 403, so it emitted no completion
receipt. A fresh interactive login and WebAuthn authorization advanced the same deletion through
the authentication challenge; the registry then returned HTTP 400. The npm registry API describes
a package's `dist-tags` object as containing at least `latest`. Together these observations falsify
the previous removal oracle and support the first-only exception above.

## Public completion

Fresh provider reads after the final write cover all five package identities, three native assets,
both image indexes, each selected image platform, checksums, npm channel tags and required
provenance. The completion receipt is retained as operational evidence and its exact run and
artifact URLs are recorded in the change outcome. A failed, partial or unverifiable read is a
release failure, not a residual accepted by elapsed time.

Write-enabled run 31937065763 retained all ten exact targets and emitted a receipt after reading
GHCR with the workflow token. Independent anonymous reads returned HTTP 403 for both images because
their packages were private, so that receipt is adverse evidence rather than completion evidence.
After the owner changed both packages to public, anonymous registry reads returned API digest
`85c76fa563950b75dc3e5bece5e72618d322aedd9dad965d26dee4679bdac329` and web digest
`25704694ebb7bebbff77832018ba90fb516d502c5795f78604d27e13b2a6a719`; both indexes contained Linux
AMD64 and ARM64. Regression evidence must require `skopeo --no-creds` and reject a completion job
that configures registry authentication. Those observations made a fresh hosted receipt necessary.

No-write run 31938652438 then retrieved all ten targets with the corrected adapter, selected zero
writes and zero npm normalizations, and preserved the two expected image index digests. Final run
31938723090 again preserved all ten targets, attached both image attestations and completed an
anonymous public retrieval at publication revision
`7cabe21714add2c45ce2c4ebcc359464fe527908`. Its retained completion receipt has SHA-256
`6c03769f4ade8709d7356a1629a4fce9617135a3254b8b9724446cdab7eda0ce`. Both npm targets recorded
`alpha` and the provider-required first-version `latest` at `0.1.0-alpha.1` with empty stable-version
populations. All eight downloadable subjects have direct provenance; both images have complete
retained-to-published provenance chains.

## Residual: provider authorization probes vary

The write-enabled run on 2026-08-15 demonstrated that a crates.io `publish-new` token cannot use the
legacy `/me` identity endpoint and that a repository read does not prove a job-scoped GitHub token's
declared write permissions. NuGet and GHCR likewise expose no non-mutating request that proves
permission to create an unused identity. Presence and authenticated npm administration are weaker
than complete publish authorization. The outcome must name which adapters proved identity and
which first learned authorization from an attempted write.
