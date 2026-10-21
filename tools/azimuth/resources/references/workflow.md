# Explicit workflow coordination

The CLI coordinates work; the harness assigns it, executes agents, obtains credentials and enforces permissions. Local workflow state is not model authority, an authenticated identity or Assurance State. A project account is governed by its own validation and acceptance gates, not by workflow completion.

## Commands

```text
azimuth workflow definition --workflow change-development [--out <file>]
azimuth workflow init --workflow change-development --inputs <file> --instance <file> --repositories <file>
azimuth workflow init --definition <file> [--inputs <file>] --instance <file> --repositories <file>
azimuth workflow next --instance <file> --repositories <file> [--out <file>]
azimuth workflow update --instance <file> --repositories <file> --handoff <file> --actor <id> --result <file> --expected-revision <number> [--out <file>]
azimuth workflow verify --instance <file> --repositories <file> [--out <file>]
```

`--change <directory>` is an alternative to `--instance`, resolving exactly that directory's `workflow.json`. A change ID is not searched across checkouts. The initial bootstrap can live outside the repository; workflow-local records resolve relative to the instance directory. This increment does not relocate the instance on change creation or implement archive handoff.

`init` pins the validated definition and an empty history at revision zero; an existing instance is not overwritten. `next` and `verify` read the same current dependencies without state changes. They return complete handoffs in `eligible`, without choosing or prioritizing work. There are no `next`, `blocked`, `current` or `superseded` fields. An empty eligible set alone is not completion: configured obligations may remain unsatisfied. Eligibility and stale-result assessment remain internal to Azimuth. The harness chooses which eligible work to execute, assigns agents and isolates concurrent edits.

`update` accepts a compact authored result and the captured `next` handoff, derives its exact supporting fingerprints and actor assertion from the harness, validates it against an eligible task's current basis and atomically records it under an exclusive sibling lock. Expected revision protects against competing writers. A new result increments revision once; retrying its exact ID and content returns its original `applied_revision` and the current revision without another transition. ID reuse for different content fails. A lost output receipt can be recovered by replay. Locks left by interrupted processes require operator inspection, not automatic deletion. Definition changes, captures and deployments are not performed by querying.

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

Ordinary coordination selects the shipped `change-development` definition. `workflow definition --workflow change-development` returns its exact reusable JSON. `init` embeds that template, its declared revision and canonical fingerprint, its portable parameters, and the resolved concrete tasks. The instance's definition fingerprint pins the entire expansion. Template/version identity and exact task definitions remain available offline; an upgrade of the CLI never silently changes an existing instance. The workflow definition, instance, template, handoff and result formats are version 2; obsolete intent-action spellings are replaced without aliases.

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

Pass the captured handoff with `--handoff` and the executing actor with `--actor`; workers do not transcribe basis or supporting fingerprints. Use the current revision for each actual update; zero applies only to the first update. The request and optional instructions must be retained inside `workflow/` for these example references. Repository paths remain only in the separately supplied local map. No manually authored task DAG is required.

Historical revision 6 stages were: establish scope → author/review proposal → author/review module spec, design and verification → engineering plan → real implementation approval → implementation → structural change-account validation → candidate preparation and automatic target derivation → Requirement Review → Check and Verification Challenge execution → exact-target Requirement Status → candidate registration/publication. Design and verification authoring can proceed independently after spec review. Verification authoring includes machine Verification Challenges; implementation includes their code/configuration. Independent readiness reviews remain distinct from post-implementation Requirement Review.

Rejected Requirement Reviews expose correction and independent reassessment. Historical authored Requirement Status workflows also expose adverse/unresolved Requirement Status for correction. Check execution records actual violations and measurement failures rather than an invented passing result. Verification Challenges target an exact assessment; adverse findings affect reliance through native assessment rules. The harness selects eligible work, providers and concurrency. No `completed` result substitutes for an accepted review, supported Requirement Status or human permission.

## Change-account validation

`validate-change-account` is one workflow objective after implementation. Capture current ecosystem and deployment manifests and exact repository Verification Targets, then validate the complete change account against those captures. Its `candidate-validation` result retains findings and exact dependencies, including source-support manifests and capture completion. Capture failure leaves validation incomplete; a capture may be reused only while its dependencies remain current. Use `azimuth change validate --scope structure` for this task. Its report excludes independent decision and acceptance obligations; assess those later with `--scope acceptance`. Structural success does not establish definition credibility or exact-target Requirement Status.

