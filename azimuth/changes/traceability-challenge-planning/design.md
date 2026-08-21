# Design: Traceability Challenge Planning

## Decision authority

The repository owns reviewed decision meaning. A Claim Judgment is a repository decision over one
case Claim's complete applicable composition. It is not a Run result and it is not Subject-specific
Assurance State.

Claim Judgments live in the owning `verification.md` beside Checks, bindings, Qualifications,
Challengers and Challenge Plans. The new block is not an interpretation of the retired alpha 1
`judgments.md` format. The old plural facet remains a hard error and no compatibility reader exists.

Every applicable standard or critical case Claim has exactly one Claim Judgment. The verdict is
`accepted | rejected`. Only a fingerprint-current accepted Judgment is an executable Challenge
target. Routine Claims stop at intent and reject Judgment declarations.

## Total-composition identity

The canonical Claim Judgment fingerprint binds:

- Claim semantic identity, proposition, case steps, criticality and applicable obligations;
- stable realization identities and source fingerprints;
- applicable design mechanisms, attachment, enforcement declarations, structural bindings, exact
  resolved artifacts and optional marker-derived source identity and fingerprint;
- relevant surface and area-obligation identities;
- Evidence Binding fingerprints;
- recomputed expected Qualification fingerprints and authored verdicts;
- the resolved decision-policy digest;
- the Judgment verdict, ordered basis and ordered residual-risk statements.

The recomputed expected Qualification fingerprint is used even when the authored Qualification has
gone stale. A relevant Check, binding, context or policy edit therefore changes the dependent
Judgment expectation too. Missing or structurally invalid dependencies make the expected Judgment
unavailable rather than substituting an authored stale value.

Date, judge, incidental declaration and locator paths, line and explanatory rationale are excluded.
Model-authoritative member identities defined by D13 are not incidental locators and remain
included. Runs, Observations and Challenge Results are execution facts and never participate in
repository decision identity.

A Challenger declares one or more closed semantic scope kinds it requires in addition to its open
form and objection proposition. A mutation Challenger can require `realization` and
`check-implementation`; a mechanism fault Challenger can require `mechanism` and
`check-implementation`; broad static analysis can require `claim`. Core validates only the closed
scope kinds and never interprets the open form name.

The Challenger fingerprint binds id, open form, objection proposition and the sorted required
scope kinds. Its rationale and location are excluded.

## Policies and scheduling

F replaces the pre-F `Qualification Policy` name in place with one project-global `Decision Policy`
namespace. In `verification.md`, Evidence Bindings and Claim Judgments both use `Policy: <id>`.
In `azimuth/standards/verification.md`, a strict `## Decision Policy: <id>` block contains distinct
repeatable `Required challenge: <form>` lines. The policy digest binds its id and sorted forms only.
The Evidence Binding preimage uses literal `decision_policy_digest`; there is no reader for the
earlier heading, binding label or `qualification_policy_digest` field.

Validation requires at least one current Challenger and authored Challenge Plan of each required
form to resolve every accepted decision with all of that Challenger's required scope kinds.
Declared plan coverage means the objection can be selected; it does not mean the Challenger ran or
returned clean.

Scheduling is deliberately separate. The same standards file contains exactly one strict
`## Challenge Schedule: current` block with distinct `Gate challenge: <form>` and
`Scheduled challenge: <form>` lines. Either lane may be empty, but their union is non-empty and
every form required by a policy or declared by a Challenger appears exactly once.
Gate forms are expected in the current acceptance lane; scheduled forms may be deferred because
they are expensive. The scheduling account participates in complete-model, semantic Plan and
Challenge-selection identity, but not in Qualification, Claim Judgment or policy digests. F
introduces no cache validity, Subject equivalence or temporal reuse rule.

Planning resolves the fixed union of requested Plans. For every selected decision, the union must
contain at least one runnable `(target fingerprint, required form)` selection for every form in the
decision policy. The request names an explicit capability for each Plan; planning never auto-picks
or auto-adds one. Missing coverage fails rather than yielding a smaller Plan. Every semantic
Challenge records its lane. The adapter may omit scheduled work only through an incomplete Run
with an exact diagnostic and no Challenge Result for the omitted selection. The planned selection
remains visible in the bundle. Gate work can also be incomplete because execution failed; its lane
changes later acceptance policy, not protocol truth.

