# Finding registry

A Finding is one deterministic defect derived from the complete model. Findings are the product of `azimuth validate` and are carried inside `azimuth export` under the root `findings` key; they are part of the exported account and therefore part of the model digest.

A Finding is never authored. Its kind, category, severity and remediation sentence are owned by the tool; only the location, Requirement or other entity reference and detail vary with the model.

## Record

```json
{
  "kind": "unbound-scenario",
  "category": "verification",
  "severity": "error",
  "entity_kind": "scenario",
  "requirement": "rounds-half-to-even",
  "file": "azimuth/model/billing/invoices/spec.md",
  "line": 18,
  "detail": "Scenario of a Requirement that declares `Verification:` has no Evidence Binding",
  "help": "Bind at least one deliberately enrolled Check to the Scenario."
}
```

These are the only keys, in exactly this order, and all nine are always present.

- `kind` is one stable string from the registry below.
- `category` is derived from `kind` and is never independent of it.
- `severity` is `error | warning`.
- `entity_kind` is `requirement | scenario | mechanism | check | null`, assigned from the finding’s typed emission context rather than inferred from ID text. It distinguishes equal stable ID names in different namespaces. It names the kind of model entity the Finding is attributed to; it is not an execution Verification Target.
- `requirement` carries the stable ID named by `entity_kind`, which is not necessarily a Requirement, or `null` when no entity is attributable. Module and parent paths do not form this reference.
- `file` is the path of the artifact carrying the defect — a spec, realization, verification authority, workspace or source file, depending on the kind.
- `line` is the one-based source line, or `0` when the defect has no line — a manifest-derived relation or a workspace declaration.
- `detail` is a generated sentence naming the specific defective entity.
- `help` is the fixed remediation sentence owned by the kind. Two Findings of one kind always carry the same `help`.

Findings are sorted by `(file, line, kind, requirement, detail)`.

## Severity

Severity is not a property of the kind. Every kind in the current registry is emitted as `error`; `warning` remains in the closed severity set, and no current kind produces it. Kinds that concern review obligations are raised only for Requirements that declare the matching label: realization kinds and `missing-realization-review` for `Realization:`, verification kinds and `missing-requirement-assurance` for `Verification:`.

## Categories

The category set is closed:

```text
intent | realization | verification | mechanism | assurance | surface | execution
```

No current kind maps to `execution`. The category exists in the closed set and is unreachable from the current registry. A consumer that switches on `category` must still accept it.

The mapping from kind to category is exhaustive by construction: a kind not named in one of the explicit groups falls through to `verification`.

## Kinds

The registry is exhaustive: 29 kinds.

| Kind | Category | Severity |
|---|---|---|
| `condition-unknown-component` | intent | error |
| `condition-unknown-leaf` | intent | error |
| `condition-type-mismatch` | intent | error |
| `condition-conflict` | intent | error |
| `check-unknown-leaf` | verification | error |
| `unrealized` | realization | error |
| `dangling-realization` | realization | error |
| `dangling-realization-entry` | mechanism | error |
| `dangling-verification-scenario` | verification | error |
| `unresolved-realization-binding` | mechanism | error |
| `enforcement-mismatch` | mechanism | error |
| `missing-surface` | surface | error |
| `unknown-surface` | surface | error |
| `enumerator-unsound-or-underived` | surface | error |
| `invariant-breach` | surface | error |
| `missing-required-realization` | realization | error |
| `dangling-realization-obligation` | realization | error |
| `dangling-mechanism-implementation` | mechanism | error |
| `unbound-scenario` | verification | error |
| `check-without-binding` | verification | error |
| `missing-requirement-assurance` | assurance | error |
| `missing-realization-review` | assurance | error |
| `unknown-review-head` | assurance | error |
| `verification-element-support-unresolved` | verification | error |
| `scenario-check-contribution-unresolved` | verification | error |
| `check-without-scenario-contribution` | verification | error |
| `mechanism-support-unresolved` | verification | error |
| `unimplemented-check` | verification | error |
| `dangling-check-implementation` | verification | error |

The table lists kinds in registry declaration order, which is not the order Findings are emitted or sorted.

### What each kind reports

