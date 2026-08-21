# Outcome: Validation Command Surface

Status: accepted

## Result

Azimuth now exposes deterministic model validation only as `azimuth validate`. The top-level
`check` command, the `rtm` validator identity, positional validator selection, the public `check`
module, Hole result types and the exported `holes` field have no compatibility path.

Every Finding belongs to one of seven closed categories and carries corrective help. One
exhaustive registry covers all 33 current Finding kinds and drives summary counts.

`azimuth report traceability` emits a stable derived JSON projection of selected case-level Claims
and their realizations. It writes to standard output by default and writes identical bytes only to
the requested file when `--out` is present.

## Engineering evidence

- The complete Rust suite passed: 187 tests, zero failures.
- The focused CLI suite passed: eight tests covering help, rejection, exit classes,
  initialization and traceability output.
- Composed validation passed over five fresh release manifests: 41 Claims, zero Findings.
- The assurance-extension and polyglot validation gates passed with six and seven Claims.
- Shell syntax, Rust formatting, diff integrity, active-command, criticality and prohibited-name
  audits passed.
- All six dependency-ordered work packages are complete and the routine intent is applied.

## Departures

None.

## Residual decisions

The verification-evidence-bindings change will add Check and Evidence Binding relationships to
the traceability projection after their repository format exists. No transitional evidence shape
was placed in this report.
