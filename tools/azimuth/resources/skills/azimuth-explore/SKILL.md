---
name: azimuth-explore
description: Research, approve and explicitly archive an uncertain initiative before creating Azimuth changes. Use for a new domain, substantial refactor, multi-change effort, or a later request to archive an approved exploration.
---

# Explore an initiative

Turn uncertainty into confirmed shared understanding and a candidate change graph. An exploration is non-normative and never authorizes implementation.

## Deliberate

1. Read the target repository's applicable instructions, current Azimuth account, relevant implementation and existing explorations.
2. Establish discoverable facts by inspection or primary-source research. Keep facts, inferences and owner decisions distinct.
3. Ask exactly one unresolved material decision question per turn. Present credible alternatives, consequences and a recommendation.
4. Recompute the decision frontier after each answer. Pressure-test contradictions, hidden dependencies, weak success criteria and material residual risks.
5. Continue until no material decision remains or the owner deliberately leaves a named decision unresolved.

Do not create a change, edit product deliverables or persist a provisional exploration during deliberation.

## Approve and persist

1. Present the synthesis one section at a time: objective and boundaries; facts and inferences; decisions and rationale; rejected alternatives and residual risks; unresolved questions or experiments; candidate change graph.
2. Obtain approval for every section, then ask whether shared understanding has been reached and persistence is authorized.
3. Run `azimuth reference show exploration` and `azimuth explore create <id> --title "<title>"`.
4. Write `exploration.md`, adding `research.md` when sources obscure the anchor and `change-map.md` when several changes are likely.
5. Self-review the actual files and ask for file approval while `Status: exploring` remains unchanged.
6. After approval of the actual files, change only the anchor to `Status: approved` and stop. Exploration approval does not approve any candidate change.

## Archive only when requested later

Require an active `Status: approved` exploration whose decisions have dispositions. Run `azimuth explore archive <id> --date <YYYY-MM-DD>`. Never archive implicitly after approval or proposal creation.
