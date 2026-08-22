# Exploration: Agent tier

Id: agent-tier-representation Status: active

## Objective

Define what the agent tier is for the alpha 2 verification graph: which roles it holds, which plane each belongs to, what each may constitute, whether those roles should be separated from one another, how a specialized reviewer could earn trust, and what representation — if any — the model owes any of it.

## Boundaries

- Rewritten from artifacts. An earlier draft rested on `docs/decisions.md`, which was deleted as unverified. Claims that depended only on that narrative are dropped rather than restated, and the arguments that survive are re-derived from the contracts, the parsers and the change archive.
- Design no notation before the experiment in E2 reports.
- Agent *quality* is out of scope. Agent *independence* and *specialization* are in scope; they are different questions and the second is not a solution to the first.
- Do not write or run tests.

## Existing context

- The only surviving normative statement about the tier is `docs/glossary.md:248`, and it is negative: review "happens outside the tool and reaches the repository only as an authored `Qualifier:` or `Judge:` identity on a decision." Every positive description of what the tier does lived in the deleted narrative.
- The archive preserves the removal as a deferral. `azimuth/changes/archive/2026-08-21-verification-evidence-bindings/proposal.md:48` states the intent to "remove the obsolete `azimuth judge` surface until total Claim Judgment has an accepted format", and `:91` requires that no replacement command be invented.
- `azimuth/changes/archive/2026-08-22-traceability-challenge-planning/proposal.md:23` accepted that format: "Azimuth authors one total-composition Claim Judgment for every applicable non-routine case Claim."
- `.agents/skills/` holds `azimuth-apply`, `azimuth-archive`, `azimuth-coordinate`, `azimuth-explore` and `azimuth-propose`. There is no verify pass.
- All 41 requirements in `azimuth/model/` are routine, so every Qualification and Claim Judgment in this repository lives in a fixture and none describes Azimuth.

## Findings

### F1 — The removal was a deferral, and its condition has been met

The deferral and its condition are both in the archive, and the condition was satisfied the following day. Nothing since has argued that the tier should remain unrepresented; it was not revisited. The burden for restoring something is therefore lower than for designing it, and the design space is correspondingly narrower.

### F2 — A Challenger cannot constitute a decision

`contracts/run-launch-plan.md:80`: "Only a `selected` candidate is runnable. Any other disposition in a requested Plan fails planning." A binding with no authored decision resolves as `missing-decision` (`contracts/verification.md:261`), so a Challenge Plan aimed at an unqualified binding does not return an empty search — it fails planning outright.

The bar is deeper than selection. A Challenge Result is exactly `clean | findings | inconclusive` with no verdict field and no rationale, and the authority split gives repositories the decisions and Run producers the facts. The Run and adapter path cannot produce a Qualification at any configuration.

Proposal is repository authoring; challenge is execution. They are not two settings of one mechanism, and the sequencing propose → accept → challenge is forced rather than chosen.

### F3 — Alpha 2 has no prior positive state, so the tier's first role is constitutive

Nothing in alpha 2 reports a Claim as covered before a human or agent decides. A Qualification is authored; until it exists the binding has no decision, and a Claim Judgment is authored over a total composition. There is no machine-derived coverage for a reviewer to withdraw.

That makes the tier's first act necessarily constitutive: producing a decision where none exists. Any account of the tier as purely negative — able to take away belief but never to establish it — describes an architecture in which the machine already asserts coverage. This one does not.

### F4 — The tier holds three roles in two planes

| Role | Plane | Target | Status |
|---|---|---|---|
| Proposer | repository authoring | a binding or Claim with no current decision | unmodelled |
| Acceptor | repository authoring | the proposed verdict | identity exists; may it be an agent? |
| Challenger | execution | an existing decision fingerprint | fully modelled |

Conflating any two produces either F2's category error or F6's correlation problem.

### F5 — Proposal needs no protocol

Because proposal is authoring rather than execution, it needs no adapter capability, no Run, no bundle and no new record type to be useful. It needs a skill and somewhere to put a candidate verdict for review. The deleted verify pass was exactly that. The surface is far smaller than the Challenger route implies and is available without touching any contract.

### F6 — Proposer and challenger cannot be the same agent on the same inputs

If one agent proposes a Qualification and an agent challenges it, the challenge is correlated with the proposal and establishes little. Independence must be arranged deliberately — by differing model, differing inputs, or a non-agent challenger.

This is not the recursion question. Whether a Challenger is itself qualified concerns its *quality*; this concerns its *correlation* with the decision it audits, and no stopping rule addresses it.

### F7 — Nothing in the format requires a human

`contracts/verification.md:112` and `:133` require `Qualifier:` and `Judge:` to be "a non-empty accountable identity". The corpus never says human. A named owner accepting an agent-drafted verdict is already within the model's letter. Whether an agent may hold the identity itself is undecided in both directions and should not be settled by omission.

### F8 — The separation exists for accountability, not capability

