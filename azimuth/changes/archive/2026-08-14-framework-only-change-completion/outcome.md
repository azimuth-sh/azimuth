# Outcome: framework-only-change-completion

Status: accepted

## Result

Azimuth now distinguishes an explicitly unchanged intent account from an omitted intent delta. A
proposal declares adjacent `Intent delta: none` and `Because:` metadata before its first section.
The parser preserves the rationale in the change report, `change check` displays it, and local
finalization removes only the supported-delta issue.

The parser rejects a missing or separated rationale, duplicate declaration, unsupported metadata
value and any proposal that combines unchanged-intent metadata with a parsed addition or
criticality transition. Existing plan, proposal-status, outcome, model-finding, finalization
freshness and archive-location gates remain unchanged.

The current framework model now owns two routine lifecycle claims. They carry no linkage, evidence
or judgment ceremony, as required by D20. Their deterministic tests remain ordinary project
evidence.

## Evidence executed

- Twelve targeted change-lifecycle unit tests passed, including the unchanged-intent acceptance,
  missing rationale, duplicate declaration, contradictory delta and retained-gate cases.
- Four CLI tests passed. The new case observed the rationale in `change check`, completed local
  finalization and wrote `finalization.json` against an intentionally empty synthetic model.
- The complete Rust suite passed: 27 library tests, 4 assurance tests, 34 machine-check tests,
  4 CLI tests, 22 design tests, 33 federation tests, 4 package tests, 27 verification-plan tests
  and 22 spec-parser tests.
- The accepted framework model contains 2 routine claims in 1 spec with zero holes, errors or
  warnings.
- All five transferred framework changes still report complete plans and zero accepted-state
  errors or warnings against the new current model.
- `./scripts/check.sh` passed from the canonical checkout, including .NET and TypeScript
  extractors, all polyglot experiments, assurance-extension conformance, assurance domain tests,
  the Next.js production build and the HTTP/PostgreSQL lifecycle component test.
- `cargo fmt --check` and `git diff --check` passed.

## Departures

The approved plan originally combined recording the outcome, finalizing and archiving in one item.
The apply workflow owns implementation evidence but explicitly does not archive. The item was split
at that boundary, and this outcome leaves the change ready for a separate acceptance pass.

Malformed or contradictory metadata fails during change inspection with exit code 2 rather than
appearing as a completion issue with exit code 1. This is stricter than deferring the same error to
finalization and keeps `change check` from presenting an ambiguous report.

## Residual decisions

- Each transferred framework-only change must add its own declaration and specific rationale;
  absence is not inferred from its existing prose.
- Their demo-era evidence summaries must be replaced or qualified with fresh canonical evidence
  before acceptance.
- Replacement, removal and arbitrary model-edit operations remain unsupported intent deltas.

## Measurements

- new accepted claims: 2 routine, 0 non-routine;
- new proposal metadata lines: 2;
- existing completion gates removed: 0;
- existing completion issues bypassed by the declaration: 1, the supported-delta requirement;
- new targeted regression cases: 5 unit and 1 CLI;
- external repository reads in build, test or change checks: 0.
