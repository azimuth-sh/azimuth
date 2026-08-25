# Exploration: Assurance Run Planning

Id: assurance-run-planning
Created: 2026-08-24
Status: current; direction recorded, design decisions open

## Objective

Explore whether Azimuth should extend its current strict Run-request resolution into a
policy-driven planner that can determine the assurance work required for one exact Subject and
SDLC event.

The immediate purpose of this account is to preserve an architectural discussion. It does not
authorize implementation or claim that automatic assurance-work selection exists today.

## Boundaries

- Keep the repository model authoritative for Claims, Checks, Evidence Bindings, Qualifications,
  Claim Judgments, Challengers and policies.
- Keep CI and development harnesses responsible for SDLC events and execution.
- Keep AST, call-graph and schema analysis in ecosystem extractors; core may consume manifests but
  must not perform heavy source analysis.
- Do not turn Azimuth into a general CI planner, build scheduler or test runner.
- Do not let an agent's free-form judgment define the mandatory assurance minimum.
- Do not infer historical applicability, result freshness or cross-Subject reuse before the Run
  ledger defines those meanings.
- Do not broaden or narrow work silently when impact, policy or selection cannot be resolved.
- Keep this exploration separate from the consulting offer that motivated the discussion. No
  consumer vocabulary, paths or fixtures become repository dependencies.

## Existing context

The current Run journey is implemented and described in
[`contracts/run-launch-plan.md`](../../../contracts/run-launch-plan.md) and
[`docs/framework.md`](../../../docs/framework.md). A strict planning request names:

- one exact Subject;
- an `execute | import` operation;
- required context;
- explicitly requested Checks;
- explicitly requested authored Challenge Plans;
- one configured capability and finite units for each request.

The planner loads the complete unselected current model. For requested Checks it derives current
Check identity and the complete implementation set. For requested Challenge Plans it resolves
current decision targets, Challenger forms, `gate | scheduled` lanes, semantic scope and
accountable launch inputs. It validates explicit capabilities and produces an immutable launch
plan with exact Subject, model, adapter, configuration and route fingerprints.

The planner does not currently:

- infer affected Claims from a source change;
- choose which Checks are mandatory for an SDLC transition;
- choose a capability, provider selector or broader fallback;
- derive Subject-specific Assurance State;
- decide whether an earlier Observation remains applicable;
- define cache validity, cadence or cross-Subject reuse.

The related
[`continuous-assurance-service`](../continuous-assurance-service/exploration.md) exploration owns the
repository/service authority split and the direction toward a durable execution ledger. This
exploration depends on that work for result applicability, but isolates the distinct planning,
policy and impact-analysis questions.

## Findings

### F1 - Current planning resolves an explicit request; it does not decide the required work

The current implementation already supports semantic Plan and launch-plan generation. Its input is
an explicitly formed request. For Checks, the caller names exact Check ids. For Challenges, the
caller names authored Challenge Plans, and core resolves them to exact current Qualifications or
Claim Judgments.

Calling this a planner is accurate, but calling it an automatic change-impact or assurance-work
planner would overstate current behaviour.

### F2 - The current model is a strong substrate for future required-work planning

Azimuth already relates:

- Claims and criticality;
- realization and mechanism sites;
- Checks and their implementations;
- Evidence Bindings and Qualification contexts;
- Claim Judgments and residual risk;
- Challengers, Challenge Plans, Decision Policies and the current Challenge Schedule;
- exact typed Subjects and bounded Run results;
- configured adapter capabilities.

Once an affected Claim set and an organizational execution policy are known, this graph can resolve
much of the exact semantic work and accountable context that a Run requires.

### F3 - Complete-model loading is not complete-plan derivation

Loading the complete model prevents partial-model blindness during resolution. It does not prove
that the caller requested every Check required by one change or lifecycle transition. Required-work
completeness needs explicit semantics rather than an inference from the amount of model loaded.

### F4 - A future planner needs an SDLC trigger and impact account

The required work depends on why planning occurs: a local edit, pull request, release candidate,
deployment, canary, incident, scheduled Challenge or bounded monitoring window can require
different selections for the same Claim.

Direct realization and mechanism linkage can identify Claims attached to changed sites. It cannot
by itself account for every transitive effect of a shared library, route, schema or configuration.
Ecosystem extractors therefore need to produce a bounded impact manifest derived from the same
source the system builds. Unknown or incomplete impact must remain visible rather than trigger a
silent whole-suite fallback.

### F5 - Required Check execution needs an authored organizational policy

Current Decision Policies and the Challenge Schedule govern which Challenge forms are required and
whether they are `gate | scheduled`. Automatic Check selection needs a corresponding policy account
for at least:

- SDLC trigger or transition;
- organizational risk class and its mapping to Azimuth criticality;
- required and strengthening Checks;
- required context and finite work units;
- mandatory, conditional and scheduled execution;
- escalation when a required Check cannot run;
- the human-confirmation boundary associated with the configured autonomy level.

Without this account, an agent-selected minimum is an opinion rather than a reproducible control.

### F6 - Historical applicability depends on the future Run ledger

A future planner may eventually omit work because an earlier result is still applicable. That
decision requires exact rules for Subject identity, changed Check implementation, Qualification
drift, context, measurement windows, cadence and expiry. The current model intentionally defines
none of those reuse rules. The first planning slice must either run every selected item or depend on
the ledger change that owns applicability.

