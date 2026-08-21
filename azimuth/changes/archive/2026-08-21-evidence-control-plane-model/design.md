# Design: Evidence Control Plane Model

## Semantic layers

The public model separates durable meaning from execution facts.

Repository authority owns:

- requirement-level and case-level Claims;
- Check definitions and Evidence Bindings;
- Qualifications and Claim Judgments;
- qualification standards and residual rationale.

Execution authority owns:

- Runs and exact Subjects;
- Observations and Challenge Results;
- provider-native artifact references;
- derived Assurance State, gates and exceptional work.

The optional service can persist execution authority, but a local bundle has the same meaning.

## Check and Evidence Binding

A Check is sparse and deliberate. Untagged native tests remain ordinary engineering inventory.
Every Check has at least one Evidence Binding to a product or operational Claim.

One Check may bind to several Claims only when its outcome is atomic. Each binding states the
evidence proposition, required context, evidence form, challenge domain and Qualification policy.
Each binding is qualified separately.

Source linkage establishes where a Check is implemented. It does not establish that the source
covers a Claim.

## Qualification and Claim Judgment

A Qualification answers whether one exact Check-to-Claim edge is credible under required context.
A Claim Judgment answers whether the total assurance composition for a Claim is sound enough given
realizations, mechanisms, guarantees, Evidence Bindings, Qualifications and residual risk.

Recurring Observations affect Subject-specific Assurance State. Passing runs do not establish a
Qualification automatically.

## Run and outcomes

A Run is a neutral envelope, not a test process. Its Subject may identify a developer workspace, CI
candidate, artifact, deployment or bounded monitoring window. It records the plan, actual
selection, context, provenance and terminal outcomes.

Each `(Run, Check)` has one terminal Observation: `satisfied`, `violated` or `inconclusive`.
Challenger execution creates a separately targeted Challenge Result. One physical fault execution
may produce both result kinds.

## Challenge boundary

A Check directly evaluates a product or operational Claim. A Challenger searches for a reason to
distrust a Qualification or Claim Judgment. Classification follows the proposition, not the tool
brand.

Mutation testing, broad static analysis and qualification-oriented fault injection are normally
Challengers. Fault recovery assertions or claim-specific analyzers with independent oracles are
Checks. Challengers do not recursively require Qualifications in alpha 2.

Challenge targeting traverses stable realization and mechanism identities to the exact semantic
decision. A challenged Qualification affects downstream judgments and state through the graph; it
does not manufacture duplicate Challenge Results.

## Provider boundary

Core owns model interpretation and bounded plans. Configured provider-family adapters translate a
plan to native selectors or import an existing report, report actual selection, and return
normalized outcomes plus artifact references. Adapters do not parse the repository model or become
semantic authorities.

Provider protocol, command syntax and persistence are intentionally deferred to dependent changes.

## Alpha policy

The transition is one-way. Alpha 1 concepts may remain in historical decisions and archives, but
active formats and commands gain no compatibility aliases or dual readers.

Every active requirement is routine. Ordinary tests still verify implementation during
development, but the current model gains no new evidence or judgment facets until criticality is
raised by a later accepted change. This change removes obsolete current verification and judgment
facets from lowered requirements instead of carrying stale alpha 1 assurance through the new model.
