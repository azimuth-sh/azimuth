# Work Provenance

Work Provenance is a ledger of completed model-shaping workflow attempts: which action was performed with which outcome, by which execution instance and what kind of agent, when, and what it used and produced. It is a separate domain from assurance. **Nothing in it is ever an input to review currentness, Requirement Status, acceptance or any other assessment.** Provenance never enters a model, Requirement, review, workflow result or attempt fingerprint.

The ledger records activities, not their products. A review, a realization or an implementation is an entity that lives in its own domain; the ledger refers to it by fingerprint. An attempt carries one summary of the model work it took (`usage`: token and call counts); everything finer, including per-call telemetry, transcripts, tool inputs and cost, is out of scope, and an attempt carries at most an opaque `trace` reference to it.

Background and decisions: `azimuth/explorations/work-provenance/`.

## Agent descriptor

Every attempt carries a descriptor with a closed field set. Each field states its basis:

- `harness` — read by the harness integration from the executing harness's own records;
- `declared` — supplied by an agent, orchestrator or person without harness attestation;
- `unknown` — not available. An unknown value is `null`; it is never guessed.

```json
{
  "kind": {"value": "model", "basis": "harness"},
  "model": {"value": "claude-opus-5-5", "basis": "harness"},
  "effort": {"value": "high", "basis": "harness"},
  "agent_type": {"value": "general-purpose", "basis": "harness"},
  "harness": {"name": "claude-code", "version": "2.1.309", "basis": "harness"}
}
```

`kind.value` is `model | human | null`. `model.value` is the provider's canonical model id. `effort.value` is the provider-native effort label, verbatim; no cross-provider scale is defined. `agent_type.value` is the harness's own name for the agent configuration that ran the attempt (a Claude Code subagent type such as `Explore`, a Codex agent role); a top-level session the harness does not name is `unknown`. `harness.name` and `harness.version` share one basis. With basis `unknown` every value is `null`; with any other basis the value is a nonempty string (for `harness`, `name` is required and `version` may be `null`).

The descriptor file accepted by `workflow update --descriptor` is:

```json
{"format": "azimuth-agent-descriptor", "version": 1, "kind": {…}, "model": {…}, "effort": {…}, "agent_type": {…}, "harness": {…}, "trace": null, "usage": null}
```

`trace` is `null` or a nonempty opaque string. `usage` is `null` or a usage object (below). Unknown fields are rejected. Omitting `--descriptor` records every field as `unknown`, and `trace` and `usage` as `null`.

## Usage

`usage` summarizes the model work of one attempt: the calls made between the handoff (`handoff_captured_at_ms`) and the recorded result.

```json
{
  "basis": "harness",
  "model_calls": 14, "tool_calls": 31,
  "input_tokens": 1820, "cache_read_input_tokens": 912000, "cache_write_input_tokens": 41000,
  "output_tokens": 18400, "reasoning_output_tokens": 2100,
  "overlapping": false,
  "delegated": {"agents": 2, "model_calls": 40, "input_tokens": 310, "cache_read_input_tokens": 1400000,
                "cache_write_input_tokens": 90000, "output_tokens": 22000, "reasoning_output_tokens": null}
}
```

- `basis` is `harness` or `declared`; usage that is not known is `null`, never zeros.
- Counts are the agent's own work: calls made by the execution instance named by `actor`, deduplicated by model call. `tool_calls` excludes the `workflow update` call that records the result.
- Token fields keep the provider's semantics and are not normalized: an OpenAI `input_tokens` includes cached input while an Anthropic one excludes it, and an Anthropic `reasoning_output_tokens` (thinking) is part of `output_tokens`. A field the provider does not report is `null`; `input_tokens` and `output_tokens` are always present.
- `delegated` sums every agent the instance launched during the window, transitively; their tokens are never added to the instance's own.
- `overlapping` is `true` when the same instance recorded another workflow result inside the window, so the tokens are shared between attempts and cannot be split.

`actor` is unchanged: it names one execution instance (a session, a subagent run or a person) and remains the reviewer-independence key. It encodes no role, attempt number or model; those are recorded by the action, the attempt id and the descriptor.

## Workflow capture

The workflow instance, handoff and result formats are version 3. Version 1 and 2 instances are read only by `provenance ledger` for historical export; version 1 action and record names are exported verbatim.

