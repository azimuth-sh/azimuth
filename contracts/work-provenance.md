# Work Provenance

Work Provenance is a ledger of completed workflow attempts: which action was performed with which outcome, by which execution instance and what kind of agent, when, and what it used and produced. It is a separate domain from assurance. **Nothing in it is ever an input to Requirement Review currentness, Requirement Status, acceptance or any other assessment.** Provenance never enters a model, Requirement, review, workflow result or attempt fingerprint.

The ledger records activities, not their products. A Requirement Review, a design or an implementation is an entity that lives in its own domain; the ledger refers to it by fingerprint. Telemetry (tokens, cost, transcripts, tool calls) is out of scope; an attempt carries at most an opaque `trace` reference.

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
  "harness": {"name": "claude-code", "version": "2.1.309", "basis": "harness"}
}
```

`kind.value` is `model | human | null`. `model.value` is the provider's canonical model id. `effort.value` is the provider-native effort label, verbatim; no cross-provider scale is defined. `harness.name` and `harness.version` share one basis. With basis `unknown` every value is `null`; with any other basis the value is a nonempty string (for `harness`, `name` is required and `version` may be `null`).

The descriptor file accepted by `workflow update --descriptor` is:

```json
{"format": "azimuth-agent-descriptor", "version": 1, "kind": {…}, "model": {…}, "effort": {…}, "harness": {…}, "trace": null}
```

`trace` is `null` or a nonempty opaque string. Unknown fields are rejected. Omitting `--descriptor` records every field as `unknown` and `trace` as `null`.

`actor` is unchanged: it names one execution instance (a session, a subagent run or a person) and remains the reviewer-independence key. It encodes no role, attempt number or model; those are recorded by the action, the attempt id and the descriptor.

## Workflow capture

The workflow instance, handoff and result formats are version 3. Version 2 instances are read only by `provenance ledger` for historical export.

- `workflow next` adds `captured_at_ms` (tool clock, integer Unix milliseconds) to the outer handoff object. It is not part of any task basis.
- `workflow update` accepts optional `--descriptor FILE`. Each appended history entry is `{result, result_fingerprint, provenance}` where `provenance` is:

```json
{
  "handoff_captured_at_ms": 1761200000000,
  "recorded_at_ms": 1761200400000,
  "agent": { …descriptor without format/version/trace… },
  "guidance": [{"kind": "skill", "id": "azimuth-review-requirement", "fingerprint": "sha256:…"}],
  "trace": null
}
```

`provenance` is a sibling of `result`: it enters neither the attempt id, `result_fingerprint`, any task basis nor dependency comparison. `guidance` lists the handoff's guidance entries in order; the tool fingerprints embedded skills and references by their shipped content and records `null` for a `package` locator. An idempotent replay of an already recorded attempt returns the original entry and never rewrites its provenance.

- `workflow init` accepts optional `--predecessor FILE`, a prior instance being replaced by a fresh one. The new instance records `predecessor: {"id": "<instance id>", "ref": "<path relative to the new instance directory>", "revision": N, "fingerprint": "sha256:…"}`, where `fingerprint` is the canonical fingerprint of the predecessor file's JSON. The predecessor file must remain retained at `ref`.
- `workflow abandon --instance FILE --expected-revision N --reason TEXT` records `abandoned: {"at_ms": …, "reason": "…"}` on the instance. An abandoned instance accepts no further updates.

## Committed ledger

`<change>/provenance/ledger.json` is committed with the change and archived with it. It is generated, never hand-edited, by:

```text
azimuth provenance ledger --change <change-directory> [--instance <file>...] [--out <file>]
```

By default it reads `<change>/.azimuth/workflow.json` and follows `predecessor.ref` links. Extra `--instance` files are added for historical instances that predate predecessor links; they are ordered by their first recorded result. `change finalize` writes the ledger when `<change>/.azimuth/workflow.json` exists; `change archive` fails when the committed ledger differs from a fresh export.

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
          "agent": {"kind": {"value": null, "basis": "unknown"}, "model": {"value": null, "basis": "unknown"}, "effort": {"value": null, "basis": "unknown"}, "harness": {"name": null, "version": null, "basis": "unknown"}},
          "guidance": [],
          "trace": null,
          "used": [{"kind": "initial-request", "repository": null, "ref": "inputs/request.md", "fingerprint": "sha256:…"}],
          "generated": [{"kind": "proposal", "repository": "application", "ref": "azimuth/changes/x/proposal.md", "fingerprint": "sha256:…", "record": null}]
        }
      ]
    }
  ]
}
```

