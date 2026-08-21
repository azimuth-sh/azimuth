# Outcome: canonical-development-authority

Status: accepted

## Result

`azimuth-sh/azimuth` now declares itself the canonical framework development and release
repository. Its agent instructions and README direct generic source, model, documentation, skills,
pull requests and version history to one authority. Published tags remain immutable while `main`
continues development.

D2 is revised rather than silently replaced. It retains the discovery-era reason for colocating
the framework and ride-hailing fixture, marks the frozen-repository rule as superseded and converts
the former extraction rule into a continuing independence boundary. `azimuth-demo` now identifies
itself as an external fixture and no longer directs framework development to its checkout.

## Evidence executed

- `./scripts/check.sh` passed from the canonical repository. It exercised the Rust core and CLI,
  .NET and TypeScript packages and extractors, all experimental language paths, assurance-extension
  conformance, the assurance service and web build, Docker-backed lifecycle qualification, release
  packaging and release orchestration checks.
- The final repository check derived 29 claims in 4 specs with zero holes, errors or warnings.
- `azimuth change check canonical-development-authority` with the five release linkage manifests
  preserved 29 current and target claims and reported zero accepted-state errors or warnings.
- `git diff --check` passed in both repositories for the affected files.
- Searches over current root instructions and D2 found no remaining direction to develop Azimuth
  in the demo or to keep the public repository frozen.

## Departures

The released repository has no `azimuth/changes/README.md`, although the proposal skill names it as
required orientation. The change used the CLI contract and the archived
`framework-only-change-completion` precedent instead. No README was invented as part of this
authority transition.

## Residual decisions

- Active or uncommitted generic work that remains in `azimuth-demo` is not transferred by this
  change. Each item needs an explicit disposition before work resumes in the canonical repository.
- The dogfooding exploration was copied as requested and remains present in both checkouts. The
  canonical copy is the one under `azimuth-sh/azimuth`; deleting or replacing the source copy was
  not authorized.
- No later version is selected or published by this change.

## Measurements

- accepted framework claims changed: 0;
- current and target claims: 29;
- canonical repository instruction and decision files revised: 3;
- external fixture instruction and decision files revised: 2;
- archived changes or explorations edited: 0; and
- complete repository-check errors and warnings: 0.