Source capture and validation commands remain independently callable for diagnostics and reuse. The standard workflow does not schedule capture as a separate task. A custom workflow may separate operations when they require different environments or owners. Existing instances pin their definitions; updating the CLI does not migrate them. Before explicitly replacing unexecuted stages in an alpha instance, preserve its original state and retain every completed task definition and history record unchanged. Verify the replacement instance and retain a migration record; do not reinterpret previous capture or validation results as completion of the combined objective.

## Reusable definition contract

A reusable template contains exactly `format: "azimuth-workflow-template"`, `version: 1`, `id`, positive integer `revision`, and nonempty `actions`. Action `id` is a well-known name such as `author-spec-delta`; it identifies the reusable process action. Each action contains exactly:

- `id`, `kind`: definition action identity and registered semantic profile. Shipped actions use the same name for both. A package-defined action may use `produce-artifact` while preserving its own meaningful declared action ID.
- `collection`: `change`, `modules`, `requirement-definitions`, `checks` or `requirements` in the current template. Requirement definitions carry no execution Verification Target; Check and Requirement Status targets require explicit Verification Target selection. Historical templates may retain `methods` and `contributions`.
- `depends_on`: ordered `{action, match, condition}` declarations. `condition` is exactly `current-success`, requiring a current dependency result in the prerequisite's declared positive `satisfies` set. `match` is `all`, `same-module`, `same-check`, `same-verification-target`, `same-requirement` or `requirement-contributions`. Same-Check matching also pins Verification Target when the prerequisite carries one; Requirement-contribution matching pins Requirement and Verification Target. A required matcher resolving to no target fails expansion.
- `inputs`: ordered binding names from `request`, `instructions`, `snapshot`. Assurance profiles require `snapshot`. Missing files make the affected work ineligible; they are not fabricated inputs.
- `guidance`: registered skill/reference/package locators; the harness resolves installed guidance and must report unavailable resources.
- `required_output`: `{kind, description}`. Native authoring, approval, execution and assurance profiles enforce their real record kinds.
- `outcomes`, `satisfies`: nonempty unique bounded outcome sets. Positive transitions can use only permitted `completed`, `accepted`, `approved` or `supported` outcomes. Review rejection, failed work, denied/deferred permission and adverse Judgment remain retained facts and never satisfy a downstream positive gate.
- `correction`: null or a direct prerequisite action resolving to exactly one concrete target. Authoring reviews must depend on their one matching author and reopen that author.

Actions are declared in dependency order. Unknown actions/outcomes/bindings, duplicate fields/dependencies, forward dependencies, cycles, invalid target matchers, unavailable required targets, mismatched native outputs and impossible positive outcome sets fail before initialization or expansion. All finite dependency paths originate at an available root; arbitrary expressions, scripts, plugin code and scheduler behavior are excluded. Inspect the shipped definition for the complete ordinary process, or pass another strict reusable template through `--definition FILE --inputs FILE` to the same engine.

A concrete work reference is exactly `{action, target}`. `action` is the definition action ID; `target` contains the declared typed locators. The target includes explicit project/change and, for assurance, exact model revision; Requirement Review tasks carry Requirement identity without Verification Target, while Check execution and Requirement Status tasks carry exact Verification Target. References are compared by canonical JSON, independent of field order, inventory order and local checkout paths. No target hash is an identity. Each handoff's `id`, result `task`, concrete definition `id`/`depends_on`/`correction`, and review or approval `task` use this same structure. Artifact fingerprints, basis fingerprints and definition fingerprints remain hashes because they detect content drift rather than name work. Unknown action/target pairs are rejected.

The previous alpha hashed/string work references are replaced without aliases or automatic migration. Preserve an old instance and its immutable records, then initialize a fresh instance and reassess its obligations. Do not rewrite accepted historical results or their basis fingerprints. The format remains version 1.

## Automatic assurance targets

