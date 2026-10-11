# Explicit workflow coordination

The CLI coordinates work; the harness assigns it, executes agents, obtains credentials and enforces permissions. Local workflow state is not model authority, an authenticated identity or Assurance State. A project account is governed by its own validation and acceptance gates, not by a workflow's `current` booleans.

## Commands

```text
azimuth workflow init --definition <file> --instance <file> --root <id>=<canonical-directory>...
azimuth workflow next --instance <file> [--out <file>]
azimuth workflow update --instance <file> --result <file> --expected-revision <number> [--out <file>]
azimuth workflow verify --instance <file> [--out <file>]
```

`--change <directory>` is an alternative to `--instance`, resolving exactly that directory's `workflow.json`. A change ID is not searched across checkouts. The initial bootstrap can live outside the repository; supplying explicit runtime roots keeps generated state out of examined source identity. This increment does not relocate the instance on change creation or implement archive handoff.

`init` pins the validated definition, explicit roots and an empty history at revision zero; an existing instance is not overwritten. `next` and `verify` read the same current dependencies without state changes. They return full eligible handoffs, blockers and a task-ID recommendation in `next` (null when none are eligible). A null recommendation with blockers is not completion. The harness may choose another eligible task or several in parallel.

`update` validates a result against an eligible task's current basis and atomically records it under an exclusive sibling lock. Expected revision protects against competing writers. A new result increments revision once; retrying its exact ID and content returns its original `applied_revision` and the current revision without another transition. ID reuse for different content fails. A lost output receipt can be recovered by replay. Locks left by interrupted processes require operator inspection, not automatic deletion. Definition changes, captures and deployments are not performed by querying.

Outputs cannot alias the instance, command inputs, configured task inputs or historical supporting records. Output parents must exist. JSON parsing rejects duplicate/unknown fields; portable references are canonical regular files below explicit roots, without symlink traversal or escaping components. Fingerprints use lowercase `sha256:`. File references hash the exact bytes; task basis and definition identity use canonical JSON. No source-directory inventory is inferred from file presence.

## Request, plan steps and workflow tasks

A request states the desired product outcome; represent it as a task input of kind `initial-request`. Plan steps describe concrete engineering work in `plan.md`. Workflow tasks identify process actions such as authoring, review, implementation or assurance assessment. One implementation workflow task can cover several engineering steps.

This initial engine uses an explicit finite task graph. It does not automatically instantiate every Check or infer module ownership. A configured definition registers each output kind and description; generic outputs have reference/content validation, not a fabricated semantic validator. Specialized authoring, approval, Run and assurance records have the checks below. No standalone source marker creates approval.

## Definition

```json
{
  "format": "azimuth-workflow-definition",
  "version": 1,
  "id": "change-assurance",
  "tasks": [
    {
      "id": "author-contract",
      "kind": "author-intent-delta",
      "target": {"module": "example/boundary"},
      "depends_on": [],
      "inputs": [
        {"kind": "initial-request", "repository": "work", "ref": "request.md"}
      ],
      "guidance": [{"kind": "skill", "id": "azimuth-propose"}],
      "required_output": {"kind": "intent-delta", "description": "The proposed Claims and Cases."},
      "correction": null
    },
    {
      "id": "review-contract",
      "kind": "review-intent-delta",
      "target": {"module": "example/boundary"},
      "depends_on": ["author-contract"],
      "inputs": [
        {"kind": "initial-request", "repository": "work", "ref": "request.md"}
      ],
      "guidance": [{"kind": "skill", "id": "azimuth-propose"}],
      "required_output": {"kind": "authoring-review", "description": "Independent completeness and precision assessment."},
      "correction": "author-contract"
    }
  ]
}
```

Each task has exactly `id`, `kind`, `target`, `depends_on`, `inputs`, `guidance`, `required_output` and `correction`. `id` identifies this concrete task within the instance; `kind` names the registered process action. Dependencies name task IDs and must be known and acyclic. `target` contains typed locators from `change`, `module`, `claim`, `case`, `check`, `probe` and `subject`; subject values are exact fingerprints. Guidance kinds are `skill`, `reference` or `package`; the harness must resolve installed guidance and report unavailable resources rather than invent them.

Registered task kinds cover scope establishment; intent, design and verification authoring/review; engineering planning; implementation approval; implementation; source capture and validation; candidate revision registration; Method Qualification; Check execution; Applicability Decision; and Claim Judgment. The authoritative list is in `workflow_state.rs`; package-defined task registration is not implemented in this increment.

`correction` is null or a direct prerequisite task ID. An adverse result reopens that prerequisite and blocks reassessment until a newer correction result is recorded. Authoring reviews with null correction derive the matching direct author task. Configure qualification correction to implementation and applicability correction to execution where that is the actual corrective boundary. For unresolved Judgments, choose the relevant prerequisite or leave diagnosis to the harness; the tool does not invent the substantive fix.

A completed scope result can include one additional supporting record of kind `workflow-definition`. That file must retain the definition identity and every existing task unchanged in the same prefix, and may append validated tasks for the discovered module scope. The new definition and its fingerprint are recorded atomically with the result. This permits a one-task scope bootstrap followed by explicit module task expansion; it does not infer modules from directory scanning or rewrite previous work.

Inputs are `{kind, repository, ref}`. Roots map explicit repository/work IDs to canonical absolute local directories. Missing inputs block the corresponding task, not the whole initial authoring graph. Handoffs materialize exact `{kind, repository, ref, fingerprint}` references and retain prerequisite result records. Keep historical review/report files immutable; changed current artifacts make dependent results stale without erasing their history.

