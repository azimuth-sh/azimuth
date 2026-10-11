# Explicit workflow coordination

The CLI coordinates work; the harness assigns it, executes agents, obtains credentials and enforces permissions. Local workflow state is not model authority, an authenticated identity or Assurance State. A project account is governed by its own validation and acceptance gates, not by workflow completion.

## Commands

```text
azimuth workflow definition --workflow change-development [--out <file>]
azimuth workflow init --workflow change-development --inputs <file> --instance <file> --repositories <file>
azimuth workflow init --definition <file> [--inputs <file>] --instance <file> --repositories <file>
azimuth workflow next --instance <file> --repositories <file> [--out <file>]
azimuth workflow update --instance <file> --repositories <file> --result <file> --expected-revision <number> [--out <file>]
azimuth workflow verify --instance <file> --repositories <file> [--out <file>]
```

`--change <directory>` is an alternative to `--instance`, resolving exactly that directory's `workflow.json`. A change ID is not searched across checkouts. The initial bootstrap can live outside the repository; workflow-local records resolve relative to the instance directory. This increment does not relocate the instance on change creation or implement archive handoff.

`init` pins the validated definition and an empty history at revision zero; an existing instance is not overwritten. `next` and `verify` read the same current dependencies without state changes. They return complete handoffs in `eligible`, without choosing or prioritizing work. There are no `next`, `blocked`, `current` or `superseded` fields. An empty eligible set alone is not completion: configured obligations may remain unsatisfied. Eligibility and stale-result assessment remain internal to Azimuth. The harness chooses which eligible work to execute, assigns agents and isolates concurrent edits.

`update` validates a result against an eligible task's current basis and atomically records it under an exclusive sibling lock. Expected revision protects against competing writers. A new result increments revision once; retrying its exact ID and content returns its original `applied_revision` and the current revision without another transition. ID reuse for different content fails. A lost output receipt can be recovered by replay. Locks left by interrupted processes require operator inspection, not automatic deletion. Definition changes, captures and deployments are not performed by querying.

Outputs cannot alias the instance, command inputs, configured task inputs or historical supporting records. Output parents must exist. JSON parsing rejects duplicate/unknown fields; portable references resolve to canonical regular files below the selected repository or workflow directory, without symlink traversal or escaping components. Fingerprints use lowercase `sha256:`. File references hash the exact bytes; task basis and definition identity use canonical JSON. No source-directory inventory is inferred from file presence.

## Local repository locations

Every command accepts `--repositories FILE`. Alternatively, `--project FILE` selects that project explicitly and defaults to `local-repositories.json` alongside the project file, conventionally `azimuth/local-repositories.json`. An explicit `--repositories` overrides that default. There is no current-directory or parent-directory discovery. Ignore this file in Git; it contains only machine-local checkout paths, not workflow storage.

```json
{
  "repositories": {
    "application": "/home/developer/application",
    "deployment": "/home/developer/deployment"
  }
}
```

The file has exactly the `repositories` map, with valid IDs and existing canonical absolute directory paths. Duplicate IDs and unknown fields are errors. There is no artificial `work` repository. A workflow-local input is `{"kind":"initial-request","ref":"inputs/request.md"}`; a repository input additionally has `"repository":"application"`.

Workflow state contains no local locations. To move an in-progress workflow, copy the instance directory with all referenced local records, check out the required repository contents, and supply the new machine's repository file. Recorded fingerprints and history remain unchanged. Missing or altered records block or stale dependent work; a changed local path alone does not. State queries never rewrite history during relocation.

The prior alpha instance shape with embedded `roots` and workflow commands using `--root` are replaced, not supported as aliases. Before switching an existing instance, preserve it and its supporting records; explicitly remove its machine-local `roots` field and convert workflow-storage references to repository-free references. Because that conversion changes reference identities, existing results require explicit reassessment rather than silently rewriting their historical fingerprints.

## Request, plan steps and workflow tasks

