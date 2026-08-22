# Execution receipt format

An execution receipt is a content-addressed result of composed engineering checks naming the exact
repository revisions that were evaluated. It is not a Run: it carries no Subject, no semantic Plan,
no Check identity and no Challenge Result, and it establishes no Assurance State. The file is
strict JSON with format `azimuth-execution-receipt` and version `1`.

```json
{
  "format": "azimuth-execution-receipt",
  "version": 1,
  "id": "integrated",
  "project": "rides",
  "outcome": "passed",
  "subjects": [
    { "repository": "backend", "revision": "9f1c1a2b3c4d5e6f708192a3b4c5d6e7f8091a2b" },
    { "repository": "experience", "revision": "1b2c3d4e5f60718293a4b5c6d7e8f90a1b2c3d4e" }
  ]
}
```

## Reading rules

Unrecognized fields are ignored. Duplicate object keys are accepted and the first occurrence wins.
Every diagnostic names the receipt file. `format` must be the string `azimuth-execution-receipt`
and `version` the number `1`; a different version reports `unsupported-version`. A receipt that is
unreadable, not UTF-8 or not well-formed JSON is a violation of the assembly that pinned it.

## Fields

| Field | Type | Required | Meaning |
|---|---|---|---|
| `format` | string `azimuth-execution-receipt` | yes | document kind |
| `version` | number `1` | yes | format version |
| `id` | non-empty string | yes | the catalog `required_receipts` id this receipt answers |
| `project` | non-empty string | yes | the catalog project id |
| `outcome` | non-empty string | yes | the composed result |
| `subjects` | array of objects | yes | the exact revisions evaluated |
| `subjects[].repository` | non-empty string | yes | catalog repository id |
| `subjects[].revision` | non-empty string | yes | the revision that repository was evaluated at |

`subjects` must be present and an array. The receipt carries no timestamp, no producer, no
provider locator and no per-check detail.

## Identity and digest

A receipt is identified by its `id`, not by its path: a workset may locate the file anywhere, and
two workset entries resolving to the same `id` are a violation. The receipt's digest is the SHA-256
of the exact file bytes, lowercase hex with no algorithm prefix, and the workset entry pointing at
it must pin that digest. Any edit to the file, including a changed `outcome`, changes the digest
and is reported as `receipt-digest-mismatch`.

## Requirements

A receipt is only meaningful against a catalog requirement of the same id. In an assembly:

- an `id` the catalog does not require is an unexpected receipt and is a violation;
- `project` must equal the catalog project;
- `outcome` must be exactly `passed`. Every other value, including an otherwise well-formed
  failure report, fails the assembly;
- a repeated `subjects[].repository` is a violation, and a subject naming a repository the catalog
  does not declare is a violation;
- the set of subject repositories must equal the requirement's `subjects` exactly. An omitted
  subject and an extra subject are both violations;
- each subject's `revision` must equal the revision the workset selected for that repository,
  otherwise `receipt-revision-mismatch`.

A required receipt that no workset entry supplies is `missing-input: receipt:<id>`, which is a
violation of a complete account and a recorded missing input of a local one. A subject that is
absent from the workset is a violation of a complete account only.

## Relationships

The catalog declares which receipts a complete account requires and which repositories each must
cover. The workset locates and pins receipt files. A project snapshot records each receipt's `id`
and digest. Nothing re-derives a receipt: the exact revision set is the only thing that binds it to
an account, so a receipt can never float from the revisions it was produced against to a newer set.