- `condition-unknown-component` — a Requirement or Scenario condition names a component the project catalog does not declare.
- `condition-unknown-leaf` — a condition names a leaf its component's execution-context declaration does not declare.
- `condition-type-mismatch` — a condition compares a leaf with a value of a different type than the leaf declares.
- `condition-conflict` — two conditions on one Requirement, or a Scenario and its inherited conditions, require different values of the same leaf.
- `check-unknown-leaf` — a Check's `Deployment:` contexts depend on a component leaf the project catalog and its execution-context declaration do not declare.
- `unrealized` — a Requirement that declares `Realization:` has no production site realizing it.
- `dangling-realization` — a `Realizes` site names a Requirement that does not exist.
- `dangling-realization-entry` — a realization entry targets a Requirement that does not exist, or a Scenario realization or mechanism's
  explicit Scenario relevance names no declared Scenario.
- `dangling-verification-scenario` — a Scenario verification declaration names no Scenario owned by its Requirement.
- `unresolved-realization-binding` — a mechanism has no artifact support, mixes an explicit binding with marker-derived sites, or its binding does not resolve to exactly one emitted artifact.
- `enforcement-mismatch` — a mechanism's declared enforcement contradicts the derived properties of the artifact it binds.
- `missing-surface` — a site-domain Requirement declares no `Over:` surface.
- `unknown-surface` — a site-domain Requirement's `Over:` value names no declared workspace surface.
- `enumerator-unsound-or-underived` — a surface contribution produced no successful enumeration witness, so tag-derived membership is not complete.
- `invariant-breach` — an enumerated surface member discharges nothing.
- `missing-required-realization` — a realization obligation's required area contains no realization of the Requirement.
- `dangling-realization-obligation` — an obligation names a Requirement that does not exist, or one that is not a `behaviour` Requirement declaring `Realization:`, or duplicates Areas authored in the realization.
- `dangling-mechanism-implementation` — an implementation marker names no realization-owned mechanism.
- `unbound-scenario` — a Scenario of a Requirement that declares `Verification:` has no Evidence Binding.
- `check-without-binding` — a declared Check is bound to nothing.
- `missing-requirement-assurance` — a Requirement declares `Verification:` and has no ratified Verification Review: no current accepted record among the change reviews and model review heads ([assurance-reviews.md](assurance-reviews.md#review-heads-in-the-model)). Validation always raises it for such a Requirement; ratification removes it. Definitions alone never clear it.
- `missing-realization-review` — a Requirement declares `Realization:` and has no ratified Realization Review, under the same rule.
- `unknown-review-head` — a model review head names a Requirement that is absent or no longer declares that review.
- `verification-element-support-unresolved` — a required verification element has no source or configuration support.
- `scenario-check-contribution-unresolved` — a Scenario has no source-authored Check contribution.
- `check-without-scenario-contribution` — a Check declares no Scenario it contributes to.
- `mechanism-support-unresolved` — a selected mechanism has no implementation support.
- `unimplemented-check` — a document-authored Check has no stable extracted source implementation.
- `dangling-check-implementation` — a source marker names a Check that does not exist.

A Verification Challenge has no aggregate score, and no Finding kind expresses one. Findings are reviewed against the specific predicate they attack.

## Domain gating

Which kinds can apply to a Requirement depends on its domain. The domain value set is closed:

```text
behaviour | sites
```

No spec field declares it. The parser assigns it structurally: a `## Invariant:` heading yields
`sites`, and a `## Requirement:` heading with its `### Scenario:` children yields `behaviour`. There is no
`Domain:` label, and an invariant header accepts only `Title:`, `Realization:`, `Verification:` and `Over:` labels.

The gating that follows:

- `missing-surface`, `unknown-surface`, `enumerator-unsound-or-underived` and `invariant-breach` are raised only for `sites` Requirements.
- `dangling-realization-obligation` is raised for any realization obligation whose Requirement is not a `behaviour` Requirement declaring `Realization:`.
- Only the `behaviour` Requirements of the spec whose id equals a surface id contribute tag-derived
  membership to that surface. A `sites` Requirement's synthesized Scenario never counts as a member of the
  surface it ranges over.

No other kind consults the domain. Because an invariant carries one synthesized Scenario, a `sites`
Requirement participates in evidence coverage and review on the same terms as a `behaviour`
Requirement.

The domain participates in the Requirement and Scenario digests and therefore in every fingerprint derived
from them; it is serialized on exported Requirements.

## Command boundary

```text
azimuth validate [--model <dir>] [--workspace <file>]
  [--manifest <file>...] [--only <pattern>...]
azimuth export [--model <dir>] [--workspace <file>]
  [--manifest <file>...] [--only <pattern>...] [--out <file>]
```

`azimuth validate` exits one when at least one error-severity Finding exists and zero otherwise; warnings alone do not fail it. `azimuth export` exits zero whenever the model loads, error-severity Findings included: there, the Findings are the output. Both exit two on load or usage failure.