A request states the desired product outcome; represent it as a task input of kind `initial-request`. Plan steps describe concrete engineering work in `plan.md`. Workflow tasks identify process actions such as authoring, review, implementation or assurance assessment. One implementation workflow task can cover several engineering steps.

The engine resolves declarative reusable templates into explicit finite task graphs. Inventories declare targets; directory presence never declares process state or module ownership. A configured definition registers each output kind and description; generic outputs have reference/content validation, not a fabricated semantic validator. Specialized authoring, approval, Run and assurance records have the checks below. No standalone source marker creates approval.

## Select the standard change workflow

Ordinary coordination selects the shipped `change-development` definition. `workflow definition --workflow change-development` returns its exact reusable JSON. `init` embeds that template, its declared revision and canonical fingerprint, its portable parameters, and the resolved concrete tasks. The instance's definition fingerprint pins the entire expansion. Template/version identity and exact task definitions remain available offline; an upgrade of the CLI never silently changes an existing instance. The alpha formats remain version 1; obsolete intent-action spellings are replaced without aliases.

Create the instance directory and its `inputs/request.md`. Supply `inputs.json`:

```json
{
  "project": "example-project",
  "change": "example-change",
  "request": {"kind": "initial-request", "ref": "inputs/request.md"},
  "instructions": [{"kind": "task-instructions", "ref": "inputs/instructions.md"}],
  "modules": [{"module": "example-boundary"}, {"module": "example-controls"}],
  "assurance": null
}
```

`instructions` may be empty. A referenced instruction may direct an exercise worker to copy retained artifacts rather than compose them, but it cannot change action names, review independence, dependencies or implementation permission. Every instruction reference is a hashed task input; unavailable instruction files block work. Initial input paths resolve beside the **instance**, not beside `inputs.json`. `project` and `change` are explicit identities, not path discovery. The nonempty module inventory is an explicit scope declaration; the scope task must retain an `azimuth-change-scope` version 1 record containing exactly `format`, `version`, `project`, `change`, `modules` and `rationale`, matching those declared identities and modules. Core checks consistency; the harness assesses whether the supplied account inventory is complete and correct. Unknown scope must be established before selecting module targets; no directory scan proves it.

```text
azimuth workflow init --workflow change-development --inputs inputs.json \
  --instance workflow/workflow.json --repositories azimuth/local-repositories.json
azimuth workflow next --instance workflow/workflow.json \
  --repositories azimuth/local-repositories.json --out handoff.json
azimuth workflow update --instance workflow/workflow.json \
  --repositories azimuth/local-repositories.json \
  --result result.json --expected-revision 0
```

Use the returned revision and basis for each actual result; zero applies only to the first update. The request and optional instructions must be retained inside `workflow/` for these example references. Repository paths remain only in the separately supplied local map. No manually authored task DAG is required.

The shipped stages are: establish scope → author/review proposal → author/review spec delta for each module → author/review design and verification against accepted intent → engineering plan → real implementation approval → implementation → source support → candidate validation → candidate registration → validated assurance inventory → Method Qualification → Check execution → exact-Subject Applicability Decision → Claim Judgment for non-routine Claims. The design and verification authoring branches can proceed independently after their module's spec review. Planning and approval depend on all required reviewed artifacts. Qualifier, applicability and Judgment reviews are native assurance records, distinct from authoring reviews. Rejections reopen the author; method rejection reopens implementation; applicability rejection reopens Check execution; adverse Judgment reopens implementation. Each requires newer corrective work before reassessment. No `completed` result can substitute for an accepted review or approved human gate.

## Reusable definition contract

A reusable template contains exactly `format: "azimuth-workflow-template"`, `version: 1`, `id`, positive integer `revision`, and nonempty `actions`. Action `id` is a well-known name such as `author-spec-delta`; it identifies the reusable process action. Each action contains exactly:

