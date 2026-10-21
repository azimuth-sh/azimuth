# Finding registry

A Finding is one deterministic defect derived from the complete model. Findings are the product of `azimuth validate` and are carried inside `azimuth export` under the root `findings` key; they are part of the exported account and therefore part of the model digest.

A Finding is never authored. Its kind, category, severity and remediation sentence are owned by the tool; only the location, Requirement or other entity reference and detail vary with the model.

## Record

```json
{
  "kind": "unbound-scenario",
  "category": "verification",
  "severity": "error",
  "verification_target_kind": "scenario",
  "requirement": "rounds-half-to-even",
  "criticality": "critical",
  "file": "azimuth/model/billing/invoices/spec.md",
  "line": 18,
  "detail": "non-routine Scenario has no Evidence Binding",
  "help": "Bind at least one deliberately enrolled Check to the Scenario."
}
```

These are the only keys, in exactly this order, and all ten are always present.

- `kind` is one stable string from the registry below.
- `category` is derived from `kind` and is never independent of it.
- `severity` is `error | warning`.
- `verification_target_kind` is `requirement | scenario | mechanism | check | null`, assigned from the finding’s typed emission context rather than inferred from ID text. It distinguishes equal stable ID names in different namespaces. Despite its key name it names the kind of model entity the Finding is attributed to, not an execution Verification Target.
- `requirement` carries the stable ID named by `verification_target_kind`, which is not necessarily a Requirement, or `null` when no entity is attributable. Module and parent paths do not form this reference.
- `criticality` is `critical | standard | routine`, or `null`. It is `null` on every Finding raised through the shared error path, including ones whose attributed entity does have a declared criticality.
- `file` is the path of the artifact carrying the defect — a spec, design, verification authority, workspace or source file, depending on the kind.
- `line` is the one-based source line, or `0` when the defect has no line — a manifest-derived relation or a workspace declaration.
- `detail` is a generated sentence naming the specific defective entity.
- `help` is the fixed remediation sentence owned by the kind. Two Findings of one kind always carry the same `help`.

Findings are sorted by `(file, line, kind, requirement, detail)`.

## Severity

Severity is not a property of the kind. Most kinds are always `error`. Six kinds derive severity
from the criticality of the Requirement they attach to: `routine` yields `warning`, and `standard`,
`critical` and an undeclared criticality all yield `error`. One kind is always `warning`.

Three of the criticality-derived kinds are unreachable for a routine Requirement because their enclosing pass skips routine Requirements first, so they are `error` in practice; they are listed as criticality-derived because that is what the code does, and a change to the enclosing filter would change the outcome without touching the severity call.

## Categories

The category set is closed:

```text
intent | realization | verification | mechanism | judgment | assurance | surface | execution
```

No current kind maps to `execution`. The category exists in the closed set and is unreachable from the current registry. A consumer that switches on `category` must still accept it.

The mapping from kind to category is exhaustive by construction: a kind not named in one of the explicit groups falls through to `verification`.

## Kinds

The registry is exhaustive. `derived` in the severity column means the criticality rule above; `derived*` marks the three kinds whose enclosing pass makes `warning` unreachable today.

