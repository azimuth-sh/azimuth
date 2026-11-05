# Work Provenance

Work Provenance is a ledger of completed workflow attempts: which action ran, with which outcome, by which execution instance and what kind of agent, when, and what it used and produced. It is a separate domain from assurance. Nothing in it is an input to Verification or Realization Review currency, Requirement Status, acceptance or any other assessment, and it never enters a workflow result, attempt identity or task basis. An attempt carries one summary of its model work (`usage`); per-call telemetry, transcripts and cost are out of scope, and an attempt carries at most an opaque `trace` reference.

## Actor and descriptor

`--actor` names one execution instance: a session, a subagent run or a person. It stays the reviewer-independence key. Never encode a role, attempt number or model in it; the action, the attempt id and the descriptor already record those.

When the repository's agent integration is installed, its harness hook records the executing agent for you: it saves a harness-basis descriptor beside the handoff of every `azimuth workflow update` you run, however the binary is spelled, and the update records it. Keep handoff files inside the change's ignored `.azimuth/` directory so the saved descriptor is never committed.

The session that performed the work records its own result, as one plain `azimuth workflow update …` command: never chained with `&&` or `;`, piped, redirected or wrapped in a substitution, and without `--descriptor` where a hook exists. Any of those turns capture off. The hook reads the session that runs the command, so a coordinator recording another agent's work cannot attest that agent's provenance; hand the update to the agent that did the work. Without a captured descriptor, `--descriptor` or `--provenance unknown`, `update` refuses rather than record `unknown` silently; retry it as one plain command from the working session.

Without a hook, `workflow update --descriptor FILE` records the executing agent:

```json
{
  "format": "azimuth-agent-descriptor",
  "version": 1,
  "kind": {"value": "model", "basis": "harness"},
  "model": {"value": "claude-opus-5-5", "basis": "harness"},
  "effort": {"value": "high", "basis": "declared"},
  "agent_type": {"value": null, "basis": "unknown"},
  "harness": {"name": "claude-code", "version": "2.1.309", "basis": "harness"},
  "trace": null,
  "usage": null
}
```

Every field states its basis. Use `harness` only for values read from the executing harness's own records, `declared` for values an agent, orchestrator or person supplies without attestation, and `unknown` with `null` values otherwise. `kind.value` is `model` or `human`. `model.value` is the provider's canonical model id, `effort.value` the provider-native label, verbatim, and `agent_type.value` the harness's name for the agent configuration, such as a subagent type. `usage` is `null` or the attempt's token and call counts; never estimate it. With basis `harness`, `harness.name` is required and `harness.version` may be `null`. Unknown fields are rejected. Never guess a value: where no hook exists and nothing can be stated, pass `--provenance unknown` and every field is recorded as `unknown`.

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

## Reconstructing provenance

When an attempt was recorded without captured provenance and the harness log of its window still exists, append an attestation rather than touching the attempt:

```text
azimuth provenance reconstruct --change <dir> --attempt <id> --harness <claude-code|codex> \
  --source <recording|work> --transcript <file> --by <actor> [--tool-use <id>]
```

Use `--source recording` with the log of the session that ran the `workflow update` (it yields what the hook would have captured there), and `--source work` with the log of the session that did the work but did not record it, such as a reviewer whose result a coordinator recorded. Claude Code logs are `~/.claude/projects/<project>/<session>.jsonl`, subagents `…/<session>/subagents/agent-<id>.jsonl`, and a named teammate is its own top-level session whose records carry its `agentName`. Codex logs are rollouts under `~/.codex/sessions/YYYY/MM/DD/`. The attestation is appended to `<change>/provenance/attestations.json`; commit it with the ledger. Its values have basis `reconstructed`, anything the log cannot establish stays `unknown`, and the command refuses a window the log holds no work for. `azimuth report provenance` shows attestations beside the provenance recorded at record time, which is never replaced, and names the attempt's agent in `attributed_*` columns with their origin in `attributed_from`:
- `work-attestation`: the attempt has a `work` attestation, and its latest one is the agent. The work was done by a session other than the one that recorded it, so the record-time capture stays beside it as the recording session.
- `record`: the record-time capture is the agent.
- `record-completed-by-recording-attestation`: the record-time capture is the agent, with descriptor fields captured as `unknown` and usage captured as `null` filled from the latest `recording` attestation of the same session. A captured value is never replaced. Never reconstruct from a log you are not sure belongs to the attempt; a wrong attestation can only be answered by appending a correct one.

## Failure Attribution

When someone actually investigates an adverse fact, they may append an attribution to `azimuth/provenance/failure-attributions.md`. It is optional; never write one to clear a gate.

```markdown
## Failure attribution: <id>

- Requirement: `<requirement-id>`
- Adverse fact: requirement-status | check-result | verification-challenge-result | verification-review | realization-review `<exact reference>`
- Investigator: <name>
- Agent kind: human | model | none
- Model: <canonical id> | none
- Effort: <provider label> | none
- Agent type: <harness agent type> | none
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

`azimuth report provenance [--root <repository>] [--change <id>] [--out <file>]` writes one JSON object per attempt (JSONL) from every committed ledger of active and archived changes, with the descriptor and usage flattened. It records facts; drawing conclusions about models or harnesses from it is an analysis responsibility.
