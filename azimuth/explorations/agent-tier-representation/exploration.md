# Exploration: Agent tier

Id: agent-tier-representation
Status: active

> **Provenance warning.** This exploration was written against `docs/decisions.md`, a 3,013-line
> narrative that was later found to have been generated in one unsupervised commit and was deleted.
> Every `D<n>` citation below is unresolvable, and findings resting on them are unverified. Three
> corrections are known: the corpus claimed six Claim domains where the parser has two
> (`behaviour | sites`); every measurement it reported is unreproducible in this repository; and an
> accepted Claim Judgment is a required positive decision, not a negative-only one. Read this as
> recorded reasoning, not as a source. Re-derive anything load-bearing from `azimuth/formats/`,
> `tools/azimuth/tests/` and `azimuth/changes/archive/`.


## Objective

Define what the agent tier is in the alpha 2 verification graph: which roles it holds, which plane
each role belongs to, what each may and may not constitute, and what representation — if any — the
model owes them. The tier is currently prose-only while the framework's own falsifiers make its
core claim depend on it.

## Boundaries

- Cover the whole tier, not one role. An earlier draft of this exploration treated agent review as
  a single activity expressible as a Challenger. That was wrong for a structural reason recorded in
  F2, and the correction is the reason this exploration was widened.
- Treat the missing representation as an unfinished transition. D45 deferred it against a named
  condition and D48 satisfied that condition without returning to it.
- Do not design notation before the experiment in E2 reports.
- Agent quality is out of scope; D43 fixes the regress stop. Agent *independence* is in scope, and
  is a different question (F6).
- Make no claim here about whether an agent may hold an accountable identity. That is the frontier.

## Existing context

- D43, current, states the lifecycle: "The machine tier validates structure, identities,
  fingerprints and applicability. The agent tier proposes a verdict and rationale. The evidence
  owner accepts it through review. CI may then challenge that candidate decision against the
  revision being accepted."
- The glossary scopes the capability: the agent tier "can propose a binding-level Qualification for
  evidence-owner review. It does not turn its own review into product evidence."
- D14 introduced representation to give the tier "ownership, a freshness clock, and visibility", and
  fixed its ceiling: "An agent judgment is never proof-strength, whatever its confidence."
- D18 revised D14 to make the effect negative, with verdicts `sound`, `toothless`, `dishonest-tag`
  and `spec-gap`. D18.1 measured ten Claims judged "on a matrix the machine tier reported as green":
  six `sound`, two `toothless`, two `dishonest-tag`, concluding "A standard that is expensive to
  satisfy honestly is cheap to satisfy dishonestly" and that the tier "is not optional".
- D28 folded realization honesty into the agent judgment. D30 separated accountability from
  authorship and contemplates an owner who may "implement it directly or direct an agent".
- D45: "`azimuth judge` is removed until a total Claim Judgment format is accepted." D48 accepted
  that format the next day and does not mention the tier; neither do D45, D46 or D47.
- D44 retains a Finding category `judgment` "for agent decisions and freshness" with nothing behind
  it. `.agents/skills/` no longer holds the verify pass recorded at `docs/decisions.md:131`.
- Open question 7 names this seam: of D14, "worth testing whether it holds." A phase-0 falsifier
  reads: "The core claim fails if the agent tier cannot reliably detect a dishonest tag."

## Findings

### F1 — The removal was a deferral, and its condition has been met

D45's deletion of the alpha 1 judgment machinery was correct on its own terms: that machinery was
entangled with `Covers`, evidence strength, imported observations and a judgment parser alpha 2
replaced wholesale. But the deletion was conditional, the condition is satisfied, and nothing since
has argued the tier should stay unrepresented. It was not revisited.

### F2 — A Challenger cannot constitute a decision, so proposal and challenge are different planes

`run-launch-plan.md:79`: "Only a `selected` candidate is runnable. Any other disposition in a
requested Plan fails planning." A binding with no authored decision resolves as `missing-decision`
(`verification.md:261`), so a Challenge Plan aimed at an unqualified binding does not return an
empty search — it fails planning.