Candidate preparation supplies the complete `candidate-authority` and an `assurance-snapshot` output. The snapshot retains exactly `{project, model_revision, authority, reviews, bundles}` and must match the prepared candidate. The update derives and pins the assurance inventory and expands tasks atomically; there is no `inventory-assurance-targets` agent task or authored inventory. Requirement Review targets are declared Requirements, independent of execution Verification Target selection. Fresh authority Requirements contain an explicit Verification Challenge array: an empty array establishes no declared Verification Challenge work; a missing historical field establishes no such absence. Derivation creates no review, Check Result, human decision or ASS publication.

Revision 10, like revision 9, requires explicit exact Verification Targets for local Check execution; its `requirements` selection may be empty because deployment Requirement Status is computed after rollout. Earlier workflow revisions also selected exact Verification Targets for authored Requirement Status. A preparation or subsequent review update may additionally supply an output of kind `assurance-verification-target-selection`. Its JSON has exactly:

```json
{
  "format": "azimuth-assurance-verification-target-selection",
  "version": 2,
  "project": "example-project",
  "model_revision": "sha256:<prepared candidate revision>",
  "checks": [
    {"check": "example-check", "verification_target": {"kind": "verification-target", "ref": "artifacts/application-verification-target.json"}}
  ],
  "requirements": [
    {"requirement": "example-requirement", "verification_target": {"kind": "verification-target", "ref": "artifacts/application-verification-target.json"}}
  ]
}
```

References may include `repository`. Each resolves to a real native Run-protocol Verification Target descriptor. Azimuth validates and computes its identity, validates each selected Check/Requirement against the candidate, and derives Check–Scenario contributions from the authority. Multiple explicitly selected Verification Targets for one target are allowed. It never assigns all Checks to a captured workspace, infers deployed Verification Targets from repository paths, or uses a model revision as an execution Verification Target.

Selections may be absent or partial. `workflow next` reports missing selections in `required_inputs`, independently of its eligible Requirement Review tasks. Each entry names kind `assurance-verification-target-selection`, typed target `{kind: check | requirement, id}` and explanatory reason. Execution tasks exist only for selected Check/Verification Target pairs. A Requirement Status task exists only when the Requirement has an explicit selected Verification Target and every declared Check–Scenario contribution has a matching execution target on that same Verification Target. Incomplete selections leave the requirement visible and prevent registration. Empty eligible work is not successful assurance.

A source Run and a deployment Run are different Verification Targets. Historical authored Requirement Status records do not compose their evidence into a common Verification Target, and matching candidate revision alone is insufficient. Current deployment Requirement Status instead uses the explicit deployment applicability contract: exact release/build lineage, observed deployment composition, effective contexts, required unit coverage and fresh monitoring. These validated relations permit applicability without rewriting a Run's actual Verification Target.

The inventory is internal portable workflow metadata: Requirement definitions, selected execution targets, derived contributions, complete Requirement Status targets, and pending Verification Target selections. A corrective candidate preserves prior generations and immutable results. New derivation cannot rewrite any completed task definition. Within a candidate, selections can be completed explicitly while preserving completed work. Snapshots remain actual retained inputs and native currentness rules still apply.

`azimuth workflow assurance-targets --instance FILE --project PROJECT --snapshot RELATIVE_REF [--verification-targets RELATIVE_REF] --out FILE` computes a proposed expanded definition without changing the instance. It supports inspection and explicit migration of already-prepared instances. It does not execute work, synthesize a result or apply a state transition. Snapshot and Verification Target selection references resolve beneath the workflow directory. New ordinary instances derive the same definition through their preparation update.

## Definition