`not qualified` and `qualified` are different kinds of statement. A negative verdict carries a demonstration — this test would also pass against a wrong system, and here is how — which can be checked. A positive verdict is a claim of exhaustive absence, and nothing accompanies it that can be checked.

The framework already refuses that move elsewhere: a clean Challenge Result "records only that the configured search found no objection; it creates neither positive product evidence nor repository credibility." If a clean search creates no credibility, a clean agent read should not either.

But the argument proves too much. A human qualifier is also saying "I looked and found nothing." If negative search cannot create credibility, humans cannot qualify either. What distinguishes them is not epistemic: a Qualification is an *accountable* claim, and someone's name is on it. The separation is worth exactly as much as that accountability does work, and nothing more.

### F9 — Seven weak spots in the separation

1. **Rubber-stamping inverts it.** At scale the signature carries no information, and the model records a *false* accountability — strictly worse than an honestly recorded agent verdict.
2. **The acceptor may be less competent than the proposer.** Judging whether an oracle is discriminating requires code fluency that an evidence owner may not have.
3. **Nobody has defined what the acceptor does.** Redoing the analysis saves nothing; not redoing it is acceptance on trust. No middle mode is defined, and the separation only pays if one exists.
4. **An agent verdict has no stable identity.** Everything else in the model is content-addressed and deterministic; re-running the same agent may yield a different verdict with different reasoning.
5. **The agent reads the artifacts the binding author wrote.** A dishonest binding whose prose and test agree with each other, and both quietly overstate, has no external anchor to catch it.
6. **Required context is uninterpretable.** Exact string equality, no ranges. An agent can judge whether the declared context is satisfied, never whether it is sufficient.
7. **Realization sites are declarations, not facts.** An enumerator bounds domain membership; semantic realization remains declared, so the agent is reading claims someone made.

### F10 — Specialization is consistent with the framework and already expressible

The framework is ecosystem-specific where semantics demand it: `contracts/verification.md` carries a distinct qualified-site profile per ecosystem, and language support is an extractor property rather than a core feature. A reviewer that knows a framework's dispatch, validation and query idioms is the same kind of artifact — ecosystem knowledge at the edge, core unchanged.

It also has an identity slot already. Challenger forms are open lower-kebab ids, and a Challenger's fingerprint binds its id, open form, objection proposition and required scope kinds. A specialized form is expressible today with no format change.

### F11 — Specialization buys capability, not trust

A specialized reviewer can be wrong, drift from a codebase's conventions, or degrade silently. If specialization is adopted, the specialization becomes an artifact whose quality matters, and it needs what any quality-bearing artifact needs: a version, a content fingerprint, a declared scope, and a conformance suite of **seeded defects** it must catch — a deliberately toothless test, an overstated form, an unearned realization tag.

That measured detection rate is the trust mechanism. It is falsifiable in a way that a better prompt is not, and it is the same discipline mutation testing applies to tests, turned on the reviewer.

Specialization also buys F6's independence cheaply: a differently specialized challenger reads different aspects with different priors, which is more practical than "use a different model".

### F12 — Realization tag placement is semantic, and its cost lands in one decision only

Whether a site deserves a `realizes` tag is not a question of depth in a call tree. The test is whether the site establishes part of the named predicate, and a reviewer must be able to say which part. A handler that enforces the predicate qualifies; a generic helper three levels down does not, however load-bearing it is.

The consequence for staleness is asymmetric and useful. A Qualification's fingerprint combines Check, binding and context and contains **no** realizations, so tag placement cannot stale one. A Claim Judgment's preimage carries `realizations` with their source fingerprints, so any edit to a realizing symbol stales the Judgment. The entire depth question lives in Claim Judgment and nowhere else.

Realization fingerprints are therefore not a behavioural change detector, and treating them as one is a category error: a helper that silently changes behaviour should break a Check, not stale a judgment. That division only works once execution facts reach the model, which is deferred, so the fingerprint is currently doing double duty.

### F13 — Tag count drives judgment labour, not only staleness

If a judge must say which part of the predicate each site establishes, then twenty tags means twenty contribution statements in every re-judgment. Over-tagging is expensive twice: more staleness and more work per re-decision. A Claim needing fifteen realization tags is also reporting something about the code — diffuse enforcement, which the enforcement ladder already ranks below a choke point.

### F14 — The proposer role is where the scale story rests, and it leaves no trace

If agent drafting is what reduces a re-decision from careful re-reading to review, then the tractability of the non-routine tier depends on a step the model does not record. An agent-drafted, owner-accepted Qualification is indistinguishable from one a person reasoned through. Nothing records who proposed, whether the proposal was accepted unchanged, or when the decision last received real attention.

### F15 — Recording proposals would add churn to the model whose churn motivates the question

Anything entering a fingerprint stales its dependents, so a proposal record participating in identity would make drafting activity a source of re-decision. This is the strongest argument for keeping proposal provenance outside fingerprints, or outside the model entirely.

