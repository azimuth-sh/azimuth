# Outcome: verification-evidence-bindings

Status: accepted

## Result

Azimuth now represents repository-owned verification as first-class Checks, sparse Evidence
Bindings and one Qualification per Check-to-Claim edge. Check implementations carry only stable
implementation identity; source annotations no longer own Claim coverage, evidence form or
credibility. Challenge Plans resolve exact current decision fingerprints through semantic graph
relations without falling back to source paths or whole suites.

The CLI loads this graph before selection, derives assembly-owned source identities, validates it
through categorized Findings and exports format version 2. All alpha 1 Covers, verification-plan,
judgment and imported-result readers are removed without compatibility aliases. Every current
framework Claim remains routine, so the canonical repository declares no Checks, Bindings or
Qualifications merely to account for ordinary engineering tests.

## Acceptance checks

- The complete Rust CLI suite passed 145 tests, including strict parsing, versioned fingerprints,
  source-identity spoofing, selected-graph closure, challenge resolution and recursive export
  shape.
- The Assurance Service workspace passed its pure-domain and PostgreSQL-backed lifecycle tests
  while remaining isolated on the D42 version-1 wire.
- The polyglot gate built and tested all seven language integrations, emitted seven Realizes and no
  routine Check enrollment, and validated its combined strict manifest without Findings.
- TypeScript, .NET, Go, Python, Rust and C++ extractor suites passed explicit retired-marker and
  exact SHA-256 fingerprint cases; the JVM fixture passed its direct compiler suite.
- The assurance-extension integrity fixture passed, and the four release suites passed 66 tests.
- The five release manifests validated 53 Claims across seven specs with zero errors or warnings.
  Traceability selected all 12 new case Claims, and the selected export was version 2 with no
  retired evidence keys.
- The package graph, formatting, public links, skill frontmatter, current command surface,
  all-routine criticality, forbidden-name and no-compatibility audits passed.

## Departures

- Review found that several source extractors silently ignored recognizable alpha 1 markers. They
  now reject those markers explicitly; unrelated language symbols with similar names remain
  ordinary source.
- Required Context originally hashed a bare canonical map. It now uses a versioned canonical
  envelope, matching the fingerprint contract.
- Locator rejection originally made an assembly-derived Next route such as `GET /payments/[id]`
  unselectable. The selector grammar now retains the complete address and distinguishes semantic
  route identity from source-file locators.
- Public guidance initially removed the immutable consumer provenance citation required by the
  release-isolation gate. The restored citation is documentary and commit-pinned; no build, test,
  release or acceptance step reads that repository.

## Residual decisions

- Run bundles, adapters, actual-selection validation, normalized execution outcomes and durable
  ledger replacement remain dependent changes from the accepted alpha 2 exploration.
- Claim Judgment selectors remain reserved and resolve no targets until total-composition Judgment
  authority is accepted.
- Package versions and release authority intentionally remain at alpha 1. The coordinated
  `alpha2-release` change owns the one-time version transition and publication; this change does
  not publish altered artifacts under the existing immutable version.
- Ignored local TypeScript build residue can retain deleted importer JavaScript in a dirty checkout.
  It is untracked, excluded from the package allowlist and absent from a clean build.
