---
name: azimuth-propose
description: Create or revise one bounded, approval-ready Azimuth change from a clear request or approved exploration. Use to establish singular authority, author current intent deltas, decide solution boundaries and validate a comprehensive proposal before implementation.
---

# Propose one change

A change proposal defines what should change in the product, why, and what would justify accepting it—including proposed intent, scope, evidence requirements, design decisions, and delivery plan. Use the term “change proposal” consistently.

Create the smallest transition that can be reviewed, implemented and accepted independently. Stop after explicit approval of the actual proposal files unless implementation is separately authorized.

## Establish readiness

1. Read the target repository's applicable instructions and change guidance.
2. Confirm the request is sufficiently decided. Require `Status: approved` when carrying an exploration and cite only decisions assigned to this change.
3. Locate singular change authority in a federated project.
4. Inspect current accepted intent, relevant archived decisions, implementation, data, interfaces and permitted engineering checks.
5. Run the repository's normal `azimuth validate` invocation and record pre-existing Findings honestly.

## Author

1. Run `azimuth change list`, choose a stable lower-kebab id and run `azimuth change create <id> --title "<title>"`.
2. Run `azimuth reference show proposal`, `azimuth reference show intent-delta` and any other reference needed for the artifacts being authored.
3. Define the present problem, observable outcome, in-scope and excluded work, exact affected Requirement identities, originating exploration decisions and inspectable completion conditions.
4. Add intent deltas only for observable obligations. Do not invent operations omitted by the installed reference.
5. Add `design.md` when implementation would otherwise decide identity, authority, data ownership, compatibility, migration, failure, security, operational or rollback semantics.
6. Write a dependency-ordered `plan.md`. Add `work-packages.md` only for independently implementable, non-overlapping path ownership.
7. Keep `proposal.md` at `Status: proposed` throughout authoring.

For an intent delta, author only operations supported by `azimuth reference show intent-delta`.
Run `azimuth change intent-capture <id>` for replacements and removals so the reviewed delta pins
the accepted blocks it changes. A Requirement addition uses:

```markdown
# Intent delta: <spec-id>

## Add requirement: <requirement-id>
Criticality: routine

Non-empty free-form normative Markdown stating this Requirement.

### Add scenario: <scenario-id>
Non-empty free-form normative Markdown stating this Scenario.
```

Requirement and Scenario bodies may use any human language, EARS, unrestricted prose, tables, diagrams or
code fences. Core preserves and fingerprints their Markdown but does not interpret keywords,
translations or notation. Reserve the top three structural headings for Spec, Requirement or Invariant,
and Scenario declarations; use level-four headings or fenced content inside a body. Keep orientation,
rationale and generated duplicate views outside normative bodies. Use lower-kebab declarative ids,
put universal meaning in the Requirement, make each Scenario stand alone, and never author test mechanics as
intent. The installed `intent-delta` reference remains the complete release-matched operation set.

Run `azimuth change intent-preview <id> --out <file>` and review the exact target spec and diff
with the proposal. Keep the preview as a reviewable artifact of the change. For edits to an
existing spec, inspect preserved surrounding Requirements, explicitly removed Scenarios and affected
references. Preview states a target, not completed or accepted behavior.

## Author candidate Requirement-first design

When the project uses the candidate Requirement-first design format, a selected Mechanism may sit directly under its Requirement design at level three. Prefer this placement before Scenario designs when the control contributes to several Scenarios. Declare its stable project-wide lower-kebab ID and optional explicit Scenarios metadata using stable Scenario IDs. Module membership and Scenario ownership do not determine identity. Nesting and order do not establish applicability or evidence. Elements still require a Section or Scenario design owner. Consult contracts/account-design.md for structural grammar.

Use ordinary Markdown captions for candidate verification prose groups; do not declare a Section identity just to organize Checks. Keep each Check one heading level below its structural owner or an intervening caption, through level six. Caption ancestry preserves Requirement/Scenario ownership; a sibling caption closes prior scope at its level. Retain explicit Sections and Elements where named support is required. Captions and order supply no evidence.

Declare Requirements, Scenarios, Mechanisms and Checks with stable project-wide lower-kebab IDs, unique within their entity kind. Preserve those IDs when splitting or merging modules or reparenting a Scenario. Module headers and Scenario nesting author mutable relationships only. Source APIs carry one stable target ID; contextual locators explicitly retain project, entity kind, ID and optional revision. Do not invent module-derived IDs, aliases, UUID requirements or cross-kind uniqueness.

For candidate verification Checks, Mechanism references and inline Scenario bindings refer directly to stable project-wide IDs. Validate target existence and duplicates without deriving identities from the document module or enclosing Requirement. A dependency does not supply evidence or qualification.

## Review authored intent

Before validation, read each proposed Requirement and its Scenarios as an independent reviewer. Apply these semantic criteria alongside the installed format reference:

- Give each Requirement one independently governable, falsifiable proposition. Name its lower-kebab id as a compressed declaration of what is true, not as a topic noun or an implementation step. Split Requirements when separate obligations could succeed, fail, or need different criticality independently.
- Put the general domain and outcome in the Requirement. Keep implementation mechanisms, entity-field decisions, test procedures, and rationale in design or plan unless they are themselves the externally observable obligation.
- Give each Scenario a stand-alone declarative id and a concrete condition and outcome within its parent Requirement. Include meaningful failure or boundary conditions. A Scenario that merely paraphrases the Requirement, bundles several unrelated situations, or has no observable result needs revision.
- Prefer a Given/When/Then shape when it clarifies a Scenario: state the relevant starting condition, the request or event, and the observable result. Name a response status or other wire detail when it is a deliberate contract, and state the protected effect or absence of one on a denial. Split materially different conditions or outcomes into separate Scenarios instead of hiding them in one broad sentence. Keep test setup, probes and observation methods in verification.
- Check Requirement-to-Scenario composition: Scenarios sample the Requirement without narrowing its universal meaning to one example or pretending that one result proves the whole Requirement. Given/When/Then is authoring guidance, not parser syntax or a required template; use equivalent prose when it is clearer.