## Resolution and candidate dispositions

Every selector is evaluated against the complete model and returns a deterministic resolution
account. `azimuth validate`, model export and traceability expose the same account; `run plan`
refuses a requested Plan containing any non-selected disposition. Each reached decision candidate
has one of these dispositions:

- `selected` for a current accepted decision;
- `missing-decision`;
- `stale-decision`;
- `rejected-decision`;
- `invalid-decision`;
- `inapplicable`; or
- `unresolved-relation`.

Successful candidates do not hide adverse siblings. There is no implicit whole-suite fallback.

The candidate universe is exact:

- `qualification from binding` reaches that binding, or one unresolved relation when it is absent;
- `qualification from check` reaches every binding naming the Check, or one unresolved relation
  when the Check or its binding set is absent;
- `qualification from realization | mechanism` reaches every binding of every related case Claim;
  a binding outside the named relation's challenge domain is an inapplicable candidate;
- `claim-judgment from claim` reaches that case Claim, or one unresolved relation when absent; and
- `claim-judgment from realization | mechanism` reaches every related case Claim, independent of
  binding challenge domains, or one unresolved relation when no relation exists.

When a realization or mechanism relation exists but one related Claim has no binding,
Qualification traversal creates one unresolved-relation record for that Claim. Claim Judgment
traversal still creates the related Claim's decision candidate. For a reached candidate, routine
criticality is `inapplicable`, no authored decision is `missing-decision`, an unavailable expected
fingerprint is `invalid-decision`, a fingerprint mismatch is `stale-decision` and a current negative
verdict is `rejected-decision`, in that precedence order. Only a current positive verdict is
`selected`. A malformed authored fingerprint is a parse error rather than a candidate.
An unresolved-relation record retains the canonical selector and has no invented target id.
Underlying structural Findings remain independently visible.

Qualification selectors from binding and Check resolve directly. Realization and mechanism
selectors traverse Claims and Evidence Bindings only when the binding authorizes the relation in
its challenge domain. Claim Judgment selectors from Claim, realization or mechanism reach the
related total-composition decision without consulting binding challenge domains.

Selections union, sort and deduplicate by exact decision identity. A target-derived id binds the
Challenger fingerprint, target kind and target fingerprint. Directly challenging a Judgment is
different from propagating impact from a challenged Qualification: the latter creates one graph
edge, never a fabricated second Challenge Result.

## Semantic scope and launch inputs

An exact target fingerprint says which decision is challenged but not what a provider should
perturb or inspect. Each D46 Challenge selection therefore gains a canonical provider-neutral scope:

```json
{
  "anchors": [
    {"kind": "realization", "id": "area|rust-item|module::item", "fingerprint": "sha256:..."}
  ],
  "inputs": [
    {"kind": "check", "id": "checks/recovery", "fingerprint": "sha256:..."}
  ],
  "fingerprint": "sha256:<scope-fingerprint>"
}
```

Anchor and input arrays are sorted and unique by `(kind, id, fingerprint)`. Their closed kinds are
`claim`, `binding`, `qualification`, `claim-judgment`, `check`, `check-implementation`,
`realization`, `mechanism`, `mechanism-implementation`, `artifact`, `context`, `policy`, `area`,
`realization-obligation`, `surface`, `surface-member` and `enumeration`. Scope identity is versioned
canonical JSON over both arrays. Overlapping selectors union their anchors and inputs; conflicting
fingerprints for one `(kind, id)` fail rather than deduplicate.

Core applies one total mapping for all seven selector forms. Each selector contributes only its
exact binding, Check, realization, mechanism or Claim origin to `anchors`. Every Qualification
selection contributes the Qualification, binding, Claim, Check, complete Check implementations,
context and policy to `inputs`; mechanism-origin selection always contributes the exact artifact
and contributes a mechanism implementation only for a marker-derived route. Every Claim Judgment
selection expands the Judgment composition item by item: Claim, realizations, mechanisms,
artifacts, optional marker implementations, bindings, Qualifications and each binding's Check,
implementations, context and policy, plus applicable surface, members, enumerations, areas and
obligation and the Judgment's own policy. Overlapping selectors union both arrays.

Core never branches on the open Challenger form. Mutation, fault injection and static analysis see
the same typed semantic vocabulary; the configured adapter interprets form plus scope.