| Kind | Category | Severity |
|---|---|---|
| `unclassified` | intent | error |
| `unrealized` | realization | derived* |
| `dangling-realization` | realization | error |
| `dangling-design-entry` | mechanism | error |
| `dangling-verification-scenario` | verification | error |
| `undeclared-mechanism` | mechanism | error |
| `unresolved-design-binding` | mechanism | error |
| `enforcement-mismatch` | mechanism | error |
| `missing-surface` | surface | derived |
| `unknown-surface` | surface | error |
| `enumerator-unsound-or-underived` | surface | derived |
| `invariant-breach` | surface | derived |
| `missing-required-realization` | realization | derived* |
| `dangling-realization-obligation` | realization | error |
| `dangling-mechanism-implementation` | mechanism | error |
| `unbound-scenario` | verification | derived* |
| `check-without-binding` | verification | error |
| `binding-missing-check` | verification | error |
| `binding-missing-scenario` | verification | error |
| `binding-missing-policy` | verification | error |
| `missing-method-qualification` | judgment | error |
| `dangling-method-qualification` | judgment | error |
| `rejected-method-qualification` | judgment | error |
| `stale-method-qualification` | judgment | error |
| `missing-applicability-decision` | judgment | error |
| `dangling-applicability-decision` | judgment | error |
| `rejected-applicability-decision` | judgment | error |
| `stale-applicability-decision` | judgment | error |
| `missing-requirement-judgment` | judgment | error |
| `pending-requirement-review` | judgment | error |
| `missing-requirement-assurance` | assurance | error |
| `invalid-requirement-review` | judgment | error |
| `verification-element-support-unresolved` | verification | error |
| `scenario-check-contribution-unresolved` | verification | error |
| `check-without-scenario-contribution` | verification | error |
| `mechanism-support-unresolved` | verification | error |
| `rejected-requirement-judgment` | judgment | error |
| `stale-requirement-judgment` | judgment | error |
| `invalid-requirement-judgment` | judgment | error |
| `unimplemented-check` | verification | error |
| `dangling-check-implementation` | verification | error |
| `unstable-check-implementation` | verification | error |
| `inapplicable-verification` | verification | warning |
| `missing-verification-challenge` | verification | error |
| `unresolved-challenge-plan` | verification | error |
| `missing-challenge-decision` | verification | error |
| `stale-challenge-decision` | verification | error |
| `rejected-challenge-decision` | verification | error |
| `invalid-challenge-decision` | verification | error |
| `inapplicable-challenge-decision` | verification | error |
| `unresolved-challenge-relation` | verification | error |
| `invalid-challenge-resolution` | verification | error |
| `missing-required-challenge` | verification | error |
| `insufficient-challenge-scope` | verification | error |

The table lists kinds in registry declaration order, which is not the order Findings are emitted or sorted.

### What each kind reports

- `unclassified` — a Requirement declares no criticality. A missing declaration is a semantic gap, not a parse error.
- `unrealized` — a non-routine Requirement has no production site realizing it.
- `dangling-realization` — a `Realizes` site names a Requirement that does not exist.
- `dangling-design-entry` — a design entry targets a Requirement that does not exist, or a mechanism's
  explicit Scenario relevance names no declared project Scenario.
- `dangling-verification-scenario` — a Scenario verification declaration names no declared spec Scenario.
- `undeclared-mechanism` — a critical Requirement declares no enforcement mechanism. The whole pass is gated on the design artifact being in use at all: a project with no design file is never told that every critical Requirement is a Finding.
- `unresolved-design-binding` — a mechanism has no artifact support or mixes an explicit binding with marker-derived sites.
- `enforcement-mismatch` — a mechanism's declared enforcement contradicts the derived properties of the artifact it binds.
- `missing-surface` — a site-domain Requirement declares no `Over:` surface.
- `unknown-surface` — a site-domain Requirement's `Over:` value names no declared workspace surface.
- `enumerator-unsound-or-underived` — a surface contribution produced no successful enumeration witness, so tag-derived membership is not complete.
- `invariant-breach` — an enumerated surface member discharges nothing.
- `missing-required-realization` — a realization obligation's required area contains no realization of the Requirement.
- `dangling-realization-obligation` — an obligation names a Requirement that does not exist, or one that is not a standard or critical behavioural Requirement.
- `dangling-mechanism-implementation` — an implementation marker names no design-owned mechanism.
- `unbound-scenario` — a Scenario of a non-routine Requirement has no Evidence Binding.
- `check-without-binding` — a declared Check is bound to nothing.
- `binding-missing-check`, `binding-missing-scenario`, `binding-missing-policy` — an Evidence Binding names a Check, Scenario or Decision Policy that does not exist.
- `missing-method-qualification`, `dangling-method-qualification`, `rejected-method-qualification`,
  `stale-method-qualification` — the shared method decision is absent, unreferenced, rejected or
  fingerprint-stale.
- `missing-applicability-decision`, `dangling-applicability-decision`,
  `rejected-applicability-decision`, `stale-applicability-decision` — the exact binding decision is
  absent, unreferenced, rejected or fingerprint-stale.
