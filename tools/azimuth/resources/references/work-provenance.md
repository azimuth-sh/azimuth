# Work Provenance

Work Provenance is a ledger of completed workflow attempts: which action ran, with which outcome, by which execution instance and what kind of agent, when, and what it used and produced. It is a separate domain from assurance. Nothing in it is an input to Requirement Review currentness, Requirement Status, acceptance or any other assessment, and it never enters a workflow result, attempt identity or task basis. Telemetry (tokens, cost, transcripts, tool calls) is out of scope; an attempt carries at most an opaque `trace` reference.

## Actor and descriptor

`--actor` names one execution instance: a session, a subagent run or a person. It stays the reviewer-independence key. Never encode a role, attempt number or model in it; the action, the attempt id and the descriptor already record those.

`workflow update --descriptor FILE` records the executing agent:

```json
{
  "format": "azimuth-agent-descriptor",
  "version": 1,
  "kind": {"value": "model", "basis": "harness"},
  "model": {"value": "claude-opus-5-5", "basis": "harness"},
  "effort": {"value": "high", "basis": "declared"},
  "harness": {"name": "claude-code", "version": "2.1.309", "basis": "harness"},
  "trace": null
}
```

Every field states its basis. Use `harness` only for values read from the executing harness's own records, `declared` for values an agent, orchestrator or person supplies without attestation, and `unknown` with `null` values otherwise. `kind.value` is `model` or `human`. `model.value` is the provider's canonical model id and `effort.value` the provider-native label, verbatim. With basis `harness`, `harness.name` is required and `harness.version` may be `null`. Unknown fields are rejected. Never guess a value: omit `--descriptor` and every field is recorded as `unknown`.

The tool stamps `captured_at_ms` when `workflow next` produces the handoff and `recorded_at_ms` when `update` appends the attempt, and fingerprints the shipped skills and references listed in the handoff's guidance (a `package` locator, or guidance this CLI does not ship, is recorded with a `null` fingerprint). Each history entry is `{result, result_fingerprint, provenance}`. Replaying an already recorded attempt returns the original entry; its provenance is never rewritten.

## Restarting an instance

Workflow instances, handoffs and results are version 3. A version 1 or 2 instance is read only by `azimuth provenance ledger`; the other workflow commands reject it. To replace an instance, keep the old file inside the change's `.azimuth/` directory and run:

```text
azimuth workflow abandon --instance <file> --expected-revision <n> --reason <text>
azimuth workflow init ... --instance .azimuth/workflow.json --predecessor .azimuth/workflow-before-1.json
```

`abandon` is optional when the old instance is simply superseded; an abandoned instance accepts no further updates. The fresh instance records the predecessor's id, relative `ref`, revision and canonical fingerprint. Do not modify or delete a predecessor afterwards: the ledger export fails when its content no longer matches.

## Committed ledger

`change finalize` writes `<change>/provenance/ledger.json` when `<change>/.azimuth/workflow.json` exists. Commit it with the change; it is generated, never hand-edited, and archived with the change. `change archive` fails when the committed ledger differs from a fresh export. Export on demand, adding historical instances that predate predecessor links:

```text
azimuth provenance ledger --change <change-directory> [--instance <file>...] [--out <file>]
```

Exported attempts are immutable: a re-export that would change or drop an attempt already in the ledger fails. Instance dispositions are `completed`, `active`, `superseded` or `abandoned`; attempt `scope` is derived from the work target (`requirement`, `check`, `verification-challenge`, `module`, `change`).

## Failure Attribution

When someone actually investigates an adverse fact, they may append an attribution to `azimuth/provenance/failure-attributions.md`. It is optional; never write one to clear a gate.

```markdown
## Failure attribution: <id>

- Requirement: `<requirement-id>`
- Adverse fact: requirement-status | check-result | verification-challenge-result | requirement-review `<exact reference>`
- Investigator: <name>
- Agent kind: human | model | none
- Model: <canonical id> | none
- Effort: <provider label> | none
- Harness: <name> [<version>] | none
- Recorded at: YYYY-MM-DDTHH:MM:SSZ
- Confidence: low | medium | high
- Caused:
  - `attempt-…`
- Allowed through: none
- Revises: none | `<earlier attribution id>`

### Rationale

Substantive Markdown.
```

Fields appear in this order. Name at least one `Caused` or `Allowed through` attempt; each attempt has one role. Descriptor lines are recorded with basis `declared`, or `unknown` for `none`. Never edit an earlier entry; write a new one that revises it. Compile with `azimuth provenance attributions --input azimuth/provenance/failure-attributions.md [--out <file>]`.

## Dataset

`azimuth report provenance [--root <repository>] [--change <id>] [--out <file>]` writes one JSON object per attempt (JSONL) from every committed ledger of active and archived changes, with the descriptor flattened. It records facts; drawing conclusions about models or harnesses from it is an analysis responsibility.