The bar is deeper than selection. `framework.md:141`: "A Run never authors or repairs one." D43
splits authority so repositories own decisions and Run producers own facts about Subjects. A
Challenge Result is exactly `clean | findings | inconclusive`, with no verdict and no rationale.

So the Run and adapter path cannot produce a Qualification at any configuration. Proposal is
repository authoring; challenge is execution. They are not two settings of one mechanism, and the
sequencing propose → accept → challenge is forced by the model rather than chosen.

### F3 — D18's directionality was architecture-specific and does not transfer

In alpha 1, coverage was machine-derived from `Covers` tags, so a Claim could read green before
anyone judged it. That prior positive state is what made D18's negative-only reading correct, and
D18.1's measurement is explicit that the matrix was already green.

Alpha 2 has no machine-derived coverage. Nothing is qualified until a Qualification is authored
against a binding, so there is no prior belief for an agent to withdraw. The tier's role became
constitutive the moment D45 made Qualification an authored decision, which is what D43 already
describes. D18 therefore revised D14 for a reason that no longer obtains, and the change that
invalidated the argument is the same change that removed the tier's representation.

This does not restore D14. It reopens the question D18 closed.

### F4 — The tier holds three roles with different homes

| Role | Plane | Target | Current status |
|---|---|---|---|
| Proposer | repository authoring | binding with no current decision | unmodelled |
| Acceptor | repository authoring | the proposed verdict | identity exists; may it be an agent? |
| Challenger | execution | an existing decision fingerprint | fully modelled |

Conflating any two of these produces either F2's category error or F6's independence problem.

### F5 — Proposal needs no protocol

Because proposal is authoring rather than execution, it requires no adapter capability, no Run, no
bundle and no new record type to be useful. It requires a skill and a place to put a candidate
verdict for review. The deleted verify pass was exactly that. The surface is far smaller than the
Challenger route implies, and it is available without touching any format.

### F6 — Proposer and challenger cannot be the same agent

If one agent proposes a Qualification and an agent challenges it, the challenge is correlated with
the proposal and establishes little. Alpha 1 did not face this: the machine-derived matrix was an
independent baseline the agent attacked. Alpha 2 removed that baseline, so independence must now be
arranged deliberately — by differing model, differing inputs, or a non-agent challenger.

D43's regress stop does not address this. That boundary concerns a Challenger's *quality*; this
concerns its *correlation* with the decision it audits.

### F7 — Nothing in the format requires a human

`Qualifier:` and `Judge:` are each "a non-empty accountable identity". The corpus never says human,
and D30 makes ownership accountability rather than authorship. A named owner accepting an
agent-drafted verdict is already within the model's letter. Whether an agent may hold the identity
is undecided in both directions and should not be settled by omission.

### F8 — The proposer role is where the scale story rests, and it leaves no trace

If agent drafting is what reduces a re-decision from careful re-reading to review, then the
tractability of the non-routine tier at 10^4 Claims depends on a step the model does not record.
An agent-drafted, owner-accepted Qualification is today indistinguishable from one a person reasoned
through. Nothing records who proposed, whether the proposal was accepted unchanged, or when the
decision last received real attention.

### F9 — `spec-gap` has no home, and the gap was found twice independently

D18's fourth verdict — "the code is right, the evidence is toothy, and a reader would still be
surprised" — is a finding about intent completeness with no decision to attach to. The
`security-assurance-fit` exploration reached the same gap from an unrelated direction. Two
structurally different concerns demanding it is the condition the framework sets for notation.

### F10 — Recording proposals would add churn to the model whose churn motivates the question

D14's freshness clock and alpha 2's fingerprint staleness are the same mechanism seen twice.
Anything entering a fingerprint stales its dependents, so a proposal record participating in
identity would make drafting activity a source of re-decision. D19.1 measured eight stale judgments
from a notation-only edit and D22 thirty-six from one shared file. This is the strongest argument
for keeping proposal provenance outside fingerprints, or outside the model entirely.

