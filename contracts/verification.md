# Verification declarations

The repository-owned decision facet for alpha 4. This file declares what a Check means, how its
terminal result bears on a Case, whether the shared method is credible, whether it applies to each
exact Check-to-Case edge and whether one Claim's total assurance composition is accepted. It never
records an execution result.

Only standard and critical Claims may receive verification declarations. Routine Claims stop at intent. An ordinary native test without deliberate Check enrollment remains outside Azimuth.

## File and identity

```markdown
# Verification: <owning-spec-id>
```

At most one `verification.md` sits beside an owning `spec.md`. The header identifies repository authority and location; it is not a namespace.

Check, Evidence Binding, Method Qualification, Challenger and Challenge Plan ids are
project-global, path-independent lower kebab path ids. An Applicability Decision id is exactly its
Evidence Binding id. A Claim Judgment id is exactly its parent Claim id `<spec-id>#<claim-id>`.
Case ids use `<spec-id>#<claim-id>/<case-id>`. Mechanism ids are
`<spec-id>#<mechanism-id>`. Moving this file or its package changes no identity.

Declarations may appear in any order. Every id and referenced identity is validated over the complete project account before `--only` selection retains the closure for selected Claims.

## Check

```markdown
## Check: payments/recovery-under-broker-loss
Method: inject broker loss after the accepted write
Method: observe replay after broker recovery
Terminal: the accepted write is replayed exactly once after recovery

The two method lines describe one atomic observable result.
```

A Check has one project-global id, one or more `Method:` lines and exactly one `Terminal:` proposition. The terminal proposition must describe one satisfied-or-violated result. If outcomes can vary independently, declare separate Checks.

Prose after the label block is required review rationale and does not enter the fingerprint. Every Check must have at least one Evidence Binding. A Check may have several source implementations; their semantic identities compose one implementation set.

## Evidence Binding

```markdown
## Evidence Binding: payments/recovery-replay-edge
Check: payments/recovery-under-broker-loss
Case: payments/recovery#accepted-write/replayed-after-broker-loss
Method qualification: payments/recovery-method
Proposition: replay after injected broker loss directly exercises the recovery predicate
Context: {"platform":"linux-x86_64","storage":"postgres-17"}
Challenge domain: ["realization","mechanism","check-implementation","oracle","context"]
Policy: applicable-edge

This edge is narrower than the whole recovery suite and can be challenged independently.
```

A binding names exactly one Check, one Case and one Method Qualification. One Check may bind to
several Cases, and one Case may receive several Checks. The pair `(Check, Case)` is unique.

`Proposition:` explains why the Check's terminal result bears on this Case. `Context:` holds only
Case- or edge-specific requirements. Shared method context belongs to the Method Qualification.

There is no Strength field. Executable Checks demonstrate sampled behavior. Structural proof remains in design mechanisms and contributes to Claim Judgment rather than becoming a fictitious Observation.

`Context:` is a required JSON object from string keys to uninterpreted string values. `{}` is explicit and valid. Equality is exact in alpha 4: values have no range, wildcard or provider-expression semantics.

`Challenge domain:` is a non-empty JSON array drawn from the closed set `realization | mechanism | check-implementation | oracle | context`. Values are sorted and deduplicated semantically. It constrains relation-based challenge traversal; it is not a list of tools.

`Policy:` names one project Decision Policy from `azimuth/standards/verification.md`. Prose after the labels is required rationale and is excluded from fingerprints. `Qualification policy:` is not an alias and is rejected.

The policy's required forms enter this binding's fingerprint. Capability coverage and actual challenge execution are validated by the dependent adapter and Run-planning contracts, not inferred from the presence of repository prose.

## Method Qualification

```markdown
## Method Qualification: payments/recovery-method
Check: payments/recovery-under-broker-loss
Scope: component
Quantification: example
Oracle: relational
Context: {"platform":"linux-x86_64","storage":"postgres-17"}
Challenge domain: ["check-implementation","oracle","context"]
Policy: credible-method
Verdict: qualified
Fingerprint: sha256:<64-lowercase-hex>
Qualified: 2026-08-21
Qualifier: evidence-owner@example

The Check implementation, oracle and common context make this method credible.
```

A Method Qualification names one Check plus exact form, common context, challenge domain and
Decision Policy inputs. Its id is independent of binding identity, so several bindings may reuse it
only when those inputs are genuinely shared. Duplicate ids are parse errors; missing, rejected or
stale Method Qualifications are Findings.

`Verdict:` is `qualified | rejected`. `Qualified:` is an ISO date and `Qualifier:` is a non-empty accountable identity. Rationale is required and never changes the expected fingerprint.

## Applicability Decision

```markdown
## Applicability Decision: payments/recovery-replay-edge
Verdict: applicable
Fingerprint: sha256:<64-lowercase-hex>
Decided: 2026-08-21
Decider: evidence-owner@example

The qualified recovery method directly establishes this Case under the edge context.
```