- Instance `id` is unique within a ledger. Version 3 `workflow init` assigns `<definition-id>-<init unix ms>`; a version 2 historical instance exports as `<recorded id>-<first 12 hex of its canonical file fingerprint>`, so restart snapshots stay distinct and deterministic. Predecessor links are matched by `predecessor.fingerprint`, not by id. The Assurance Service keys instances on `(project, change, id)`.
- `disposition`: `abandoned` when recorded; `superseded` when another listed instance names it as predecessor or a later historical instance follows it; `completed` when the change has a `finalization.json` or is archived and the instance is the last; otherwise `active`.
- `sequence` is the 1-based position in the instance history. `target` is the work target flattened to sorted `{name, value}` string pairs; nested values are rendered as canonical JSON.
- `scope` is derived from the target: `requirement` when it names a Requirement, else `check`, `verification-challenge`, `module`, and finally `change`.
- `report` is the authored result's `report`, or `null` when the authored file is absent or has none.
- `used` are the attempt's recorded inputs; `generated` are its output records (`record` names a selected review-bundle entry).
- Version 2 history entries export with every descriptor field `unknown`, both times `null` and `guidance` `[]`.

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

Adverse fact kinds: `requirement-status | check-result | verification-challenge-result | requirement-review`, with an exact reference string. `Confidence` is `low | medium | high`. At least one `Caused` or `Allowed through` attempt is required. `Revises` names an earlier attribution id. Descriptor lines are recorded with basis `declared`, or `unknown` when `none`.

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

reads every committed `azimuth/changes/**/provenance/ledger.json` (active and archived) and writes one JSON object per line (JSONL), one per attempt: the ledger's `project` and `change`, the instance `id` and `disposition`, and every attempt field with the descriptor flattened to `agent_kind`, `agent_kind_basis`, `model`, `model_basis`, `effort`, `effort_basis`, `harness_name`, `harness_version`, `harness_basis`. Lines are ordered by change, instance order and sequence.

## Assurance Service

Work Provenance is stored in ordinary relational tables with typed columns and no JSON. Writes require the publisher credential and are recorded with `provenance_basis: publisher-transported-assertion`; reads are open to every project role.

| Route | Purpose |
|---|---|
| `POST /v1/projects/{project}/provenance/ledgers` | Submit one change ledger. Existing attempts must match exactly; new attempts and instances are appended; a changed attempt is a conflict. |
| `POST /v1/projects/{project}/provenance/attributions` | Submit one compiled Failure Attribution. Immutable by id. |
| `GET /v1/projects/{project}/provenance/changes/{change}` | The stored ledger, reassembled. |
| `GET /v1/projects/{project}/provenance/requirements/{requirement}` | The Requirement's work history and attributions. |
| `GET /v1/projects/{project}/provenance/attempts?cursor=&limit=` | Paged dataset rows in the `report provenance` shape. |

The Requirement history follows the release lineage. Releases are ordered by `integration_sequence`; for each release, its authority is compared with the previous release's authority. A Requirement's closure changed when the canonical fingerprint of its review dependency packet differs or the Requirement is new. The change is labelled `intent` when the Requirement's entry in `requirement_intents` or any of its Scenario fingerprints differs, otherwise `dependency`. Every change recorded for that release is listed with its stored attempts; a release change without a stored ledger is listed with no attempts. The response is:

```json
{
  "requirement": "…",
  "releases": [
    {
      "release": "sha256:…", "label": "…", "integration_sequence": 3, "model_revision": "sha256:…",
      "relation": "intent",
      "changes": [{"change": "…", "attempts": [ …attempt rows… ]}]
    }
  ],
  "attributions": [ …compiled attributions for this Requirement… ]
}
```

The authority projection carries a top-level `requirement_intents: [{"id", "fingerprint"}]`, the Requirement intent digest, outside every review dependency packet, so adding it never stales a Requirement Review.
