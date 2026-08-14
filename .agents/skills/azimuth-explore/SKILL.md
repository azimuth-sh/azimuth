---
name: azimuth-explore
description: Research and shape an uncertain initiative through strict one-question-at-a-time deliberation before creating Azimuth changes. Use for a new domain or module, a substantial refactor, curriculum or content design, or any effort likely to span several changes; research discoverable facts, pressure-test user-owned decisions, and persist only confirmed shared understanding under azimuth/explorations/.
---

# Explore an initiative

Turn an uncertain objective into a researched, defensible direction and a candidate change graph.
An exploration is above changes: it may produce several changes, an experiment, or no work at all.

<HARD-GATE>
Do not create a change, plan implementation, edit the deliverable, scaffold code, or take any other
implementation action during deliberation. Do not persist an exploration until the user approves
the conversational synthesis. After the written exploration is approved, stop and let the user
choose the next action; never invoke `azimuth-propose` implicitly.
</HARD-GATE>

## Non-negotiable conversation contract

- Ask exactly one unresolved decision question per assistant turn, then stop and wait.
- Never batch questions unless the user explicitly requests batching.
- For a material decision, present three or four credible alternatives with consequences and
  trade-offs. Present two when only two genuine alternatives exist; never invent filler.
- Number the alternatives so the user can answer with a number and optional reasoning.
- End the question with a concise recommendation and its reasoning.
- Use an open-ended question only when honest alternatives cannot yet be formed.
- Do not answer a user-owned decision, interpret silence as agreement, or present a complete
  direction while material decisions remain unsettled.
- Treat an approval question for one synthesis section as the only question in that turn.

## Model the exploration

Maintain a decision tree. A decision enters the frontier only when all of its prerequisites are
settled. Even when several decisions are available, ask only the most consequential one and
recompute the tree after the answer.

A decision is material when it can change the objective, scope, behaviour, architecture, risks,
success criteria, required experiments, or candidate change graph. Exhaust the material frontier,
not every conceivable preference.

Keep epistemic roles explicit:

- Establish discoverable facts with repository inspection, history, documentation, primary
  sources, or other read-only tools. Do not ask the user to retrieve facts the environment can
  establish.
- Label consequential inferences as inferences and retain sources for material facts.
- Put value judgments, priorities, and trade-offs to the user and wait for an answer.
- Keep questions downstream of an unresolved fact, decision, or experiment blocked while working
  any unaffected branch.

## Workflow

1. Locate the Azimuth project and read current model packages, active changes, relevant source,
   documentation, and recent history. In a federated checkout, run `azimuth project locate` before
   assuming authority.
2. Establish the objective, boundaries, and success criteria through the conversation contract.
   Keep adjacent ambitions outside the boundary.
3. Research the factual frontier. Separate facts, inferences, and unknowns before asking the next
   user-owned decision.
4. Work the decision frontier one question at a time. After every answer, recompute prerequisites
   and pressure-test the answer only when a concrete concern warrants it.
5. Continue until no material decision remains or the user deliberately leaves named decisions
   unresolved.
6. Audit the material decisions, then present the resulting direction in sections. Ask for approval
   after each section and revise it before continuing.
7. After every section is approved, ask explicitly whether shared understanding has been reached
   and whether the exploration may be persisted.
8. Only after confirmation, form a concise stable topic id and run
   `azimuth explore create <id> --title <title>` when no package exists. Write the creation date as
   `Created: YYYY-MM-DD` in the account rather than embedding it in the active id. Write the approved
   account, self-review it, then ask the user to review the actual file.
9. Correct requested issues and repeat the file-review gate. After approval, stop and offer explicit
   next steps without choosing one for the user.

## Pressure-test decisions

Challenge a selected direction when it creates a contradiction, material risk, hidden dependency,
weak success criterion, or conflict with established facts. Make the challenge the next single
question. Do not challenge choices performatively or require the user to defend routine details.

Before synthesis, ensure every material decision has:

- an agreed rationale;
- a strongest rejected alternative;
- a known residual risk or condition that would reopen it.

When a pattern of unreasoned agreement appears across consequential recommendations, run one
focused alignment check. Ask which decision is least certain, what the strongest objection is, or
what fact would change the direction. Treat this as a diagnostic signal, not a demand to manufacture
disagreement.

If discovered decisions cease to contribute to one independently understandable outcome, pause
detailed questioning. Propose three or four credible decomposition boundaries with trade-offs and a
recommendation, then let the user choose which branch to continue. Do not use a fixed question limit
or split the initiative without consent.

## Handle uncertainty honestly

When the user does not know an answer, classify the uncertainty:

- Research a discoverable fact.
- For an empirical question, propose the smallest experiment or prototype, what it measures, and
  how its result would settle the decision. Wait for authorization before creating it.
- Leave a genuinely user-owned decision explicitly unresolved.

Never convert uncertainty into acceptance of the recommendation. Do not keep rephrasing a question
that discussion cannot settle.

## Synthesize in sections

Cover the following, scaling each section to its complexity:

- objective and boundaries;
- established facts and consequential inferences;
- agreed decisions and rationale;
- rejected alternatives and residual risks;
- unresolved questions and authorized or proposed experiments;
- candidate change dependency graph.

Present only one coherent section at a time and ask whether it is correct. Requested corrections
reopen that section and any downstream decision affected by them.

## Persist the approved account

- Use `azimuth/explorations/<id>/exploration.md` as the active anchor. Keep the stable id undated,
  record `Created: YYYY-MM-DD` in the account and reserve the `YYYY-MM-DD-<id>` directory form for
  archival transitions, symmetric with changes. The exploration is not accepted product truth.
- Add `research.md` only when sourced findings obscure the anchor. Add `change-map.md` only when more
  than one likely change exists.
- Keep proposed target behaviour out of `azimuth/model/`; that directory describes current truth.
- Do not put a multi-change exploration inside the first change.
- Let a downstream proposal declare `Exploration: <id>` and the decision ids it carries. Derive the
  reverse exploration-to-change map rather than maintaining two authoritative copies.
- Archive an exploration when every decision has a disposition and its intended changes or
  experiments are identified. Do not wait for those changes to finish.

Self-review the written artifact for placeholders, contradictions, ambiguity, unsupported factual
claims, scope drift, and accidental presentation of proposals as current truth. File review is a
separate user gate; conversational approval does not substitute for it.

## Stop at the boundary

After file approval, offer explicit choices such as proposing one bounded change, running an agreed
experiment, conducting more research, retaining unresolved decisions, or abandoning the initiative.
Wait for the user's choice.

## Failure signals

Stop and return to the current phase when tempted by any of these thoughts:

- "The user asked for a proposal, so I should provide the complete proposal now."
- "These questions are independent, so I can bundle them."
- "I know enough context to skip the remaining decision frontier."
- "The user accepted my recommendation, so its rationale needs no audit."
- "I can write a provisional exploration now and obtain approval later."
- "Approval of the exploration authorizes the first change."

Each is a violation of the exploration contract.

## Completion

Finish with one of: an approved candidate change map, an approved experiment, an explicit
abandonment, or named open decisions the user chose not to settle. Never turn silence, uncertainty,
or repeated assent into agreement.