The id is exactly one Evidence Binding id. Every binding has exactly one Applicability Decision.
`Verdict:` is `applicable | rejected`; `Decided:` is an ISO date and `Decider:` is a non-empty
accountable identity. Missing, rejected or stale decisions are Findings. An accepted Method
Qualification never implies edge applicability.

## Claim Judgment

```markdown
## Claim Judgment: payments/recovery#accepted-write
Verdict: accepted
Policy: credible-executable
Fingerprint: sha256:<64-lowercase-hex>
Judged: 2026-08-21
Judge: assurance-owner@example
Basis: the recovery realizations and transactional mechanism compose the Claim
Basis: every Case has current applicable evidence and all shared methods are qualified
Residual risk: correlated broker and storage loss remains outside the accepted composition

The reviewed composition is sufficient for the declared Claim and residual risk.
```

The id is exactly one existing parent Claim id. Every standard or critical Claim has exactly one
Claim Judgment; a routine Claim rejects one. The Judgment consumes every Case and evidence edge;
Cases own no separate Judgment. `Verdict:` is `accepted | rejected`. `Policy:` names one Decision
Policy. `Judged:` is an ISO date and `Judge:` is a non-empty accountable identity. There is one or
more `Basis:` line and one or more `Residual risk:` line; their declaration order is semantic. Use
an explicit statement such as `none identified` rather than omitting a required account. Review
rationale after the labels is required and non-semantic.

Only a structurally valid, fingerprint-current `accepted` Judgment is an executable Challenge target. A rejected Judgment is a current negative repository decision and remains a Finding; a clean Challenge Result cannot convert it to accepted.

## Challenger

```markdown
## Challenger: mutation/implementation-perturbation
Form: implementation-perturbation
Searches for: an implementation change that leaves the bound Check satisfied
Required scope: ["check-implementation","realization"]

Surviving changes are objections to credibility, not product outcomes.
```

`Form:` is an open lower kebab path id interpreted by Decision Policy and adapter capabilities. `Searches for:` is the objection proposition. `Required scope:` is a non-empty JSON array sorted and unique over the closed semantic scope kinds listed below. Core tests coverage by kind and never infers scope from the open form. It uses the same fixed kind order as semantic scope. A Challenger never directly evaluates a product Claim and is not recursively qualified in alpha 4.

## Challenge Plan

```markdown
## Challenge Plan: payments/recovery-credibility
Challenger: mutation/implementation-perturbation
Select: method-qualification from method-qualification payments/recovery-method
Select: method-qualification from check payments/recovery-under-broker-loss
Select: method-qualification from realization payments|rust-item|recovery::replay
Select: method-qualification from mechanism payments/recovery#transactional-outbox
Select: applicability-decision from binding payments/recovery-replay-edge
Select: applicability-decision from case payments/recovery#accepted-write/replayed-after-broker-loss
Select: applicability-decision from check payments/recovery-under-broker-loss
Select: applicability-decision from realization payments|rust-item|recovery::replay
Select: applicability-decision from mechanism payments/recovery#transactional-outbox
Select: claim-judgment from claim payments/recovery#accepted-write
Select: claim-judgment from realization payments|rust-item|recovery::replay
Select: claim-judgment from mechanism payments/recovery#transactional-outbox

The plan states semantic reach; a Run freezes the exact resolved fingerprints.
```

A plan names one Challenger and one or more repeatable `Select:` lines using only the twelve forms
shown above. Resolution retains every reached candidate disposition. Executable selections union,
sort and deduplicate exact current positive decision fingerprints.

Method Qualification and Applicability Decision traversal from realizations or mechanisms reaches
Cases and Evidence Bindings only when the corresponding challenge domain authorizes that relation.
Method findings fan out through every dependent edge; Applicability Decision findings remain local
to their edge. Claim Judgment traversal reaches related total-composition decisions without
consulting binding challenge domains. Zero resolution is a Finding. Source paths, line numbers,
globs and whole-suite fallback are invalid.

The selector retains the complete semantic source address after the relation token, including spaces. An assembly-derived address such as `web|next-route|GET /payments/[id]` is semantic; a source-file locator remains invalid even when placed in the address field.

## Resolution account

Every `Select:` line normalizes to this strict selector object:

```json
{"target":"method-qualification","from":"realization","id":"payments|rust-item|recovery::replay"}
```

`target` is `method-qualification | applicability-decision | claim-judgment`. The permitted
`(target, from)` pairs are exactly `(method-qualification, method-qualification | check |
realization | mechanism)`, `(applicability-decision, binding | case | check | realization |
mechanism)` and `(claim-judgment, claim | realization | mechanism)`. The address is retained
byte-for-byte after the single separating space in Markdown.

Each reached candidate has the strict derived shape:

```json
{
  "selector": {
    "target": "method-qualification",
    "from": "realization",
    "id": "payments|rust-item|recovery::replay"
  },
  "relation": {
    "kind": "method-qualification",
    "id": "payments/recovery-method"
  },
  "target": {
    "kind": "method-qualification",
    "id": "payments/recovery-method",
    "expected_fingerprint": "sha256:<64-lowercase-hex>",
    "authored_fingerprint": "sha256:<64-lowercase-hex>"
  },
  "disposition": "selected"
}
```

`relation.kind` is `binding | case | check | claim | method-qualification | realization |
mechanism`; it identifies the exact model relation through which the candidate was reached.
`target` is `null` only for `unresolved-relation`; otherwise it has exactly the fields above.
`authored_fingerprint` is `null` only when no decision is authored; a malformed authored
fingerprint is a parse error and creates no model. `expected_fingerprint` is `null` only when
composition cannot be computed. Disposition is exactly `selected | missing-decision |
stale-decision | rejected-decision | invalid-decision | inapplicable | unresolved-relation`.

Records sort by canonical selector `(target, from, id)`, then relation `(kind, id)`, then target `(kind, id)` with `null` first. That tuple is also candidate identity; duplicate records fail. One successful record never hides an adverse sibling. Selector lines deduplicate by their canonical object before resolution. There is no fallback when all records are adverse.

The twelve mappings and their exact relation objects are total:

- Method Qualification from Method Qualification reaches that exact decision;
- Method Qualification from Check reaches every Method Qualification naming it;
- Method Qualification from realization or mechanism reaches the distinct Method Qualifications
  used by related Case bindings whose challenge domain permits that relation;
- Applicability Decision from binding reaches that binding's decision;
- Applicability Decision from Case reaches every binding of that exact Case;
- Applicability Decision from Check reaches every binding naming it;
- Applicability Decision from realization or mechanism reaches the bindings of each related Case
  whose challenge domain permits that relation;
- Claim Judgment from Claim reaches that Claim with `{"kind":"claim","id":<claim-id>}`; an absent Claim retains the requested id in the same relation object and has no target;
- Claim Judgment from realization reaches every related Claim, each with a Claim relation object, or one `{"kind":"realization","id":<SourceIdentity>}` unresolved relation when none exists; and
- Claim Judgment from mechanism applies the preceding mapping with `{"kind":"mechanism","id":<spec-id#mechanism-id>}` only when no Claim relation exists.

For a reached target, disposition precedence is exactly: routine criticality is `inapplicable`; no authored declaration is `missing-decision`; an unavailable expected fingerprint is `invalid-decision`; unequal expected and authored fingerprints is `stale-decision`; a current negative verdict is `rejected-decision`; otherwise a current positive verdict is `selected`.

For every current positive decision and every form its Decision Policy requires, at least one current Challenger has that exact form and at least one authored Plan resolves the decision with all of that Challenger's required scope kinds. Additional forms are permitted strengthening. This is declaration coverage, not evidence that a Challenge executed or returned clean.

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

The closed scope kinds are `applicability-decision | case | claim | binding |
method-qualification | claim-judgment | check | check-implementation | realization | mechanism |
mechanism-implementation | artifact | context | policy | area | realization-obligation | surface |
surface-member | enumeration`. Every item has exactly `kind`, non-empty stable `id` and
fingerprint. Arrays sort and are unique by `(kind, id, fingerprint)` in that order; two
fingerprints for one `(kind, id)` are a conflict rather than two items. An item may occur once in
each array because selector provenance and decision composition are distinct roles.

An anchor is the exact authored selector origin: Method Qualification, binding, Case, Check,
realization, mechanism or Claim. Inputs are the complete semantic dependencies of the selected
decision. The exact twelve projections follow the selector relations above.

Every Method Qualification selection input contains that decision, Check, complete Check
implementation set, common context and Decision Policy. Every Applicability Decision selection
contains that decision, binding, Case, parent Claim, Method Qualification composition, edge
context and Decision Policy. Every Claim Judgment selection input contains, item by item:

- the Claim Judgment, Claim and Judgment's Decision Policy;
- every exact parent-Claim realization;
- every applicable mechanism, its Case relevance, resolved artifact and marker implementation when present;
- every Case, Evidence Binding, Method Qualification and Applicability Decision for the Claim;
- each binding's Check, complete Check implementation set, exact context and Decision Policy;
- the applicable surface, every contribution area, enumeration witness and tagged or enumerated surface member; and
- the exact realization obligation and each obligated area when one exists.

Relation-specific inputs above are additional. Overlapping selectors and Plans union the arrays before fingerprinting. Required-scope coverage tests kinds over the union of `anchors` and `inputs`.

Stable item ids and fingerprints are exact: Case and Claim use their nested and parent ids with
their semantic digests; binding, Method Qualification, Applicability Decision, Claim Judgment,
Check and policy use their declared ids and corresponding fingerprint or digest. Context uses its
owning decision or binding id and the canonical context-object fingerprint. Realization and Check
implementation use SourceIdentity and source fingerprint; all other established component rules
below are unchanged. Locator paths never substitute for another kind's id.