### F7 - Plan construction belongs to Azimuth only at the assurance boundary

Azimuth can own the deterministic compilation of an exact Subject, impact account and authored
policy into the minimum semantic assurance Plan. It should not own build scheduling or tool-native
execution. Adapters translate the frozen Plan, while the existing CI or harness performs the work.

An agent may explain impact, propose strengthening work or identify missing relationships. The
mandatory minimum must remain deterministically derivable and reviewable.

## Shared direction

If pursued, the future flow is:

```text
SDLC event + exact Subject
            |
            v
ecosystem impact extractor
            |
            v
bounded impact manifest
            |
            v
Azimuth model traversal + organizational planning policy
            |
            v
minimum required semantic Plan + explanation + unresolved gaps
            |
            v
configured confirmation boundary
            |
            v
Launch Plan
            |
            v
CI / harness / adapters
```

The status boundary discussed for an early consumer is:

- **Current** - strict request-to-Launch-Plan resolution;
- **Pilot prerequisite** - policy-driven selection of mandatory assurance work;
- **Joint integration** - SDLC events and impact manifests supplied by the consumer environment.

## Decisions

These decisions record the shared direction of the exploration. They are not accepted framework
behaviour and do not authorize a change.

- **D1 - Treat required-work planning as a plausible Azimuth responsibility.** Azimuth owns the
  assurance graph and is the natural place to compile explicit impact and policy into a semantic
  Plan. The strongest alternative is to let each CI or harness own selection; that is simpler
  locally but fragments Claim meaning and policy enforcement across integrations. Residual risk:
  planning may expand Azimuth beyond a small semantic core if the SDLC boundary is not kept strict.
- **D2 - Keep source impact analysis outside core.** Ecosystem extractors produce accountable impact
  manifests; core consumes them. The strongest alternative is direct core diff and dependency
  analysis, which would centralize planning but violate the language-boundary and zero-dependency
  constraints. Residual risk: extractor accounts may differ in precision across ecosystems.
- **D3 - Make the mandatory minimum deterministic and policy-derived.** Agents may propose or
  explain work but do not freely author the required minimum at runtime. The strongest alternative
  is agent-driven selection, which adapts quickly but is not reproducible or auditable. Residual
  risk: authored policy may become cumbersome or lag product architecture.
- **D4 - Preserve unresolved impact as an adverse planning fact.** Do not silently run less, and do
  not automatically broaden to a whole suite. The strongest alternative is conservative whole-suite
  fallback, which can reduce immediate omission risk but hides missing semantics and may be
  prohibitively expensive. Residual risk: fail-closed planning can block delivery until ownership
  and escalation paths are defined.

## Rejected alternatives

- **Run every system Check after every change.** This avoids selection design but is too expensive,
  does not address stage-specific meaning and makes scheduled or production work incoherent.
- **Let an agent choose all required work from the diff.** This lowers initial policy cost but makes
  the assurance minimum nondeterministic and hard to audit.
- **Move AST, call-graph and schema analysis into core.** This would collapse ecosystem semantics
  into the zero-dependency kernel.
- **Present the current request resolver as the complete future planner.** It would conceal the
  missing impact, execution-policy and applicability semantics.
- **Silently widen or truncate a plan when selection cannot resolve.** This would report a different
  execution question under the identity of the requested one.

## Residual risks

- Incomplete traceability or impact manifests can create false confidence in plan completeness.
- An organizational execution policy can become heavier than the risk it controls.
- A planner without a ledger cannot honestly optimize repeated execution through reuse.
- Automatic planning can hide rationale unless every selected item retains a human-readable reason.
- Capability routing cannot become heuristic selection without a separately accepted policy and
  authority model.

## Open questions

1. What is the minimum Check-execution policy that can express stage, risk, trigger, context and
   inability to execute without becoming a generic workflow language?
2. What exact impact-manifest contract is sufficient for the first ecosystem, and how does it state
   incompleteness?
3. Does unresolved impact always fail planning, or may an accountable human approve a declared
   conservative scope?
4. Which planning transitions require human confirmation, and how are local R0-R5 classes mapped to
   that boundary?
5. Must the first policy-driven planning slice wait for the Run ledger, or can it prohibit result
   reuse and execute every selected item?
6. Which explanatory account must accompany a generated Plan so that every included and excluded
   item is reconstructable?
7. Should this direction become a branch of the continuous-assurance-service change graph, or a
   separate dependency-ordered series that consumes the ledger contract?

## Candidate change dependency graph

This graph is exploratory. No node is authorized.

```text
SDLC trigger and Work/Action identity
                 |
                 +--------------------+
                 v                    v
       impact-manifest contract   Check-execution policy
                 |                    |
                 +----------+---------+
                            v
             deterministic required-work planner
                            |
                            v
        existing semantic Plan and Launch Plan generation

continuous-assurance-service / Run ledger
                            |
                            v
          optional applicability and result reuse
```

## Result

The discussion establishes a bounded direction, not an implementation commitment. Current Azimuth
can resolve an explicit strict request into an exact Launch Plan. Its existing semantic graph is a
credible foundation for future required-work planning, but it cannot derive a necessary plan from a
change without additional SDLC-trigger, impact-manifest, Check-policy and applicability semantics.

Retain the exploration with the open questions above. Do not create a change until the policy and
impact boundaries are deliberately resolved, and do not present the future planner as a current
capability.