- `id`, `kind`: definition action identity and registered semantic profile. Shipped actions use the same name for both. A package-defined action may use `produce-artifact` while preserving its own meaningful declared action ID.
- `collection`: `change`, `modules`, `methods`, `checks`, `contributions` or `claims`. `methods` deduplicates declared Check IDs, without inventing a Subject for qualification.
- `depends_on`: ordered `{action, match, condition}` declarations. `condition` is exactly `current-success`, requiring a current dependency result in the prerequisite's declared positive `satisfies` set. `match` is `all`, `same-module`, `same-check`, `same-subject` or `claim-contributions`. Same-Check matching also pins Subject when the prerequisite carries one; Claim-contribution matching pins Claim and Subject. A required matcher resolving to no target fails expansion.
- `inputs`: ordered binding names from `request`, `instructions`, `snapshot`. Assurance profiles require `snapshot`. Missing files make the affected work ineligible; they are not fabricated inputs.
- `guidance`: registered skill/reference/package locators; the harness resolves installed guidance and must report unavailable resources.
- `required_output`: `{kind, description}`. Native authoring, approval, execution and assurance profiles enforce their real record kinds.
- `outcomes`, `satisfies`: nonempty unique bounded outcome sets. Positive transitions can use only permitted `completed`, `accepted`, `approved` or `supported` outcomes. Review rejection, failed work, denied/deferred permission and adverse Judgment remain retained facts and never satisfy a downstream positive gate.
- `correction`: null or a direct prerequisite action resolving to exactly one concrete target. Authoring reviews must depend on their one matching author and reopen that author.

Actions are declared in dependency order. Unknown actions/outcomes/bindings, duplicate fields/dependencies, forward dependencies, cycles, invalid target matchers, unavailable required targets, mismatched native outputs and impossible positive outcome sets fail before initialization or expansion. All finite dependency paths originate at an available root; arbitrary expressions, scripts, plugin code and scheduler behavior are excluded. Inspect the shipped definition for the complete ordinary process, or pass another strict reusable template through `--definition FILE --inputs FILE` to the same engine.

A concrete work reference is exactly `{action, target}`. `action` is the definition action ID; `target` contains the declared typed locators. The target includes explicit project/change and, for assurance, exact model revision; qualifying methods carry Check identity without Subject, while execution/contribution/Judgment tasks carry exact Subject. References are compared by canonical JSON, independent of field order, inventory order and local checkout paths. No target hash is an identity. Each handoff's `id`, result `task`, concrete definition `id`/`depends_on`/`correction`, and review or approval `task` use this same structure. Artifact fingerprints, basis fingerprints and definition fingerprints remain hashes because they detect content drift rather than name work. Unknown action/target pairs are rejected.

The previous alpha hashed/string work references are replaced without aliases or automatic migration. Preserve an old instance and its immutable records, then initialize a fresh instance and reassess its obligations. Do not rewrite accepted historical results or their basis fingerprints. The format remains version 1.

## Later assurance inventories

The registration task retains a strict local `azimuth-candidate-registration` version 1 receipt with exactly `format`, `version`, `project`, `model_revision`. The harness obtains actual registration through the existing owner API; a local receipt does not authenticate service transport. The following `inventory-assurance-targets` action outputs `assurance-target-inventory` referencing:

```json
{
  "format": "azimuth-assurance-target-inventory",
  "version": 1,
  "project": "example-project",
  "model_revision": "sha256:<registered candidate projection>",
  "snapshot": {"kind": "assurance-snapshot", "ref": "inputs/candidate-snapshot.json"},
  "checks": [{"check": "example-check", "subject": "sha256:<exact Subject>"}],
  "contributions": [{"check": "example-check", "case": "example-case", "claim": "example-claim", "subject": "sha256:<exact Subject>"}],
  "claims": [{"claim": "example-claim", "subject": "sha256:<exact Subject>"}]
}
```

