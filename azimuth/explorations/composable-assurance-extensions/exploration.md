# Exploration: Composable assurance extensions

Id: composable-assurance-extensions
Status: complete

## Objective

Determine whether Azimuth can keep a small claim/linkage/judgment core while accepting
tool-specific assurance methods as optional extensions, including methods whose one execution
relates to several claims or spans several repositories.

## Boundaries

- Preserve `Realizes`, `Covers`, mechanism linkage, evidence form and judgment as the semantic
  core. Do not add one core artifact type per assurance tool.
- Include operational configuration, load testing, chaos testing, static analysis and the existing
  mutation integration as validation cases.
- Do not claim that a broad tool run proves every claim in its target files.
- Keep native tool execution outside ordinary `azimuth check`; import immutable results.

## Existing context

- D10 already defines the manifest and exported model as extension seams.
- D18 makes agent judgment negative qualification of evidence rather than evidence itself.
- D33 permits intent, realization and evidence to be owned by different repositories while areas
  provide stable source identity.
- The current mutation prototype added a dedicated `MutationAssessment` to the Rust core. That
  repetition would continue for SARIF, load and chaos unless the transport is generalized.
- One test site can already carry several `Covers` relations. The missing normalization is one
  execution identity with several claim-specific interpretations.

## Findings

### F1 — Repository placement and semantic role are independent

The federation fixture already has a `rides-operations` repository and a `monitoring` area.
Prometheus rules, dashboards and their repository tests can therefore contribute realization,
mechanism and evidence relations from that repository. Checked-in configuration establishes the
declared rule; a live receipt is still needed for claims about deployment or notification delivery.

### F2 — Testing techniques do not require new core semantics

Load, stress, spike, soak, chaos, recovery, security, compatibility, concurrency, accessibility
and manual testing differ in execution and oracle. They can still supply either claim evidence,
mechanism evidence or judgment context through the existing relations. Tool brand does not decide
which role a result plays.

### F3 — One execution may support several propositions

A load run can test latency, throughput and error-rate assertions; a chaos run can test degraded
behaviour, recovery and alerting. The execution metadata is shared, but each claim binding needs
its own assertion, form, outcome and oracle. A shared `passed` bit would be an unauditable blanket
claim.

### F4 — Broad analysis is usually a challenge, not covering evidence

A repository-wide static scan or mutation run may challenge many realization and evidence sites.
No findings does not establish the product predicates. Findings and inconclusive outcomes must be
available to the judge and must stale affected judgments when the report, configuration or targets
change.

### F5 — The smallest useful extension protocol has two levels

An immutable observation records the execution once. Its bindings relate that observation to a
claim as direct evidence or as a judgment challenge. Challenge bindings name the existing sites or
mechanisms they targeted; evidence bindings carry their actual scope, quantification and oracle.

## Decisions

- **E1 — Generic observation envelope.** Replace mutation-specific core records with one
  observation format carrying id, kind, producer, report, configuration inputs, observation time,
  expiry and a payload fingerprint.
- **E2 — Explicit many-to-many bindings.** One observation may bind to several claims, but every
  evidence binding declares a separate assertion, outcome and evidence form.
- **E3 — Two roles only.** `evidence` contributes a `Covers` relation; `challenge` contributes
  freshness-tracked agent judgment context and never covers a claim.
- **E4 — Resolved challenge subjects.** Challenge targets reuse existing realization, evidence and
  mechanism identities. A deleted or renamed target becomes a machine-tier hole.
- **E5 — Validation set.** Migrate Stryker.NET, import a SARIF scan, model one k6 load execution and
  one chaos execution, and assemble operational Prometheus realization/evidence from the separate
  operations repository.

## Open questions

None blocking this experiment. Native adapters beyond Stryker.NET and SARIF remain later extension
work; the load and chaos cases validate the provider-neutral boundary first.

## Result

Proceed with one framework change, `generic-assurance-observations`. Accept the protocol only if
all four validation cases use the same core collections and adding the second judgment tool does
not require another Rust domain type.
