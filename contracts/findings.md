# Finding registry

A Finding is one deterministic defect derived from the complete model. Findings are the product of `azimuth validate` and are carried inside `azimuth export` under the root `findings` key; they are part of the exported account and therefore part of the model digest.

A Finding is never authored. Its kind, category, severity and remediation sentence are owned by the tool; only the location, claim and detail vary with the model.

## Record

```json
{
  "kind": "unbound-case",
  "category": "verification",
  "severity": "error",
  "claim": "billing/invoices#rounding/rounds-half-to-even",
  "criticality": "critical",
  "file": "azimuth/model/billing/invoices/spec.md",
  "line": 18,
  "detail": "non-routine Case has no Evidence Binding",
  "help": "Bind at least one deliberately enrolled Check to the Case."
}
```

These are the only keys, in exactly this order, and all nine are always present.

- `kind` is one stable string from the registry below.
- `category` is derived from `kind` and is never independent of it.
- `severity` is `error | warning`.
- `claim` retains its historical field name. Its value is the most exact affected parent Claim or
  nested Case id (`<spec>#<claim>` or `<spec>#<claim>/<case>`), or `null` when no intent entity is
  attributable. The key is not duplicated as `case`.
- `criticality` is `critical | standard | routine`, or `null`. It is `null` on every Finding raised through the shared error path, including ones whose subject does have a declared criticality.
- `file` is the path of the artifact carrying the defect — a spec, design, verification authority, workspace or source file, depending on the kind.
- `line` is the one-based source line, or `0` when the defect has no line — a manifest-derived relation or a workspace declaration.
- `detail` is a generated sentence naming the specific subject.
- `help` is the fixed remediation sentence owned by the kind. Two Findings of one kind always carry the same `help`.

Findings are sorted by `(file, line, kind, claim, detail)`.

## Severity

Severity is not a property of the kind. Most kinds are always `error`. Six kinds derive severity
from the criticality of the Claim they attach to: `routine` yields `warning`, and `standard`,
`critical` and an undeclared criticality all yield `error`. One kind is always `warning`.

Three of the criticality-derived kinds are unreachable for a routine Claim because their enclosing pass skips routine Claims first, so they are `error` in practice; they are listed as criticality-derived because that is what the code does, and a change to the enclosing filter would change the outcome without touching the severity call.

## Categories

The category set is closed:

```text
intent | realization | verification | mechanism | judgment | surface | execution
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
| `unbound-case` | verification | derived* |
| `check-without-binding` | verification | error |
| `binding-missing-check` | verification | error |
| `binding-missing-case` | verification | error |
| `binding-missing-policy` | verification | error |
| `missing-method-qualification` | judgment | error |
| `dangling-method-qualification` | judgment | error |
| `rejected-method-qualification` | judgment | error |
| `stale-method-qualification` | judgment | error |
| `missing-applicability-decision` | judgment | error |
| `dangling-applicability-decision` | judgment | error |
| `rejected-applicability-decision` | judgment | error |
| `stale-applicability-decision` | judgment | error |
| `missing-claim-judgment` | judgment | error |
| `rejected-claim-judgment` | judgment | error |
| `stale-claim-judgment` | judgment | error |
| `invalid-claim-judgment` | judgment | error |
| `unimplemented-check` | verification | error |
| `dangling-check-implementation` | verification | error |
| `unstable-check-implementation` | verification | error |
| `inapplicable-verification` | verification | warning |
| `missing-challenger` | verification | error |
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

- `unclassified` — a Claim declares no criticality. A missing declaration is a semantic gap, not a parse error.
- `unrealized` — a non-routine Claim has no production site realizing it.
- `dangling-realization` — a `Realizes` site names a Claim that does not exist.
- `dangling-design-entry` — a design entry targets a Claim that does not exist, or a mechanism's
  explicit Case relevance names no local Case under that Claim.
- `undeclared-mechanism` — a critical Claim declares no enforcement mechanism. The whole pass is gated on the design artifact being in use at all: a project with no design file is never told that every critical Claim is a Finding.
- `unresolved-design-binding` — a mechanism resolves to zero or several artifact bindings.
- `enforcement-mismatch` — a mechanism's declared enforcement contradicts the derived properties of the artifact it binds.
- `missing-surface` — a site-domain Claim declares no `Over:` surface.
- `unknown-surface` — a site-domain Claim's `Over:` value names no declared workspace surface.
- `enumerator-unsound-or-underived` — a surface contribution produced no successful enumeration witness, so tag-derived membership is not complete.
- `invariant-breach` — an enumerated surface member discharges nothing.
- `missing-required-realization` — a realization obligation's required area contains no realization of the Claim.
- `dangling-realization-obligation` — an obligation names a Claim that does not exist, or one that is not a standard or critical behavioural Claim.
- `dangling-mechanism-implementation` — an implementation marker names no design-owned mechanism.
- `unbound-case` — a Case of a non-routine Claim has no Evidence Binding.
- `check-without-binding` — a declared Check is bound to nothing.
- `binding-missing-check`, `binding-missing-case`, `binding-missing-policy` — an Evidence Binding names a Check, Case or Decision Policy that does not exist.
- `missing-method-qualification`, `dangling-method-qualification`, `rejected-method-qualification`,
  `stale-method-qualification` — the shared method decision is absent, unreferenced, rejected or
  fingerprint-stale.
- `missing-applicability-decision`, `dangling-applicability-decision`,
  `rejected-applicability-decision`, `stale-applicability-decision` — the exact binding decision is
  absent, unreferenced, rejected or fingerprint-stale.
- `missing-claim-judgment` — a standard or critical Claim has no total-composition Judgment.
- `rejected-claim-judgment` — the current Claim Judgment's verdict is `rejected`.
- `stale-claim-judgment` — the authored Claim Judgment fingerprint does not equal the derived one.
- `invalid-claim-judgment` — the Claim composition is incomplete, so no current Judgment can be derived against it.
- `unimplemented-check` — a Check has no stable source implementation.
- `dangling-check-implementation` — a source marker names a Check that does not exist.
- `unstable-check-implementation` — a Check implementation has no resolved semantic source identity and fingerprint.
- `inapplicable-verification` — an evidence or judgment declaration targets a routine Claim or one
  of its Cases. Routine Claims reject bindings, Method Qualifications, Applicability Decisions and
  Claim Judgments targeted to them.
- `missing-challenger` — a Challenge Plan names a Challenger that does not exist.
- `unresolved-challenge-plan` — a Challenge Plan resolves no current accepted decision.
- `missing-challenge-decision`, `stale-challenge-decision`, `rejected-challenge-decision`, `invalid-challenge-decision`, `inapplicable-challenge-decision`, `unresolved-challenge-relation` — one per adverse candidate disposition of a Challenge Plan selector, mapped one-to-one from the dispositions `missing-decision`, `stale-decision`, `rejected-decision`, `invalid-decision`, `inapplicable` and `unresolved-relation`. The `selected` disposition raises nothing.
- `invalid-challenge-resolution` — conflicting declarations, currently a duplicate candidate reached through different selectors of one Plan.
- `missing-required-challenge` — a Decision Policy requires a Challenge form that has no Challenger at all, or no runnable Plan reaching the decision.
- `insufficient-challenge-scope` — a runnable Plan reaches the decision but its selected semantic scope does not cover every scope kind the Challenger requires.

A Challenger has no aggregate score, and no Finding kind expresses one. Findings are reviewed against the specific predicate they attack.

## Domain gating

Which kinds can apply to a Claim depends on its domain. The domain value set is closed:

```text
behaviour | sites
```

No spec field declares it. The parser assigns it structurally: a `## Invariant:` heading yields
`sites`, and a `## Claim:` heading with its `### Case:` children yields `behaviour`. There is no
`Domain:` label, and an invariant accepts only `Criticality:` and `Over:`.

The gating that follows:

- `missing-surface`, `unknown-surface`, `enumerator-unsound-or-underived` and `invariant-breach` are raised only for `sites` Claims.
- `dangling-realization-obligation` is raised for any realization obligation whose Claim is not a standard or critical `behaviour` Claim.
- Only the `behaviour` Claims of the spec whose id equals a surface id contribute tag-derived
  membership to that surface. A `sites` Claim's synthesized Case never counts as a member of the
  surface it ranges over.

No other kind consults the domain. Because an invariant carries one synthesized Case, a `sites`
Claim participates in evidence coverage and parent Judgment on the same terms as a `behaviour`
Claim.

The domain participates in the Claim and Case digests and therefore in every fingerprint derived
from them; it is serialized on exported Claims.

## Command boundary

```text
azimuth validate [--model <dir>] [--standards <file>] [--workspace <file>]
  [--manifest <file>...] [--only <pattern>...]
azimuth export [--model <dir>] [--standards <file>] [--workspace <file>]
  [--manifest <file>...] [--only <pattern>...] [--out <file>]
```

`azimuth validate` exits one when at least one error-severity Finding exists and zero otherwise; warnings alone do not fail it. `azimuth export` exits zero whenever the model loads, error-severity Findings included: there, the Findings are the output. Both exit two on load or usage failure.