Auxiliary component digests use this file's canonical serialization over these exact envelopes:

```json
<canonical-context-object>
{"format":"azimuth-mechanism-record-digest","version":1,"mechanism":<mechanism-record>}
{"format":"azimuth-artifact-property-digest","version":1,"artifact":<artifact-account>}
{"format":"azimuth-area-digest","version":1,"id":<area-id>}
{
  "format": "azimuth-realization-obligation-digest",
  "version": 1,
  "claim": <parent-claim-id>,
  "areas": <sorted-distinct-area-ids>
}
{"format":"azimuth-surface-account-digest","version":1,"surface":<surface-account>}
{
  "format": "azimuth-surface-member-digest",
  "version": 1,
  "surface": <surface-id>,
  "kind": "enumerated",
  "file": <site-member-file-identity>
}
```

The surface account has exactly `id`, `contributions` and `members`. Each contribution has `area`, `mount`, `enumerator` and `witness`; the witness has enumeration `kind`, stable `identity` and `source_fingerprint`. Contributions sort by `(area, mount, enumerator, witness.identity)`. A member is either `{"kind":"tagged","identity":<SourceIdentity>,"source_fingerprint":<fingerprint>}` or `{"kind":"enumerated","file":<site-member-file-identity>}`. Members sort first by kind in `tagged | enumerated` order and then by identity or file. Every set-like array is unique on its sort key; a collision with different content makes the expected Judgment unavailable.

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

Its SHA-256 value is `sha256:29cdeb0c856e2172f0693eba0962d038f6916f0ed0f3dde464c1b0052326a977`.

## Canonical fingerprints

Repository decision fingerprints use canonical JSON: object keys sort recursively by their exact strings, set-like arrays use their declared order, strings preserve code points, expanded serialization uses two spaces and LF, and exactly one terminal LF is hashed. This is intentionally the existing model `canonical_sha256` contract, not the RFC 8785 serialization used by the Run-bundle format. There is no legacy reader.

- Check fingerprint: format version, Check id, ordered methods, terminal proposition, and sorted implementation semantic identities plus source fingerprints.
- Method Qualification fingerprint: format version, id, Check id and fingerprint, form tuple,
  canonical common context, sorted challenge domain and Decision Policy digest.
- Binding fingerprint: format version, binding id, Check id, semantic Case digest, current Method
  Qualification fingerprint, Proposition, canonical edge context, sorted challenge domain and
  Decision Policy digest.
- Applicability Decision fingerprint: an exact envelope over the Evidence Binding fingerprint.
- Context fingerprint: the canonical context object itself.
- Claim Judgment fingerprint: the exact total-composition preimage below.
- Challenger fingerprint: id, open form, objection proposition and sorted required scope kinds.

Paths, lines, mounts and explanatory prose are excluded. Criticality remains excluded from Method
Qualification and Applicability Decision identity but is included in Claim Judgment identity. A
source, Case, binding, policy or context change stales only the decisions whose inputs include it.

The Method Qualification preimage is exactly:

```json
{
  "format": "azimuth-method-qualification-fingerprint",
  "version": 1,
  "id": <method-qualification-id>,
  "check": <check-id>,
  "check_fingerprint": <check-fingerprint>,
  "form": {"scope": <scope>, "quantification": <quantification>, "oracle": <oracle>},
  "context": <common-context-object>,
  "challenge_domain": <sorted-distinct-domain-kinds>,
  "decision_policy_digest": <decision-policy-digest>
}
```

The Evidence Binding preimage is exactly:

```json
{
  "format": "azimuth-evidence-binding-fingerprint",
  "version": 1,
  "id": <binding-id>,
  "check": <check-id>,
  "case_digest": <semantic-case-digest>,
  "method_qualification_fingerprint": <current-method-qualification-fingerprint>,
  "proposition": <binding-proposition>,
  "context": <edge-context-object>,
  "challenge_domain": <sorted-distinct-domain-kinds>,
  "decision_policy_digest": <decision-policy-digest>
}
```

The Applicability Decision preimage is exactly:

```json
{
  "format": "azimuth-applicability-decision-fingerprint",
  "version": 1,
  "binding_fingerprint": <evidence-binding-fingerprint>
}
```