## Handoff and result

A handoff includes `id`, `task` (the registered kind), `target`, `basis`, `reason`, `guidance`, `inputs`, `dependencies`, `corrections` and `required_output`. The outer response contains format/version, instance/revision, definition fingerprint, `eligible`, `next`, `blocked` and `current`. All eligible tasks have complete handoffs; recommendation is an ID rather than a duplicate handoff. Definitions and returned `current` values describe only that configured graph, not every project obligation.

```json
{
  "format": "azimuth-workflow-result",
  "version": 1,
  "id": "author-contract-attempt-1",
  "task": "author-contract",
  "basis": "sha256:<handoff basis>",
  "actor": "author",
  "outcome": "completed",
  "records": [
    {"kind": "intent-delta", "repository": "work", "ref": "spec.md", "fingerprint": "sha256:<file bytes>"}
  ]
}
```

Angle-bracket fingerprints are explanatory placeholders, not valid inputs. Every result has exactly these fields; `task` names the concrete task ID. Supporting record references are unique. A successful generic result needs exactly one record matching its declared output kind. A failed generic task needs a `failure-report` and remains incomplete. Completed execution capture can retain violated or inconclusive Observations without turning them into workflow failure or positive assurance.

| Task class | Outcomes |
| --- | --- |
| Generic authoring/implementation/capture/validation/registration/execution | `completed`, `failed` |
| Authoring review, Method Qualification, Applicability Decision | `accepted`, `rejected` |
| Claim Judgment | `supported`, `violated`, `unresolved` |
| Implementation approval | `approved`, `denied`, `deferred` |

Protocol-valid adverse results are recorded normally. Structural mismatch, invalid basis, conflicting revision or malformed record fails without a transition. Currentness rechecks actual supporting content and dependency semantics, not manually authored completion flags.

## Independent authoring reviews and human approval

An `azimuth-authoring-review` version 1 record contains exactly `format`, `version`, `task`, `reviewer`, `outcome`, `dependencies`, `findings` and `rationale`. Task/actor/outcome must match the submitted result. Dependencies are the exact handoff input references followed by direct prerequisite records in their declared order. Findings are prose strings; rejection requires at least one finding. The reviewer must differ from direct prerequisite authors. Actor identities are local harness assertions; neither the file nor core establishes authenticated human independence.

Review intent against the original request and boundaries; review design against every Case and relevant architecture; review verification for coverage, credible observations, independent oracles, controls and limitations. Acceptance is distinct from parsing and from artifact authoring. Rejected reviews expose author correction and independent reassessment. Changing a pinned dependency invalidates current acceptance; the historical review remains.

Implementation approval uses an `azimuth-workflow-approval` version 1 record containing `format`, `version`, `task`, `actor`, `outcome`, `dependencies` and `rationale`. Exact dependencies cover the proposal package being approved. Only `approved` satisfies the gate. The harness must obtain the real human decision and enforce authority; the CLI does not authenticate the local actor label or create permission.

## ASS integration and the three assurance reviews

ASS project creation, credential assignment and accepted-baseline onboarding precede this workflow. Register immutable candidate projections with owner `POST /v1/projects/{project}/model-revisions`; do not select the candidate as accepted just to assess it. Producers submit Runs within that existing project. Independent reviewers submit existing strict records to the exact revision's `/reviews` endpoint, then the harness fetches `/snapshot` over authenticated transport.

Each assurance task supplies one input of kind `assurance-snapshot`, containing exactly `{project, model_revision, authority, reviews, bundles}`. Core validates the projection, review schemas and Run protocol, matches the configured Check/Case/Claim and mandatory exact Subject for applicability and Judgment, and requires the unchanged review record to be present in the supplied snapshot. A local verdict alone is insufficient. The harness is responsible for authenticating snapshot transport; a local JSON file cannot prove where it came from.

Qualification output is a `method-qualification` record; applicability output is an `applicability-decision`; Judgment output is a `claim-judgment`. Existing `assurance_review` validators and dependency assessments govern their meaning. A superseded review cannot satisfy a task: current record selection follows the native typed-target and review-time rules, including ambiguous latest records. Method currentness projects the relevant Check identity rather than renewing qualification whenever a new Run or unrelated review arrives. Applicability and Judgment are reassessed against actual current snapshot evidence and definitions; stale reviews cannot satisfy downstream obligations.

Check execution output includes a protocol-valid `run-bundle`. Native execution, capture, normalization and service submission use existing adapter and Run commands, not an agent-invented Observation. Authoring review examines the intended account; Method Qualification examines an implemented method; Applicability Decision examines a particular contribution; Claim Judgment examines composition. None is synthesized from another's success.

## Retention and limits

State embeds append-only result records and retains exact references to supporting content. Locks serialize writers; hashes detect input drift during update. This is not an adversarial filesystem sandbox or a multi-host transactional scheduler. A coordinator crash may require inspecting a lock or replaying a result; an agent is not automatically relaunched.

No automatic `.azimuth` exclusion, source capture, directory relocation, service transport, deployment, full-federation finalization, accepted-revision promotion gate, archival or production scheduling is claimed. Keep operational data outside examined source roots until capture ownership is explicit. Before deleting a local working directory, retain every required referenced record in durable storage. Runtime history does not turn missing source/hosted evidence into support.
