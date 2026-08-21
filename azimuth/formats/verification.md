# Verification declarations

The repository-owned decision facet for alpha 2 (D43, D45, D48). This file declares what a Check
means, how its terminal result bears on a case-level Claim, whether that exact edge is credible in
its required context and whether one Claim's total assurance composition is accepted. It never
records an execution result.

Only standard and critical Claims may receive verification declarations. Routine Claims stop at
intent. An ordinary native test without deliberate Check enrollment remains outside Azimuth.

## File and identity

```markdown
# Verification: <owning-spec-id>
```

At most one `verification.md` sits beside an owning `spec.md`. The header identifies repository
authority and location; it is not a namespace.

Check, Evidence Binding, Challenger and Challenge Plan ids are project-global, path-independent
lower kebab path ids. A Claim Judgment id is exactly its case-level Claim id
`<spec-id>#<case-id>`. Mechanism ids are `<spec-id>#<mechanism-id>`. Moving this file or its
package changes no identity.

Declarations may appear in any order. Every id and referenced identity is validated over the
complete project account before `--only` selection retains the closure for selected Claims.

## Check

```markdown
## Check: payments/recovery-under-broker-loss
Method: inject broker loss after the accepted write
Method: observe replay after broker recovery
Terminal: the accepted write is replayed exactly once after recovery

The two method lines describe one atomic observable result.
```

A Check has one project-global id, one or more `Method:` lines and exactly one `Terminal:`
proposition. The terminal proposition must describe one satisfied-or-violated result. If outcomes
can vary independently, declare separate Checks.

Prose after the label block is required review rationale and does not enter the fingerprint. Every
Check must have at least one Evidence Binding. A Check may have several source implementations;
their semantic identities compose one implementation set.

## Evidence Binding

```markdown
## Evidence Binding: payments/recovery-replay-edge
Check: payments/recovery-under-broker-loss
Claim: payments/recovery#accepted-write-is-replayed
Proposition: replay after injected broker loss directly exercises the recovery predicate
Scope: component
Quantification: example
Oracle: relational
Context: {"platform":"linux-x86_64","storage":"postgres-17"}
Challenge domain: ["realization","mechanism","check-implementation","oracle","context"]
Policy: credible-executable

This edge is narrower than the whole recovery suite and can be challenged independently.
```

A binding names exactly one Check and one case-level Claim. One Check may bind to several Claims,
and one Claim may receive several Checks. The pair `(Check, Claim)` is unique.

`Proposition:` explains why the Check's one terminal result bears on this Claim. The form is the
actual evidence form for this edge:

- `Scope:` is `unit | component | e2e`;
- `Quantification:` is `example | universal`; and
- `Oracle:` is `direct | golden | relational | metamorphic | model-based | contract`.

There is no Strength field. Executable Checks demonstrate sampled behavior. Structural proof
remains in design mechanisms and contributes to Claim Judgment rather than becoming a fictitious
Observation.

`Context:` is a required JSON object from string keys to uninterpreted string values. `{}` is
explicit and valid. Equality is exact in alpha 2: values have no range, wildcard or
provider-expression semantics.

`Challenge domain:` is a non-empty JSON array drawn from the closed set
`realization | mechanism | check-implementation | oracle | context`. Values are sorted and
deduplicated semantically. It constrains relation-based challenge traversal; it is not a list of
tools.

`Policy:` names one project Decision Policy from `azimuth/standards/verification.md`. Prose after
the labels is required rationale and is excluded from fingerprints. `Qualification policy:` is
not an alias and is rejected.

The policy's required forms enter this binding's fingerprint. Capability coverage and actual
challenge execution are validated by the dependent adapter and Run-planning contracts, not inferred
from the presence of repository prose.

## Qualification

```markdown
## Qualification: payments/recovery-replay-edge
Verdict: qualified
Fingerprint: sha256:<64-lowercase-hex>
Qualified: 2026-08-21
Qualifier: evidence-owner@example

The Check implementation, oracle and exact platform context make this edge credible.
```