Structural validation cannot perform this semantic review. Resolve weak or ambiguous wording before requesting approval, even if `azimuth change validate` passes.

## Validate and hand off

Run `azimuth change validate <id>`, `azimuth change work-packages <id>` when present, the repository's normal `azimuth validate`, and `azimuth change show <id>`. Inspect the working diff and ask whether an implementer can proceed without inventing product behavior, ownership, compatibility, migration, failure or completion decisions. Present the actual files for explicit approval. Do not implement, finalize, archive or commit implicitly.

## Author compact verification packages

Use compact Check method prose and inline Evidence bindings when the project adopts that account. Keep the Contribution narrower than the whole Scenario where the observation is narrower. Inherit Scenario only through its structural Scenario owner; use an explicit Scenario for shared Checks. Do not synthesize qualification, applicability or judgments from implementation tags.

Surface and network package declarations follow the installed verification reference. Enable packages explicitly, publish field names without duplicating language types, link a source producer through its package marker, and pin independently authored expectation data. State discovery completeness, controls and inconclusive conditions in module prose. Keep package mechanics in the package reference rather than repeating them in every module. Ordinary captions have no IDs.

## Explicit workflow handoffs

When a workflow instance is configured, use `azimuth workflow next --instance <file>` or `--change <directory>` and read `azimuth reference show workflow`. Select work from `eligible`; the harness owns scheduling and may execute eligible work in parallel with appropriate isolation. Read its full guidance and pinned inputs. Perform the work, retain its actual records, then call `workflow update` with the result and expected revision. Querying never advances the instance. Do not infer completion, approval or independent assurance from file presence.

Review the proposal independently before authoring module contracts: assess the request, scope, ownership, exclusions, assumptions and readiness. Accepted proposal review permits further authoring, not implementation. Delegate required authoring reviews to another agent or an independent human when the runtime permits it. The reviewer reads the original request, artifact and dependencies directly, records exact input references and findings, and remains distinct from the recorded author. Rejected reviews require correction and reassessment; changed dependencies require fresh assessment. Local actor labels express harness assertions rather than authenticated identity. Service review ingestion must match the actual reviewer credential.

Use Requirement Review and deployment Requirement Status contracts for concrete assurance. Retain unchanged reviewer records in a complete local candidate snapshot. Local authoring and review do not require AAS. Finish with the compact Git-trackable publication-inputs envelope validated by `azimuth release inputs`; authorized CI later registers the candidate alongside its named release and retains the actual AAS receipt. Registering a candidate never ratifies the model (ratification is the merge to main) or authorizes deployment. Agents may run locally while CI supplies Runs. Never manufacture reviewed-at times, independent reviewer decisions, Run facts or permission records to satisfy a workflow gate.

## Standard change workflow selection

Select `azimuth workflow init --workflow change-development --inputs <file> --instance <file> --repositories <file>` for ordinary change coordination. Inspect `azimuth workflow definition --workflow change-development` and `azimuth reference show workflow` for the exact input and inventory contracts. Supply explicit project/change identities, original request, optional referenced task instructions and declared module inventory; do not manually compose a process DAG or invent context-specific action names. Instruction overrides can direct retained-artifact copying while preserving approval and independent review gates.

Read `workflow next` using the same explicit repository map (or selected `--project`); handoffs carry portable references plus complete local execution locations. Act on `action` and typed `target`, and author one compact result (`work`, optional `target`, `outcome`, optional `report`, `outputs`). Pass the captured handoff, actual actor and current revision to `workflow update`; Azimuth derives identities, fingerprints and dependencies. `--actor` names one execution instance — a session, a subagent run or a person — never a role, attempt number or model; those are carried by the action, the attempt id and the descriptor. When the harness or orchestrator can state the executing agent, also pass `--descriptor <file>` (an `azimuth-agent-descriptor` with kind, model, effort and harness, each marked `harness`, `declared` or `unknown`); never guess a value, and omit the flag rather than invent one. Read `azimuth reference show work-provenance`. Reuse the result as the report instead of writing duplicate wrapper files. The harness chooses workers and obtains real permission. A local approval label grants no permission itself.

After structural validation, prepare the complete local candidate authority and snapshot. Azimuth derives targets automatically. Independently review each non-routine Requirement definition, Scenarios, realization and verification through Requirement Review; no execution Verification Target belongs to that review. Execute Checks against explicitly selected exact Verification Targets, and run declared machine Verification Challenges against exact current assessments with resolved source scope. Publish actual Runs and effective context captures. The deployment assessor computes Requirement Status from current Requirement Review, exact release lineage, observed deployment composition, applicable evidence and fresh monitoring; do not author a separate positive status record. Different Verification Targets are joined only through those validated applicability relations. Keep local iterations local; publish the relied-on review and execution package from CI or another authorized boundary. Preserve adverse facts and reassessments; clean Challenge Results never establish support.
