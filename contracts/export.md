# Export format

`azimuth export` writes the complete derived model as one strict JSON document. It is the extension seam: dashboards, CI integrations and downstream tools consume this document, and nothing else re-parses specs, designs, verification authorities or manifests.

Output goes to stdout, or to `--out <file>` when given. The rendering is deterministic: two-space indentation, one member per line, object keys in construction order rather than sorted order, empty arrays and objects rendered inline as `[]` and `{}`, integral numbers rendered without a fractional part, and a single trailing newline. `sha256` over exactly these bytes is the model digest used by execution planning and accepted change records, so key order and whitespace are part of the format.

## Root

```json
{
  "version": 4,
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
  "method_qualifications": [],
  "applicability_decisions": [],
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
exactly the value `4`.

The export root carries no `format` identifier. A consumer distinguishes version 4 export from
traceability by root keys: the export has `specs`, the projection has `cases` and
`decision_impacts`.

`challenge_schedule` is an object when a Decision Standards file was loaded and `null` otherwise. Every other value is an array.

## Ordering

Ordering is derivation order, not a canonical sort, except where stated:

- `specs` follow sorted spec file paths; Claims and Cases follow declaration order within their file.
- `checks`, `evidence_bindings`, `method_qualifications`, `applicability_decisions`,
  `claim_judgments`, `challengers` and `challenge_plans` follow sorted `verification.md` paths, then
  declaration order within each file.
- `mechanisms` follow design load order, then design entry order, then mechanism order.
- `realizes`, `mechanism_implementations`, `check_implementations`, `class_members`, `enumerations` and `artifacts` follow `--manifest` argument order, then the order inside each manifest. Core concatenates and does not re-sort them.
- `decision_policies` follow declaration order in the Decision Standards file.
- `challenge_resolutions` are sorted by `(plan, challenger)`.
- `findings` are sorted by `(file, line, kind, claim, detail)`.

## Selection

`--only <pattern>` restricts the exported model to matching spec ids and the relations reachable from them. An export produced under `--only` is a partial account and is not the input to the model digest, which requires the complete unselected model.

## Specs

```json
{
  "id": "billing/invoices",
  "path": "azimuth/model/billing/invoices/spec.md",
  "claims": [
    {
      "id": "invoice-totals-are-exact",
      "criticality": "critical",
      "statement": "An invoice total SHALL equal the sum of its lines.",
      "line": 12,
      "domain": "behaviour",
      "over": null,
      "cases": [
        {
          "id": "rounds-half-to-even",
          "line": 18,
          "statement": "For a two-line invoice, ties round to the nearest even minor unit."
        }
      ]
    }
  ]
}
```

`criticality` is `critical | standard | routine`, or `null` when the Claim declares none. `line` is
the one-based source line of the heading. `domain` is `behaviour | sites`; `over` is a surface id
for the sites domain and `null` otherwise. Claim and Case `statement` values preserve their
free-form normative Markdown after outer blank lines and newline encoding are normalized.

A site-domain Claim—one written as `## Invariant:`—appears here with exactly one synthesized Case
whose id equals the Claim id and whose `statement` is empty.

Every Claim digest includes its domain, `over` value and the complete ordered Case set. Every Case
digest includes the parent Claim statement and domain plus that Case's free-form statement.

## Realizes

