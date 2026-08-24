# Exploration: Scenario-bound mechanisms

Id: scenario-bound-mechanisms
Created: 2026-08-24
Status: direction agreed; open design questions named

## Objective

Shape a mechanism model in which every observable scenario Claim can account for:

- the production sites that realize its behaviour;
- the product mechanisms that causally support its correctness;
- the Checks that search for violations;
- the reviewed credibility of each Check-to-Claim edge; and
- one total Claim Judgment with explicit residual risk.

The candidate composition is:

```text
Scenario Claim
  |-- Realization
  |-- Mechanism Binding --> Mechanism --> mechanism implementation
  `-- Evidence Binding  --> Check     --> Check implementation
                                  |
                             Qualification
                                  |
                           Claim Judgment
```

The intended result is a Claim-centred model that distinguishes causal support from sampled evidence, permits one mechanism to support Claims in several specs without duplicating its identity, and does not introduce a second qualification system for mechanisms.

This exploration records candidate direction, not accepted product truth. Every implementation slice still requires its own proposal and authority.

## Boundaries

- Realization, Mechanism Binding, Evidence Binding and Claim Judgment address only scenario Claims. A requirement remains a normative grouping and criticality owner, not an assurance edge target.
- The explored mechanisms belong to the product or operational system being described. Applying the same ontology to the development harness itself is outside this direction.
- A Mechanism is reusable. One declaration may be referenced by consuming specs through separate Mechanism Bindings.
- One Mechanism Binding relates exactly one Mechanism to exactly one scenario Claim.
- Checks and Evidence Bindings continue to address scenario Claims only. They do not target a Mechanism or Mechanism Binding.
- Qualification continues to decide the credibility of one Evidence Binding. There is no Mechanism Qualification.
- A reusable mechanism's independently important behaviour is expressed as scenario Claims in its owning spec and checked through ordinary Evidence Bindings.
- Requirement discovery and completeness remain outside the method. The model can expose a missing Claim but cannot establish that all necessary Claims were authored.
- Exact Markdown notation, role extensibility, fingerprints, migration and implementation-artifact cardinality remain downstream design work.
- No consumer vocabulary, repository, document or executable dependency enters this exploration.
- This exploration does not authorize implementation or a downstream proposal.

## Existing context

The following repository facts constrain the direction:

- `contracts/design.md` and `tools/azimuth/src/design.rs` currently nest each Mechanism under a requirement- or scenario-targeted design entry. There is no standalone Mechanism Binding.
- Current Mechanism identity is spec-scoped. The design contract advises placing reusable controls in concern-oriented specs, but the format has no explicit cross-spec edge from a shared Mechanism to a consuming scenario Claim.
- The current `Enforcement` enum classifies mechanisms as `type | schema | constraint | choke-point | middleware | guard`. The authored mechanism therefore combines a structural kind, a Claim attachment and an artifact binding.
- A current Mechanism resolves exactly one structural binding. Code linkage uses the existing two-argument `ImplementsMechanism` marker; extractors derive ecosystem-semantic source identity and a companion Artifact.
- `contracts/verification.md` and `tools/azimuth/src/verification.rs` already restrict Evidence Bindings to case-level Claims. One Qualification decides one Evidence Binding.
- Claim Judgment already consumes mechanism records directly as part of the total Claim composition. No current Mechanism Qualification exists.
- Mechanism and Check implementation linkage are structurally distinct. A marker records implementation identity rather than evidentiary meaning.
- The alpha design phase has no backward-compatibility obligation unless an accepted change states one explicitly.
- Repository rules require evidence for a new notation before the notation enters the model and require consumer-domain findings to remain outside executable framework dependencies.

## Findings

### F1 - Realization, Mechanism and Check have different semantic jobs

A Realization implements part of a scenario predicate. A Mechanism causally supports the Claim's correctness by preventing a violation, preserving correctness under declared failures or restoring correctness after a violation. A Check searches for a violation and yields sampled evidence when executed.

The same source artifact may participate in more than one relation, but the relations must not collapse. Causal contribution and knowledge about the outcome answer different questions.

### F2 - Enforcement is a relation-specific role, not a complete replacement for Mechanism

Renaming the general entity to Enforcement gives a crisp account for types, storage constraints and mandatory guards, but excludes redundancy, failover, isolation and reconciliation. Conversely, those controls do not justify weakening Enforcement to mean any risk reduction.

The consequential inference is that Mechanism remains the general entity while the Mechanism-to-Claim edge states the role. The same circuit breaker can enforce a local call limit for one Claim and merely contribute resilience to another.

### F3 - Scenario-only addressability removes implicit assurance fan-out

A requirement-level edge would silently apply one causal or evidentiary statement to every child scenario. Those scenarios may have different assumptions, checks and residual risks.

Repeated atomic bindings are therefore meaningful rather than accidental duplication. A shared Mechanism identity prevents implementation duplication while each scenario retains an independently reviewable edge.

### F4 - Mechanism checking does not require another Check-like entity

A dedicated Mechanism Check would repeat Check identity, implementation, execution, result and qualification semantics while differing only in target. Allowing Evidence Bindings to target Claims, Mechanisms and Mechanism Bindings would instead create several assurance centres.

When a mechanism has independently important behaviour, that behaviour can be specified as an ordinary scenario Claim in the mechanism's owning spec. Checks remain Claim-directed. A Challenger can attack the causal account in a Claim Judgment without becoming product evidence.

### F5 - Mechanism Qualification duplicates Claim Judgment

Evidence Qualification answers whether one Check implementation, oracle and context credibly bear on one scenario Claim. The adequacy of Mechanism Bindings is part of the total-composition question already owned by Claim Judgment.

Because each Mechanism Binding belongs to exactly one scenario Claim, no independently reusable mechanism-edge decision has been demonstrated. A separate Mechanism Qualification would create another review and staleness layer without a distinct consumer.

### F6 - Property-based testing is a Check method, not another ontology

A property-based Check searches generated inputs for a counterexample to a general scenario Claim. Finite generation does not establish a universal proposition unless the domain is actually exhausted or a proof-producing method is used.

Generator relevance, oracle independence and input-domain boundaries remain Evidence Qualification concerns. Whether the current `example | universal` quantification needs a `sampled` form is unresolved.

## Decisions

### D1 - Address assurance relations only to scenario Claims

Realization, Mechanism Binding, Evidence Binding and Claim Judgment target scenario Claims. Requirement-level targeting is removed from the candidate mechanism direction. Invariants are expressed as general scenario Claims rather than a second terminal target kind.

Rationale: one exact assurance centre avoids implicit fan-out and makes assumptions, evidence and residual risk independently reviewable.

### D2 - Retain Mechanism as the general causal entity

Mechanism denotes a product or operational control that causally supports a Claim. `enforcement`, `resilience` and `recovery` are the currently evidenced role families, not yet an approved closed syntax.

Rationale: Enforcement alone is too narrow, while an undifferentiated mechanism would obscure materially different causal claims.

### D3 - Put the role on Mechanism Binding

The edge from Mechanism to scenario Claim owns the role, the exact causal proposition and its assumptions.

Rationale: a control's role is relative to the Claim. It is not an intrinsic property of the source artifact.

### D4 - Declare a shared Mechanism once

A Mechanism is owned in a concern-oriented spec and may be referenced by Mechanism Bindings in other specs.

Rationale: implementation identity and shared design intent must not be copied into every consumer, while each consuming Claim still needs its own explicit edge.

### D5 - Keep Mechanism Bindings atomic

One Mechanism Binding names one Mechanism and one scenario Claim. Composition of several Mechanisms occurs only in Claim Judgment.

Rationale: grouped bindings would hide individual roles, assumptions and impact propagation.

### D6 - Link code to Mechanism identity only

`ImplementsMechanism` continues to name the owning spec and Mechanism. It does not name a Claim, Mechanism Binding, role or consumer list. Extractors derive source identity and Artifact linkage. Explicit artifact binding remains available for non-code artifacts that cannot carry a marker.

Rationale: code owns implementation identity; the design model owns the mechanism's roles in product Claims.

### D7 - Keep Checks and Evidence Bindings Claim-directed

No Mechanism Check or additional Evidence Binding target kind is introduced. Independently verifiable mechanism behaviour becomes scenario intent in the owning spec.

Rationale: Check semantics depend on the proposition being evaluated, not on whether its implementation happens to be architectural infrastructure.

### D8 - Do not introduce Mechanism Qualification

Qualification remains an exact Evidence Binding decision. Claim Judgment evaluates Mechanism Bindings together with realizations, qualified Evidence Bindings and residual risk.

Rationale: the proposed atomic edge has no demonstrated reusable decision boundary outside its one Claim Judgment.

### D9 - Treat property-based tests as ordinary Checks

The property belongs to a scenario Claim. Generation is one Check method and its finite result remains sampled evidence unless exhaustive or proof-producing.

Rationale: generation strategy does not create a new semantic entity.

### D10 - Keep harness self-application outside the direction

The candidate model covers product and operational Claims. Modelling the development harness, permission ladder or delivery gates through the same Claims and Mechanisms requires a separate exploration.

Rationale: self-application is a plausible but currently unsupported scope expansion.

## Rejected alternatives

- **Replace Mechanism with Enforcement.** This excludes resilience and recovery. Reopen only if real examples show that those roles never contribute independently to Claim Judgment.
- **Attach a Mechanism to a requirement.** This reduces authoring but creates implicit assurance fan-out. Reopen only if scenario-level authoring proves operationally infeasible and cannot be improved without weakening identity.
- **Allow one Binding to group Mechanisms or Claims.** This shortens notation but hides independent causal propositions and impact.
- **Let Evidence Binding target Mechanism or Mechanism Binding.** This permits direct internal checks but creates multiple assurance centres and more qualification variants.
- **Create Mechanism Check.** No lifecycle or result semantics distinct from Check were found.
- **Create Mechanism Qualification.** It repeats the total-composition responsibility of Claim Judgment under the selected one-edge-to-one-Claim constraint.
- **Retain a separate addressable Invariant target.** This represents timeless properties naturally but creates a second terminal assurance target. Reopen if real invariants cannot be expressed as scenarios without semantic distortion.
- **Model the development harness with the same ontology now.** This introduces ownership, recursion and authority questions unrelated to the product mechanism problem.

## Residual risks

- Authors may still struggle to distinguish a Realization from a Mechanism when one artifact participates in both relations.
- `enforcement`, `resilience` and `recovery` may overlap or prove incomplete as a role set.
- The direction does not yet decide whether one Mechanism may have several implementation artifacts or must decompose into atomic Mechanisms.
- Cross-spec and federated Mechanism references require singular ownership and fail-closed behaviour when an owning repository is absent.
- Scenario atomicity improves precision but may create substantial binding volume.
- Without Mechanism Qualification, Claim Judgment needs enough structured input to assess causal sufficiency without hiding it in unreviewable prose.
- Current contracts, Rust types, export, validation, traceability and current model packages all assume nested Mechanisms; migration will span several bounded changes.
- The current `example | universal` evidence form may describe finite generative evidence poorly.
- Fingerprint and staleness propagation through a new Mechanism Binding are not yet designed.
- Even a semantically coherent model may remain too abstract for adopters without small, concrete examples.

## Open questions

1. Can one source artifact simultaneously realize a scenario Claim and implement a Mechanism for it, or must the conceptual identities be distinct?
2. Should mechanism roles be a closed enum, an open namespaced vocabulary or introduced one evidenced role at a time?
3. Does the current structural ladder remain as a separate Mechanism kind after role moves to Mechanism Binding?
4. Can one Mechanism have several implementation artifacts, and what makes such a set one mechanism rather than a composition?
5. Which Mechanism Binding fields make the causal proposition and assumptions exact without turning the format into unrestricted design prose?
6. How do cross-spec and federated projects resolve ownership and incompleteness for shared Mechanisms?
7. Which Mechanism, Mechanism Binding, implementation and Artifact inputs participate in Claim Judgment identity and staleness?
8. Are current `Basis` and `Residual risk` fields sufficient for causal assessment without Mechanism Qualification?
9. How do traceability and Challenger selection traverse a shared Mechanism through scenario-specific Mechanism Bindings?
10. Does finite property-based evidence require a `sampled` quantification distinct from `example` and `universal`?
11. What one-way migration removes nested requirement-level Mechanisms without retaining a parallel reader?

## Proposed validation cases

- **Shared enforcement:** declare one synthetic access-control Mechanism and bind it to two scenario Claims in separate specs. The implementation identity must not duplicate, while assumptions and Claim Judgments remain independent.
- **Resilience:** model synthetic service failover for an availability scenario. The role must state a bounded fault model rather than an absolute guarantee.
- **Recovery:** model synthetic reconciliation after data divergence. The account must distinguish restoration after violation from prevention.
- **Judgment without Mechanism Qualification:** compose several Mechanism Bindings and Evidence Bindings for one critical scenario and determine whether one Claim Judgment remains clear and reviewable.
- **Property-based Check:** bind finite generated inputs to a general scenario Claim and determine the necessary quantification semantics.
- **Atomicity:** model a synthetic middleware implementation, route configuration and several entry points to find the boundary between one Mechanism and a composition.

No experiment is authorized by this exploration.

## Candidate change graph

The candidate dependency graph is maintained in `change-map.md`. It is a direction account, not an implementation plan or proposal authority.

## Result

Shared direction has been reached for a scenario-bound, reusable Mechanism model with atomic Mechanism Bindings and Claim-directed evidence. The exact notation, implementation cardinality, role vocabulary, judgment composition and migration remain named downstream decisions.

The next permitted action is review of this written account. No proposal or implementation follows implicitly.
