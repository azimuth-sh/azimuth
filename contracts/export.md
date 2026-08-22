# Export format

`azimuth export` writes the complete derived model as one strict JSON document. It is the extension
seam: dashboards, CI integrations and downstream tools consume this document, and nothing else
re-parses specs, designs, verification authorities or manifests.

Output goes to stdout, or to `--out <file>` when given. The rendering is deterministic: two-space
indentation, one member per line, object keys in construction order rather than sorted order, empty
arrays and objects rendered inline as `[]` and `{}`, integral numbers rendered without a fractional
part, and a single trailing newline. `sha256` over exactly these bytes is the model digest used by
execution planning and accepted change records, so key order and whitespace are part of the format.

## Root

```json
{
  "version": 2,
  "specs": [],
  "realizes": [],
  "workspace": {},
  "mechanism_implementations": [],
  "check_implementations": [],
  "class_members": [],
  "enumerations": [],
  "artifacts": [],
  "mechanisms": [],
  "checks": [],
  "evidence_bindings": [],
  "qualifications": [],
  "claim_judgments": [],
  "decision_policies": [],
  "challenge_schedule": null,
  "challengers": [],
  "challenge_plans": [],
  "challenge_resolutions": [],
  "findings": []
}
```

These are the only root keys, in exactly this order. Every one is always present. `version` has
exactly the value `2`.

The export root carries no `format` identifier. Every other serialized artifact — run bundle, run
launch plan, workspace, run inspection, challenge resolution, every fingerprint preimage — names
itself with a `format` string; the export does not. That is the current shape. A consumer
distinguishes an export from a traceability projection, which also carries `"version": 2` and no
`format`, by root keys: the export has `specs`, the projection has `claims` and `decision_impacts`.

`challenge_schedule` is an object when a Decision Standards file was loaded and `null` otherwise.
Every other value is an array.

## Ordering

Ordering is derivation order, not a canonical sort, except where stated:

- `specs` follow sorted spec file paths; requirements, scenarios and steps follow declaration
  order within their file.
- `checks`, `evidence_bindings`, `qualifications`, `claim_judgments`, `challengers` and
  `challenge_plans` follow sorted `verification.md` paths, then declaration order within each file.
- `mechanisms` follow design load order, then design entry order, then mechanism order.
- `realizes`, `mechanism_implementations`, `check_implementations`, `class_members`, `enumerations`
  and `artifacts` follow `--manifest` argument order, then the order inside each manifest. Core
  concatenates and does not re-sort them.
- `decision_policies` follow declaration order in the Decision Standards file.
- `challenge_resolutions` are sorted by `(plan, challenger)`.
- `findings` are sorted by `(file, line, kind, claim, detail)`.

## Selection

`--only <pattern>` restricts the exported model to matching spec ids and the relations reachable
from them. An export produced under `--only` is a partial account and is not the input to the model
digest, which requires the complete unselected model.

## Specs

```json
{
  "id": "billing/invoices",
  "path": "azimuth/model/billing/invoices/spec.md",
  "requirements": [
    {
      "id": "invoice-totals-are-exact",
      "criticality": "critical",
      "statement": "An invoice total SHALL equal the sum of its lines.",
      "line": 12,
      "scenarios": [
        {
          "id": "rounds-half-to-even",
          "line": 18,
          "steps": [{ "kind": "given", "text": "a two-line invoice" }]
        }
      ]
    }
  ]
}
```

`criticality` is `critical | standard | routine`, or `null` when the requirement declares none.
`line` is the one-based source line of the heading. Step `kind` is `given | when | then | and`.

A site-domain requirement — one written as `## Invariant:` — appears here with exactly one
synthesized scenario whose id equals the requirement id and whose `steps` array is empty.

The export omits the requirement's `domain` and `over` values. Both exist in the derived model and
both participate in the case-Claim digest, but neither is serialized here. A consumer cannot
recover a Claim's domain from an export. The synthesized-scenario shape above is not a reliable
discriminator: the parser does not require a behavioural scenario to declare steps, so an empty
`steps` array is possible in either domain.

## Realizes