```json
{
  "spec": "billing/invoices",
  "claim": "invoice-totals-are-exact",
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

`spec`, `claim`, `site`, `file` and `lang` are always present. `source_fingerprint` is emitted only
when non-empty. The record then carries exactly one of two tails:

- when the relation has a semantic source identity, the four keys `area`, `address_kind`, `address` and `mount`, in that order; or
- otherwise `derived_area`, holding the id of the workspace area whose longest mount path contains `file`.

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

These are the only workspace keys, in this order. `path` is the workspace file path as resolved, and is the empty string when no workspace file exists at that path. The default path is `workspace.json` beside the model directory. The block re-states declared workspace facts only; derived surface membership is not part of it. Enumerated surface members reach the export through `class_members` and `enumerations` instead.

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

`source_fingerprint` is always present here, empty string included. The optional `area`, `address_kind`, `address` and `mount` tail follows when the relation has a semantic source identity; there is no `derived_area` fallback on this record.

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

One derived member of a site class. The optional source-identity tail follows when present. There is no fingerprint on this record.

## Enumerations

```json
{
  "class": "trips/rider-view",
  "kind": "next-routes",
  "source": ".next/routes-manifest.json",
  "source_fingerprint": "sha256:<64-lowercase-hex>"
}
```

Evidence that a class was enumerated from a system-produced source. The optional source-identity tail is built from the enumeration's own identity when it has one.

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

`id`, `kind` and `file` are always present. `unique` appears only when the extractor derived it, `columns` only when non-empty, and `predicate` only when present. The optional source-identity tail follows last.

## Mechanisms

```json
{
  "spec": "billing/invoices",
  "target_kind": "claim",
  "target": "invoice-totals-are-exact",
  "id": "unique-invoice-number",
  "cases": ["rounds-half-to-even"],
  "enforcement": "schema",
  "rung": 1,
  "binding": "billing.invoice_number_unique",
  "expected_unique": true,
  "expected_columns": ["tenant_id", "invoice_number"],
  "expected_predicate": null
}
```

All eleven keys are always present in this order. `target_kind` is always `claim`. `cases` is the
sorted local Case relevance list and is empty when the mechanism bears on the complete Claim.
`enforcement` is `type | schema | constraint | choke-point | middleware | guard`, and `rung` is the
integer that enforcement maps to: `type` and `schema` are 1, `constraint` and `choke-point` are 2,
`middleware` is 3, `guard` is 4.

`binding` is the mechanism's single resolved artifact binding. The candidate set is the mechanism's own declared binding, if any, followed by the binding of every mechanism implementation naming that spec and mechanism. `binding` is the sole candidate when there is exactly one, and `null` when there are none or several. `expected_unique` and `expected_predicate` are `null` when the design declares none; `expected_columns` is an empty array when the design declares none.

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
  "case": "billing/invoices#invoice-totals-are-exact/rounds-half-to-even",
  "method_qualification": "billing/invoice-total-method",
  "proposition": "the suite exercises half-to-even rounding on two-line invoices",
  "context": { "locale": "en-US" },
  "challenge_domain": ["oracle"],
  "policy": "standard-evidence",
  "context_fingerprint": "sha256:<64-lowercase-hex>",
  "applicability_fingerprint": "sha256:<64-lowercase-hex>"
}
```

Each `challenge_domain` entry is `realization | mechanism | check-implementation | oracle |
context`.

`context` is the exact edge context object; it is `{}` when the binding requires no edge-specific
context.

`applicability_fingerprint` is present only when the expected Applicability Decision fingerprint is
derivable. Its absence marks incomplete composition, not a stale decision.

## Method qualifications

```json
{
  "id": "billing/invoice-total-method",
  "check": "billing/invoice-total-suite",
  "scope": "unit",
  "quantification": "example",
  "oracle": "direct",
  "context": {"locale":"en-US"},
  "challenge_domain": ["oracle"],
  "policy": "standard-evidence",
  "verdict": "qualified",
  "fingerprint": "sha256:<64-lowercase-hex>",
  "qualified": "2026-08-21",
  "qualifier": "a.reviewer",
  "expected_fingerprint": "sha256:<64-lowercase-hex>"
}
```

`verdict` is `qualified | rejected`. `fingerprint` is authored; `expected_fingerprint` is present
when the complete method composition is derivable.

## Applicability decisions

```json
{
  "id": "billing/invoice-total-binding",
  "verdict": "applicable",
  "fingerprint": "sha256:<64-lowercase-hex>",
  "decided": "2026-08-21",
  "decider": "a.reviewer",
  "expected_fingerprint": "sha256:<64-lowercase-hex>"
}
```

`verdict` is `applicable | rejected`. The id is exactly the binding id. The expected fingerprint is
present only when the binding, Case, policy and referenced Method Qualification resolve.

## Claim judgments

