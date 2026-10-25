# Candidate execution verification

`azimuth run verify-candidate --authority <file> --bundle <file> --out <file>` evaluates one native Run against a supplied complete current authority. It creates no Requirement Review, Check Result, Challenge finding disposition, deployment applicability or Requirement Status. The caller selects the exact candidate and independently compares the Run Verification Target with its required source/artifact vector.

The evaluator returns exactly:

```json
{
  "format": "azimuth-candidate-execution",
  "version": 1,
  "status": "passed",
  "checks": [],
  "challenges": [],
  "diagnostics": []
}
```

Status is `passed | failed | incomplete`. Observed violated Checks, failed native tests or Challenge findings make status failed. Missing, stale, malformed or inconclusive examination makes status incomplete unless an adverse result also exists. Every retained adverse result stays visible; a failed result does not erase an execution gap. Diagnostics and per-execution rows retain the reasons. An empty Run is incomplete, not vacuous success. Invalid authority is an input error rather than a positive report.

A Check row has exactly `{id, status, scenarios, units, diagnostics}`. The evaluator requires its current authority fingerprint, exact bound Scenario set, complete Run/activities and native execution-context equality. Each unit needs measured attempts with the required Scenario outcomes. Skipped or not-executed native test records are incomplete, failed tests are adverse, and inconclusive Check outcomes are incomplete. Native protocol validation remains responsible for identity, reduction and fingerprint consistency. The evaluator rejects skipped or missing outcomes explicitly recorded by the native provider. It cannot infer an omitted test case from a generic source-scan Check: launchers must compare any expected native test inventory against the emitted records before accepting coverage.

When `Deployment.units` explicitly declares units, execution must cover that exact set. A `whole` unit cannot erase explicit fixture/control obligations. Without an authored finite unit inventory, the default certifiable scope is `whole`; a narrow unit set needs an explicit complete unit manifest and is otherwise reported incomplete. Project integrations may separately establish unit partition coverage; this API does not invent a reuse proof or join earlier results.

A Challenge row has exactly `{id, status, outcome, target_fingerprint, issues, diagnostics}`. New candidate examination must target `implementation`, and the implementation fingerprint must recompute against the current authority, source scope and exact Run Verification Target. Historical assessment-targeted Runs remain inspectable but cannot satisfy this pre-review gate. Inconclusive attempts, non-completed activities, and non-informational execution/normalization diagnostics remain gaps. A clean complete search can pass this execution gate; findings retain stable issue IDs and require independent disposition. An equivalence proposal never changes native findings to clean.

The actual Run context must equal its planned required context. This is not proof that the launch selected the correct environmental obligations: the caller must derive and compare the required candidate context before execution. The evaluator reads one bundle; validating retained correction chains and selecting their active heads remain caller responsibilities through native Run history validation. Results from another target, skipped work, an engineering summary or an artifact download cannot substitute for native results.

CI uses this report as an execution validation input and retains its diagnostics. It creates no work package, assignment, agent launch or deployment permission.
