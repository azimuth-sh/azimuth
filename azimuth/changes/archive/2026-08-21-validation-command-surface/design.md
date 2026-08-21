# Design: Validation Command Surface

## Command boundary

The top-level command surface becomes:

```text
azimuth validate [options]
azimuth report traceability [options]
azimuth export [options]
azimuth judge [options]
```

The parser has no positional validator ids. `azimuth check` is unknown, and `azimuth validate rtm`
is an unexpected positional argument. Neither path invokes validation or prints a compatibility
redirect. Nested change and project `check` operations retain their lifecycle meaning.

## Finding contract

The current 33 kinds remain stable where their semantics remain current. One exhaustive
`FindingKind::ALL` registry drives summaries so a newly added kind cannot disappear from counts.
Each kind supplies one corrective help string and belongs to one closed category:

- `intent` for incomplete or contradictory Claim declarations;
- `realization` for production linkage and required-area participation;
- `verification` for evidence plans, forms, bindings and execution-derived evidence;
- `mechanism` for design declarations, implementation bindings and enforcement;
- `judgment` for agent decisions and freshness;
- `surface` for derived domains and member discharge; and
- `execution` for imported execution identity and target resolution.

Detailed text and JSON include kind, category, severity, source location, optional Claim,
criticality, detail and help. The exported complete model uses `findings`; it never emits a second
`holes` field.

The later breaking verification change may remove alpha 1 Finding kinds whose underlying concepts
disappear. This change does not preserve them as compatibility aliases after that semantic removal.

## Traceability report

The first report is a pure projection over selected current case-level Claims and their realization
relations. Its deterministic JSON records Claim identity, parent requirement, criticality,
statement and ordered realization source identities. It does not copy model authority and does not
include alpha 1 evidence under a transitional name.

The Evidence Binding change can add Check relationships after their repository format exists. That
extension must preserve deterministic ordering and the report's derived-only status.

`--out` writes the same JSON emitted to standard output. Without `--out`, the command performs no
filesystem write.

## Transition

Active Rust modules, tests, scripts, skills and documentation move atomically. Immutable archives
and historical decision paragraphs retain their original commands. Current decision text receives
an explicit alpha 2 revision rather than being silently rewritten.

No compatibility alias, deprecated field or dual reader survives the change.
