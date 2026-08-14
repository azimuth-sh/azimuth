# Assurance extensions

Azimuth's small core contains claims, realization, evidence, mechanisms and judgment. External
tools extend those concepts; they do not add a facet or a tool category to the core (D39).

## Choose the binding by proposition

The executable does not determine the role. Ask what the result establishes.

| Result | Usual binding | Why |
|---|---|---|
| Load threshold over a declared workload | evidence | workload, threshold and measured outcome form an oracle |
| Stress, spike, soak or capacity threshold | evidence | each threshold is a claim-specific assertion |
| Chaos degradation, recovery or alert check | evidence | each observed predicate has its own oracle |
| Broad SAST/SARIF scan | challenge | absence of reported findings does not establish product behaviour |
| Mutation run | challenge | killed mutations qualify test sensitivity, not the expected result |
| Claim-specific static rule with independent oracle | evidence | its proposition is narrower than “the scanner passed” |
| Race detector, sanitizer or leak checker | challenge by default | a clean sampled run is negative search over executions |
| Contract or schema compatibility test | evidence | the accepted/rejected relation is the product predicate |
| Backup restoration or rollback drill | evidence | recovery time and restored invariants are observable claims |
| Penetration or exploratory session | evidence receipt or challenge | use evidence only for explicit executed cases and outcomes |

The same observation may have several bindings. One load execution can cover latency and error
rate; one chaos execution can cover degraded service, queue recovery and alert delivery. Each
binding declares a separate assertion, outcome, scope, quantification and oracle. There is no
blanket `passed` relation.

## Provider-neutral boundary

```json
{
  "observations": [{
    "id": "expected-load-42",
    "kind": "load-test",
    "tool": "k6",
    "tool_version": "1.0.0",
    "report": "reports/load.json",
    "inputs": ["tests/load.js"],
    "observed_at": "2026-08-11T12:00:00Z",
    "expires_at": 4102444800,
    "source_fingerprint": "...",
    "bindings": [{
      "role": "evidence",
      "spec": "checkout/performance",
      "scenario": "latency-objective",
      "assertion": "p95 latency is below 300 milliseconds",
      "outcome": "satisfied",
      "subjects": [],
      "scope": "e2e",
      "quantification": "example",
      "oracle": "direct"
    }],
    "payload": {"p95_ms": 241}
  }]
}
```

Adapters own native schemas and reject unknown versions or statuses. `azimuth-import-observation`
validates provider-neutral exports. `azimuth-import-mutation`, `azimuth-import-pit` and
`azimuth-import-sarif` derive challenge bindings from existing linkage so they need no second claim
map. PIT additionally fingerprints an exact resolved class and test selection because its XML report
does not enumerate every selected test.

The opaque payload is for agent inspection and export consumers. It cannot change a binding's role,
outcome or form; those remain explicit core fields. The payload and native inputs are covered by the
observation fingerprint.

## Repository placement

An observation belongs to the repository or assurance system that produces its immutable result.
Realization and evidence may be in another repository from intent. For example:

- an application repository emits the backlog metric;
- an operations repository owns the Prometheus rule and `promtool` rule test;
- an assurance repository owns a broker-loss experiment and alert-delivery receipt;
- one model source remains authority for the operational claim.

Checked-in rules establish declared configuration. They do not establish that production loaded
the rule, scraped the metric or delivered the notification. Claims about those boundaries require a
live observation or a composed revision-bound receipt.

## Extension acceptance test

A new adapter is composable when it can be added without a Rust tool type and satisfies all of:

- native schema and status vocabulary fail closed;
- every evidence proposition is an explicit binding with a complete form;
- broad analysis creates no implicit coverage;
- challenge subjects resolve to existing linkage and fail after deletion or rename;
- report, configuration or subject changes stale affected judgments;
- one execution may bind to several claims without duplicating run metadata.

The executable fixtures are under `experiments/assurance-extensions/`.
