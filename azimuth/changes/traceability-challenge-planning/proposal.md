# Change: traceability-challenge-planning

Status: proposed

Exploration: evidence-control-plane-alpha-2
Carries decisions: E5, E6, E8, E9
Depends on changes: verification-evidence-bindings, adapter-capability-protocol

## Problem

D45 can resolve authored Challenge Plans only to current Qualification fingerprints and deliberately
leaves Claim Judgment selectors unresolved. D47 can transport hand-authored Challenge selections,
but the current planner emits `challenges: []`. There is no current Claim Judgment authority,
canonical Challenger fingerprint, required-form coverage check or model-to-Run projection.

An opaque decision fingerprint is also insufficient input for a real mutation, fault-injection or
static-analysis adapter. Without a frozen traceability-derived scope, an adapter would have to load
the Azimuth model independently or depend on a second path/glob map. Either choice would move
semantic authority out of core and recreate the ambiguity this change is intended to remove.

## Outcome

Azimuth authors one total-composition Claim Judgment for every applicable non-routine case Claim,
resolves all seven Challenge Plan selector forms to exact current accepted decision fingerprints,
and expands requested plans into provider-neutral Run Challenge selections. Each selection carries
canonical semantic scope and one launch route carries the corresponding accountable source
locators for a configured capability with the exact Challenger form.

Qualification and Claim Judgment policies require open challenge forms. A separate `gate |
scheduled` lane classifies when those forms are expected without changing decision fingerprints.
Missing, stale, rejected, inapplicable or deferred work stays visible; clean results remain negative
search facts and never become product evidence. Durable result application and Assurance State
remain Change G work.

## Scope

In scope:

- add strict Claim Judgment blocks to `verification.md` without restoring alpha 1 judgment files;
- define Claim Judgment, Challenger and semantic challenge-scope fingerprints, including each
  Challenger's required closed semantic scope kinds;
- make one current accepted Claim Judgment the total-composition decision for each applicable
  standard or critical case Claim;
- include Claim semantics and criticality, realization, mechanism, surface, binding,
  Qualification, policy, basis and residual-risk inputs in Claim Judgment identity;
- resolve every Qualification and Claim Judgment selector with explicit selected, missing, stale,
  rejected, invalid, inapplicable or unresolved disposition;
- require every policy form to have an authored Challenger and Challenge Plan that resolves the
  decision with the Challenger's required semantic scope, while allowing strengthening forms;
- classify required forms in one project scheduling account as `gate` or `scheduled` without
  putting scheduling into the Qualification or Claim Judgment fingerprint;
- extend Run planning requests with exact Challenge Plan ids, explicit configured capabilities,
  finite work units and a fail-closed target cap;
- permit Check-only, Challenge-only and mixed Run plans while preserving one complete-model
  fingerprint, one Subject, one exact context and one configured adapter;
- project canonical selector anchors and decision dependencies into each semantic Challenge scope;
- project matching source locators into launch routes so adapters can derive native selectors
  without loading the repository model;
- map the Challenger's exact form to `challenge.execute | challenge.import` capability coverage;
- expose pure Qualification-to-Claim-Judgment impact edges for the later ledger; and
- conformance-test mutation, qualification-oriented fault injection and broad static analysis.

Out of scope:

- executing or importing `model.extract` capabilities;
- provider-specific mutation, fault, analyzer or CI configuration in core;
- automatic inference of changed files, risk, budget, native shards or provider selectors;
- cache TTLs, cross-Subject result reuse, cadence, historical applicability or time-based deferral;
- durable Run ingestion, authorization, retention, revocation or Assurance State;
- applying a Challenge Result to repository-authored Qualification or Claim Judgment text;
- manufacturing a Claim Judgment Challenge Result from an upstream Qualification result;
- long-running adapters, daemons, webhooks or provider event hosting;
- production provider adapters or package publication;
- raising any current framework Claim above routine; and
- any alpha 1 judgment parser, result importer, alias or compatibility reader.

## Affected claims

Add seven routine requirements under `framework/traceability-challenge-planning`:

- Claim Judgments bind one exact total assurance composition;
- Challenge resolution preserves every candidate disposition;
- generated Run plans bind exact current accepted decisions and context;
- adapters receive frozen semantic scope and accountable launch inputs;
- challenge outcome, deferral and selection-mismatch meanings remain separate;
- required forms and scheduling lanes remain visible without false success; and
- Qualification impact reaches dependent Judgments without duplicate Challenge Results.

The requirements are routine and owe no Check, Evidence Binding, Qualification or Claim Judgment.
All format, parser, planner, adapter and conformance tests are ordinary engineering tests over
synthetic non-routine models.

## Completion conditions

- `verification.md` accepts one strict Claim Judgment block per case Claim with `accepted |
  rejected` verdict, exact policy, fingerprint, date, judge, repeatable basis, repeatable residual
  risk and rationale. Routine targets, duplicate ids, dangling Claims and unknown fields fail.
- Every standard or critical case Claim has exactly one current Claim Judgment. A missing,
  rejected, stale or structurally invalid Judgment is a Finding; routine Claims reject it.