It suggests a separable repair for F12's churn: bind realization *identities* in the Judgment fingerprint and track *currency* separately. A judge would then re-read the three sites that moved rather than re-author a total composition, and the composition would stay exact.

### F16 — The tier is now entirely unrepresented, including in prose

It has no command, no skill, no record, no Finding kind and no export representation. As of the narrative's deletion it has no positive description either: the sole surviving statement says the model records no agent-tier artifact. A tier that the framework's own reasoning treats as load-bearing now exists only as an absence.

### F17 — An unattributed finding has no home

`contracts/findings.md` lists 42 kinds and none of them records "something was found here and no proposition governs it". A reviewer who notices behaviour no Claim covers has nowhere to put it. This gap was reached independently from the security direction, so two structurally different concerns demand it.

## Decisions

- **E1 — Model the tier as three roles, not one.** Proposer, acceptor and challenger have different planes, targets and constraints.
- **E2 — Restore proposal first, as a skill, with no format change.** A verify pass that reads a Check, its implementations, the binding and required context and drafts a candidate verdict for review. Smallest step that makes the tier real; per F5 it needs no notation.
- **E3 — Cut the separation at assertion versus demonstration, not at propose versus accept.** The agent should not emit a bare `qualified`.
- **E4 — Require the agent to state what it could not check.** That account, not the conclusion, is what an acceptor can review cheaply and substantively, and it is the missing middle mode in F9.3.
- **E5 — Gate the separation on criticality.** For a critical Claim an accountable signature carries organisational weight; for standard it may not. Criticality already decides facet applicability.
- **E6 — If measurement shows near-universal acceptance unchanged, delete the separation rather than defend it,** and let the agent qualify under its own identity. An honest agent signature beats a hollow human one.
- **E7 — Keep agent challenge as a Challenger, scoped to existing decisions only.** Valid and already modelled; it answers a different question and cannot substitute for E2.
- **E8 — Require proposer and challenger independence.** Differently specialized reviewers are the cheapest sufficient form.
- **E9 — Treat a specialized reviewer as a versioned artifact with a seeded-defect conformance suite.** Trust is the measured detection rate, not the specialization.
- **E10 — Hold the provenance record pending measurement from E2.** The case is an inference about scale, and F15 gives a real reason for restraint.
- **E11 — Route the unattributed finding through general work,** not an agent-tier mechanism.

## Rejected alternatives

- **Express the whole tier as a Challenger.** Rejected by F2; structurally impossible for the proposer role. This was an earlier draft's position.
- **Restore `azimuth judge` as it was.** Its inputs, verdicts and freshness model were entangled with removed alpha 1 machinery. The removal was right; only the deferral went unfinished.
- **Make an agent an admissible `Judge:` now.** No evidence supports it, and settling it by omission is how the present ambiguity arose.
- **Put proposal provenance into fingerprints.** Rejected by F15 unless measurement shows no churn.
- **Treat Git and pull-request history as sufficient and close the question.** Plausible and untested; E2 is cheaper than assuming either way.
- **Specialize on the full cross product of ecosystem, framework and test type.** Specialize on the axis that changes the reading; the rest is a maintenance surface that will rot.

## Open questions

1. May an agent hold an accountable identity for a Qualification, a Claim Judgment, both or neither?
2. Does a candidate verdict live in the repository, or only in review metadata outside the model?
3. If recorded, does proposal provenance enter any fingerprint, and how is churn bounded?
4. What is the observable for "accepted unchanged", and can it be derived rather than authored?
5. What form of independence is sufficient, and can the model express it or must it stay policy?
6. Claim Judgment has no stated lifecycle anywhere. Qualification's died with the narrative, and a total composition may not admit the same propose-accept shape.
7. Would decoupling realization identity from currency in the Judgment fingerprint (F15) change the answer to F14, by making re-decision cheap enough that drafting need not be recorded?

## Result

No change is created and the frontier is deliberately undisposed. One experiment is identified: restore the verify pass as a skill under E2, run it against real bindings, and measure what it catches that the machine tier passes and how often owners accept a proposal unchanged. The second measurement decides F14 and therefore E10. This exploration finishes when that experiment reports.

## What would falsify this

- **F2 is wrong** if a Challenge Plan can reach a `missing-decision` candidate without weakening the rule that only current positive decisions execute.
- **F3 is wrong** if alpha 2 retains some machine-derived positive state that a reviewer can withdraw.
- **F8 is wrong** if the separation turns out to add discrimination rather than only accountability — that is, if acceptors reliably catch defects proposers miss.
- **F11 is wrong** if a seeded-defect suite proves unbuildable because the defects worth catching cannot be synthesized convincingly.
- **F14 is wrong** if agent drafting is not load-bearing, either because re-decision volume is low or because owners rewrite proposals rather than accepting them.
- **E2 is wrong** if a skill with no model record cannot produce reviewable proposals in practice, making representation a precondition rather than an outcome.
