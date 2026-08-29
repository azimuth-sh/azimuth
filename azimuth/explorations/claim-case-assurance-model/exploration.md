# Exploration: Claim/Case assurance model

Id: claim-case-assurance-model
Created: 2026-08-25
Status: direction agreed; open design questions named

## Objective

Evaluate a Claim-centred assurance model that does not depend on OpenSpec's
Requirement/Scenario ontology:

```text
Claim
  |-- Case
  |-- Case
  `-- Case
```

A Claim is the independently governable product proposition. Cases are normative
condition/outcome clauses within its predicate, not examples. The intended result preserves one
assurance centre while making every materially different situation addressable for evidence, Run
selection and adverse-result provenance.

The candidate composition is:

```text
production site --Realizes------------------------> Claim
Mechanism implementation --> Mechanism ----------> Claim
Check implementation ------> Check --binding-----> Case --belongs to--> Claim
                                   |                  |
                            Method Qualification     Applicability Decision
                                   \__________________/
                                            |
                                  total Claim Judgment
                                            |
                                   Claim Assurance State
```

This exploration records candidate direction, not accepted product truth. Every behavioral or
format transition still requires its own proposal and authority.

## Boundaries

- Claim is the only independently governed intent entity. It owns criticality, production
  realization, total Claim Judgment and Claim Assurance State.
- A Case is a normative constituent of one Claim. Adding, removing or changing a Case changes the
  Claim's intent.
- Cases are addressable by Evidence Bindings, Run selection, Observations and Challenger impact,
  but own no criticality, production realization, separate Claim Judgment or separate Assurance
  State.
- Within an assurance-bearing non-routine Claim, every Case requires at least one deliberately
  enrolled verification route. One Check may bear on several Cases and one Case may receive several
  Checks.
- Functional and non-functional Claims use the same assurance semantics. Workload profiles,
  deadlines, convergence windows and freshness boundaries belong to their predicates and contexts,
  not to a second intent ontology.
- The exploration covers sparse source markers, Check identity, split credibility decisions,
  Challenger targeting and impact, Claim composition, migration and a synthetic cardinality stress
  model.
- The exploration revisits decisions in the earlier `scenario-bound-mechanisms` exploration but
  does not rewrite that preserved reasoning.
- It does not change the accepted Azimuth model, contracts, tooling or documentation; create a
  proposal; authorize implementation; or manufacture Qualification, Judgment or execution facts.
- External domain vocabulary and fixtures do not enter this repository or become executable
  Azimuth dependencies.

## Existing context

- The current framework treats an OpenSpec scenario as a case-level Claim. An OpenSpec requirement
  groups those Claims and owns criticality.
- Current production realization, Mechanism, Evidence Binding, Qualification and Claim Judgment
  relations terminate at case-level Claims.
- A Check has one project-global identity and one atomic terminal proposition. One Check may bind to
  several case-level Claims, and several Checks may bind to one case-level Claim.
- Each current Evidence Binding has exactly one complete Qualification. Its fingerprint combines
  Check, Claim, binding, required context and policy credibility inputs.
- A Challenger searches for a reason to distrust an exact Qualification or Claim Judgment. A
  finding is an adverse execution fact and impact edge; it does not automatically rewrite the
  repository decision. A clean result is only a negative search fact.
- Challenge selection currently resolves exact decision fingerprints through binding, Check,
  realization, Mechanism or Claim relations. Results retain every exact target and do not infer
  broader impact from a sibling result.
- Source linkage is already sparse for Checks: `ImplementsCheck` names only the Check identity.
  Production and Mechanism markers likewise name implementation identity rather than evidence
  meaning.
- The `scenario-bound-mechanisms` exploration selected scenario Claims as independent assurance
  centres, atomic scenario-directed Mechanism Bindings, and one complete Qualification per Evidence
  Binding. Its reusable Mechanism identity, relation-specific role, sparse implementation marker
  and rejection of Mechanism Qualification remain useful; its assurance target and Qualification
  cardinality are reconsidered here.
- The alpha design phase has no general backward-compatibility obligation, but a brownfield
  migration must not invent renewed assurance authority.

## Synthetic stress model

A synthetic model exercises collection CRUD, constrained path planning, offline operation,
latency, convergence and publication freshness. Its predicates, criticalities, workloads,
deadlines, mechanisms and verification methods are hypotheses used only to expose model cardinality
and assurance relationships.

The model contains:

| Entity or decision | Count |
|---|---:|
| Claims | 8 |
| Normative Cases | 33 |
| Candidate Checks | 25 |
| Check-to-Case evidence edges | 33 |
| Parent Claim Judgments | 8 |

The six-Case collection CRUD Claim is the decisive pressure case. One model-conformance Check can
credibly exercise create, metadata update, idempotent addition, note update, removal and deletion,
but method-wide defects and Case-specific coverage defects have different impact.

## Findings

### F1 - Cases preserve intent rather than illustrate it

A scenario such as retrying deletion, failing closed when no legal route exists or reconnecting an
offline device specifies a materially different part of the product proposition. Omitting it loses
intent even when the parent predicate remains readable.

The consequential inference is that every authored Case contributes to the Claim digest and must
remain independently addressable.

### F2 - Evidence cardinality should follow normative Cases

Thirty-three Cases producing at least thirty-three evidence edges is not an explosion to eliminate.
It is the explicit account that every normative clause has a verification route. Compressing those
edges would create implicit evidence fan-out and hide coverage gaps.

What should not automatically follow Case cardinality is production tagging, criticality, total
Judgment or Assurance State.

### F3 - Check identity and Case coverage are independent

One semantic Check may implement a parameterized, model-based or otherwise shared method whose
atomic terminal proposition bears on several Cases. Conversely, one Case may need several
independent methods.

Each relationship remains explicit even when source carries only one Check implementation marker.
Agents can propose stable Check ids, but review owns the proposition and atomicity.

### F4 - Claim-level production linkage avoids duplicate Case tags

The same command boundary, routing pipeline or publication projection commonly realizes several
Cases of one promise. Tagging every Case at those sites repeats intent identity and makes Case
refactoring invasive.

Claim-level realization deliberately trades some local Case-to-site precision for sparse and stable
source linkage. Traceability may show inherited parent context but must not fabricate direct
Case-to-site assertions.

### F5 - Complete per-binding Qualification repeats shared credibility

When six bindings share one Check implementation, independent oracle, common context and method,
six complete Qualifications restate the costly credibility account. Challenger execution against
that shared method may also repeat substantially identical work against six decision fingerprints.

The binding count is justified; repeating all method credibility on every binding is not.

### F6 - Check-only Qualification is too coarse

A Challenger may discover that a state-model oracle reuses production code. That finding bears on
every Case served by the method. It may instead discover that generated sequences never exercise
deletion. That finding bears on the deletion edge without necessarily discrediting the other five
Cases.

One Check-level Qualification cannot express both outcomes without either over-broad impact or an
unsoundly retained decision.

### F7 - Credibility contains two independently challengeable propositions

Method Qualification asks whether the Check implementation, oracle, common context and method are
credible. Applicability Decision asks whether that method actually establishes one Case under the
edge's required context.

A Check may require more than one Method Qualification when common context or policy differs. Reuse
is permitted only when the exact method-level inputs are shared; it is never inferred from Check id
alone.

### F8 - Challenger impact follows the proposition attacked

Method-oriented findings should reach every dependent applicability edge and parent Claim.
Coverage- or relevance-oriented findings should remain local to one edge unless an explicit
additional target establishes broader impact.

Azimuth must retain the adverse target and impact graph without converting a Challenger result into
an automatic repository decision or manufacturing sibling results.

### F9 - One Claim Judgment can preserve exact adverse provenance

One total Claim Judgment can consume the complete Claim and Case digests, realizations, Mechanisms,
method decisions, edge decisions, policy, basis and residual risks. Claim Assurance State can remain
parent-level while every Observation retains exact Check, Case, Subject and outcome identity.

No scoring or averaging is needed: an adverse Case remains adverse even when sibling results are
clean.

### F10 - Structural migration is easier than authority migration

Sparse source markers allow the model files to change without retagging product and Check
implementation sites. Existing binding rationale and provenance can seed the two new decisions.

An accepted old Qualification nevertheless judged one combined proposition. It cannot honestly
authorize two newly separated propositions, and historical Challenger Results cannot be relabelled
with new fingerprints.

## Decisions

### D1 - Make Claim the sole assurance centre

Claim owns the independently governable product proposition, criticality, production realization,
total Claim Judgment and Claim Assurance State.

Rationale: one intent centre preserves focus and prevents concrete situations from multiplying
governance entities.

### D2 - Make Cases normative constituents of Claim intent

A Case is an authored condition/outcome clause whose presence contributes to Claim meaning and
identity. It is not an example.

Rationale: concrete operating situations carry intent that a broad parent sentence cannot safely
imply.

### D3 - Address Cases without governing them independently

Cases are selectable through traceability, Evidence Bindings, Runs, Observations and Challenger
impact. They own no separate criticality, production realization, Claim Judgment or Assurance
State.

Rationale: verification needs exact situation identity while product governance concerns the whole
promise.

### D4 - Keep implementation markers sparse

Production sites identify Claims, Check implementations identify Checks and Mechanism
implementations identify Mechanisms. No marker carries Case, binding, Qualification or consumer
lists.

Rationale: implementation identity and assurance meaning evolve at different rates and belong to
different authorities.

### D5 - Require explicit evidence coverage for every non-routine Case

Every Case under an assurance-bearing Claim receives at least one Evidence Binding. Checks and Cases
retain a many-to-many relationship.

Rationale: a normative Case without a deliberately enrolled verification route is an assurance gap.

### D6 - Retain semantic project-global Check identity

Check identity is independent of test file, function, provider activity and Case identity. Agents
may propose ids and reuse candidates; reviewers decide semantic stability and atomicity.

Rationale: implementation can move or compose without changing the proposition being evaluated.

### D7 - Split method credibility from edge applicability

Method Qualification decides the credibility of exact shared method inputs. Applicability Decision
decides whether that qualified method bears on one Check-to-Case Evidence Binding.

Rationale: costly shared credibility should be reviewed once while Case-specific relevance remains
independently visible and challengeable.

### D8 - Target Challengers to the decision proposition

Method Challengers target Method Qualification and their impact fans out through dependent edges.
Coverage and relevance Challengers target Applicability Decisions and remain local unless another
explicit relation establishes broader impact.

Rationale: fault localization must not sacrifice the ability to expose shared method failure.

### D9 - Keep one total Claim Judgment

Claim Judgment consumes all applicable composition, including every Case and evidence edge, without
creating a separate readiness decision per Case.

Rationale: the parent promise is accepted or rejected as a whole, and clean siblings must never hide
an adverse or unsupported Case.

### D10 - Keep Claim Assurance State parent-level and non-scoring

Observations retain exact Case provenance. Parent state propagation distinguishes violation,
inconclusive work and missing policy-required work without percentages or averaging.

Rationale: operational response concerns the promise while diagnosis requires the exact adverse
situation.

### D11 - Migrate authorship but never authority

A one-way migration may generate Method Qualifications and Applicability Decisions from current
binding Qualifications and copy their rationale and provenance. Generated decisions begin
unreviewed or stale. Historical decisions and Challenger Results retain their original identities.

Rationale: mechanical work can be automated, but no tool may manufacture review of newly separated
propositions.

## Rejected alternatives

- **Keep every Case as an independent Claim.** Precise locally, but it multiplies realization,
  criticality, policy, Judgment and Assurance State for one product promise.
- **Treat Cases as non-addressable examples.** Compact, but it loses exact intent, coverage, Run
  selection and adverse provenance.
- **Require one unique Check implementation per Case.** This duplicates credible model-based and
  parameterized methods without improving the evidence relation.
- **Attach production realization markers to Cases.** This improves local impact precision at the
  cost of repeated tags and invasive Case refactoring.
- **Keep one complete Qualification per binding.** Exact, but repeats shared method review and
  Challenge work.
- **Use only one Check Qualification.** Compact, but cannot distinguish method-wide defects from
  one Case's missing coverage.
- **Qualify an explicit group of bindings.** Group membership churns identity, and one local finding
  either over-invalidates the group or forces a split.
- **Automatically retain accepted verdicts during migration.** This grants authority to propositions
  no reviewer accepted.

## Residual risks

- A Claim may still become too broad and assemble an unwieldy realization and Judgment context.
- Authors need a usable boundary between a Case and an independently governable Claim.
- Common method context and Case-specific context may be difficult to separate honestly.
- Policy ownership and required Challenge forms may differ between Method Qualification and
  Applicability Decision.
- A local Challenger finding may reveal a method-wide problem; escalation needs an explicit account
  rather than inferred fan-out.
- Shared Check execution may have an aggregate result too coarse for exact Case Observations.
- Editing one Case stales the parent Claim Judgment even when sibling applicability decisions remain
  current.
- Promoting a Case into a Claim changes identity and requires retargeting evidence and realization.
- Mechanism relevance needs Case precision in the model without Case ids entering source markers.
- Existing contracts, types, fingerprints, traceability, validation, Challenge Plans and Run
  planning assume the combined Qualification model.
- The synthetic stress model demonstrates plausible cardinality but not yet sustainable human
  authoring and review in a maintained project.

## Open questions

1. What final names best distinguish Method Qualification from per-binding Applicability Decision?
2. Does Applicability Decision need an explicit verdict, or can an accepted Evidence Binding carry
   the reviewed applicability assertion?
3. Which Check, implementation, oracle, common-context, policy and method-definition inputs form
   Method Qualification identity?
4. Which Case, edge proposition, Case-specific context and method identity form Applicability
   Decision identity?
5. How does the format keep authors from moving inconvenient context into the less frequently
   reviewed layer?
6. Do the two decision kinds use separate Decision Policies and required Challenger forms?
7. How is a local applicability finding explicitly escalated into a method-wide objection?
8. When one Check serves several selected Cases, does execution produce one projected Observation,
   separate Case-addressed Observations or a structured result reduced to both?
9. How does Run planning deduplicate shared execution without losing Case coverage or adverse
   provenance?
10. How does Case relevance participate in shared Mechanism Bindings and Claim Judgment?
11. Which edits stale Applicability Decision, Method Qualification and total Claim Judgment?
12. What operational rule identifies a Claim that is too broad?
13. How do historical Challenge Plans and Results remain inspectable after decision identities
    change?
14. Which prior `scenario-bound-mechanisms` decisions are formally superseded by each accepted
    downstream transition?

## Validation experiments

The synthetic stress model is completed and authorized as non-authoritative research. No further
experiment is authorized by this account.

Candidate experiments are:

- introduce shared-oracle, shared-implementation and deletion-only coverage defects into the
  collection Check and inspect local versus fan-out Challenger impact;
- select one, several and all collection Cases and require one physical Check execution with
  complete Case coverage and result provenance;
- give two bindings genuinely different contexts and determine whether they can share a Method
  Qualification honestly;
- migrate a synthetic current-format project, preserve rationale and history, and confirm that no
  generated decision is accepted;
- promote a routing Case into a Claim and account for every identity and historical-reference
  transition; and
- compare human and agent authoring of the same additional domain for granularity, id quality,
  repeated rationale and review effort.

## Candidate change graph

The dependency-ordered candidate graph is maintained in `change-map.md`. It is a research result,
not an implementation plan or proposal authority.

## Result

Shared direction has been reached for a Claim-centred model with normative addressable Cases,
sparse Claim-level production linkage, explicit Case evidence coverage, shared Method Qualification,
per-binding Applicability Decision, proposition-exact Challenger impact and fail-closed migration.

The direction materially revises the scenario-as-assurance-centre and complete-per-binding
Qualification choices in `scenario-bound-mechanisms`. That exploration remains preserved; only a
future accepted change can establish formal supersession in the current model.

Exact notation, decision fingerprints, policy ownership, Challenger forms, result projection,
Mechanism relevance and migration mechanics remain named downstream decisions. No proposal,
implementation or follow-up experiment follows implicitly from this exploration.