- `workflow next` adds `captured_at_ms` (tool clock, integer Unix milliseconds) to the outer handoff object. It is not part of any task basis.
- `workflow update` accepts optional `--descriptor FILE`. Without it, `update` uses the descriptor a harness hook saved beside its handoff, at `<handoff file name>.agent-descriptor.json` in the handoff's directory, when that file was saved no earlier than the handoff's `captured_at_ms`; an older file belongs to an earlier attempt that reused the handoff path and is ignored. A saved descriptor that fails validation fails the update without a transition. Recording the attempt, or replaying it, removes the saved file. Each appended history entry is `{result, result_fingerprint, provenance}` where `provenance` is:

```json
{
  "handoff_captured_at_ms": 1761200000000,
  "recorded_at_ms": 1761200400000,
  "agent": { …descriptor without format/version/trace… },
  "guidance": [{"kind": "skill", "id": "azimuth-review-verification", "fingerprint": "sha256:…"}],
  "trace": null,
  "usage": null
}
```

`provenance` is a sibling of `result`: it enters neither the attempt id, `result_fingerprint`, any task basis nor dependency comparison. `guidance` lists the handoff's guidance entries in order; the tool fingerprints embedded skills and references by their shipped content and records `null` for a `package` locator. An idempotent replay of an already recorded attempt returns the original entry and never rewrites its provenance.

- `workflow init` accepts optional `--predecessor FILE`, a prior instance being replaced by a fresh one. The new instance records `predecessor: {"id": "<instance id>", "ref": "<path relative to the new instance directory>", "revision": N, "fingerprint": "sha256:…"}`, where `fingerprint` is the canonical fingerprint of the predecessor file's JSON. The predecessor file must remain retained at `ref`.
- `workflow abandon --instance FILE --expected-revision N --reason TEXT` records `abandoned: {"at_ms": …, "reason": "…"}` on the instance. An abandoned instance accepts no further updates.

## Harness capture

`azimuth provenance capture --harness <claude-code|codex>` is a PreToolUse hook. It reads the hook event on standard input. For a single plain `azimuth workflow update` command with `--handoff` and without `--descriptor` (no shell operators, redirections or substitutions), it saves a descriptor beside the handoff (see Workflow capture), resolving a relative handoff path against the event's `cwd`. It prints nothing, never alters the tool call and never decides permission; Codex applies a command rewrite only together with a permission grant, so the descriptor travels through the file instead. A hook failure is reported on standard error and the command proceeds without a descriptor.

Values have `harness` basis only when read from the harness's own records; anything the records lack stays `unknown`:

| Field | Claude Code | Codex |
|---|---|---|
| `model` | `message.model` of the transcript record that issued the tool call (the hook waits up to 5 s for it) | the event's `model` |
| `effort` | the event's `effort.level` | `effort` of the rollout's `turn_context` for the event's `turn_id` |
| `agent_type` | the event's `agent_type` (inside a subagent) | the event's `agent_type` (inside a subagent) |
| `harness.version` | `version` of the issuing record | `session_meta.cli_version` |
| `usage` | the agent's transcript (a subagent's own `subagents/agent-<id>.jsonl`) from the handoff on; delegated agents are found by the `toolUseId` of their `.meta.json` | the thread's rollout from the handoff on; delegated threads are rollouts whose `session_meta` names this thread as spawning parent |
| `trace` | `claude-code:session/<id>[/agent/<id>]/tool/<id>` | `codex:session/<id>/thread/<id>/turn/<id>/tool/<id>` |

`kind` is `model`. The usage window opens at the `captured_at_ms` of the handoff file the command names with `--handoff`; without a readable handoff, `usage` is `null`.

`azimuth update` installs the hook as a managed resource of each agent integration: `.claude/settings.json` for `claude` and `.codex/hooks.json` for `codex`. Both invoke `azimuth` from `PATH`, which must be a release that ships this command. An unmanaged file at either path blocks installation. Codex runs project hooks only in a trusted project and only after the hook itself is trusted through `/hooks`; a changed hook must be trusted again.

## Committed ledger

`<change>/provenance/ledger.json` is committed with the change and archived with it. It is generated, never hand-edited, by:

```text
azimuth provenance ledger --change <change-directory> [--instance <file>...] [--out <file>]
```

By default it reads `<change>/.azimuth/workflow.json` and follows `predecessor.ref` links; a predecessor file must live inside the successor's instance directory. Extra `--instance` files are added for historical instances that predate predecessor links. They precede the predecessor chain, ordered by their first `recorded_at_ms`; untimed version 1 and 2 instances keep command-line order. A ledger built with extra instances cannot be reproduced by `change finalize` or `change archive`, which follow only the default chain, so it suits an already archived backfill. When a change has no local `.azimuth/workflow.json`, `change archive` validates only the committed ledger's format. `change finalize` writes the ledger when `<change>/.azimuth/workflow.json` exists; `change archive` fails when the committed ledger differs from a fresh export.