Placeholder fingerprints are explanatory, not valid values. Supply the candidate revision's authenticated snapshot through the declared reference. The update verifies its native authority/reviews/Run protocol, project and registered revision, every Check and Case/Claim relationship, every declared Check–Case contribution, each non-routine Claim and the completeness of its declared binding composition for each Subject. Routine Claims owe no Judgment; an entirely routine account may supply three empty arrays. Unknown targets, omitted non-routine Claims or required contributions, invalid snapshots and a missing registration fail without recording the inventory. Missing snapshot files leave this stage incomplete; no assurance tasks or completion are inferred. An empty method/contribution collection cannot satisfy a nonempty dependent Claim matcher.

A valid inventory result atomically pins the inventory and instantiates assurance tasks from the selected definition; the coordinator never writes assurance task names or a DAG. Subsequent `next` calls require actual snapshot content, matching project/revision and each native review's currentness. Changed or unavailable snapshot content makes affected work ineligible or stale. The harness supplies Runs and reviews through existing transports; the workflow does not synthesize either.

After corrective work registers a different immutable candidate, the inventory task becomes stale through its dependencies and accepts a newly validated inventory. Expansion appends the new candidate's concrete tasks, retaining prior inventory parameters, task definitions, record references and results. Tasks for earlier candidate revisions remain in the retained definition and history; they are not eligible or current obligations. A superseded candidate cannot be silently reactivated. Within one candidate revision, target inventory and snapshot reference remain fixed; refresh the referenced snapshot content to retain native currentness. Each new candidate generation pins its own snapshot reference. Exact unchanged Check definitions can retain native Method Qualification; re-execution alone does not require inventing another review. A later workflow result still has to identify the current retained native record in the selected snapshot.

## Definition

```json
{
  "format": "azimuth-workflow-definition",
  "version": 1,
  "id": "change-assurance",
  "tasks": [
    {
      "id": {"action": "author-spec-delta", "target": {"module": "example/boundary"}},
      "kind": "author-spec-delta",
      "target": {"module": "example/boundary"},
      "depends_on": [],
      "inputs": [
        {"kind": "initial-request", "ref": "request.md"}
      ],
      "guidance": [{"kind": "skill", "id": "azimuth-propose"}],
      "required_output": {"kind": "intent-delta", "description": "The proposed Claims and Cases."},
      "correction": null
    },
    {
      "id": {"action": "review-spec-delta", "target": {"module": "example/boundary"}},
      "kind": "review-spec-delta",
      "target": {"module": "example/boundary"},
      "depends_on": [{"action": "author-spec-delta", "target": {"module": "example/boundary"}}],
      "inputs": [
        {"kind": "initial-request", "ref": "request.md"}
      ],
      "guidance": [{"kind": "skill", "id": "azimuth-propose"}],
      "required_output": {"kind": "authoring-review", "description": "Independent completeness and precision assessment."},
      "correction": {"action": "author-spec-delta", "target": {"module": "example/boundary"}}
    }
  ]
}
```

Each concrete task has `id`, `kind`, `target`, `depends_on`, `inputs`, `guidance`, `required_output` and `correction`. Template-expanded tasks additionally retain `action`, `outcomes` and `satisfies`; these fields are validated, not arbitrary metadata. `id` identifies this concrete task within the instance; `kind` names the registered process action. Dependencies carry structured work references and must be known and acyclic. `target` contains typed locators from `project`, `change`, `module`, `claim`, `case`, `check`, `probe`, `subject` and `model_revision`; Subject and revision values are exact fingerprints. Guidance kinds are `skill`, `reference` or `package`; the harness must resolve installed guidance and report unavailable resources rather than invent them.

Registered task kinds cover scope establishment; intent, design and verification authoring/review; engineering planning; implementation approval; implementation; source capture and validation; candidate revision registration; Method Qualification; Check execution; Applicability Decision; and Claim Judgment. The authoritative semantic profiles are in `workflow_state.rs`. Reusable templates register their own action IDs against these bounded profiles, including `produce-artifact` for package-defined artifact work. Packages supply a template through `--definition`; they cannot register executable code or bypass semantic review/approval contracts.