A resolved Plan covers its Challenger form only when every required scope kind occurs in the
selection. A binding-only selector therefore cannot satisfy a mutation Challenger that requires a
realization, and a non-mechanism selector cannot satisfy a fault Challenger that requires a
mechanism. Missing required scope is a model Finding and a planning failure.

The scope contains no provider selector syntax. The D47 launch route carries a sorted, unique
`inputs` array for every source-backed item in the union of scope anchors and inputs. An item that
occurs in both scope arrays projects to one launch input. A source launch input repeats semantic
kind, id and fingerprint plus file, language and site locators. Artifact and enumeration inputs
repeat stable SourceIdentity, accountable source locators and derived metadata. A tagged
surface-member input also repeats stable SourceIdentity. An enumerated surface member instead
repeats its D13 authoritative file identity and derived locator metadata. Scope participates in the
semantic Plan fingerprint; the complete route, including locator projection, participates in the
existing D47 launch-fingerprint preimage and returned provenance. Missing, extra or substituted
items are mismatches.

Mutation from a realization therefore receives that realization plus the bound Check and its
implementations. Mechanism-oriented fault injection receives the selected mechanism plus the bound
Check. Claim-level broad analysis receives the exact total composition. A provider adapter
translates these frozen inputs but does not load or reinterpret the Azimuth model.

## Planning and exact context

The strict planning request accepts Check requests and Challenge Plan requests. Each Challenge Plan
request names an explicit configured capability, finite work units and a nonzero `max_candidates`.
The combined Check and Challenge request is non-empty. Challenge-only and mixed Runs are valid.

Core loads and fingerprints the complete unselected model, resolves the authored plan and current
Challenger, expands one semantic Challenge per selected decision and validates the capability's
operation class and exact open form. It never trusts a caller-supplied form and never chooses among
several matching capabilities.

One Run has one exact context. A Qualification target is applicable only when its binding context
equals the Run context. A fan-out over different contexts fails with guidance to create separate
Runs. Direct Claim Judgment challenges use the Run context as execution context while their
repository identity remains context-independent beyond the composition inputs already bound.

For each requested Plan, `max_candidates` counts unique reached candidate records, including
adverse dispositions and unresolved relations, after selector union but before cross-plan
deduplication. The cap fails rather than truncates. Conflicting duplicate routes or work units fail.
The D47 one-adapter-per-Run rule remains, so provider families that cannot share one configured
adapter run in separate Runs.

## Exact composition algorithm

The Claim Judgment preimage is one versioned D45 canonical object. It contains the Claim id,
semantic Claim digest, criticality, applicable surface account, exact realization-obligation areas,
sorted exact case realization `(SourceIdentity, source fingerprint)` pairs, sorted applicable
mechanism records, sorted Evidence Binding `(id, fingerprint)` records, sorted recomputed expected
Qualification `(id, expected fingerprint, verdict)` records, resolved decision-policy digest,
Judgment verdict, ordered basis and ordered residual risk.

Realizations are unique by SourceIdentity. A repeated identity is a duplicate when its fingerprint
agrees and makes composition unavailable when the fingerprint conflicts; neither case becomes two
realizations.

An applicable mechanism is attached to the case or its parent requirement. Its record includes
that attachment, design id, enforcement, explicit expectations and exactly one resolved Artifact
with stable SourceIdentity and canonical derived properties. An explicit Design binding has no
marker implementation. A marker-derived binding additionally includes exactly one stable
SourceIdentity, source fingerprint and artifact binding id. Missing or multiple marker relations,
artifacts or stable identities are structural Findings and prevent a current accepted Judgment;
the fingerprint never invents a placeholder digest.

Audit found that the pre-correction extractor binding derived marker SourceIdentity from
`<file>#<site>`, contradicting D33 relocation stability. D48 now requires a semantic `site` field
and exact `<address-kind>:<site>` binding. The site is compiler-qualified by module or package,
declaring type or receiver and overload signature where supported. The accountable emitter derives
that meaning and fails when its compiler or runtime account is ambiguous; core cannot prove it from
opaque bytes and instead checks syntax, equality and assembled consistency.

