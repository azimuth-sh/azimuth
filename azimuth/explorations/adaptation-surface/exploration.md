# Exploration: Adaptation surface

Id: adaptation-surface Status: active

## Objective

Establish what a team adopting Azimuth can actually adapt to its own needs, what the framework fixes on their behalf, and whether the adaptable surface matches the intent that motivated it.

## Boundaries

- Derive the surface from artifacts: `contracts/`, the parsers under `tools/azimuth/src/`, and the instance artifacts in `azimuth/`. The narrative that previously described this reasoning was deleted as unverified.
- Record the surface. Do not widen a vocabulary, add a field or propose notation from argument.
- Do not write or run tests; this exploration observes only.
- Adoption experience is out of scope. No team outside this repository has used the surface, so every claim here is about what the artifacts permit, not about what proved workable.

## Existing context

- Project-declarable artifacts are exactly what `azimuth init` scaffolds (`tools/azimuth/src/workflow.rs:58-85`): `model/`, `changes/archive/`, `explorations/archive/`, `standards/verification.md` and `workspace.json`, plus `adapters.json` when a provider is used.
- The standards grammar is now `contracts/standards.md`; the instance file holds only this project's declarations.
- Every Claim in `azimuth/model/` is routine, so the policy surface governs nothing here today.

## Findings

### F1 — The declarative policy surface is two block types and one field

A `## Decision Policy: <id>` block carries one or more `Required challenge:` lines and nothing else. There is no severity threshold, no candidate count, no scope requirement, no cadence and no expiry. A `## Challenge Schedule: current` block — exactly one per project — assigns every form to `Gate challenge:` or `Scheduled challenge:`.

That is the whole of `azimuth/standards/`. An Evidence Binding or Claim Judgment selects a policy by id through its `Policy:` label.

### F2 — Every value vocabulary is closed and framework-fixed

Criticality (three), scope (three), quantification (two), oracle (six), challenge domain (five), enforcement kinds (six), candidate dispositions (seven), semantic scope kinds (seventeen), capability classes (five), Subject kinds (six), Observation outcomes (three), Challenge Results (three), Claim domains (two), and the Finding registry with severity derived from criticality by a fixed rule in `tools/azimuth/src/validation.rs`. A project changes none of them.

The closure is deliberate — comparability across teams was preferred to extensibility — but it means the adaptable surface is narrow by construction rather than by omission.

### F3 — The intended criticality-to-evidence mapping was never built

The obvious knob is "what does this criticality level require here", so that a team can say critical Claims need an independent oracle while standard Claims do not. No such mapping exists. Criticality decides which *facets* are applicable — `contracts/spec.md` — and nothing more. Required objections attach to a binding, never to a level.

A team therefore cannot express a rule of the form "every critical Claim requires mutation testing". They can only attach the same policy id to each binding by hand and hope none is missed, and no Finding detects the omission because a binding without the intended policy is simply a binding with a different policy.

### F4 — Policy selection is per binding, with no default and no inheritance

`Policy:` names one policy on one Evidence Binding or Claim Judgment. There is no project default, no per-spec default and no inheritance from requirement to case. At any scale this is the same problem as a hand-maintained matrix, one level up: the correctness of the account depends on every author remembering the same id.

### F5 — Criticality has no bound

Nothing caps the share of requirements at any level, and no review gate exists at the change boundary. The analyst who declares a level does not pay for it, so the level carries information only while everyone is disciplined. This is recorded as a working rule in `AGENTS.md` and enforced by nothing.

### F6 — There is no time dimension anywhere

`contracts/standards.md` and the instance file both state it: no cache, cadence, TTL, historical applicability, cross-Subject reuse or time-based deferral. A team cannot declare that a decision must be re-established quarterly, that a manual result expires, or that a Challenger runs on a recurrence. Staleness is fingerprint equality and nothing else.

### F7 — Challenge forms are the one genuinely open vocabulary

A Challenger declares an open lower-kebab form id, and a configured capability declares which forms it supports. This is the single place a team invents its own terms, and it is the hinge the whole policy surface turns on: a Decision Policy is a list of form ids, and the schedule is a partition of them. Everything adaptable about assurance rigor is expressed in that one vocabulary.

### F8 — Adapter configuration is the other real surface, and it is pinned by content

`azimuth/adapters.json` fixes provider and adapter identity, executable and resource content, the expected description, semantic settings, exact non-secret environment literals, process limits and the capability dictionary. It is genuinely per-project, but it configures *how* work is dispatched rather than *what* the project demands. Version 1 supports no secret value, secret reference or interpolation, which bounds which providers can be configured at all.

### F9 — Workspace is filed as topology and behaves as a standard