The Qualification id is exactly the Evidence Binding id. Every applicable binding has exactly one
Qualification. Duplicate blocks are parse errors; missing, rejected or stale Qualifications are
Findings.

`Verdict:` is `qualified | rejected`. `Qualified:` is an ISO date and `Qualifier:` is a
non-empty accountable identity. Rationale is required and never changes the expected fingerprint.

## Claim Judgment

```markdown
## Claim Judgment: payments/recovery#accepted-write-is-replayed
Verdict: accepted
Policy: credible-executable
Fingerprint: sha256:<64-lowercase-hex>
Judged: 2026-08-21
Judge: assurance-owner@example
Basis: the recovery realizations and transactional mechanism compose the Claim
Basis: both current Evidence Bindings are qualified in their declared contexts
Residual risk: correlated broker and storage loss remains outside the accepted composition

The reviewed composition is sufficient for the declared Claim and residual risk.
```

The id is exactly one existing case-level Claim id. Every standard or critical case Claim has
exactly one Claim Judgment; a routine Claim rejects one. `Verdict:` is `accepted | rejected`.
`Policy:` names one Decision Policy. `Judged:` is an ISO date and `Judge:` is a non-empty
accountable identity. There is one or more `Basis:` line and one or more `Residual risk:` line;
their declaration order is semantic. Use an explicit statement such as `none identified` rather
than omitting a required account. Review rationale after the labels is required and non-semantic.

Only a structurally valid, fingerprint-current `accepted` Judgment is an executable Challenge
target. A rejected Judgment is a current negative repository decision and remains a Finding; a
clean Challenge Result cannot convert it to accepted.

## Challenger

```markdown
## Challenger: mutation/implementation-perturbation
Form: implementation-perturbation
Searches for: an implementation change that leaves the bound Check satisfied
Required scope: ["check-implementation","realization"]

Surviving changes are objections to credibility, not product outcomes.
```

`Form:` is an open lower kebab path id interpreted by Decision Policy and adapter capabilities.
`Searches for:` is the objection proposition. `Required scope:` is a non-empty JSON array sorted
and unique over the closed semantic scope kinds listed below. Core tests coverage by kind and never
infers scope from the open form. It uses the same fixed kind order as semantic scope. A Challenger
never directly evaluates a product Claim and is not recursively qualified in alpha 2.

## Challenge Plan

```markdown
## Challenge Plan: payments/recovery-credibility
Challenger: mutation/implementation-perturbation
Select: qualification from binding payments/recovery-replay-edge
Select: qualification from check payments/recovery-under-broker-loss
Select: qualification from realization payments|rust-item|recovery::replay
Select: qualification from mechanism payments/recovery#transactional-outbox
Select: claim-judgment from claim payments/recovery#accepted-write-is-replayed
Select: claim-judgment from realization payments|rust-item|recovery::replay
Select: claim-judgment from mechanism payments/recovery#transactional-outbox

The plan states semantic reach; a Run freezes the exact resolved fingerprints.
```

A plan names one Challenger and one or more repeatable `Select:` lines using only the seven forms
shown above. Resolution retains every reached candidate disposition. Executable selections union,
sort and deduplicate exact current accepted decision fingerprints.

Qualification traversal from realizations or mechanisms reaches case-level Claims and their
Evidence Bindings only when each binding's Challenge domain authorizes that relation. Claim
Judgment traversal reaches related total-composition decisions without consulting binding
challenge domains. Zero resolution is a Finding. Source paths, line numbers, globs and whole-suite
fallback are invalid.

The selector retains the complete semantic source address after the relation token, including
spaces. An assembly-derived address such as `web|next-route|GET /payments/[id]` is semantic; a
source-file locator remains invalid even when placed in the address field.

## Resolution account

Every `Select:` line normalizes to this strict selector object:

```json
{"target":"qualification","from":"realization","id":"payments|rust-item|recovery::replay"}
```

`target` is `qualification | claim-judgment`. The permitted `(target, from)` pairs are exactly
`(qualification, binding | check | realization | mechanism)` and
`(claim-judgment, claim | realization | mechanism)`. The address is retained byte-for-byte after
the single separating space in Markdown.

Each reached candidate has the strict derived shape:

```json
{
  "selector": {
    "target": "qualification",
    "from": "realization",
    "id": "payments|rust-item|recovery::replay"
  },
  "relation": {
    "kind": "claim",
    "id": "payments/recovery#accepted-write-is-replayed"
  },
  "target": {
    "kind": "qualification",
    "id": "payments/recovery-replay-edge",
    "expected_fingerprint": "sha256:<64-lowercase-hex>",
    "authored_fingerprint": "sha256:<64-lowercase-hex>"
  },
  "disposition": "selected"
}
```

`relation.kind` is `binding | check | realization | mechanism | claim`; it identifies the exact
model relation through which the candidate was reached. `target` is `null` only for
`unresolved-relation`; otherwise it has exactly the fields above. `authored_fingerprint` is `null`
only when no decision is authored; a malformed authored fingerprint is a parse error and creates
no model. `expected_fingerprint` is `null` only when composition cannot be computed. Disposition is
exactly
`selected | missing-decision | stale-decision | rejected-decision | invalid-decision |
inapplicable | unresolved-relation`.

Records sort by canonical selector `(target, from, id)`, then relation `(kind, id)`, then target
`(kind, id)` with `null` first. That tuple is also candidate identity; duplicate records fail.
One successful record never hides an adverse sibling. Selector lines deduplicate by their
canonical object before resolution. There is no fallback when all records are adverse.

The seven mappings and their exact relation objects are total:

- qualification from binding reaches that binding with
  `{"kind":"binding","id":<binding-id>}`; an absent binding retains the requested id in the
  same relation object and has no target;
- qualification from Check reaches every binding naming it, each with
  `{"kind":"binding","id":<binding-id>}`; a missing Check or empty binding set yields one
  `{"kind":"check","id":<check-id>}` unresolved relation;
- qualification from realization reaches every binding of every related case Claim, each with a
  binding relation object; a binding whose domain omits `realization` is `inapplicable`, a related
  Claim without a binding yields `{"kind":"claim","id":<claim-id>}`, and no related Claim yields
  `{"kind":"realization","id":<SourceIdentity>}`, both unresolved;
- qualification from mechanism applies the preceding mapping with domain `mechanism`; no related
  Claim uses `{"kind":"mechanism","id":<spec-id#mechanism-id>}`;
- Claim Judgment from Claim reaches that Claim with `{"kind":"claim","id":<claim-id>}`; an
  absent Claim retains the requested id in the same relation object and has no target;
- Claim Judgment from realization reaches every related Claim, each with a Claim relation object,
  or one `{"kind":"realization","id":<SourceIdentity>}` unresolved relation when none exists;
  and
- Claim Judgment from mechanism applies the preceding mapping with
  `{"kind":"mechanism","id":<spec-id#mechanism-id>}` only when no Claim relation exists.

For a reached target, disposition precedence is exactly: routine criticality is `inapplicable`; no
authored declaration is `missing-decision`; an unavailable expected fingerprint is
`invalid-decision`; unequal expected and authored fingerprints is `stale-decision`; a current
negative verdict is `rejected-decision`; otherwise a current positive verdict is `selected`.

For every current positive decision and every form its Decision Policy requires, at least one
current Challenger has that exact form and at least one authored Plan resolves the decision with
all of that Challenger's required scope kinds. Additional forms are permitted strengthening. This
is declaration coverage, not evidence that a Challenge executed or returned clean.

## Semantic challenge scope

Every generated Challenge selection carries:

```json
{
  "anchors": [
    {
      "kind": "realization",
      "id": "payments|rust-item|recovery::replay",
      "fingerprint": "sha256:<64-lowercase-hex>"
    }
  ],
  "inputs": [
    {
      "kind": "check-implementation",
      "id": "payments|rust-item|recovery::replay-after-loss",
      "fingerprint": "sha256:<64-lowercase-hex>"
    }
  ],
  "fingerprint": "sha256:<scope-fingerprint>"
}
```

The closed scope kinds are `claim | binding | qualification | claim-judgment | check |
check-implementation | realization | mechanism | mechanism-implementation | artifact | context |
policy | area | realization-obligation | surface | surface-member | enumeration`. Every item has
exactly `kind`, non-empty stable `id` and fingerprint. Arrays sort and are unique by
`(kind, id, fingerprint)` in the order just listed; two fingerprints for one `(kind, id)` are a
conflict rather than two items. An item may occur once in each array because selector provenance
and decision composition have different meanings.

An anchor is the exact authored selector origin: binding, Check, realization, mechanism or Claim.
Inputs are the complete semantic dependencies of the selected decision. The exact seven
projections are:

- qualification from binding anchors the binding;
- qualification from Check anchors the Check;
- qualification from realization anchors that exact realization;
- qualification from mechanism anchors that exact mechanism and always adds its resolved artifact
  to inputs; a marker-derived route additionally adds its one mechanism implementation;
- Claim Judgment from Claim anchors the Claim;
- Claim Judgment from realization anchors that exact realization; and
- Claim Judgment from mechanism anchors that exact mechanism.

Every Qualification selection input contains its Qualification, binding, Claim, Check, complete
Check implementation set, exact context and Decision Policy. Every Claim Judgment selection input
contains, item by item:

- the Claim Judgment, Claim and Judgment's Decision Policy;
- every exact case realization;
- every applicable mechanism, its resolved artifact and its marker implementation when present;
- every Evidence Binding and Qualification for the Claim;
- each binding's Check, complete Check implementation set, exact context and Decision Policy;
- the applicable surface, every contribution area, enumeration witness and tagged or enumerated
  surface member; and
- the exact realization obligation and each obligated area when one exists.

Relation-specific inputs above are additional. Overlapping selectors and Plans union the arrays
before fingerprinting. Required-scope coverage tests kinds over the union of `anchors` and
`inputs`.

Stable item ids and fingerprints are exact: Claim uses its id and semantic Claim digest; binding,
Qualification, Claim Judgment, Check and policy use their declared id and corresponding canonical
fingerprint or digest. Context uses its owning Evidence Binding id and the D45 context fingerprint;
two contexts for one binding are structurally impossible. Realization and Check implementation use
SourceIdentity and source fingerprint; mechanism uses `<spec>#<mechanism>` and its canonical
mechanism-record digest; source mechanism implementation uses SourceIdentity and source
fingerprint; artifact uses its artifact id and canonical source-identity and derived-property
digest; area uses area id and canonical area digest; realization obligation uses its Claim id and
canonical sorted-area digest; and surface uses its id and canonical surface-account digest. An
enumeration uses
`<surface>|<area>|<mount>|<enumerator>|<SourceIdentity>` and its source fingerprint. A tagged
surface member uses `<surface>|tagged|<SourceIdentity>` and its source fingerprint; an enumerated
member uses `<surface>|enumerated|<file>` and the canonical digest of that D13 model-authoritative
file identity. Locator paths never substitute for another kind's id.

Auxiliary component digests use D45 canonical serialization over these exact envelopes:

