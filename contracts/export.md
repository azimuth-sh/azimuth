# Export format

`azimuth export` writes the complete derived model as one strict JSON document. It is the extension seam: dashboards, CI integrations and downstream tools consume this document, and nothing else re-parses specs, realizations, verification authorities or manifests.

Output goes to stdout, or to `--out <file>` when given. The rendering is deterministic: two-space indentation, one member per line, object keys in construction order rather than sorted order, empty arrays and objects rendered inline as `[]` and `{}`, integral numbers rendered without a fractional part, and a single trailing newline. `sha256` over exactly these bytes is the model digest used by execution planning and accepted change records, so key order and whitespace are part of the format.

## Root

```json
{
  "version": 8,
  "extensions": [],
  "specs": [],
  "realizes": [],
  "workspace": {},
  "mechanism_implementations": [],
  "account_checks": [],
  "check_implementations": [],
  "class_members": [],
  "enumerations": [],
  "artifacts": [],
  "mechanisms": [],
  "checks": [],
  "verification_challenges": [],
  "findings": []
}
```

These are the only root keys, in exactly this order. Every one is always present. `version` has
exactly the value `8`.

The export root carries no `format` identifier. A consumer distinguishes version 8 export from
traceability by root keys: the export has `specs`, the projection has `scenarios`.

`version` is an integer, `workspace` is an object and every other value is an array.

## Ordering

Ordering is derivation order, not a canonical sort, except where stated:

- `specs` follow sorted spec file paths; Requirements and Scenarios follow declaration order within their file. Terms sort by local id in each spec export.
- `checks` and `verification_challenges` follow sorted `verification.md` paths, then declaration order within each file.
- `mechanisms` follow realization load order, then realization entry order, then mechanism order.
- `realizes`, `mechanism_implementations`, `check_implementations`, `class_members`, `enumerations` and `artifacts` follow `--manifest` argument order, then the order inside each manifest. Core concatenates and does not re-sort them.
- `findings` are sorted by `(file, line, kind, requirement, detail)`.

## Selection

`--only <pattern>` restricts the exported model to matching spec ids and the relations reachable from them. An export produced under `--only` is a partial account and is not the input to the model digest, which requires the complete unselected model.

## Extensions

Each `extensions` record is one extracted verification-package producer with `package`, `contract`,
`entity`, `site`, `file`, `lang`, `source_fingerprint` and `inputs`, then optional `schema` and the
optional `area`, `address_kind`, `address` and `mount` source-identity tail. See
[verification-packages.md](verification-packages.md).

## Specs

Each spec carries `id`, `path`, `terms` and `requirements` in that order. A Term has only `id` and `definition`; source line is retained by the parser but is not exported. An empty vocabulary is `[]`. Definitions preserve their authored Markdown. Term order in export is sorted by id, so moving an unchanged Term block does not change the model digest.

```json
{
  "id": "billing/invoices",
  "path": "azimuth/model/billing/invoices/spec.md",
  "terms": [
    {
      "id": "invoice-line",
      "definition": "One chargeable line in an invoice."
    }
  ],
  "requirements": [
    {
      "id": "invoice-totals-are-exact",
      "realization": "reviewed",
      "verification": "reviewed",
      "statement": "An invoice total SHALL equal the sum of its lines.",
      "line": 12,
      "domain": "behaviour",
      "over": null,
      "scenarios": [
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

`realization` and `verification` are each `"reviewed"` when the Requirement declares that label and `null` when it does not. `line` is
the one-based source line of the heading. `domain` is `behaviour | sites`; `over` is a surface id
for the sites domain and `null` otherwise. Requirement and Scenario `statement` values preserve their
free-form normative Markdown after outer blank lines and newline encoding are normalized. An
optional `title` follows `scenarios` when the Requirement declares one.

A site-domain Requirement—one written as `## Invariant:`—appears here with exactly one synthesized Scenario
whose id equals the Requirement id and whose `statement` is empty.

Every Requirement digest includes its domain, `over` value, review declarations and the complete ordered Scenario set. Every Scenario
digest includes the parent Requirement statement and domain plus that Scenario's free-form statement.

## Realizes