`workspace.json` declares areas and mounts, surfaces assembled from enumerator contributions, and realization obligations naming areas that must contain a realization. It expresses no rigor and no policy, so it reads as topology.

But its content enters decision identity. The Claim Judgment preimage carries `realization_obligation_areas` and an applicable surface account (`contracts/verification.md:483`, `:364`), so redefining a surface or an obligation stales every Judgment that composed it. A team adjusting its source topology re-opens accountable decisions without any indication that it is doing so.

### F10 — Standard and configuration are distinguishable, and the split surprises

The useful test is whether editing an artifact stales an accountable human decision. A standard is a commitment decisions are measured against; a configuration is a knob that changes behaviour and invalidates no judgment.

| Artifact | Enters decision identity | Kind |
|---|---|---|
| `standards/verification.md` | `decision_policy_digest`, binding preimage | standard |
| `workspace.json` | obligations and surface account, Judgment preimage | standard |
| `adapters.json` | nothing; binds Run and launch identity only | configuration |

Only one of the three is configuration in the plain sense. This is why `standards` is the right word for the policy file: calling it configuration would imply it can be adjusted freely, when editing it re-opens judgments that a named person accepted.

### F11 — In this repository the surface is inert

`azimuth/standards/verification.md` declares one policy, `credible-executable`, that zero Evidence Bindings reference, and a schedule whose two forms zero Challengers declare. It is loaded, parsed, digested and fingerprinted, and it governs nothing. That is honest for a project whose Claims are all routine, but it means the surface has never been exercised even by its author.

### F12 — The parser tolerates one unchecked region

`tools/azimuth/src/verification.rs:1006` excepts `## Semantics` from the unrecognized-heading error. Anything beneath that heading is skipped. This is how the instance file carried a specification of its own format, unchecked against the parser, until that specification moved to `contracts/standards.md`. One claim in it was wrong on inspection: the retired `## Policy:` and `## Qualification Policy:` spellings were described as "rejected rather than aliased", implying a named check, when the parser emits one generic `unrecognized Decision Standards heading` diagnostic for any unknown heading.

The exception now has no remaining user.

## Decisions

- **E1 — Record the surface; change no vocabulary from argument.** The closure in F2 is a decided trade and nothing observed here falsifies it.
- **E2 — Name the criticality-to-policy gap (F3) as the primary adoption question.** It is the one place where the surface does not do what its own vocabulary implies, and it is the first thing an adopting team will reach for.
- **E3 — Treat counter-pressure (F5) as a prerequisite rather than an enhancement.** Levels that cost nothing to raise stop distinguishing anything, and the surface has no other rigor dial.
- **E4 — Do not add a time dimension here.** F6 is a general repair recorded elsewhere, not a configuration feature; introducing cadence as a policy field would scope a core question to one construct.
- **E5 — Challenge forms remain the open vocabulary.** Any further openness must be demanded by two structurally different concerns first.

## Rejected alternatives

- **Adding fields to Decision Policy now** — a candidate cap, a required scope, a minimum quantification. Each is plausible and none is demanded by an observed case; the cap already exists per request rather than per policy.
- **Making criticality configurable** — letting projects define their own levels. This forfeits the comparability the closure was chosen for, and F3's gap is about what a level *requires*, not about which levels exist.
- **Treating `workspace.json` as the policy surface.** It adapts to a codebase's shape and expresses no rigor.

## Open questions

1. Should required objections be derivable from criticality, or is per-binding selection correct and the missing piece merely a default?
2. What form should counter-pressure take — a declared cap, review at the change boundary, or a derived report that makes the distribution visible?
3. Should a Decision Policy gain any field beyond `Required challenge:`, and what evidence would justify the first one?
4. Is the closed-vocabulary trade still right once a second team adopts the framework, and what observation would show it is not?
5. Should the `## Semantics` exception be removed from the parser now that it has no user?
6. Does the adapter surface need a secret-reference form before any commercial provider can be configured, or is that deliberately out of scope for version 1?

## Result

No change is created. The surface is recorded as observed: two block types, one field, one open vocabulary, and four framework-fixed dimensions around them. The exploration finishes when questions 1 and 2 are dispositioned, since those decide whether the surface is thin by design or thin by omission.

## What would falsify this

- **F1 is wrong** if a project-level construct exists that this exploration did not find.
- **F3 is wrong** if the criticality-to-evidence relation is expressible through a composition of existing constructs that no artifact names.
- **E1 is wrong** if an adopting team cannot express a rigor requirement they consider essential, and the gap is a closed vocabulary rather than F3.
- **E3 is wrong** if criticality distribution stays disciplined without any mechanism, measured once non-routine Claims exist.