```json
{
  "format": "azimuth-context-fingerprint",
  "version": 1,
  "required_context": <binding-context-object>
}
{"format":"azimuth-mechanism-record-digest","version":1,"mechanism":<mechanism-record>}
{"format":"azimuth-artifact-property-digest","version":1,"artifact":<artifact-account>}
{"format":"azimuth-area-digest","version":1,"id":<area-id>}
{
  "format": "azimuth-realization-obligation-digest",
  "version": 1,
  "claim": <case-claim-id>,
  "areas": <sorted-distinct-area-ids>
}
{"format":"azimuth-surface-account-digest","version":1,"surface":<surface-account>}
{
  "format": "azimuth-surface-member-digest",
  "version": 1,
  "surface": <surface-id>,
  "kind": "enumerated",
  "file": <D13-member-file-identity>
}
```

The surface account has exactly `id`, `contributions` and `members`. Each contribution has `area`,
`mount`, `enumerator` and `witness`; the witness has enumeration `kind`, stable `identity` and
`source_fingerprint`. Contributions sort by `(area, mount, enumerator, witness.identity)`. A member
is either `{"kind":"tagged","identity":<SourceIdentity>,"source_fingerprint":<fingerprint>}` or
`{"kind":"enumerated","file":<D13-member-file-identity>}`. Members sort first by kind in
`tagged | enumerated` order and then by identity or file. Every set-like array is unique on its sort
key; a collision with different content makes the expected Judgment unavailable.

Scope identity is SHA-256 over RFC 8785 canonical UTF-8 for this exact object:

```json
{
  "format": "azimuth-challenge-scope-fingerprint",
  "version": 1,
  "anchors": <complete-anchors>,
  "inputs": <complete-inputs>
}
```

### Canonical scope vector

This preimage is already in RFC 8785 form:

```json
{"anchors":[{"fingerprint":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","id":"demo|rust-item|demo::subject","kind":"realization"}],"format":"azimuth-challenge-scope-fingerprint","inputs":[{"fingerprint":"sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","id":"demo|rust-item|demo::check","kind":"check-implementation"}],"version":1}
```

Its SHA-256 value is
`sha256:29cdeb0c856e2172f0693eba0962d038f6916f0ed0f3dde464c1b0052326a977`.

## Canonical fingerprints

Repository decision fingerprints use D45 canonical JSON: object keys sort recursively by their
exact strings, set-like arrays use their declared order, strings preserve code points, expanded
serialization uses two spaces and LF, and exactly one terminal LF is hashed. This is intentionally
the existing model `canonical_sha256` contract, not D46 RFC 8785. There is no legacy reader.

- Check fingerprint: format version, Check id, ordered methods, terminal proposition, and sorted
  implementation semantic identities plus source fingerprints.
- Binding fingerprint: format version, binding id, Check id, semantic Claim digest, Proposition,
  form tuple, sorted challenge domain and Decision Policy digest.
- Context fingerprint: format and version plus the canonical required-context object.
- Qualification fingerprint: Check, Binding and Context fingerprints.
- Claim Judgment fingerprint: the exact total-composition preimage below.
- Challenger fingerprint: id, open form, objection proposition and sorted required scope kinds.

Paths, lines, mounts and explanatory prose are excluded. Criticality remains excluded from
Qualification identity but is included in Claim Judgment identity. A source, Claim, binding,
policy or context change stales the decision it actually affects.

The D48 Evidence Binding preimage is exactly:

```json
{
  "format": "azimuth-evidence-binding-fingerprint",
  "version": 1,
  "id": <binding-id>,
  "check": <check-id>,
  "claim_digest": <semantic-claim-digest>,
  "proposition": <binding-proposition>,
  "form": {
    "scope": <scope>,
    "quantification": <quantification>,
    "oracle": <oracle>
  },
  "challenge_domain": <sorted-distinct-domain-kinds>,
  "decision_policy_digest": <decision-policy-digest>
}
```

The literal `decision_policy_digest` replaces D45's unpublished
`qualification_policy_digest` in place. The earlier field is rejected and there is no alternate
preimage.

Decision Policy uses this exact digest preimage:

```json
{
  "format": "azimuth-decision-policy-digest",
  "version": 1,
  "id": <policy-id>,
  "required_challenges": <sorted-distinct-forms>
}
```

