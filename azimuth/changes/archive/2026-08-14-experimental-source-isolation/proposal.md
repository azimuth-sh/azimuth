# Change: experimental-source-isolation

Status: accepted and complete

Exploration: canonical-alpha-release
Carries decisions: CAR3, CAR17; change-map node C3

## Problem

The release catalog classifies eleven roots as experimental source, and `scripts/check.sh` runs the
three present experiment suites plus the polyglot extractors. That account does not prove that
every classified root remains attached to an executable gate. A new root can be added without a
check, or a gate call can be removed while the release classification remains unchanged.

The canonical repository also has no GitHub Actions workflow. Local success therefore does not
establish that the experimental corpus runs from a clean canonical checkout. Three current
references to retained domain evidence are immutable commit-pinned citations, but no check prevents
a future local path, mount or mutable branch URL from becoming an executable dependency.

Without these controls, the first alpha can contain experimental source that silently depends on
the Drim or demo checkout. That would contradict the canonical authority boundary and make a local
project result appear portable when it is not.

## Outcome

One derived experimental-source account proves that every root classified by the release catalog
is exercised by a repository-owned gate. The root repository check executes that account, and a
GitHub Actions workflow executes the same root check from a clean checkout rather than restating a
second command matrix.

Isolation checks reject executable inputs outside the canonical repository, including local Drim
or demo paths and mutable domain-repository references. Commit-pinned external links remain
citation-only provenance and are never opened as build, test or acceptance inputs.

## Scope

In scope:

- a machine-derived relation from all eleven declared experimental roots to executable checks;
- fail-closed validation for an unaccounted root, a missing gate and an external domain locator;
- clean-checkout execution of polyglot, assurance-extension and assurance-service experiments;
- one canonical GitHub Actions workflow that invokes the root repository gate;
- immutable commit-pinned citations for the retained real-domain federation trial and concern
  evidence; and
- exact-revision evidence from the first successful canonical workflow run.

Out of scope:

- publishing experimental packages or promising support for their languages;
- moving or executing the retained real-domain multi-repository laboratory;
- changing the semantics demonstrated by the existing experiments;
- release publication, provenance attestations or partial-publication recovery;
- private assurance deployment; and
- optimizing CI duration or adding platform matrices unrelated to experimental isolation.

## Affected claims

Add `framework/release-artifacts#all-experimental-source-is-gated`,
`framework/release-artifacts#experiment-gates-need-no-domain-checkout` and
`framework/release-artifacts#external-domain-evidence-is-citation-only` under a new
`experimental-source-isolation` requirement at standard criticality.

A violation blocks standalone alpha qualification and can hide an undeclared source dependency,
but it does not publish an artifact, corrupt durable data or affect an existing consumer. Standard
criticality therefore matches the consequence.

## Completion conditions

- Every path in `release/artifacts.json` `experimentalSource` resolves to tracked source and to at
  least one executable gate without a second hand-maintained root inventory.
- Adding an unaccounted experimental root or removing its gate relation fails with the exact root
  named; set reordering does not change the result.
- The experiment gate rejects an executable Drim/demo checkout locator and a mutable external
  domain URL while accepting the existing commit-pinned provenance citations.
- `scripts/check.sh` runs the experimental account and all three experiment suites from a clean
  checkout with no adjacent domain repository or mounted fixture.
- The canonical GitHub Actions workflow checks out only `drim-dev/azimuth`, installs declared
  toolchains and invokes `scripts/check.sh` rather than duplicating its commands.
- One successful workflow execution is attributable to the exact active-change revision. A local
  workflow parse or green developer checkout does not substitute for this rollout evidence.
- The current model has no holes, all three new standard claims have current agent judgments, and
  the complete repository gate remains green.
- No experimental package gains a public identity, publish command or support promise.
