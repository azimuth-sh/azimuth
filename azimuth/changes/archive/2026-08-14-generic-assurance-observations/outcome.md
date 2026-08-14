# Outcome: generic-assurance-observations

Status: accepted

## Result *(revised 2026-08-14)*

Azimuth now has one provider-neutral observation protocol. Evidence bindings project into ordinary
`Covers` relations; challenge bindings enter judgment freshness without creating coverage. One
observation can bind to several claims, while every evidence binding retains its own assertion,
outcome and verification form.

Stryker.NET and SARIF use the same challenge path. Provider-neutral load and chaos fixtures use the
same evidence path.

The original result included ownership of two ride-hailing alert claims and their Prometheus
relations. CAR16 assigns that applied transition to `ride-hailing-delivery-alerts`. The generic
result retains the field validation as immutable provenance, not domain authority or an executable
dependency:

- Repository: `https://github.com/drim-dev/azimuth-demo.git`
- Revision: `68a2eb5d46daf01ba087ec94b6a1ea7901c63bfd`

## Acceptance evidence

- The synthetic assurance-extension experiment emitted 2 load evidence bindings, 3 chaos evidence
  bindings and 6 SARIF challenge bindings. Its 6-claim model had zero holes, errors or warnings.
- Thirty-four Rust machine-check cases passed, including duplicate observation identity, unresolved
  challenge subjects, freshness, failed evidence and multi-claim observation behavior.
- All 50 TypeScript extractor cases passed, including Stryker.NET, PIT, SARIF, load and chaos
  provider-boundary failures and projections.
- A source search found no Stryker, SARIF, k6, Chaos Mesh or mutation-specific type name in the Rust
  core.
- The complete standalone repository check passed without reading the cited demo repository.

## Departures

This section was revised on 2026-08-14 when CAR16 separated domain authority from the generic
mechanism.

The initial change boundary mixed generic protocol work with ride-hailing intent and evidence.
`ride-hailing-delivery-alerts` now owns the domain transition, while this change owns only the
provider-neutral mechanism and synthetic validation. No product behavior or framework mechanism
changed during the split.

Native k6 and chaos-platform adapters remain deliberately outside scope; their neutral fixture
results validate the extension boundary without putting either provider into the core.

## Residual decisions

Azimuth validates an imported observation's structure, linkage and freshness, but it still trusts
the adapter to interpret the native report honestly. Signed attestations and independently
re-running external tools remain possible extensions rather than core responsibilities.

Checked-in Prometheus configuration and rule tests establish declared operational behaviour. They
do not prove that a production deployment loaded the rule, routed the notification, or elicited an
operator response. Those propositions still require live operational receipts with appropriate
expiry.

The opaque provider payload is intentionally agent-facing. The machine tier fingerprints it and
fails closed on broken bindings, but does not acquire tool-specific policy for mutation scores,
SARIF severities, performance thresholds or chaos-resource states.

The framework-only completion blocker was resolved on 2026-08-14 by the accepted
`framework-only-change-completion` change. This proposal now declares unchanged intent explicitly;
an unrelated product delta was not invented to pass the gate.

## Measurements

- The Rust domain contains one `Observation` abstraction and no mutation-, SARIF-, load-, k6- or
  chaos-specific core type, parser collection, hole or fingerprint role.
- One synthetic load execution supplies two separately formed evidence bindings; one chaos
  execution supplies three. A single SARIF observation challenges all six fixture claims without
  covering any of them.
- The experiment model contains 6 claims and is hole-free after importing those 3 observations.
  The current canonical model contains 2 routine claims in 1 spec and is also hole-free with zero
  errors or warnings. The earlier 90-claim, 11-spec application account remains pre-extraction
  provenance.
- At the cited pre-split revision, the federation suite contained 33 passing cases and the
  operations repository supplied both Prometheus realizations and rule-test evidence relations.
- Federation testing found one real source-identity collision: a Prometheus alert rule and its rule
  test originally shared an untyped address. Giving them distinct `prometheus-alert` and
  `prometheus-rule-test` source kinds fixed the ambiguity rather than weakening duplicate-source
  detection.
- At the cited pre-split revision, the complete repository gate passed the Rust core and federation
  suites, 19 .NET extractor tests, 36 TypeScript extractor tests, all service/component and
  composed end-to-end tests, both web production builds, both Prometheus rule suites, and
  assurance-extension conformance.