```json
{
  "spec": "billing/invoices",
  "scenario": "rounds-half-to-even",
  "site": "Billing.Invoice.total",
  "file": "services/billing/invoice.cs",
  "lang": "csharp",
  "source_fingerprint": "sha256:<64-lowercase-hex>",
  "area": "billing",
  "address_kind": "assembly",
  "address": "Billing/Billing.Invoice.total",
  "mount": "code"
}
```

`spec`, `scenario`, `site`, `file` and `lang` are always present. `source_fingerprint` is emitted
only when non-empty. The record then carries exactly one of two tails:

- when the relation has a semantic source identity, the four keys `area`, `address_kind`, `address`
  and `mount`, in that order; or
- otherwise `derived_area`, holding the id of the workspace area whose longest mount path contains
  `file`.

When the relation has no source identity and no mount contains the file, neither tail appears.

## Workspace

```json
{
  "path": "azimuth/workspace.json",
  "areas": [{ "id": "billing", "mounts": [{ "id": "code", "path": "services/billing" }] }],
  "surfaces": [
    {
      "id": "trips/rider-view",
      "contributions": [{ "area": "billing", "mount": "code", "enumerator": "next-routes" }]
    }
  ],
  "realization_obligations": [
    { "spec": "billing/invoices", "claim": "rounds-half-to-even", "areas": ["billing"] }
  ]
}
```

These are the only workspace keys, in this order. `path` is the workspace file path as resolved,
and is the empty string when no workspace file exists at that path. The default path is
`workspace.json` beside the model directory. The block re-states declared workspace
facts only; derived surface membership is not part of it. Enumerated surface members reach the
export through `class_members` and `enumerations` instead.

## Mechanism implementations

```json
{
  "spec": "billing/invoices",
  "mechanism": "unique-invoice-number",
  "site": "Billing.Invoice",
  "binding": "billing.invoice_number_unique",
  "file": "services/billing/invoice.cs",
  "lang": "csharp",
  "source_fingerprint": "sha256:<64-lowercase-hex>"
}
```

`source_fingerprint` is always present here, empty string included. The optional `area`,
`address_kind`, `address` and `mount` tail follows when the relation has a semantic source
identity; there is no `derived_area` fallback on this record.

## Check implementations

```json
{
  "check": "billing/invoice-total-suite",
  "site": "Billing.Tests.TotalTests",
  "file": "services/billing/total_tests.cs",
  "lang": "csharp",
  "source_fingerprint": "sha256:<64-lowercase-hex>"
}
```

The optional source-identity tail follows on the same terms as mechanism implementations.

## Class members

```json
{
  "class": "trips/rider-view",
  "site": "app/rider/trips/[id]",
  "file": "app/web/rider/routes.json",
  "lang": "next-routes"
}
```

One derived member of a site class. The optional source-identity tail follows when present. There
is no fingerprint on this record.

## Enumerations

```json
{
  "class": "trips/rider-view",
  "kind": "next-routes",
  "source": ".next/routes-manifest.json",
  "source_fingerprint": "sha256:<64-lowercase-hex>"
}
```

Evidence that a class was enumerated from a system-produced source. The optional source-identity
tail is built from the enumeration's own identity when it has one.

## Artifacts

```json
{
  "id": "billing.invoice_number_unique",
  "kind": "index",
  "file": "services/billing/schema.sql",
  "unique": true,
  "columns": ["tenant_id", "invoice_number"],
  "predicate": "deleted_at IS NULL"
}
```

`id`, `kind` and `file` are always present. `unique` appears only when the extractor derived it,
`columns` only when non-empty, and `predicate` only when present. The optional source-identity tail
follows last.

## Mechanisms

```json
{
  "spec": "billing/invoices",
  "target_kind": "requirement",
  "target": "invoice-totals-are-exact",
  "id": "unique-invoice-number",
  "enforcement": "schema",
  "rung": 1,
  "binding": "billing.invoice_number_unique",
  "expected_unique": true,
  "expected_columns": ["tenant_id", "invoice_number"],
  "expected_predicate": null
}
```

All ten keys are always present in this order. `target_kind` is `requirement | scenario`.
`enforcement` is `type | schema | constraint | choke-point | middleware | guard`, and `rung` is the
integer that enforcement maps to: `type` and `schema` are 1, `constraint` and `choke-point` are 2,
`middleware` is 3, `guard` is 4.