The literal `decision_policy_digest` replaces the unpublished `qualification_policy_digest` in place. The earlier field is rejected and there is no alternate preimage.

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
    "id": <parent-claim-id>,
    "semantic_digest": <parent-claim-digest-including-every-case>,
    "criticality": <standard-or-critical>,
    "realization_obligation_areas": <sorted-distinct-area-ids>,
    "surface": <applicable-surface-account-or-null>
  },
  "realizations": <sorted-realization-records>,
  "mechanisms": <sorted-applicable-mechanism-records>,
  "bindings": <sorted-binding-id-and-fingerprint-records>,
  "method_qualifications": <sorted-id-expected-fingerprint-and-verdict-records>,
  "applicability_decisions": <sorted-id-expected-fingerprint-and-verdict-records>,
  "policy_digest": <decision-policy-digest>,
  "verdict": <accepted-or-rejected>,
  "basis": <ordered-non-empty-statements>,
  "residual_risks": <ordered-non-empty-statements>
}
```

A realization record is exactly `{"identity": <SourceIdentity>, "source_fingerprint":
<fingerprint>}` and sorts uniquely by `identity`. Realizations are the exact sites for the parent
Claim. Repeating one identity with the same fingerprint is a duplicate; repeating it with a
different fingerprint makes the expected Judgment unavailable. Every mechanism belongs to the
Claim and may declare exact Case relevance. Its record is exactly:

```json
{
  "id": <spec-id#mechanism-id>,
  "claim": <parent-claim-id>,
  "cases": <sorted-local-case-ids-or-empty-for-complete-claim>,
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
unavailable. Changing `cases` changes the mechanism record even when the Claim is unchanged.

The two routes are exclusive. An explicit Design binding that names a marker companion's raw id or derived assembled key is a structural error and never produces an explicit-binding mechanism record. Ordinary non-companion Artifacts may still be shared by several explicit Design bindings.

For a marker-derived record, the manifest `site` is already represented by the assembled SourceIdentity address and its area-qualified `binding` is already represented by the rewritten resolved Artifact id. The raw typed binding never enters the preimage. Neither `site`, assembled `binding`, `lang` nor `file` is added as another Claim Judgment preimage field. Language determines the semantic address kind. File is an accountable locator only.

The applicable surface account is exactly its `id`, sorted contribution `(area, mount, enumerator)` objects, one sorted witness per contribution and sorted member records. A contribution owns `area`, `mount` and `enumerator`; its nested witness has exactly enumeration `kind`, stable `identity` and `source_fingerprint`. A tagged member records its stable SourceIdentity and source fingerprint; an enumerated member records its file identity. Their tagged/enumerated variants cannot collapse. The mount id is an authored contribution identity; its path is excluded. The surface is `null` when the Claim has no `Over:`. The obligation areas are only the exact workspace obligation for the spec and Claim, or `[]`.

Binding records are exactly `{"id": <binding-id>, "fingerprint": <binding-fingerprint>}` sorted by
id. Method Qualification and Applicability Decision records are each exactly `{"id":
<decision-id>, "expected_fingerprint": <recomputed-expected-fingerprint>, "verdict":
<closed-verdict>}` sorted by id. Missing or structurally invalid composition makes the expected
Judgment unavailable; stale authored fingerprints are never used.

## Source boundary

### Mechanism implementation linkage

Source keeps the existing two-argument marker `ImplementsMechanism(<spec-id>, <mechanism-id>)` in the idiomatic ecosystem spelling. The annotation neither accepts nor owns `site`, `binding` or an Artifact id. A language extractor derives this strict pair:

```json
{
  "mechanism_implementations": [
    {
      "spec": "payments/capture",
      "mechanism": "completion-guard",
      "site": "cargo:lib:pay:pay::Capture::complete fn(&self)->bool",
      "binding": "rust-symbol:cargo:lib:pay:pay::Capture::complete fn(&self)->bool",
      "file": "src/capture.rs",
      "lang": "rust",
      "source_fingerprint": "sha256:<64-lowercase-hex>"
    }
  ],
  "artifacts": [
    {
      "id": "rust-symbol:cargo:lib:pay:pay::Capture::complete fn(&self)->bool",
      "kind": "rust-symbol",
      "file": "src/capture.rs"
    }
  ]
}
```

These are the only fields on a raw MechanismImplementation. `spec` is a lower-kebab path id and `mechanism` is one lower-kebab segment. `file` is one normalized workspace-relative locator, `lang` is one supported extractor language and `source_fingerprint` is exact `sha256:<64-lowercase-hex>`.

`site` is a non-empty qualified identity under the closed ecosystem profiles below. Except for the narrow C++ alpha profile, it contains a module, package or compilation-target identity and a declaring type or receiver. Where supported, it also contains the overload signature or generic arity needed to distinguish declarations. For example, a .NET extractor may emit `Payments.CaptureService.CompleteAsync(Payments.CompletionId,System.Threading.CancellationToken)`. A short method such as `CompleteAsync`, a path-plus-symbol such as `src/Capture.cs#CompleteAsync`, and a source path used only to distinguish overloads are invalid. The accountable emitter establishes those semantic facts. Core treats `site` as opaque: it can require a non-empty trimmed string without control characters or `|`, but it cannot prove from bytes that a module, receiver or overload is genuinely compiler-qualified.

The address-kind mapping for marker-derived mechanisms is closed in alpha 4:

- `csharp` maps to `dotnet-symbol`; and
- `cpp | go | java | javascript | kotlin | python | rust | typescript` maps to `<lang>-symbol`.

In raw extractor output, `binding` is split at its first `:` as `<address-kind>:<site>`. The prefix equals the mapping for `lang`; the complete remaining suffix equals `site` byte-for-byte, including receiver, type and overload syntax. Untyped bindings, a different prefix or suffix, a prequalified `<area>|<address-kind>|<site>` key and the retired `<kind>:<file>#<site>` form are rejected.

The raw companion Artifact requires `id`, `kind` and `file` and permits only the existing optional typed properties `unique`, `columns` and `predicate`. Its `id` equals the complete raw binding, `kind` equals the binding prefix and `file` equals the implementation locator. The raw pairing key is `(id, kind, file)`. Exactly one Artifact matches each implementation and exactly one MechanismImplementation target owns that companion. Repeating or ambiguously matching the triple, sharing it with another marker target, or mixing it with an ordinary Artifact is invalid. The same raw id in different files is retained until area assembly only when each collision is an exact, separately owned marker companion.

A paired raw companion is marker-only. Before the atomic rewrite, core derives its assembled key and rejects an explicit Design `Binding:` equal to either that key or the raw companion id. The MechanismImplementation resolves only the mechanism named by its exact `spec` and `mechanism`; the artifact id does not infer or fan out another target. Only an ordinary non-companion Artifact may be reused by several explicit Design bindings.

For the example above, both `Binding: rust-symbol:cargo:lib:pay:pay::Capture::complete fn(&self)->bool` and `Binding: payments|rust-symbol|cargo:lib:pay:pay::Capture::complete fn(&self)->bool` are invalid. By contrast, two explicit mechanisms may both declare `Binding: postgres-index:payments.capture_key` when that id resolves one ordinary, non-companion Artifact.

Project assembly resolves `file` to exactly one area and mount. It derives `<SourceIdentity> = <area>|<address-kind>|<site>` and, before any model identity, resolution or fingerprinting, atomically rewrites the implementation `binding` and companion Artifact `id` to that exact key. It also attaches `area`, `address_kind`, `address = site` and `mount` to both. For the raw example above in area `payments` and mount `code`, the exact assembled pair is:

```json
{
  "mechanism_implementations": [
    {
      "spec": "payments/capture",
      "mechanism": "completion-guard",
      "site": "cargo:lib:pay:pay::Capture::complete fn(&self)->bool",
      "binding": "payments|rust-symbol|cargo:lib:pay:pay::Capture::complete fn(&self)->bool",
      "file": "src/capture.rs",
      "lang": "rust",
      "source_fingerprint": "sha256:<64-lowercase-hex>",
      "area": "payments",
      "address_kind": "rust-symbol",
      "address": "cargo:lib:pay:pay::Capture::complete fn(&self)->bool",
      "mount": "code"
    }
  ],
  "artifacts": [
    {
      "id": "payments|rust-symbol|cargo:lib:pay:pay::Capture::complete fn(&self)->bool",
      "kind": "rust-symbol",
      "file": "src/capture.rs",
      "area": "payments",
      "address_kind": "rust-symbol",
      "address": "cargo:lib:pay:pay::Capture::complete fn(&self)->bool",
      "mount": "code"
    }
  ]
}
```

These are the exact assembled fields for the raw example, which omitted every optional Artifact property. Assembly preserves any emitted `unique`, `columns` and `predicate` value and rewrites only the companion id before attaching source identity. The canonical Artifact account always represents an absent property as `"unique": null`, `"columns": []` or `"predicate": null`. The assembled companion id is already its SourceIdentity key; it is not expanded again as `<area>|<kind>|<id>`.

Here, path-free means that the id is not derived from or extended with the workspace-relative `file`. A semantic package or module identity may retain its language-native separators.

Normally an Artifact's semantic address is its authored id. The marker companion is the one exception: assembly replaces its raw id with the area-qualified SourceIdentity key and uses that key directly as its identity. An explicit Design Artifact and every unrelated Artifact retain their authored kind/id address and are never reinterpreted or rewritten as companions.

Within one `(area, address-kind)`, a qualified site denotes one compiler declaration. Records for different marker targets may not reuse it. Repeating one `(spec, mechanism, SourceIdentity)`, naming another target at that SourceIdentity or supplying a conflicting source account is invalid. The same kind/site in two areas is legal and produces two distinct assembled binding and Artifact ids. One applicable mechanism still resolves zero or one qualified implementation; several distinct sites make its expected Claim Judgment unavailable.

An emitter fails before output when compiler or runtime metadata reports an ambiguous site in the compilation account for that record. Core does not reproduce that semantic proof. Local and federated assembly check syntax, raw binding equality, exact companion pairing, the atomic rewrite and uniqueness and consistency over each complete area. Both derive byte-identical assembled binding, Artifact id and SourceIdentity for the same area and raw record. Neither uses `file`, repository, revision or mount to repair an ambiguous semantic identity. Old records without `site`, path-bearing bindings and raw records carrying an assembled key are schema failures, not deprecated input. A marker companion referenced by an explicit Design binding is likewise rejected before rewrite, not reinterpreted as an ordinary Artifact.

The source locator of a mechanism-implementation Challenge scope item repeats this exact `site`; it does not substitute binding, Artifact id or file.

Moving unchanged source within the same area while preserving language, site and source fingerprint leaves the implementation, Claim Judgment and semantic Challenge scope identities unchanged. The new file remains visible in complete-model and launch locator accounts and changes their fingerprints. Moving across an area or changing language, site or source content is semantic.

#### Ecosystem semantic-site profiles

Every extractor resolves `--root` once and emits `file` as a non-empty normalized path below that root. The path uses `/`, contains no empty, `.` or `..` segment and contains no backslash. An input outside the root is invalid. Input files and directories select work; they never become semantic identity or alternate roots.

The C++ alpha profile accepts only a Clang-proven program-global declaration with external linkage. It rejects internal linkage, an anonymous namespace, a local declaration, a declaration attached to a C++ module, any function or enclosing class template, any constrained declaration and any canonical type containing a source locator. The exact site is:

```text
<program-global-qualified-name> <Clang canonical function type>
```

This is the one alpha profile without a package or module prefix. It is safe only because every accepted declaration has one program-global external identity. Supporting modules, templates, constraints or internal linkage requires a later build-target and module account; the extractor must reject them rather than append a translation-unit path.

For Python, `--root` is the declared semantic import root. The exact module is the selected `.py` file's normalized relative path with `/` changed to `.`, the `.py` suffix removed and a terminal `.__init__` removed. The site is `<module>.<__qualname__>`. Every module segment is a Python identifier. A file/package collision, two files resolving one module, an ambiguous namespace or an outside-root file is invalid. Positional input grouping does not affect the module.

TypeScript and JavaScript inputs belong to one whole configured project. Every selected file finds the same nearest unambiguous `tsconfig.json | jsconfig.json`; the nearest owning `package.json` at or above that config supplies one exact package name. Both configs at the same nearest level, no config, no package name, a file outside the configured project or inputs spanning projects are invalid. Input arguments only select files. Before Program creation, inputs are canonicalized by real path, deduplicated and sorted.

Discovery accepts exactly `.ts`, `.tsx`, `.mts`, `.cts`, `.js`, `.jsx`, `.mjs` and `.cjs` and rejects declaration files. One Program loads the complete config file set and compiler options. Every marked declaration has no relevant options, syntax or semantic diagnostic, including any diagnostic in a type declaration needed by its signature. A global diagnostic is relevant. If the emitter cannot prove another diagnostic irrelevant to marker or type resolution, it fails closed.

The exact TypeScript/JavaScript site account is:

```text
<package>::<module-specifier>::<receiver-kind>::<qualified-symbol><overload-set>
```

`receiver-kind` is exactly `static | instance | none`. `module-specifier` is the unique declared package export accepted by the configured resolver, or otherwise its unique package-relative compiler module specifier. The package-relative form keeps semantic module segments but contains neither an absolute path nor the workspace-relative `file`. A module move changes this value. Relocation evidence moves the complete project root while retaining package, config meaning and module specifier.

The overload set is sorted and non-empty. Each overload contains generic arity, generic constraints, parameter optional/rest modifiers and canonical parameter and return types. Generic parameters are replaced by `$0`, `$1` and so on in declaration order. The checker recursively resolves aliases to canonical target types; alias spelling does not enter the site. A path-bearing canonical type, duplicate canonical overload or type whose alias/canonical identity is unavailable is invalid.

A mechanism call emits only when its compiler symbol resolves to the `@azimuth-sh/annotations` package's `implementsMechanism` export through a direct, aliased or namespace import. A local homonym is ordinary source. A marker-shaped imported call with wrong arity or non-literal arguments, an anonymous or ambiguous enclosing declaration, an ambiguous symbol or any relevant diagnostic is a controlled extraction failure before output. JavaScript uses `javascript-symbol`; TypeScript uses `typescript-symbol`. The public two-argument marker and CLI syntax do not change.

Go uses `<package-import-path>.<receiver><function><signature>` from `go/types`. Receiver and callable type parameters share one zero-based account, receiver parameters first. Every occurrence of a compiler type parameter in receiver, constraints, parameters and results is replaced by the literal token `$<index>`; its source name is excluded. Parameter names are excluded. Constraint, parameter and result order and variadic position remain exact. For example:

```text
example.test/service.Apply[any]($0)->($0)
```

Rust accepts only a compiler-accepted source reachable from exactly one conventional Cargo target through its conventional module graph. It rejects ambiguous multi-target reachability, custom target paths, `#[path]`, generated or included source and unreachable files. Target kind is one of `lib | proc-macro | bin | example | test | bench`. The compiler crate name is the Cargo target name with `-` changed to `_`. The exact site prefix is:

```text
cargo:<target-kind>:<Cargo-target-name>:<compiler-crate-name>::<module>::<declaration>
```

The root module omits `<module>`. Inline and conventional file modules use the compiler-proven module graph, never a path guessed from `src/`. The declaration is qualified by every enclosing module, type or trait implementation.

One ASCII space separates the Rust qualified declaration from its normalized declared signature. The signature retains callable qualifiers, receiver form, declared parameter types, return type, generic constraints and where constraints. It excludes the callable name already present in the qualified declaration and excludes every value-parameter pattern name. Generic parameters are replaced in declared order by `$0`, `$1` and so on. Outside strings and lifetimes, token normalization removes whitespace around punctuation and uses one space only between adjacent word tokens. Declared type-path spelling is identity: an alias and its resolved underlying type produce different sites. This is intentionally weaker than resolved-type identity and is the complete Rust alpha contract.

.NET reflects over built assemblies rather than source. Compiled metadata resolves repeatable attributes, inheritance and generics; the site is derived from metadata alone and a PDB path never disambiguates it. Compiler-generated types and special-name methods are excluded. The exact site is the declaring type's metadata name for a type-level marker, and for a method:

```text
<declaring-type>.<method-name>``<generic-arity>(<parameter-type>,...)
```

The ``` ``<generic-arity> ``` segment is present only on a generic method definition and absent otherwise. A metadata type name is the CLR full name, so a namespace-qualified name with `+` between nested types and a `` ` ``-arity suffix on a generic definition. A by-reference type appends `&`, a pointer appends `*`, an array appends `[` with one comma fewer than its rank and `]`, a constructed generic is `<definition>[<argument>,...]`, and a generic parameter is `!<position>` when declared by the type and `!!<position>` when declared by the method. Positions are zero-based; parameter names are not identity. The address kind is `dotnet-symbol` and `lang` is `csharp`.

The file locator comes from the assembly's portable PDB, made relative to `--root`. An assembly built without a readable portable PDB yields no path and a warning rather than an invented one; a PDB document path that does not lie below `--root` is emitted as read. A marker fingerprint spans the sequence-point line range of the declaring method, extended upward to the line naming the method and over any immediately preceding attribute lines, and for an async or iterator method it is taken from the generated state machine's `MoveNext`. A type's fingerprint is the hash of its declared members' fingerprints, sorted and newline-joined. `ImplementsCheck` and `ImplementsMechanism` require a non-empty fingerprint and fail when metadata cannot produce one.

The JVM profile reads compiled classes from `--classes` and matches them to sources under `--source-root`, with `--root` fixing the emitted locator. Synthetic, anonymous and local classes, synthetic and bridge methods and `module-info` are excluded. `lang` is `kotlin` when the resolved source file ends `.kt` and `java` otherwise; the address kind follows as `kotlin-symbol` or `java-symbol`. The exact site is the binary class name for a type-level marker, and for a method:

```text
<binary-class-name>.<method-name>(<parameter-descriptor>...)<return-descriptor>
```

The descriptor is the erased JVM form: `V Z B C S I J F D` for `void`, `boolean`, `byte`, `char`, `short`, `int`, `long`, `float` and `double`, `L<binary-name-with-slashes>;` for a reference type and the JVM array spelling for an array. Generic parameters are erased and are not identity; overloads separate by their erased descriptor and by nothing else. Parameter names are excluded.

A class's source is resolved from its binary name with any `$` suffix removed and `.` changed to `/`, trying `.java`, then `.kt`, then the `Kt`-stripped `.kt` name for a Kotlin file facade. No unique source, or one source address reachable through two roots, is invalid. A `Realizes` or `ImplementsMechanism` fingerprint is the complete source file. An `ImplementsCheck` fingerprint is the exact enclosing method source: for Java the unique `javac`-parsed method with that name and parameter count, and for Kotlin the unique `fun <name>` declaration with its preceding annotation lines and its brace or expression body. A method that is not uniquely identified this way is invalid, and a method carrying both `ImplementsCheck` and `ImplementsMechanism` uses that same method span for both.

These profiles change no canonical fingerprint preimage. The preimages continue to carry the exact opaque `site` and exclude `file`. Re-emitting a site that violated a profile changes its dependent model and Plan fingerprints; the canonical serializer and published hash vectors do not change.

### Check implementation linkage

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

For Check records, workspace or project assembly attaches `area`, `mount`, `address_kind` and `address` from declared repository structure. Files and mounts remain locators; the semantic source identity is `<area>|<address-kind>|<address>`.

An implementation marker contains no Case, form, context, Method Qualification or Applicability
Decision. A native test without the marker emits nothing.

## Rejected alpha 1 input

The parser rejects old `## Claim`, `## Judgment` and residual headings, evidence floors, non-test evidence, Strength, detector fields, `Qualification policy:` and the plural `judgments.md` facet. Manifests reject `covers`, `mechanism_covers` and `observations`. Annotations reject Covers and CoversMechanism. Nothing is translated, deprecated or exported twice.