```json
{
  "id": "billing/invoices#invoice-totals-are-exact",
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

`verdict` is `accepted | rejected`. `fingerprint` is authored; `expected_fingerprint` is the total-composition fingerprint core derives and is present only when that composition is derivable.

## Decision policies

```json
{
  "id": "standard-evidence",
  "required_challenges": ["adversarial-review"],
  "digest": "sha256:<64-lowercase-hex>"
}
```

`required_challenges` is emitted in declaration order. The `digest` is derived over a sorted, deduplicated copy of that list, so two policies that differ only in the order or repetition of required forms share one digest.

## Challenge schedule

```json
{
  "gate_challenges": ["adversarial-review"],
  "scheduled_challenges": ["mutation-search"],
  "digest": "sha256:<64-lowercase-hex>"
}
```

The single `Challenge Schedule: current` block, or `null` when no Decision Standards file was loaded.

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

Each `required_scope` entry is one of `applicability-decision`, `case`, `claim`, `binding`,
`method-qualification`, `claim-judgment`, `check`, `check-implementation`, `realization`,
`mechanism`, `mechanism-implementation`, `artifact`, `context`, `policy`, `area`,
`realization-obligation`, `surface`, `surface-member`, `enumeration`.

## Challenge plans

```json
{
  "id": "billing/mutation-plan",
  "challenger": "billing/mutation-search",
  "selectors": ["applicability-decision from binding billing/invoice-total-binding"]
}
```

Each selector is one of the twelve canonical forms in [verification.md](verification.md).

## Challenge resolutions

One record per Challenge Plan, holding the deterministic resolution of that plan against the current model.

```json
{
  "format": "azimuth-challenge-resolution",
  "version": 1,
  "plan": "billing/mutation-plan",
  "challenger": "billing/mutation-search",
  "candidates": [
    {
      "selector": {
        "target": "applicability-decision",
        "from": "binding",
        "id": "billing/invoice-total-binding"
      },
      "relation": { "kind": "binding", "id": "billing/invoice-total-binding" },
      "target": {
        "kind": "applicability-decision",
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

These nested records are the one place in the export that names its own format.
`selector.target` and `target.kind` are `method-qualification | applicability-decision |
claim-judgment`. `selector.from` and `relation.kind` use the closed relation kinds in the
verification contract. `target` is `null` when no decision target was reached;
`expected_fingerprint` and `authored_fingerprint` are independently nullable.

`disposition` is `selected | missing-decision | stale-decision | rejected-decision | invalid-decision | inapplicable | unresolved-relation`. Adverse candidates are preserved rather than pruned: a plan reports every candidate its selectors reach, so a consumer sees the adverse siblings of a successful selector.

`issues` holds sorted, deduplicated conflict messages — currently duplicate candidates reached by different selectors. A plan is runnable only when `issues` is empty, `candidates` is non-empty and every candidate is `selected`.

## Findings

```json
{
  "kind": "unbound-case",
  "category": "verification",
  "severity": "error",
  "claim": "billing/invoices#invoice-totals-are-exact/rounds-half-to-even",
  "criticality": "critical",
  "file": "azimuth/model/billing/invoices/spec.md",
  "line": 18,
  "detail": "non-routine Case has no Evidence Binding",
  "help": "Bind at least one deliberately enrolled Check to the Case."
}
```

All nine keys are always present in this order. `claim` and `criticality` are nullable. `help` is a fixed remediation sentence owned by the kind, not authored per finding. See `contracts/findings.md` for the closed category set, the exhaustive kind registry and the severity rule.

Findings are part of the exported account and therefore part of the model digest.

## What the export does not contain

- No Run ledger data. Observations, Challenge Results, Run bundles, Assurance State and any notion of what has been executed are absent. The export is a static account of the model as authored and derived, not of anything that ran. Run facts live in run bundles; see `contracts/run-bundle.md`.
- No decision-impact edges. The impact of a change on existing decisions belongs to the traceability projection emitted by `azimuth report traceability`, which carries them under `decision_impacts` alongside the per-Case account. The export carries neither.
- No independent Case `domain` or `over` value. Cases inherit both from their parent Claim.
- No derived surface membership inside the `workspace` block.

## Command boundary

```text
azimuth export [--model <dir>] [--standards <file>] [--workspace <file>]
  [--manifest <file>...] [--only <pattern>...] [--out <file>]
```

Export exits zero whenever the model loads, including when it contains error-severity Findings: the Findings are the output, not a failure of the command. Exit two covers load failure — an unparsable spec, design, verification authority, Decision Standards file, workspace or manifest, a duplicate identity, or command usage — and emits diagnostics on stderr instead of a document. Load warnings are reported on stderr and do not change the exit code.

`--out` writes with a plain file write rather than the atomic replacement used by the Run commands.