- Claim Judgment identity changes with Claim semantics or criticality, applicable realization and
  mechanism inputs, surfaces and area obligations, Evidence Bindings, Qualification decisions,
  policy, verdict, basis or residual risk. Paths, lines, dates, judge and explanatory prose do not
  affect it.
- Challenger identity is canonical over id, form, objection proposition and sorted required scope
  kinds. It excludes paths, lines and rationale.
- Selector resolution reports every candidate disposition. One selected sibling cannot hide a
  missing, stale, rejected, invalid, inapplicable or unresolved candidate, and zero resolution
  never widens to a suite, path or glob.
- Only current `qualified` Qualifications and current `accepted` Claim Judgments become executable
  targets. A rejected decision stays a visible repository Finding rather than something a clean
  Challenge repairs.
- Binding and Check selectors resolve Qualifications directly. Realization and mechanism
  Qualification traversal honors the binding challenge domain. Claim Judgment traversal reaches
  related Claims without consulting binding challenge domains.
- A Plan covers a required form only when its resolved scope includes every closed semantic kind
  required by that Challenger. Core never infers required kinds from an open form name.
- Overlapping selectors and requested plans deduplicate by exact
  `(Challenger fingerprint, target kind, target fingerprint)`. A Qualification result may impact
  one dependent Judgment but never creates a second direct Judgment Challenge Result.
- One in-place `Decision Policy` namespace replaces the pre-F Qualification-policy name. Evidence
  Bindings and Claim Judgments both use `Policy:`; each policy names required open forms. Every
  accepted current decision has a current Challenger and authored Plan resolving it with required
  scope for every required form. Additional forms are permitted as strengthening.
- One strict scheduling account in `azimuth/standards/verification.md` assigns every required form
  to exactly one `gate | scheduled` lane. Changing the lane alters model and Run-plan identity but
  does not stale a Qualification or Claim Judgment. F defines no cache or temporal reuse.
- Planning resolves the fixed union of requested Plans. For every selected decision, that union
  must contain at least one runnable `(target, required form)` selection for each policy form;
  otherwise planning fails. Each request names its explicit capability and `max_candidates` cap.
  Every semantic Challenge records its lane. An adapter may defer work only through an incomplete
  Run with a selection diagnostic and no result for the omitted selection.
- The strict Run planning request accepts sorted Challenge Plan requests with explicit capability,
  units and `max_candidates`. It allows Check-only, Challenge-only and mixed plans, but their
  combined semantic selection is non-empty.
- Planning loads the complete unselected model, derives every Challenger and decision fingerprint,
  requires exact Qualification context equality with the Run context and rejects mixed target
  contexts rather than broadening them.
- The request supplies an explicit capability address. Core verifies operation class, exact open
  form and the one-adapter-per-Run boundary; it never picks a capability lexically or trusts a
  caller-supplied form.
- `max_candidates` counts the unique candidate records reached by one requested Plan, before
  cross-plan deduplication and regardless of disposition. Exceeding it fails before launch rather
  than truncating. A target-derived stable Challenge id is independent of selector order or which
  overlapping plan reached it first.
- Every Challenge selection carries sorted, unique selector anchors and semantic decision inputs.
  Each item has a closed kind, stable id and canonical fingerprint. Matching launch inputs carry
  stable source identities, fingerprints and accountable locators. The Plan fingerprint changes
  with semantic scope; the launch fingerprint changes with locator projection.
- Mutation scope contains the selected product realization and bound Check implementations;
  mechanism-oriented fault scope contains the exact mechanism and bound Check implementations;
  broad static-analysis scope contains the exact selected Claim composition. No provider path or
  glob becomes semantic authority.
- `clean`, `findings` and `inconclusive` retain their D46 meanings. `deferred` is not a Challenge
  Result and creates no fact. A planned target omitted from an allowed incomplete Run has one
  execution diagnostic scoped to its planned Challenge-selection id and remains outstanding; a
  complete omission or added/substituted target, context, units or scope is a selection mismatch
  and produces no published bundle.
- A Check source-only change recomputes the dependent expected Qualification fingerprint and
  therefore stales the dependent Claim Judgment, while unrelated Judgments remain current. The
  Judgment fingerprint never trusts a stale authored Qualification fingerprint as composition.
- A pure impact projection maps a Qualification fingerprint to its owning Claim and current Claim
  Judgment, and maps a direct Claim Judgment target to its Claim. It does not mutate repository
  decisions, ingest Runs or derive Assurance State.
- Synthetic conformance proves relevant mutation selection, dual-role fault execution, broad
  static analysis, missing traceability, context mismatch, required-form coverage, target caps,
  clean/findings/inconclusive outcomes, partial deferral and actual-selection drift.
- Mutation conformance maps survivors to findings, all planned mutants killed to clean, and zero
  mutants, no tests or tool skip to inconclusive. Static-analysis conformance maps warnings to
  findings and unsupported or skipped scope to inconclusive.
- One activated fault activity produces a direct Check Observation and a distinct Qualification
  Challenge Result without deriving either from the other. A claim-specific analyzer enrolled as
  a Check produces an Observation, while broad analysis remains a Claim Judgment Challenger.
- All current framework requirements remain routine, the canonical model has no current
  verification declarations, and complete Rust, conformance, isolation and composed-model gates
  pass.