- `missing-requirement-judgment` — a standard or critical Requirement has no total-composition Judgment.
- `pending-requirement-review` — a standard or critical Requirement has no Judgment but has a pending historical Requirement review; no verdict is inferred.
- `missing-requirement-assurance` — a standard or critical Requirement with a compact Requirement verification facet needs a current accepted native Requirement Review and a supported exact-target Requirement Status; definitions alone establish neither.
- `invalid-requirement-review` — a Requirement review's references are incomplete or do not match its Requirement's design and verification.
- `verification-element-support-unresolved` — a required verification element has no source or configuration support.
- `scenario-check-contribution-unresolved` — a Scenario has no source-authored Check contribution.
- `check-without-scenario-contribution` — a Check declares no Scenario it contributes to.
- `mechanism-support-unresolved` — a selected mechanism has no implementation support.
- `rejected-requirement-judgment` — the current Requirement Judgment's verdict is `rejected`.
- `stale-requirement-judgment` — the authored Requirement Judgment fingerprint does not equal the derived one.
- `invalid-requirement-judgment` — the Requirement composition is incomplete, so no current Judgment can be derived against it.
- `unimplemented-check` — a Check has no stable source implementation.
- `dangling-check-implementation` — a source marker names a Check that does not exist.
- `unstable-check-implementation` — a Check implementation has no resolved semantic source identity and fingerprint.
- `inapplicable-verification` — an evidence or judgment declaration targets a routine Requirement or one
  of its Scenarios. Routine Requirements reject bindings, Method Qualifications, Applicability Decisions and
  Requirement Judgments targeted to them.
- `missing-verification-challenge` — a Challenge Plan names a Verification Challenge that does not exist.
- `unresolved-challenge-plan` — a Challenge Plan resolves no current accepted decision.
- `missing-challenge-decision`, `stale-challenge-decision`, `rejected-challenge-decision`, `invalid-challenge-decision`, `inapplicable-challenge-decision`, `unresolved-challenge-relation` — one per adverse candidate disposition of a Challenge Plan selector, mapped one-to-one from the dispositions `missing-decision`, `stale-decision`, `rejected-decision`, `invalid-decision`, `inapplicable` and `unresolved-relation`. The `selected` disposition raises nothing.
- `invalid-challenge-resolution` — conflicting declarations, currently a duplicate candidate reached through different selectors of one Plan.
- `missing-required-challenge` — a Decision Policy requires a Challenge form that has no Verification Challenge at all, or no runnable Plan reaching the decision.
- `insufficient-challenge-scope` — a runnable Plan reaches the decision but its selected semantic scope does not cover every scope kind the Verification Challenge requires.

A Verification Challenge has no aggregate score, and no Finding kind expresses one. Findings are reviewed against the specific predicate they attack.

## Domain gating

Which kinds can apply to a Requirement depends on its domain. The domain value set is closed:

```text
behaviour | sites
```

No spec field declares it. The parser assigns it structurally: a `## Invariant:` heading yields
`sites`, and a `## Requirement:` heading with its `### Scenario:` children yields `behaviour`. There is no
`Domain:` label, and an invariant accepts only `Criticality:` and `Over:`.

The gating that follows:

- `missing-surface`, `unknown-surface`, `enumerator-unsound-or-underived` and `invariant-breach` are raised only for `sites` Requirements.
- `dangling-realization-obligation` is raised for any realization obligation whose Requirement is not a standard or critical `behaviour` Requirement.
- Only the `behaviour` Requirements of the spec whose id equals a surface id contribute tag-derived
  membership to that surface. A `sites` Requirement's synthesized Scenario never counts as a member of the
  surface it ranges over.

No other kind consults the domain. Because an invariant carries one synthesized Scenario, a `sites`
Requirement participates in evidence coverage and parent Judgment on the same terms as a `behaviour`
Requirement.

The domain participates in the Requirement and Scenario digests and therefore in every fingerprint derived
from them; it is serialized on exported Requirements.

## Command boundary

```text
azimuth validate [--model <dir>] [--standards <file>] [--workspace <file>]
  [--manifest <file>...] [--only <pattern>...]
azimuth export [--model <dir>] [--standards <file>] [--workspace <file>]
  [--manifest <file>...] [--only <pattern>...] [--out <file>]
```

`azimuth validate` exits one when at least one error-severity Finding exists and zero otherwise; warnings alone do not fail it. `azimuth export` exits zero whenever the model loads, error-severity Findings included: there, the Findings are the output. Both exit two on load or usage failure.