Challenger uses this exact fingerprint preimage:

```json
{
  "format": "azimuth-challenger-fingerprint",
  "version": 1,
  "id": <challenger-id>,
  "form": <open-form>,
  "searches_for": <objection-proposition>,
  "required_scope": <sorted-distinct-scope-kinds>
}
```

The Claim Judgment preimage is:

```json
{
  "format": "azimuth-claim-judgment-fingerprint",
  "version": 1,
  "claim": {
    "id": <case-claim-id>,
    "semantic_digest": <D45-case-claim-digest>,
    "criticality": <standard-or-critical>,
    "realization_obligation_areas": <sorted-distinct-area-ids>,
    "surface": <applicable-surface-account-or-null>
  },
  "realizations": <sorted-realization-records>,
  "mechanisms": <sorted-applicable-mechanism-records>,
  "bindings": <sorted-binding-id-and-fingerprint-records>,
  "qualifications": <sorted-id-expected-fingerprint-and-verdict-records>,
  "policy_digest": <decision-policy-digest>,
  "verdict": <accepted-or-rejected>,
  "basis": <ordered-non-empty-statements>,
  "residual_risks": <ordered-non-empty-statements>
}
```

A realization record is exactly `{"identity": <SourceIdentity>, "source_fingerprint":
<fingerprint>}` and sorts uniquely by `identity`. Realizations are the exact sites for the case
Claim's spec and scenario. Repeating one identity with the same fingerprint is a duplicate;
repeating it with a different fingerprint makes the expected Judgment unavailable. A mechanism is
applicable when it is attached to that scenario or its parent requirement and sorts by id. Its
record is exactly:

```json
{
  "id": <spec-id#mechanism-id>,
  "attachment": {
    "target_kind": <requirement-or-scenario>,
    "target_id": <declared-target-id>
  },
  "enforcement": <enforcement-kind>,
  "expect": {
    "unique": <boolean-or-null>,
    "columns": <ordered-columns>,
    "predicate": <string-or-null>
  },
  "artifact": {
    "id": <artifact-id>,
    "kind": <artifact-kind>,
    "identity": <SourceIdentity>,
    "unique": <boolean-or-null>,
    "columns": <ordered-columns>,
    "predicate": <string-or-null>
  },
  "implementation": <mechanism-implementation-or-null>
}
```

An explicit Design `Binding:` yields `implementation: null`. A marker-derived binding yields
exactly `{"identity": <SourceIdentity>, "source_fingerprint": <fingerprint>, "artifact":
<artifact-id>}`. Both paths resolve exactly one artifact and always include its canonical derived
properties and stable SourceIdentity. Zero or several marker relations, a marker without stable
source identity or fingerprint, or zero or several matching artifacts makes the expected Judgment
unavailable. Moving a mechanism between the parent requirement and case changes `attachment` even
when it remains applicable.

The applicable surface account is exactly its `id`, sorted contribution
`(area, mount, enumerator)` objects, one sorted witness per contribution and sorted member records.
A contribution owns `area`, `mount` and `enumerator`; its nested witness has exactly enumeration
`kind`, stable `identity` and `source_fingerprint`. A tagged member records its stable
SourceIdentity and source fingerprint; an enumerated member records its D13 file identity. Their
tagged/enumerated variants cannot collapse.
The mount id is an authored contribution identity; its path is excluded. The surface is `null`
when the Claim has no `Over:`. The obligation areas are only the exact workspace obligation for the
spec and Claim, or `[]`.

Binding records are exactly `{"id": <binding-id>, "fingerprint": <binding-fingerprint>}` sorted
by id. Qualification records are exactly `{"id": <binding-id>, "expected_fingerprint":
<recomputed-expected-fingerprint>, "verdict": <qualified-or-rejected>}` sorted by id. Missing or
structurally invalid composition makes the expected Judgment unavailable; a stale authored
Qualification fingerprint is never used.