The raw record has exactly `spec`, `mechanism`, `site`, `binding`, `file`, `lang` and
`source_fingerprint`. The raw companion requires id, kind and file, may retain the typed optional
Artifact properties and matches by `(id, kind, file)`. Project assembly resolves the file's area,
derives `<area>|<address-kind>|<site>` and atomically rewrites the implementation binding and
companion Artifact id to that key before identity, resolution or fingerprinting. It preserves the
optional properties, whose canonical absent values remain `null`, `[]` and `null`. The assembled
companion id is already its SourceIdentity; it is never expanded a second time. Unrelated and
explicit Design Artifacts keep authored kind/id semantics. Path-free excludes the workspace file,
not language-native package separators.

The raw companion is marker-only. Before rewriting, core rejects an explicit Design `Binding:`
equal to its raw id or derived assembled key. One MechanismImplementation and companion resolve
only the exact named `(spec, mechanism)` and never fan out by artifact id. Artifact reuse remains
valid only for ordinary non-companion Artifacts referenced through explicit Design bindings.

One `(area, address-kind, site)` identifies one compiler declaration and one marker target.
Duplicate, cross-target or conflicting accounts within an area fail; the same kind/site in two
areas produces distinct legal assembled ids. Local and federated assembly perform the same rewrite.
Several distinct qualified sites for one mechanism make the expected Judgment unavailable. The
source annotation remains the existing two-argument `ImplementsMechanism(spec, mechanism)`; all new
fields are extractor-derived.

Relocation within one area preserves semantic site, SourceIdentity, Judgment composition and scope
when language and content are unchanged. File-bearing complete-model and launch accounts change.
The exact preimages add no locator: site is already the SourceIdentity address, the rewritten
binding is already the Artifact id, the raw binding is excluded, language determines address kind,
and file remains excluded.

Surface contributions and obligations are included only when the Claim declares that surface or
the workspace names an obligation for the exact spec and Claim. A surface account pairs each
contribution with its exact enumeration witness and distinguishes tagged source members from D13
enumerated file-identity members. Unrelated areas, surfaces, mechanisms, bindings, Qualifications
and source records are excluded. Repository decision digests retain D45's expanded canonical
serializer; D46 semantic scope, selection and D47 launch identity use RFC 8785. The formats freeze
literal fields, array ordering, preimages and vectors before implementation.

## Outcomes, deferral and propagation

D46 retains exactly `clean | findings | inconclusive` for Challenge Results. Clean says only that
the configured search found no objection. Findings challenge the exact target. Inconclusive means
the selected search could establish neither conclusion. None of them is a product Observation.

Deferred work did not execute and has no Challenge Result. A planned target omitted from a partial,
cancelled or timed-out Run remains outstanding. D46 adds a `challenge-selection` diagnostic scope
whose id names the planned Challenge selection without claiming an execution. Every omitted
selection has exactly one execution diagnostic with a non-empty reason. A complete Run must match
its Plan; added, changed or substituted target, context, scope or units is a selection mismatch and
is rejected before publication.

F exposes a pure dependency projection:

```text
Qualification fingerprint -> binding -> Claim -> current Claim Judgment fingerprint
Claim Judgment fingerprint -> Claim
```

Change G later combines those edges with accepted Run facts to derive historical and current
Assurance State. F neither ingests a Run nor rewrites repository decisions.

## Rejected alternatives

Keeping Claim Judgment selectors reserved would make F's stated completion false. Restoring the
alpha 1 judgment facet would preserve the wrong evidence-wide semantics. Passing only decision
fingerprints to adapters would force provider-side model interpretation. Hand-authored path or glob
units would recreate a second traceability map. A numeric universal cost score or cache TTL would
pretend F can compare provider cost or Subject equivalence without a ledger. A `deferred` result
would manufacture an execution fact for work that did not run.

## Temporal boundary

D48 replaces the current D45/D47 deferrals atomically: Claim Judgment authoring and generated
Challenge planning become current. Hand-authored Challenge transport remains valid only in the new
strict current shape. Durable ingest and state stay absent. Because this is unpublished alpha 2,
formats change in place and earlier shapes are rejected without compatibility readers. The
identity correction likewise rejects a MechanismImplementation without semantic `site`, an untyped
or path-bearing raw binding, a raw prequalified area key and a mismatched companion Artifact; it
also rejects marker/explicit dual use and supplies no transitional reader.