`correction` is null or a direct prerequisite work reference. An adverse result reopens that prerequisite and blocks reassessment until a newer correction result is recorded. Authoring reviews with null correction derive the matching direct author task. Configure qualification correction to implementation and applicability correction to execution where that is the actual corrective boundary. For unresolved Judgments, choose the relevant prerequisite or leave diagnosis to the harness; the tool does not invent the substantive fix.

For a concrete definition, a completed result can include one additional supporting record of kind `workflow-definition`. That file must retain the definition identity and every existing task unchanged in the same prefix, and may append validated tasks for the discovered module scope. The new definition and its fingerprint are recorded atomically with the result. This advanced concrete-graph facility does not infer modules from directory scanning or rewrite previous work. A pinned template rejects such extensions; its assurance stages expand through validated inventory results only.

Inputs are `{kind, ref}` for workflow-local files, or `{kind, repository, ref}` for repository files. `ref` is always a normal relative file path. The local repository file supplies checkout locations; the instance directory supplies the location of workflow-local files. Missing inputs block the corresponding task, not the whole initial authoring graph. Handoffs materialize exact `{kind, repository, ref, fingerprint}` references. The published `inputs` list contains configured inputs followed by direct prerequisite artifact references, in declared order, with identical references deduplicated. Conflicting references to the same kind/repository/path fail closed. Prerequisite result envelopes, outcomes and author identities remain internal for dependency gates and independence validation; no `dependencies` field is published. Keep historical review/report files immutable; changed current artifacts make dependent results stale without erasing their history.

## Handoff and result

A handoff includes `id`, `action` (the definition action ID), `task` (the semantic profile), `target`, `repositories`, `workflow_directory`, `basis`, `reason`, `guidance`, `inputs`, `corrections` and `required_output`. The outer response contains format/version, instance/revision, definition fingerprint, `eligible`. All eligible tasks have complete handoffs; their order does not express execution priority. Definitions describe only that configured graph, not every project obligation. Each eligible handoff carries validated `repositories` (repository IDs mapped to absolute checkout paths) and `workflow_directory` (the canonical instance directory). A worker receiving that handoff alone can resolve both kinds of reference. These are local execution locations, not portable metadata, additional permissions or semantic identity; they are excluded from task-basis fingerprints. The harness must regenerate a handoff on the execution machine rather than reuse another machine's paths.

```json
{
  "format": "azimuth-workflow-result",
  "version": 1,
  "id": "author-spec-delta-attempt-1",
  "task": {"action": "author-spec-delta", "target": {"module": "example/boundary"}},
  "basis": "sha256:<handoff basis>",
  "actor": "author",
  "outcome": "completed",
  "records": [
    {"kind": "intent-delta", "ref": "spec.md", "fingerprint": "sha256:<file bytes>"}
  ]
}
```

Angle-bracket fingerprints are explanatory placeholders, not valid inputs. Every result has exactly these fields; `task` contains the concrete work reference. Supporting record references are unique. A successful generic result needs exactly one record matching its declared output kind. A failed generic task needs a `failure-report` and remains incomplete. Completed execution capture can retain violated or inconclusive Observations without turning them into workflow failure or positive assurance.

| Task class | Outcomes |
| --- | --- |
| Generic authoring/implementation/capture/validation/registration/execution | `completed`, `failed` |
| Authoring review, Method Qualification, Applicability Decision | `accepted`, `rejected` |
| Claim Judgment | `supported`, `violated`, `unresolved` |
| Implementation approval | `approved`, `denied`, `deferred` |

Protocol-valid adverse results are recorded normally. Structural mismatch, invalid basis, conflicting revision or malformed record fails without a transition. Currentness rechecks actual supporting content and dependency semantics, not manually authored completion flags.