### Canonical decision vectors

This Evidence Binding preimage is canonical:

```json
{
  "challenge_domain": [
    "realization"
  ],
  "check": "demo/check",
  "claim_digest": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  "decision_policy_digest": "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
  "form": {
    "oracle": "direct",
    "quantification": "example",
    "scope": "component"
  },
  "format": "azimuth-evidence-binding-fingerprint",
  "id": "demo/binding",
  "proposition": "the Check directly exercises the case",
  "version": 1
}
```

The serialized preimage has one terminal LF. Its SHA-256 value is
`sha256:58dc690f4b9ec8fab6184d542154e88104df35448bb28d3e38cb2ae59fd627e7`.

The current project policy preimage is:

```json
{
  "format": "azimuth-decision-policy-digest",
  "id": "credible-executable",
  "required_challenges": [
    "implementation-perturbation",
    "oracle-perturbation"
  ],
  "version": 1
}
```

The serialized preimage has one terminal LF. Its SHA-256 value is
`sha256:852f3fdc2d9f376403c41e215e3a06304e667df9d1e4a49eae9af53300433b06`.

This Challenger preimage is canonical:

```json
{
  "form": "implementation-perturbation",
  "format": "azimuth-challenger-fingerprint",
  "id": "mutation/implementation-perturbation",
  "required_scope": [
    "check-implementation",
    "realization"
  ],
  "searches_for": "an implementation change that leaves the bound Check satisfied",
  "version": 1
}
```

The serialized preimage has one terminal LF. Its SHA-256 value is
`sha256:383c91179c3d79e1a7e2c974376d481c674f9df12aa24ee7c73104a1c03c0390`.

This minimal Claim Judgment preimage is canonical:

```json
{
  "basis": [
    "the bound Check directly exercises the case"
  ],
  "bindings": [
    {
      "fingerprint": "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
      "id": "demo/binding"
    }
  ],
  "claim": {
    "criticality": "standard",
    "id": "demo/spec#case",
    "realization_obligation_areas": [],
    "semantic_digest": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    "surface": null
  },
  "format": "azimuth-claim-judgment-fingerprint",
  "mechanisms": [],
  "policy_digest": "sha256:eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
  "qualifications": [
    {
      "expected_fingerprint": "sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
      "id": "demo/binding",
      "verdict": "qualified"
    }
  ],
  "realizations": [
    {
      "identity": "demo|rust-item|demo::subject",
      "source_fingerprint": "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
    }
  ],
  "residual_risks": [
    "none identified"
  ],
  "verdict": "accepted",
  "version": 1
}
```

The serialized preimage has one terminal LF. Its SHA-256 value is
`sha256:98223be4d7f1cb21da47caae82aaf5f1d33dd879eee2240aeee1b643c1eeb441`.

## Source boundary

Source uses `ImplementsCheck(<check-id>)`. A language extractor emits only:

```json
{
  "check_implementations": [
    {
      "check": "payments/recovery-under-broker-loss",
      "site": "recovery::replay_after_loss",
      "file": "src/recovery.rs",
      "lang": "rust",
      "source_fingerprint": "sha256:<64-lowercase-hex>"
    }
  ]
}
```

Workspace or project assembly attaches `area`, `mount`, `address_kind` and `address` from
declared repository structure. Files and mounts remain locators; the semantic source identity is
`<area>|<address-kind>|<address>`.

An implementation marker contains no Claim, form, context or Qualification. A native test without
the marker emits nothing.

## Rejected alpha 1 input

The parser rejects old `## Claim`, `## Judgment` and residual headings, evidence floors, non-test
evidence, Strength, detector fields, `Qualification policy:` and the plural `judgments.md` facet.
Manifests reject `covers`, `mechanism_covers` and `observations`. Annotations reject Covers and
CoversMechanism. Nothing is translated, deprecated or exported twice.