```json
{
  "format": "azimuth-workflow-definition",
  "version": 2,
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
      "required_output": {"kind": "intent-delta", "description": "The proposed Requirements and Scenarios."},
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

Each concrete task has `id`, `kind`, `target`, `depends_on`, `inputs`, `guidance`, `required_output` and `correction`. Template-expanded tasks additionally retain `action`, `outcomes` and `satisfies`; these fields are validated, not arbitrary metadata. `id` identifies this concrete task within the instance; `kind` names the registered process action. Dependencies carry structured work references and must be known and acyclic. `target` contains typed locators from `project`, `change`, `module`, `requirement`, `scenario`, `check`, `probe`, `verification_target` and `model_revision`; Verification Target and revision values are exact fingerprints. Guidance kinds are `skill`, `reference` or `package`; the harness must resolve installed guidance and report unavailable resources rather than invent them.

Current registered task kinds cover scope establishment; intent, design and verification authoring/review; engineering planning; implementation approval; implementation; source capture and validation; candidate revision registration; Requirement Review; Check and machine Verification Challenge execution; and Requirement Status. Historical pinned instances retain their original Method Qualification, Applicability Decision and Requirement Judgment profiles without satisfying current gates. The authoritative semantic profiles are in `workflow_state.rs`. Reusable templates register their own action IDs against these bounded profiles, including `produce-artifact` for package-defined artifact work. Packages supply a template through `--definition`; they cannot register executable code or bypass semantic review/approval contracts.

`correction` is null or a direct prerequisite work reference. An adverse result reopens that prerequisite and blocks reassessment until a newer correction result is recorded. Authoring reviews with null correction derive the matching direct author task. Configure Requirement Review and Requirement Status correction to the actual implementation boundary. For unresolved Requirement Status, choose the relevant prerequisite or leave diagnosis to the harness; the tool does not invent the substantive fix.

For a concrete definition, a completed result can include one additional supporting record of kind `workflow-definition`. That file must retain the definition identity and every existing task unchanged in the same prefix, and may append validated tasks for the discovered module scope. The new definition and its fingerprint are recorded atomically with the result. This advanced concrete-graph facility does not infer modules from directory scanning or rewrite previous work. A pinned template rejects such extensions; its assurance stages expand through automatic candidate target derivation only.

Inputs are `{kind, ref}` for workflow-local files, or `{kind, repository, ref}` for repository files. `ref` is always a normal relative file path. The local repository file supplies checkout locations; the instance directory supplies the location of workflow-local files. Missing inputs block the corresponding task, not the whole initial authoring graph. Handoffs materialize exact `{kind, repository, ref, fingerprint}` references. The published `inputs` list contains configured inputs followed by direct prerequisite artifact references, in declared order, with identical references deduplicated. Conflicting references to the same kind/repository/path fail closed. Prerequisite result envelopes, outcomes and author identities remain internal for dependency gates and independence validation; no `dependencies` field is published. Keep historical review/report files immutable; changed current artifacts make dependent results stale without erasing their history.

## Handoff and result

A handoff includes `id`, `action` (the definition action ID), `task` (the semantic profile), `target`, `repositories`, `workflow_directory`, `basis`, `reason`, `guidance`, `inputs`, `corrections` and `required_output`. The outer response contains format/version, instance/revision, definition fingerprint, `eligible` and `required_inputs`. All eligible tasks have complete handoffs; their order does not express execution priority. Definitions describe only that configured graph, not every project obligation. Each eligible handoff carries validated `repositories` (repository IDs mapped to absolute checkout paths) and `workflow_directory` (the canonical instance directory). A worker receiving that handoff alone can resolve both kinds of reference. These are local execution locations, not portable metadata, additional permissions or semantic identity; they are excluded from task-basis fingerprints. The harness must regenerate a handoff on the execution machine rather than reuse another machine's paths.

Workers author one result per attempt:

```json
{
  "work": "author-spec-delta",
  "target": {"module": "example/boundary"},
  "outcome": "completed",
  "report": "Specified service admission and rejection behavior.",
  "outputs": [
    {"kind": "intent-delta", "repository": "application", "ref": "azimuth/changes/example/deltas/example/boundary/spec.md"}
  ]
}
```

`work`, `outcome` and `outputs` are required. `target` and nonempty prose `report` are optional. The work and supplied target locators must select exactly one eligible entry in the captured handoff; omission is valid only when unambiguous. References contain `kind`, `ref`, optional `repository` and optional `record`. Without a repository, references resolve beneath the workflow directory. With `record`, the reference selects a local ID from an `azimuth-review-bundle`; the tool validates the complete bundle and tracks that entry's native review, input and provenance. Appending unrelated records or changing derived assessment status does not invalidate that entry. No worker-authored fingerprints, result IDs, actor labels or dependency arrays are accepted.

```text
azimuth workflow next --instance .azimuth/workflow.json --project azimuth/project.json --out .azimuth/handoff.json
azimuth workflow update --instance .azimuth/workflow.json --project azimuth/project.json \
  --handoff .azimuth/handoff.json --actor independent-reviewer \
  --result .azimuth/results/spec-review-1.json --expected-revision 3