```json
{
  "format": "azimuth-work-provenance-ledger",
  "version": 1,
  "project": "drim-dev",
  "change": "private-api-service-ingress",
  "instances": [
    {
      "id": "…",
      "predecessor": null,
      "disposition": "completed",
      "definition_fingerprint": "sha256:…",
      "template": {"id": "change-development", "revision": 10},
      "attempts": [
        {
          "id": "attempt-…",
          "sequence": 1,
          "action": "author-proposal",
          "task": "author-proposal",
          "scope": "change",
          "target": [{"name": "change", "value": "private-api-service-ingress"}, {"name": "project", "value": "drim-dev"}],
          "actor": "claude-author",
          "outcome": "completed",
          "report": "…",
          "handoff_captured_at_ms": null,
          "recorded_at_ms": null,
          "agent": {"kind": {"value": null, "basis": "unknown"}, "model": {"value": null, "basis": "unknown"}, "effort": {"value": null, "basis": "unknown"}, "agent_type": {"value": null, "basis": "unknown"}, "harness": {"name": null, "version": null, "basis": "unknown"}},
          "guidance": [],
          "trace": null,
          "usage": null,
          "used": [{"kind": "initial-request", "repository": null, "ref": "inputs/request.md", "fingerprint": "sha256:…"}],
          "generated": [{"kind": "proposal", "repository": "application", "ref": "azimuth/changes/x/proposal.md", "fingerprint": "sha256:…", "record": null, "record_fingerprint": null}]
        }
      ]
    }
  ]
}
```

- Instance `id` is unique within a ledger. Version 3 `workflow init` assigns `<definition-id>-<init unix ms>`; a version 1 or 2 historical instance exports as `<recorded id>-<first 12 hex of its canonical file fingerprint>`, so restart snapshots stay distinct and deterministic. Predecessor links are matched by `predecessor.fingerprint`, not by id. The Assurance Service keys instances on `(project, change, id)`.
- `disposition`: `abandoned` when recorded; `superseded` when another listed instance names it as predecessor or a later historical instance follows it; `completed` when the change has a `finalization.json` or is archived and the instance is the last; otherwise `active`.
- `sequence` is the 1-based position in the instance history. `target` is the work target flattened to sorted `{name, value}` string pairs; nested values are rendered as canonical JSON.
- `scope` is derived from the target: `requirement` when it names a Requirement, else `check`, `verification-challenge`, `module`, and finally `change`.
- `report` is the authored result's `report`, or `null` when the authored file is absent, no longer matches its recorded fingerprint, or has none. The file resolves relative to its own instance file's directory, so retained snapshots must stay beside the instance they belong to.
- `used` are the attempt's recorded inputs; `generated` are its output records. `record` names a selected review-bundle entry and `record_fingerprint` is that native record's own fingerprint (for a Verification or Realization Review, its review fingerprint); both are `null` otherwise. This is how a stored review is linked to the attempt that produced it.
- Version 1 and 2 history entries export with every descriptor field `unknown`, both times and `usage` `null`, and `guidance` `[]`.

The ledger fingerprint is the canonical fingerprint of the ledger JSON. The ledger is append-only across exports: an attempt id, once exported, keeps identical content.

## Failure Attribution

An optional, append-only record by which an investigator names the attempts believed to have **caused** an adverse fact or **allowed it through**. It is authored in `azimuth/provenance/failure-attributions.md`:

```markdown
## Failure attribution: ingress-isolation-gap-1

- Requirement: `private-api-requires-designated-service`
- Adverse fact: requirement-status `sha256:…`
- Investigator: dima
- Agent kind: human
- Model: none
- Effort: none
- Agent type: none
- Harness: none
- Recorded at: 2026-10-24T08:00:00Z
- Confidence: medium
- Caused:
  - `attempt-…`
- Allowed through:
  - `attempt-…`
- Revises: none

### Rationale

Substantive Markdown.
```

Adverse fact kinds: `requirement-status | check-result | verification-challenge-result | verification-review | realization-review`, with an exact reference string. `Confidence` is `low | medium | high`. At least one `Caused` or `Allowed through` attempt is required. `Revises` names an earlier attribution id. Descriptor lines are recorded with basis `declared`, or `unknown` when `none`.

```text
azimuth provenance attributions --input azimuth/provenance/failure-attributions.md [--out <file>]
```

compiles every entry to:

```json
{
  "format": "azimuth-failure-attribution",
  "version": 1,
  "id": "ingress-isolation-gap-1",
  "requirement": "private-api-requires-designated-service",
  "adverse": {"kind": "requirement-status", "ref": "sha256:…"},
  "investigator": "dima",
  "agent": { …descriptor… },
  "recorded_at_ms": 1761292800000,
  "confidence": "medium",
  "attempts": [{"attempt": "attempt-…", "role": "caused"}, {"attempt": "attempt-…", "role": "allowed-through"}],
  "revises": null,
  "rationale": "…",
  "fingerprint": "sha256:…"
}
```

and writes `{"format": "azimuth-failure-attributions", "version": 1, "attributions": [...]}`. `fingerprint` covers every other field.

## Dataset

```text
azimuth report provenance [--root <repository>] [--change <id>] [--out <file>]
```

reads every committed `azimuth/changes/**/provenance/ledger.json` (active and archived) and writes one JSON object per line (JSONL), one per attempt: the ledger's `project` and `change`, the instance `id` and `disposition`, and every attempt field with the descriptor flattened to `agent_kind`, `agent_kind_basis`, `model`, `model_basis`, `effort`, `effort_basis`, `agent_type`, `agent_type_basis`, `harness_name`, `harness_version`, `harness_basis`, and `usage` flattened to `usage_basis`, `model_calls`, `tool_calls`, the five token fields, `usage_overlapping`, `delegated_agents`, `delegated_model_calls` and the five token fields prefixed `delegated_`; every usage column is `null` when usage is unknown. Lines are ordered by change, instance order and sequence.

## Assurance Service

Work Provenance is stored in ordinary relational tables with typed columns and no JSON. Writes require the publisher credential and are recorded with `provenance_basis: publisher-transported-assertion`; reads are open to every project role.

| Route | Purpose |
|---|---|
| `POST /v1/projects/{project}/provenance/ledgers` | Submit one change ledger. Existing attempts must match exactly; new attempts and instances are appended; a changed attempt is a conflict. |
| `POST /v1/projects/{project}/provenance/attributions` | Submit one compiled Failure Attribution. Immutable by id. |
| `GET /v1/projects/{project}/provenance/changes/{change}` | The stored ledger, reassembled. |
| `GET /v1/projects/{project}/provenance/requirements/{requirement}` | The Requirement's work history and attributions; 404 when no release authority contains the Requirement. |
| `GET /v1/projects/{project}/provenance/attempts?cursor=&limit=` | Paged dataset rows in the `report provenance` shape. |

The Requirement history follows the release lineage. Releases are ordered by `integration_sequence`, and a release without one is not part of the lineage; for each release, its authority is compared with the previous release's authority. A Requirement's closure changed when the canonical fingerprint of its review dependency packet differs or the Requirement is new. The change is labelled `intent` when the Requirement's entry in `requirement_intents` or any of its Scenario fingerprints differs, otherwise `dependency`. Every change recorded for that release is listed with its stored attempts; a release change without a stored ledger is listed with no attempts. The response is:

```json
{
  "requirement": "…",
  "releases": [
    {
      "release": "sha256:…", "label": "…", "integration_sequence": 3, "model_revision": "sha256:…",
      "relation": "intent",
      "changes": [{"change": "…", "attempts": [ …attempts in the ledger attempt shape, each with "instance" and "disposition" added… ]}]
    }
  ],
  "attributions": [ …compiled azimuth-failure-attribution records for this Requirement… ]
}
```

The authority projection carries a top-level `requirement_intents: [{"id", "fingerprint"}]`, the Requirement intent digest, outside every review dependency packet, so adding it never stales a review.

## Finalization closure boundary

The `finalize-change` workflow action consumes the committed shaping ledger, writes the finalization/provenance artifacts and moves the change to its archive. Its own attempts are excluded from that ledger projection. Otherwise recording completion would change the input certified by the same completion and make a fresh export differ immediately. Remaining projected sequence numbers are contiguous. The native workflow history retains actual attempts unchanged.

The authored actual completion result is retained separately as `provenance/finalization-result.json` outside ignored runtime storage, with references to the real finalized artifacts. It is not synthesized as an earlier shaping attempt, inserted into an existing immutable ledger, or fed back into a retained semantic-input manifest. Exporting before and after recording the final action therefore produces the same shaping ledger. Earlier authoring, implementation, review and execution attempts remain append-only and fully attributable.

This boundary changes neither review independence nor assurance currentness. A new correction is a related active change with its own provenance; the original archived ledger and completion record remain unchanged.