### F11 — The tier is prose-only

No command, no skill, no record, no Finding kind, no export representation. It survives in two
sentences of D43 and one glossary entry, while D18.1 calls it not optional and a phase-0 falsifier
makes the framework's core claim depend on it.

## Decisions

- **E1 — Model the agent tier as three roles, not one.** Proposer, acceptor and challenger have
  different planes, targets and constraints. Any design that treats them as one activity will
  reproduce F2 or F6.
- **E2 — Restore proposal first, as a skill, with no format change.** A verify pass that reads a
  Check, its implementations, the binding and required context and drafts a candidate Qualification
  verdict and rationale for owner review. This is the smallest step that makes the tier real, and
  per F5 it needs no notation.
- **E3 — Reopen D18's directionality.** It was correct for alpha 1's machine-derived coverage and
  does not transfer to alpha 2. This reverses an earlier draft of this exploration, which held that
  the finding should stand.
- **E4 — Keep agent challenge as a Challenger, scoped to existing decisions only.** It remains valid
  and already modelled, but it answers a different question and cannot substitute for E2.
- **E5 — Require proposer and challenger independence.** Whatever form it takes, a challenge of an
  agent-proposed decision must not be produced by the same agent on the same inputs.
- **E6 — Hold the provenance record pending measurement from E2.** The case for it is an inference
  about scale, not an observed failure, and F10 gives a real reason for restraint.
- **E7 — Route `spec-gap` through the general unattributed-finding work**, not through an
  agent-tier-specific mechanism.

## Rejected alternatives

- **Express the whole tier as a Challenger.** Rejected by F2; this was the previous draft's position
  and it is structurally impossible for the proposer role.
- **Restore `azimuth judge` as it was.** Its inputs, verdicts and freshness model were entangled
  with removed alpha 1 machinery. D45's deletion was right; only the deferral went unfinished.
- **Make an agent an admissible `Judge:` now.** No evidence supports it, and settling it by omission
  is how the present ambiguity arose.
- **Put proposal provenance into fingerprints.** Rejected by F10 unless measurement shows no
  practical churn cost.
- **Keep D18 as decided and treat the tier as negative-only.** Rejected by F3. It would leave the
  constitutive role that D43 already describes permanently unowned.

## Open questions

1. May an agent hold an accountable identity for a Qualification, for a Claim Judgment, for both or
   for neither?
2. Does a candidate verdict live in the repository as a reviewable artifact, or only in review
   metadata outside the model?
3. If recorded, does proposal provenance participate in any fingerprint, and how is churn bounded?
4. What is the observable for "accepted unchanged", and can it be derived rather than authored?
5. What form of proposer/challenger independence is sufficient, and can the model express it or must
   it remain project policy?
6. Claim Judgment has no stated lifecycle anywhere in the corpus. Qualification's is stated in D43
   and may not transfer to a total composition.

## Result

No change is created yet and the frontier is deliberately undisposed. One experiment is identified:
restore the verify pass as a skill under E2, run it against real bindings, and measure what it
catches that the machine tier passes and how often owners accept a proposal unchanged. The second
measurement decides F8 and therefore E6. This exploration finishes when that experiment reports.

## What would falsify this

- **F2 is wrong** if a Challenge Plan can be made to reach a `missing-decision` candidate without
  weakening the rule that only current positive decisions execute.
- **F3 is wrong** if alpha 2 turns out to retain some machine-derived positive state that an agent
  can withdraw, which would preserve D18's reading.
- **E2 is wrong** if a skill without any model record cannot produce reviewable proposals in
  practice, which would make representation a precondition rather than an outcome.
- **F6 is overstated** if measurement shows agent challenges of agent proposals find defects at a
  rate comparable to independent challenges.
- **F8 is wrong** if agent drafting turns out not to be load-bearing, either because re-decision
  volume is low or because owners rewrite proposals rather than accepting them.
