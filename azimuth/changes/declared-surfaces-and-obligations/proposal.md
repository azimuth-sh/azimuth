# Change: declared-surfaces-and-obligations

Status: implemented, pending acceptance

## Problem

Site-domain checking currently receives the assignment between a semantic class and its extractor
through ad-hoc command-line arguments such as `--next-app trips/rider-view=app/web/rider`. The
enumerator derives routes once configured, but the architectural assertion that this application
contributes that surface is neither a validated project declaration nor visible beside the model.

Ordinary claims have the inverse gap. A single backend `Realizes` relation can satisfy linkage even
when an accepted solution requires backend, BFF and web participation. Surface-wide universal
discharge is the wrong repair: an ordinary claim requires at least one realization in each expected
architectural area, not realization by every member of an enumerated set.

## Outcome

Azimuth has validated declarations for repository areas, independently enumerated surfaces and
optional area realization obligations. Areas remain available metadata rather than routine
ceremony. Site-domain claims fail closed without a declared, successfully derived surface;
ordinary claims may require realization in named areas without requiring one test per area.

The proposal/apply/verify skills guide agents to reuse or create a surface when a site-domain claim
is introduced, validate its negative failure path, and judge every obligated realization. Evidence
continues to follow the verification plan's scope, quantification and oracle rather than mirroring
area participation.

## Scope

In scope:

- structured monorepo area declarations and path-derived area attribution;
- surface declarations binding semantic ids to an enumerator and contributing areas;
- replacement of this repository's ad-hoc Next surface assignment with that declaration;
- area realization obligations for non-routine claims;
- machine holes for missing/failed surface derivation, unknown areas and absent obligated
  realizations;
- synthetic and corpus validation, including an untagged-route negative case;
- documentation, glossary, formats and agent-skill guidance.

Out of scope:

- mandatory roles or a controlled role vocabulary;
- one evidence definition or test per realization area;
- inference that arbitrary untagged code realizes prose;
- ASP.NET, queue-consumer or additional framework enumerators;
- automatic call-graph credit for a shared choke point;
- archive or source-control publication.

## Affected claims

No product intent changes. The current `trips/rider-view#position-confined-to-live-phases`
invariant and `referrals/rewards#referral-summary-explains-state` claim are the corpus fixtures used
to validate the two mechanics.

## Completion conditions

- `azimuth check` reads validated area, surface and realization-obligation declarations.
- A site-domain claim with no surface, a surface with an unknown area or failed enumerator, and an
  area obligation with no matching realization each produce distinct errors.
- The rider Next application is assigned to `trips/rider-view` through configuration rather than a
  semantic-id command-line pairing.
- Existing routes pass; a synthetic newly enumerated untagged route fails with
  `invariant-breach`.
- A synthetic backend-only realization fails a required rider-web obligation.
- Tags do not gain mandatory area or role arguments; area is derived from source location.
- Verification documentation states that obligations do not imply area-local tests.
- Targeted tool tests and the repository check pass, with departures and residuals recorded.
