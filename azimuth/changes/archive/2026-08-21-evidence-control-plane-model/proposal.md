# Change: evidence-control-plane-model

Status: accepted and complete

Exploration: evidence-control-plane-alpha-2
Carries decisions: E1, E2, E3, E4, E5, E6, E7, E8, E9, E10, E12, E13, E15

## Problem

Azimuth currently has evidence sites, generic assurance observations, reusable evidence
qualification, challenge bindings and lifecycle gates, but the concepts were introduced by
separate changes and do not compose into one public execution model. In particular, current
coverage linkage conflates a verification method's implementation with the meaning of its result,
an observation is both an execution and a claim binding, and mutation or broad analysis attacks a
judgment without an exact first-class Qualification target.

The ambiguity becomes blocking when local tests, CI, fault injection and monitoring all need to
enter the same control plane. Implementing commands or provider adapters before fixing the semantic
authority would freeze different interpretations into each boundary.

## Outcome

Azimuth has one explicit public conceptual model:

- requirement-level Claims own normative propositions and criticality;
- case-level Claims refine observable conditions;
- deliberately enrolled Checks execute within Runs and produce Observations;
- Evidence Bindings relate one Check outcome to one Claim aspect;
- each binding has its own Qualification;
- Claim Judgments assess whole-Claim assurance composition;
- Challengers target exact Qualification or Claim Judgment fingerprints and produce Challenge
  Results;
- Assurance State is derived for exact Subjects from execution facts and repository decisions;
- provider adapters translate bounded plans but do not interpret the repository model.

The change adds only routine intent and current terminology. Dependent changes own syntax, command
execution, adapter protocol and persistence implementation.

## Scope

In scope:

- add authoritative decisions for the evidence-control-plane model and explicitly revise the
  superseded portions of D38–D42;
- add bounded glossary definitions and a derived framework account;
- add a routine current model package for the new semantic obligations;
- lower every active standard or critical framework requirement to routine for the fast-moving
  alpha period;
- remove the current verification and judgment facets whose obligations disappear with that
  lowering while retaining useful design knowledge;
- establish identity, ownership, cardinality and the non-recursive Check/Challenger boundary;
- distinguish repository decisions from Run facts and storage policy from assurance policy; and
- name the follow-on format, command, adapter and ledger boundaries without implementing them.

Out of scope:

- changing `verification.md` syntax or extractor linkage;
- renaming or implementing CLI commands;
- defining Run JSON, adapter transport or capability configuration;
- changing Assurance Service storage or APIs;
- retaining alpha 1 compatibility aliases or readers;
- assigning non-routine criticality, evidence, Qualifications or Claim Judgments to this
  repository's new requirements; and
- releasing alpha 2.

## Affected claims

Add six routine requirements under `framework/evidence-control-plane`:

- `claims-separate-normative-and-observable-intent`;
- `checks-bind-atomic-outcomes-to-claims`;
- `qualifications-belong-to-evidence-bindings`;
- `runs-contain-subject-bound-execution`;
- `challenges-target-semantic-decisions`; and
- `core-owns-semantics-and-adapters-own-providers`.

Routine criticality is deliberate for the fast-moving alpha. These Claims require no Covers
relations or agent judgments until a later accepted change raises their consequences.

Lower the nine active standard or critical requirements under `framework/assurance-deployment`,
`framework/release-artifacts` and `framework/release-orchestration` to routine. Revisit each level
after the codebase stabilizes and its real consequence warrants evidence and judgment obligations.

## Completion conditions

- The six routine Claims and their scenarios parse as one intent delta.
- Every active framework requirement has routine criticality.
- One authoritative decision defines every public concept, relation and ownership boundary without
  presenting dependent implementation as current behavior.
- D38–D42 retain their historical reasoning and carry explicit revision notes where alpha 2
  changes their active interpretation.
- The glossary and framework use Check, Evidence Binding, Qualification, Run, Observation,
  Challenger, Challenge Result, Claim Judgment and Assurance State consistently.
- The account states that a Run contains Check and Challenger executions; a Check never emits a
  Run.
- Mutation, broad static analysis and qualification-oriented fault injection are Challengers, while
  direct product assertions remain Checks regardless of provider.
- The provider boundary keeps traceability traversal and semantic target selection in core.
- No alpha 1 compatibility behavior, evidence facet or judgment is added; obsolete current
  verification and judgment facets are removed in this change.
- `azimuth change check evidence-control-plane-model` and work-package validation pass.