`binding` is the mechanism's single resolved artifact binding. The candidate set is the mechanism's
own declared binding, if any, followed by the binding of every mechanism implementation naming that
spec and mechanism. `binding` is the sole candidate when there is exactly one, and `null` when
there are none or several. `expected_unique` and `expected_predicate` are `null` when the design
declares none; `expected_columns` is an empty array when the design declares none.

## Checks

```json
{
  "id": "billing/invoice-total-suite",
  "methods": ["execution"],
  "terminal": "services/billing tests",
  "fingerprint": "sha256:<64-lowercase-hex>"
}
```

`fingerprint` is the Check fingerprint derived over the Check and its implementations.

## Evidence bindings

```json
{
  "id": "billing/invoice-total-binding",
  "check": "billing/invoice-total-suite",
  "claim": "billing/invoices#rounds-half-to-even",
  "proposition": "the suite exercises half-to-even rounding on two-line invoices",
  "scope": "unit",
  "quantification": "example",
  "oracle": "direct",
  "context": {
    "format": "azimuth-context-fingerprint",
    "version": 1,
    "required_context": { "locale": "en-US" }
  },
  "challenge_domain": ["oracle"],
  "policy": "standard-evidence",
  "context_fingerprint": "sha256:<64-lowercase-hex>",
  "qualification_fingerprint": "sha256:<64-lowercase-hex>"
}
```

`scope` is `unit | component | e2e`. `quantification` is `example | universal`. `oracle` is
`direct | golden | relational | metamorphic | model-based | contract`. Each `challenge_domain`
entry is `realization | mechanism | check-implementation | oracle | context`.

`context` is the complete context-fingerprint preimage, with `required_context` holding the
binding's required context keys in sorted key order; the object is `{}` when the binding requires
no context.

`qualification_fingerprint` is present only when the expected Qualification fingerprint is
derivable — that is, when the binding's Check, Decision Policy and Claim all resolve in the loaded
model. Its absence marks an incomplete composition, not a stale decision.

## Qualifications

```json
{
  "id": "billing/invoice-total-binding",
  "verdict": "qualified",
  "fingerprint": "sha256:<64-lowercase-hex>",
  "qualified": "2026-08-21",
  "qualifier": "a.reviewer"
}
```

`verdict` is `qualified | rejected`. `fingerprint` is the fingerprint the reviewer authored, not
the fingerprint core derives; the derived counterpart is the binding's
`qualification_fingerprint`. Comparing the two is how a consumer detects staleness.

## Claim judgments

```json
{
  "id": "billing/invoices#rounds-half-to-even",
  "verdict": "accepted",
  "policy": "standard-evidence",
  "fingerprint": "sha256:<64-lowercase-hex>",
  "judged": "2026-08-21",
  "judge": "a.reviewer",
  "basis": ["billing/invoice-total-binding"],
  "residual_risks": ["no coverage of multi-currency invoices"],
  "expected_fingerprint": "sha256:<64-lowercase-hex>"
}
```

`verdict` is `accepted | rejected`. `fingerprint` is authored; `expected_fingerprint` is the
total-composition fingerprint core derives and is present only when that composition is derivable.

## Decision policies

```json
{
  "id": "standard-evidence",
  "required_challenges": ["adversarial-review"],
  "digest": "sha256:<64-lowercase-hex>"
}
```

`required_challenges` is emitted in declaration order. The `digest` is derived over a sorted,
deduplicated copy of that list, so two policies that differ only in the order or repetition of
required forms share one digest.

## Challenge schedule

```json
{
  "gate_challenges": ["adversarial-review"],
  "scheduled_challenges": ["mutation-search"],
  "digest": "sha256:<64-lowercase-hex>"
}
```

The single `Challenge Schedule: current` block, or `null` when no Decision Standards file was
loaded.

## Challengers

```json
{
  "id": "billing/mutation-search",
  "form": "mutation-search",
  "searches_for": "surviving mutants inside the invoice total path",
  "required_scope": ["check", "check-implementation"],
  "fingerprint": "sha256:<64-lowercase-hex>"
}
```

