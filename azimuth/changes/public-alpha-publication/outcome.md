# Outcome: public-alpha-publication

Status: implemented

## Result

Azimuth `0.1.0-alpha.1` is publicly retrievable under tag `v0.1.0-alpha.1`. The retained account
contains ten targets: five ecosystem packages, three native CLI archives and two multi-platform
assurance images. Candidate authority remains commit
`49d350b9d3cacc1cfddd8874b97ba67301090960`; reviewed publication orchestration completed from
commit `7cabe21714add2c45ce2c4ebcc359464fe527908` without rebuilding or moving the tag.

The final operation preserved all ten immutable targets and performed no npm normalization. Both
npm packages expose `alpha` and the provider-required first-version `latest` at
`0.1.0-alpha.1`; neither package has a stable version. Both GHCR indexes are anonymously
retrievable for Linux AMD64 and ARM64. Eight downloadable subjects have direct candidate
provenance, and both image indexes have retained-to-published provenance chains.

## Acceptance evidence

- Pull request 10 passed all nine hosted jobs. The canonical repository check completed in 7m20s;
  package, three native, two image, preparation and account lanes also passed.
- Local release qualification passed 63 tests, built all five real package candidates and reported
  29 claims in four specs with no holes, errors or warnings.
- No-write run 31938652438 preserved ten targets, selected zero publications and zero npm
  normalizations, and recorded zero writes through the anonymous GHCR adapter.
- Final run 31938723090 preserved ten targets, passed both image-provenance jobs and emitted public
  completion after a second anonymous retrieval.
- The retained completion receipt has SHA-256
  `6c03769f4ade8709d7356a1629a4fce9617135a3254b8b9724446cdab7eda0ce` and names candidate account
  SHA-256 `83576507f778b0103fe61f0eb0efe344f105e3a7486cd9384828ceea8664dc9d`.
- Independent registry reads downloaded the five packages and three native archives at their
  recorded digests. Anonymous GHCR reads returned API index digest
  `85c76fa563950b75dc3e5bece5e72618d322aedd9dad965d26dee4679bdac329` and web index digest
  `25704694ebb7bebbff77832018ba90fb516d502c5795f78604d27e13b2a6a719`.

## Departures

The first write attempt exposed that npm does not infer an `alpha` distribution tag and can assign
the first prerelease to `latest`. A later authenticated removal reached npm but returned HTTP 400;
the registry package contract requires `latest`. Completion therefore accepts that alias only
while no stable version exists and refuses to guess a stable target later.

NuGet.org adds a repository signature after ingestion, so raw archive equality classified both
successful publications as conflicts. The accepted identity verifies the repository signature and
compares every non-signature path and payload while retaining both public and candidate digests.

Run 31937065763 authenticated to GHCR before retrieval and consequently accepted two private image
packages as public. Independent anonymous reads returned HTTP 403. The owner made both packages
public, and pull request 10 changed every image read to `skopeo --no-creds` while removing registry
login from completion. Run 31938723090 is the first receipt emitted by that corrected oracle.

## Residual decisions

- A crates.io `publish-new` token, NuGet push key and bounded GitHub workflow token expose creation
  authorization only through their first writes. Resumable planning contains this provider limit;
  it does not turn credential presence into an authorization claim.
- GitHub provenance does not provide a complete SBOM, transparency-log or cross-ecosystem signing
  account. Those remain outside the first-alpha boundary and must be reconsidered before stable.
- The assurance images implement the private deployment profile. Public container distribution
  does not make the assurance service suitable for direct internet exposure.

## Measurements

- public immutable targets: 10 of 10;
- package targets: 5;
- native targets: 3;
- image indexes: 2, each with 2 selected runnable platforms;
- direct provenance subjects: 8;
- retained-to-published provenance subjects: 2;
- final-run immutable writes: 0;
- final-run npm normalizations: 0; and
- accepted-model account before finalization: 29 claims in 4 specs, with 0 holes.
