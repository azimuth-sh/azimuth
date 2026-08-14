# Outcome: declared-surfaces-and-obligations

Status: implemented, pending acceptance

## Result

Azimuth now has one validated local workspace account for architectural areas, independently
derived surfaces and optional area realization obligations. The rider and driver Next applications
derive their route populations through workspace contributions rather than semantic command-line
assignments. The referral-summary claim requires both Trips and rider-experience participation.

The machine distinguishes universal surface discharge from existential area participation. It
derives source areas from paths or federated identities, so tags gained neither area nor role
arguments. Routine claims remain intent-only, and area participation creates no evidence or test
obligation.

The proposal, apply and verify skills now guide agents to reuse or create a derived surface,
exercise its negative path, inspect every obligated realization and keep evidence governed by the
verification plan.

## Evidence executed

- The Rust suite passed 23 unit, 34 machine-check, 2 CLI, 22 design, 33 federation, 4 package, 27
  verification-plan and 22 spec-parser tests.
- The TypeScript extractor passed 40 tests after rebuilding, including independent Next-route
  enumeration, unknown-mount refusal, traversal refusal and duplicate-mount refusal.
- Synthetic checks prove that a missing surface, unknown surface, missing contribution witness,
  untagged member and backend-only realization each fail through the intended distinct path.
- The complete repository check passed service/component tests against real PostgreSQL and
  RabbitMQ, Prometheus rule tests, both Next production builds, 12 composed E2E tests, extraction,
  the 90-claim model, polyglot conformance and assurance-extension conformance.
- The accepted model finished with zero holes, errors or warnings. `git diff --check` passed.

## Departures

The workspace uses D33's existing `areas[].mounts[]` shape rather than introducing the proposed
`roots` synonym. This keeps monorepo and federated source identity aligned.

The BFF routes and pages remain one `rider-experience` area because they share one stable ownership
and delivery boundary. No artificial area split or mandatory role vocabulary was introduced.

Coordinator review tightened two implementation details beyond the worker result: the TypeScript
workspace consumer now rejects unsafe or duplicate mounts before extraction, and judgment inputs
canonicalize set-like declarations so JSON reordering alone does not create stale verdicts.

## Residual decisions

- Add ASP.NET endpoint and broker-consumer enumerators only when a concrete invariant needs those
  independently derived populations.
- Decide how federated repositories contribute to one surface without duplicating surface
  authority.
- Model nested applications inside one mount if a real repository layout requires them.
- Assignment completeness remains an accepted architectural assertion: an enumerator proves every
  member inside a declared application, not that every relevant application was declared.
- An area obligation proves only that a declared realization exists there. Its semantic honesty
  remains an agent judgment, and it does not guarantee a particular page, handler role or test.

## Measurements

- Workspace declarations: 9 areas, 2 surfaces, 1 realization obligation.
- New mandatory arguments on `Realizes` or `Covers`: 0.
- New evidence obligations caused by area participation: 0.
- Accepted claims after implementation: 90.
- Final machine findings: 0.