Each `required_scope` entry is one of `claim`, `binding`, `qualification`, `claim-judgment`,
`check`, `check-implementation`, `realization`, `mechanism`, `mechanism-implementation`,
`artifact`, `context`, `policy`, `area`, `realization-obligation`, `surface`, `surface-member`,
`enumeration`.

## Challenge plans

```json
{
  "id": "billing/mutation-plan",
  "challenger": "billing/mutation-search",
  "selectors": ["qualification from binding billing/invoice-total-binding"]
}
```

Each selector is its canonical string form: `qualification from binding <id>`, `qualification from
check <id>`, `qualification from realization <id>`, `qualification from mechanism <id>`,
`claim-judgment from claim <id>`, `claim-judgment from realization <id>` or `claim-judgment from
mechanism <id>`.

## Challenge resolutions

One record per Challenge Plan, holding the deterministic resolution of that plan against the
current model.

```json
{
  "format": "azimuth-challenge-resolution",
  "version": 1,
  "plan": "billing/mutation-plan",
  "challenger": "billing/mutation-search",
  "candidates": [
    {
      "selector": {
        "target": "qualification",
        "from": "binding",
        "id": "billing/invoice-total-binding"
      },
      "relation": { "kind": "binding", "id": "billing/invoice-total-binding" },
      "target": {
        "kind": "qualification",
        "id": "billing/invoice-total-binding",
        "expected_fingerprint": "sha256:<64-lowercase-hex>",
        "authored_fingerprint": "sha256:<64-lowercase-hex>"
      },
      "disposition": "selected"
    }
  ],
  "issues": []
}
```

These nested records are the one place in the export that names its own format. `selector.target`
and `target.kind` are `qualification | claim-judgment`. `selector.from` and `relation.kind` are
`binding | check | claim | mechanism | realization`. `target` is `null` when no decision target was
reached; `expected_fingerprint` and `authored_fingerprint` are independently nullable.

`disposition` is `selected | missing-decision | stale-decision | rejected-decision |
invalid-decision | inapplicable | unresolved-relation`. Adverse candidates are preserved rather
than pruned: a plan reports every candidate its selectors reach, so a consumer sees the adverse
siblings of a successful selector.

`issues` holds sorted, deduplicated conflict messages — currently duplicate candidates reached by
different selectors. A plan is runnable only when `issues` is empty, `candidates` is non-empty and
every candidate is `selected`.

## Findings

```json
{
  "kind": "unbound-claim",
  "category": "verification",
  "severity": "error",
  "claim": "billing/invoices#rounds-half-to-even",
  "criticality": "critical",
  "file": "azimuth/model/billing/invoices/spec.md",
  "line": 18,
  "detail": "non-routine Claim has no Evidence Binding",
  "help": "Bind at least one deliberately enrolled Check to the Claim."
}
```

All nine keys are always present in this order. `claim` and `criticality` are nullable. `help` is a
fixed remediation sentence owned by the kind, not authored per finding. See `contracts/findings.md`
for the closed category set, the exhaustive kind registry and the severity rule.

Findings are part of the exported account and therefore part of the model digest.

## What the export does not contain

- No Run ledger data. Observations, Challenge Results, Run bundles, Assurance State and any notion
  of what has been executed are absent. The export is a static account of the model as authored and
  derived, not of anything that ran. Run facts live in run bundles; see `contracts/run-bundle.md`.
- No decision-impact edges. The impact of a change on existing decisions belongs to the
  traceability projection emitted by `azimuth report traceability`, which carries them under
  `decision_impacts` alongside a per-Claim rollup. The export carries neither.
- No requirement `domain` or `over` value, as stated above.
- No derived surface membership inside the `workspace` block.

## Command boundary

```text
azimuth export [--model <dir>] [--standards <file>] [--workspace <file>]
  [--manifest <file>...] [--only <pattern>...] [--out <file>]
```

Export exits zero whenever the model loads, including when it contains error-severity Findings:
the Findings are the output, not a failure of the command. Exit two covers load failure — an
unparsable spec, design, verification authority, Decision Standards file, workspace or manifest, a
duplicate identity, or command usage — and emits diagnostics on stderr instead of a document. Load
warnings are reported on stderr and do not change the exit code.

`--out` writes with a plain file write rather than the atomic replacement used by the Run commands.
