# Candidate change map: Scenario-bound mechanisms

This map records candidate dependency boundaries. It is not proposal authority and does not authorize implementation.

```text
M0 - Validate semantics with synthetic cases
 |
 v
M1 - Separate Mechanism and Mechanism Binding
 |
 v
M2 - Reshape Mechanism implementation linkage
 |
 v
M3 - Compose Mechanism Bindings into Claim Judgment
 |
 v
M4 - Update traceability and Challenger traversal
 |
 v
M5 - Migrate the current model and derived docs
```

## M0 - Validate semantics with synthetic cases

Apply the direction to three structurally different synthetic cases:

- access control as enforcement;
- service failover as resilience; and
- reconciliation as recovery.

This validation should settle the initial role vocabulary, Mechanism atomicity and whether Claim Judgment can assess Mechanism Bindings without Mechanism Qualification.

## M1 - Separate Mechanism and Mechanism Binding

Candidate scope:

- standalone Mechanism declarations;
- atomic scenario-only Mechanism Bindings;
- cross-spec references to shared Mechanisms;
- a relation-specific role, causal proposition and assumptions;
- rejection of requirement-level bindings; and
- removal of the current nested design-entry semantics.

The format contract must be decided before tool behaviour changes.

## M2 - Reshape Mechanism implementation linkage

Depends on M1. Candidate scope:

- retain `ImplementsMechanism` as implementation-identity linkage only;
- derive source identity and companion Artifact without writing generated bindings into authored design;
- decide single- versus multi-artifact Mechanism implementation;
- retain an explicit route for non-code artifacts; and
- resolve shared Mechanisms in complete and federated project accounts.

## M3 - Compose Mechanism Bindings into Claim Judgment

Depends on M1 and the relevant M2 identity decisions. Candidate scope:

- include scenario-specific Mechanism Bindings in total Claim composition;
- define Mechanism, binding, implementation and Artifact identity inputs;
- propagate causal assumptions into reviewable judgment context;
- retain Qualification only for Evidence Bindings; and
- report missing, ambiguous, rejected or stale composition precisely.

## M4 - Update traceability and Challenger traversal

Depends on M3. Candidate scope:

- traverse shared Mechanisms through scenario-specific Mechanism Bindings;
- resolve every affected Claim Judgment without requirement-level fan-out;
- challenge causal composition without turning a Challenger into product evidence; and
- report cross-spec impact deterministically.

## M5 - Migrate the current model and derived docs

Depends on accepted M1-M4 behaviour. Candidate scope:

- migrate current framework Mechanisms;
- remove the old nested form and reject it rather than retaining a parallel reader;
- update contracts, glossary and derived framework prose;
- update synthetic fixtures without consumer vocabulary; and
- record inconvenient self-application findings rather than weakening the model silently.

Each node may become one or more bounded changes after its prerequisites are settled. This exploration does not create those changes.