```json
{
  "spec": "billing/invoices",
  "requirement": "invoice-totals-are-exact",
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

`spec`, `requirement`, `site`, `file` and `lang` are always present. `source_fingerprint` is emitted only
when non-empty. The record then carries exactly one of two tails:

- when the relation has a semantic source identity, the four keys `area`, `address_kind`, `address` and `mount`, in that order; or
- otherwise `derived_area`, holding the id of the workspace area whose longest mount path contains `file`.

When the relation has no source identity and no mount contains the file, neither tail appears.

## Workspace

```json
{
  "path": "azimuth/workspace.json",
  "packages": [],
  "areas": [{ "id": "billing", "mounts": [{ "id": "code", "path": "services/billing" }] }],
  "surfaces": [
    {
      "id": "trips/rider-view",
      "contributions": [{ "area": "billing", "mount": "code", "enumerator": "next-routes" }]
    }
  ],
  "realization_obligations": [
    { "spec": "billing/invoices", "requirement": "rounds-half-to-even", "areas": ["billing"] }
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

## Account Checks

Each `account_checks` record is one document-owned Check from a compact `verification.md` with
`definition` (the Check record below), `rationale`, `bindings` (`[{scenario, contribution}]`),
`inputs`, `selectors`, owning `requirement`, `scenarios`, `mechanisms` and nullable `proposition`, in
that order. See [account-verification.md](account-verification.md).

## Check implementations

```json
{
  "check": "invoice-total-suite",
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
  "target_kind": "requirement",
  "target": "invoice-totals-are-exact",
  "id": "unique-invoice-number",
  "scenarios": ["rounds-half-to-even"],
  "enforcement": "schema",
  "rung": 1,
  "binding": "billing.invoice_number_unique",
  "bindings": ["billing.invoice_number_unique"],
  "expected_unique": true,
  "expected_columns": ["tenant_id", "invoice_number"],
  "expected_predicate": null
}
```

All twelve keys are always present in this order. `target_kind` is always `requirement`. `scenarios` is the
sorted stable Scenario relevance list and is empty when the mechanism bears on the complete Requirement.
`enforcement` is `type | schema | constraint | choke-point | middleware | guard`, and `rung` is the
integer that enforcement maps to: `type` and `schema` are 1, `constraint` and `choke-point` are 2,
`middleware` is 3, `guard` is 4.

`bindings` is the sorted set of resolved Artifact bindings: either one explicit realization binding or all distinct extractor-resolved sites naming that project-global mechanism. `binding` retains the sole candidate when there is exactly one and is `null` when there are none or several; consumers of composed mechanisms use `bindings`. `expected_unique` and `expected_predicate` are `null` when the realization declares none; `expected_columns` is an empty array when the realization declares none.

## Checks

```json
{
  "id": "invoice-total-suite",
  "methods": ["execution"],
  "terminal": "services/billing tests",
  "fingerprint": "sha256:<64-lowercase-hex>"
}
```

`fingerprint` is the Check fingerprint derived over the Check and its implementations.

## Verification Challenges

```json
{
  "id": "billing/mutation-search",
  "form": "mutation-search",
  "searches_for": "surviving mutants inside the invoice total path",
  "targets": [],
  "requirements": [],
  "mechanisms": [],
  "checks": [],
  "required_scope": ["check", "check-implementation"],
  "fingerprint": "sha256:<64-lowercase-hex>"
}
```

`targets`, `requirements`, `mechanisms` and `checks` restate the authored target assessment kinds and
selected Requirements, Mechanisms and Checks. Each `required_scope` entry is one of the closed semantic scope kinds in [verification.md](verification.md#semantic-challenge-scope).

## Findings

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

All nine keys are always present in this order. `entity_kind` and `requirement` are nullable. `entity_kind` names the typed model entity the Finding concerns (`requirement | scenario | mechanism | check`), not an execution Verification Target, and `requirement` holds that entity's stable ID. Typed kind accompanies the stable entity ID; it is never inferred from ID spelling. `help` is a fixed remediation sentence owned by the kind, not authored per finding. See `contracts/findings.md` for the closed category set, the exhaustive kind registry and the severity rule.

Findings are part of the exported account and therefore part of the model digest.

## What the export does not contain

- No Run ledger data. Check Results, Challenge Results, Run bundles, Assurance State and any notion of what has been executed are absent. The export is a static account of the model as authored and derived, not of anything that ran. Run facts live in run bundles; see `contracts/run-bundle.md`.
- No per-Scenario traceability rows. `azimuth report traceability` emits them as a separate projection, version 6, with root keys `version` and `scenarios`. Each row is `{id, parent_requirement, realization_rigor, verification_rigor, statement, scenario_statement, realizations, verification}`: `realization_rigor` and `verification_rigor` are the parent Requirement's `reviewed` declaration or null, `realizations` the sorted source identities of the parent Requirement's `Realizes` sites, and `verification` the sorted `{check, contribution}` pairs of Checks bound to the Scenario. It carries no decision-impact or judgment keys.
- No independent Scenario `domain` or `over` value. Scenarios inherit both from their parent Requirement.
- No derived surface membership inside the `workspace` block.

## Command boundary

```text
azimuth export [--model <dir>] [--workspace <file>]
  [--manifest <file>...] [--only <pattern>...] [--out <file>]
```

With a project catalog beside the model directory, topology comes from the catalog and `--workspace`/`--manifest` are rejected ([project-catalog.md](project-catalog.md#local-loads-without-a-workset)). Export exits zero whenever the model loads, including when it contains error-severity Findings: the Findings are the output, not a failure of the command. Exit two covers load failure — an unparsable spec, realization, verification authority, workspace or manifest, a duplicate identity, or command usage — and emits diagnostics on stderr instead of a document. Load warnings are reported on stderr and do not change the exit code.

`--out` writes with a plain file write rather than the atomic replacement used by the Run commands.

## Stable entity identities: 2026-10-02

Requirement, Scenario, Mechanism and Check identities are stable project-wide lower-kebab IDs within their typed namespaces. Project context comes from the complete account; kind remains explicit. Module membership is navigation, not identity. Source references do not encode modules. Former qualified spellings are rejected without aliases. Regenerate current artifacts and fingerprints; historical bytes and facts remain immutable. See `contracts/entity-identity.md`.