## Independent authoring reviews and human approval

An `azimuth-authoring-review` version 1 record contains exactly `format`, `version`, `task`, `reviewer`, `outcome`, `dependencies`, `findings` and `rationale`. Task/actor/outcome must match the submitted result. Dependencies equal the exact published handoff `inputs` list, including its flattened prerequisite references. Findings are prose strings; rejection requires at least one finding. The reviewer must differ from direct prerequisite authors. Actor identities are local harness assertions; neither the file nor core establishes authenticated human independence.

Review the proposal against the original request, declared scope, exclusions, singular ownership, repository participation, unresolved assumptions and readiness to author module contracts. Proposal review requires a reviewer distinct from the proposal author and accepted current review before spec authoring; rejection reopens proposal authoring followed by independent reassessment. Proposal review does not grant implementation permission. Review intent against the original request and boundaries; review design against every Case and relevant architecture; review verification for coverage, credible observations, independent oracles, controls and limitations. Acceptance is distinct from parsing and from artifact authoring. Rejected reviews expose author correction and independent reassessment. Changing a pinned dependency invalidates current acceptance; the historical review remains.

Implementation approval uses an `azimuth-workflow-approval` version 1 record containing `format`, `version`, `task`, `actor`, `outcome`, `dependencies` and `rationale`. Dependencies equal the exact published handoff `inputs` list and cover the proposal package being approved. Only `approved` satisfies the gate. The harness must obtain the real human decision and enforce authority; the CLI does not authenticate the local actor label or create permission.

## ASS integration and the three assurance reviews

ASS project creation, credential assignment and accepted-baseline onboarding precede this workflow. Register immutable candidate projections with owner `POST /v1/projects/{project}/model-revisions`; do not select the candidate as accepted just to assess it. Producers submit Runs within that existing project. Independent reviewers submit existing strict records to the exact revision's `/reviews` endpoint, then the harness fetches `/snapshot` over authenticated transport.

Each assurance task supplies one input of kind `assurance-snapshot`, containing exactly `{project, model_revision, authority, reviews, bundles}`. Core validates the projection, review schemas and Run protocol, matches the configured Check/Case/Claim and mandatory exact Subject for applicability and Judgment, and requires the unchanged review record to be present in the supplied snapshot. A local verdict alone is insufficient. The harness is responsible for authenticating snapshot transport; a local JSON file cannot prove where it came from.

Qualification output is a `method-qualification` record; applicability output is an `applicability-decision`; Judgment output is a `claim-judgment`. Existing `assurance_review` validators and dependency assessments govern their meaning. A superseded review cannot satisfy a task: current record selection follows the native typed-target and review-time rules, including ambiguous latest records. Method currentness projects the relevant Check identity rather than renewing qualification whenever a new Run or unrelated review arrives. Applicability and Judgment are reassessed against actual current snapshot evidence and definitions; stale reviews cannot satisfy downstream obligations.

Check execution output includes a protocol-valid `run-bundle`. Native execution, capture, normalization and service submission use existing adapter and Run commands, not an agent-invented Observation. Authoring review examines the intended account; Method Qualification examines an implemented method; Applicability Decision examines a particular contribution; Claim Judgment examines composition. None is synthesized from another's success.

## Retention and limits

State embeds append-only result records and retains exact references to supporting content. Locks serialize writers; hashes detect input drift during update. This is not an adversarial filesystem sandbox or a multi-host transactional scheduler. A coordinator crash may require inspecting a lock or replaying a result; an agent is not automatically relaunched.

No automatic source-directory inference, `.azimuth` exclusion, source capture, directory relocation, service transport, deployment, full-federation finalization, accepted-revision promotion gate, archival or production scheduling is claimed. Keep operational data outside examined source roots until capture ownership is explicit. Before deleting a local working directory, retain every required referenced record in durable storage. Runtime history does not turn missing source/hosted evidence into support.