```

`--handoff` must retain the query captured before work began. It supplies the tool-owned exact basis; querying again after work is not a substitute for that capture. A newer coordination revision may be supplied after another independent task completes, but the original basis must still match. Preserve the captured handoff for each in-flight worker; overwrite or remove it after update succeeds. `--actor` is a harness assertion, not authenticated identity or permission.

Azimuth retains an internal result with generated attempt identity, exact basis, captured input references, actor, output fingerprints and the authored result reference. Authoring review and approval decisions are retained with their exact dependencies inside that internal result. Unchanged replay remains idempotent; modifying the same attempt or its supporting content fails. Historical results already committed to an instance retain their original schema, identities and fingerprints; they are not reauthored. The internal representation is not the authoring interface.

The result itself serves as the report for implementation, failed work, independent authoring reviews and human approval when no separate substantive output is supplied. These tasks require nonempty `report`; rejection describes the findings there. `outputs: []` is valid for such tasks. Other tasks require their substantive output, such as the intent artifact, native validation report, candidate snapshot or Run bundle. Candidate preparation may supply `candidate-authority`; core validates the authority and derives the preparation receipt internally. Do not author a duplicate receipt. Reviews and Runs retain their own stricter contracts and exact identities. Completed execution can retain violated or inconclusive Check Results without manufacturing positive assurance.

| Task class | Outcomes |
| --- | --- |
| Generic authoring/implementation/capture/validation/registration/execution | `completed`, `failed` |
| Authoring review, Requirement Review | `accepted`, `rejected` |
| Requirement Status | `supported`, `violated`, `unresolved` |
| Implementation approval | `approved`, `denied`, `deferred` |

Protocol-valid adverse results are recorded normally. Structural mismatch, invalid basis, conflicting revision or malformed record fails without a transition. Currentness rechecks actual supporting content and dependency semantics, not manually authored completion flags.

## Independent authoring reviews and human approval

An independent authoring reviewer can submit the compact result with `outcome: accepted | rejected`, nonempty rationale/findings in `report` and `outputs: []`. Core derives the review's work reference, reviewer assertion and exact dependencies from the captured handoff and validates independence against prerequisite authors. A separate substantive `authoring-review` output is optional; its native contract remains `format`, `version`, `task`, `reviewer`, `outcome`, `dependencies`, `findings`, `rationale`. Its task/actor/outcome and dependencies must match. Rejection requires findings. Actor identities remain local harness assertions; neither the file nor core establishes authenticated human independence.
Review the proposal against the original request, declared scope, exclusions, singular ownership, repository participation, unresolved assumptions and readiness to author module contracts. Proposal review requires a reviewer distinct from the proposal author and accepted current review before spec authoring; rejection reopens proposal authoring followed by independent reassessment. Proposal review does not grant implementation permission. Review intent against the original request and boundaries; review design against every Scenario and relevant architecture; review verification for coverage, credible observations, independent oracles, controls and limitations. Acceptance is distinct from parsing and from artifact authoring. Rejected reviews expose author correction and independent reassessment. Changing a pinned dependency invalidates current acceptance; the historical review remains.

Implementation approval can use the compact result itself, with the actual human decision and its rationale in `report`. Core captures the exact proposal-package dependencies and actor assertion. An optional separate native approval retains `format`, `version`, `task`, `actor`, `outcome`, `dependencies`, `rationale`. Only `approved` satisfies the gate. The harness must obtain the real human decision and enforce authority; a local approval label never grants permission.

## ASS integration and assurance assessments

Local development prepares the candidate and conducts Requirement Reviews without ASS connectivity. Author aggregate Markdown records, prepare exact inputs and compile them against the complete candidate. Requirement Review requires definitions/source support; Historical authored Requirement Status additionally requires actual exact-target Runs and a current accepted Requirement Review. Current deployment Requirement Status is computed after rollout, as described below. The native schema and compilation commands are in `contracts/assurance-reviews.md`. Workflow assessment hydrates retained prerequisite Runs and reviews into the candidate snapshot; no manually rewritten snapshot per task is required.

Each assurance task supplies a snapshot with exactly `{project, model_revision, authority, reviews, bundles}`. Core validates schemas and currentness, matches the exact Requirement and (for Requirement Status) Verification Target, and assesses the submitted native record against actual evidence. Ambiguous latest records, stale dependencies and relevant Verification Challenge findings prevent unsupported reliance. Historical review kinds retain their original meaning and satisfy no current Review/Status gate.

ASS onboarding and credentials are publication prerequisites. CI transports the relied-on local reviews and relevant Runs; it need not spend model tokens conducting reviews. Preserve original reviewer/model provenance and adverse records. Each review submission still requires a credential authorized for its original reviewer; producer credentials cannot impersonate reviewers. Candidate registration never selects accepted authority. Authorized hosted Runs and subsequent Requirement Status assessments follow deployment; source/artifact evidence is not implicitly transferred to a deployment.

`azimuth assurance verify-snapshot --input snapshot.json --out validation.json` validates a publication packet without service mutation. A valid packet may contain rejected, stale or unresolved assessments; validation is not acceptance. `scripts/publish-assurance.py` transports the validated packet with separately authorized owner, producer and reviewer credentials and retains partial or complete submission receipts. Do not publish all local debugging history.

At an explicit local acceptance boundary, use `azimuth change validate --scope acceptance --assurance-snapshot snapshot.json`. After authored facets are actually applied and completion obligations satisfied, `change finalize` and `change archive` accept the same explicit scope and snapshot. Each reassesses native Review/Status against the complete current model and actual retained Runs; snapshot presence is not approval. Missing or adverse assessments and unrelated findings remain blocking. These commands do not publish to ASS, select accepted authority or extend the standard workflow with automatic archive/deployment work.

## Retention and limits

State embeds append-only result records and retains exact references to supporting content. Locks serialize writers; hashes detect input drift during update. This is not an adversarial filesystem sandbox or a multi-host transactional scheduler. A coordinator crash may require inspecting a lock or replaying a result; an agent is not automatically relaunched.

No automatic source-directory inference, `.azimuth` exclusion, source capture, directory relocation, service transport, deployment, full-federation finalization, accepted-revision promotion gate, archival or production scheduling is claimed. Keep operational data outside examined source roots until capture ownership is explicit. Before deleting a local working directory, retain every required referenced record in durable storage. Runtime history does not turn missing source/hosted evidence into support.

## Local review artifacts

Keep Requirement Reviews in `reviews/requirement-reviews.md` and exact-target assessments in `reviews/requirement-status.md`. Generated inputs, compiled bundles and scope reports live under ignored `.azimuth/`. Select a compiled record with `record` in the compact result. Preserve exact dependencies and explicit reassessment relationships; appending an unrelated review does not invalidate an existing selected entry.

Existing instances retain their pinned definitions and immutable histories. Adopting revision 6 requires an explicit migration or a fresh instance; upgrading or querying never replays work. Change-wide work awaits the validated candidate inventory. Missing Verification Target selections remain visible and defer publication. An explicitly empty current collection can satisfy its `all` gate; missing inventory is not an empty collection.
## Local working storage

Keep authored task results under `.azimuth/results/`, useful generated artifacts under `.azimuth/artifacts/`, and exact assurance review inputs under `.azimuth/review-inputs/`. Normative change artifacts and authored assurance review bundles remain outside this ignored working directory. One result per attempt is enough: a separate work report, workflow-result wrapper and update receipt are not required. Query handoffs are temporary in-flight inputs, not a permanent transcript. The instance already retains the exact coordination history.

Clean only unreferenced scratch exports, command transcripts, duplicate response captures and superseded diagnostics. Preserve every supporting file referenced by immutable history, current declared input, retained review dependency or Run, and preserve the complete candidate needed for assessment. Historical supporting artifacts cannot be silently discarded merely because they no longer satisfy current work. Back up the workspace before cleanup; historical dependency closure takes precedence over a target file count.

## Acting on correction feedback

`eligible[].corrections` gives reviewed work/outcome, reviewer, the reviewed model revision when present, direct authored review sections and reviewed artifacts. Read the section using its `directory`, `ref`, and `line`; its digest identifies the exact retained review section. If source resolution fails, read the inline retained feedback and diagnostic. Compiled bundles and internal result wrappers remain tooling/audit records, not required reading for ordinary correction work.

Currentness applies only to retained input references; unchanged historical captures do not establish that current checkouts still match them. Inspect current source and artifacts before applying feedback. Assess each finding, update the existing engineering plan as needed, and implement bounded corrections. Use the normal compact result's `report` to identify each review/finding and record its changes and actual verification, justified disagreement, or unresolved limitation. Independent reassessment follows; correction work does not accept its own review. Obtain the relevant decision for material scope or intent changes. No extra correction-plan task or duplicate report file is required.

For revision-7 Verification Challenge work, select one eligible `run-verification-challenge` target. Read its declaration,
readable reviews and scope/configuration inputs. Retain a fresh-attempt readable `verification-challenge-report`
and native `run-bundle`; include the finding disposition/limitations in the work report. Execution
completion is distinct from the derived assessment: native clean means bounded no-findings, never
Requirement Status. Findings require bounded correction and independent reassessment; execution failure
or inconclusive search requires environment/configuration/scope diagnosis and retry. Preserve
surviving-mutant equivalence/relevance questions and findings alongside any execution gaps.

For findings with unchanged candidate/source/configuration, an independently accepted successor
Requirement Review can retain the exact Run/result/target `challenge_basis` proofs and justify the
disposition. Core may then close the original bounded search without relabeling its findings.
Execution gaps remain retry obligations. Changed source or scope requires fresh candidate work
and affected reruns. Captured parallel Verification Challenge results remain recordable after a sibling
finding only while their exact inputs and prerequisite basis remain unchanged.

For a selected compiled review entry, workflow assessment retains its exact reassessment ancestor chain from the validated bundle as historical context. Unselected sibling reviews are not imported as current prerequisites. Missing predecessors remain missing dependencies; retained ancestors never confer current acceptance.

## Requirement Review guidance (change-development revision 8)

`review-requirement` handoffs reference the shipped `azimuth-review-requirement` skill. Acceptance requires
both adequate verification design and faithful implemented verification, including the observations
and controls promised by each contribution. Execution may remain pending. Documented unfinished
implementation does not satisfy the gate. Existing workflow instances retain their pinned guidance
and definitions until an explicit migration; installing this skill does not rewrite review history.

## Offline publication handoff

The standard workflow's final task is prepare-publication-inputs. Its publication-inputs output is the compact native envelope validated by `azimuth release inputs`. Local preparation never connects to ASS or asserts registration. Commit the envelope in the explicitly configured release repository. Trusted CI publishes the release/candidate and retains the actual receipt. Explicit custom registration tasks remain service operations; preparation is never a registration receipt. Existing pinned workflow instances retain their definition and history; start a new instance to adopt the changed standard template rather than silently rewriting completed tasks.


## Deployment Requirement Status workflow (revision 10)

The shipped change-development revision 10 prepares local publication inputs after current Requirement Reviews, source/artifact Check executions, candidate preparation and declared Verification Challenges. It does not schedule an authored Requirement Status assessment. Historical pinned workflows and their authored assessment records retain their original requirements; earlier revision descriptions above describe those contracts.

The candidate authority's Check deployment policy determines execution scope. Checks whose execution verification target is `deployment` (including Monitors) execute after rollout and are excluded from the local execution inventory. Source and artifact Checks still require explicit exact Verification Target selection and actual Run facts before publication. A missing local Check Verification Target blocks publication preparation. A hosted-only inventory does not require an invented local execution or positive assessment.

CI publishes unchanged accepted Requirement Reviews, native Runs and their effective-context captures with the composed release. After deployment reconciliation establishes the observed composition and current process instances, ASS computes Requirement Status from applicable evidence, provenance and effective-context compatibility. Missing applicability or evidence remains unresolved; passing CI is not automatic deployment confirmation. No local workflow action accepts a composed release or authorizes deployment. Owner acceptance and archival remain separate explicit decisions.
