# Project snapshot format

A project snapshot is the finalization record over one complete, clean assembly: the catalog digest, the area topology, the selected revisions, the pinned manifest and receipt digests, the observed change authorities and the derived model fingerprint. It is strict JSON with format `azimuth-project-snapshot` and version `1`.

Core writes this document and never reads it back. It is a record of a finalization that already passed, not an input to any command.

```json
{
  "format": "azimuth-project-snapshot",
  "version": 1,
  "project": "rides",
  "catalog_digest": "<64-lowercase-hex>",
  "model_fingerprint": "<64-lowercase-hex>",
  "areas": [
    {
      "id": "payments",
      "repository": "backend",
      "mounts": [{ "id": "code", "path": "app/services/Payments" }]
    }
  ],
  "repositories": [
    {
      "id": "backend",
      "revision": "9f1c1a2b3c4d5e6f708192a3b4c5d6e7f8091a2b",
      "manifest_digest": "<64-lowercase-hex>"
    }
  ],
  "receipts": [{ "id": "integrated", "digest": "<64-lowercase-hex>" }],
  "changes": [
    {
      "id": "critical-refund",
      "state": "active",
      "repository": "backend",
      "path": "azimuth/changes/critical-refund",
      "digest": "<64-lowercase-hex>"
    }
  ]
}
```

## Fields

| Field | Type | Meaning |
|---|---|---|
| `format` | string `azimuth-project-snapshot` | document kind |
| `version` | number `1` | format version |
| `project` | string | the catalog project id |
| `catalog_digest` | string | SHA-256 of the catalog file bytes |
| `model_fingerprint` | string | SHA-256 of the derived complete model account |
| `areas` | array | the catalog area topology, restated verbatim |
| `repositories` | array | one entry per selected repository, sorted by `id` |
| `receipts` | array | one `{ id, digest }` per supplied execution receipt |
| `changes` | array | every observed change authority in the assembly |
| `accepted_change` | object | present only in an accept-change snapshot |

Every field except `accepted_change` is always written. Digests are lowercase hex with no algorithm prefix. The document is written pretty-printed, and field order is fixed as shown.

`areas[]` carries `id`, `repository` and `mounts` of `{ id, path }`, exactly as the catalog declares them. `repositories[]` carries `id`, `revision` and `manifest_digest`. `changes[]` carries `id`, `state`, `repository`, `path` and `digest`, sorted by `id`, then `state` with `active` before `archived`, then `path`, `digest` and `repository`.

`model_fingerprint` is the SHA-256 of the pretty-printed export of the complete unselected model together with its findings. It covers intent, linkage and findings, and it does not cover the workset, the checkouts or the receipts; those are covered by the revision and digest fields beside it.

A change `digest` is the SHA-256 tree digest of the change directory in its checkout: files sorted by normalized relative path, each contributing its path, a zero byte, its content and a `0xff` byte. An active change is observed at `azimuth/changes/<id>` and an archived one at `azimuth/changes/archive/<YYYY-MM-DD>-<id>`; either requires a `proposal.md` file, and a symbolic link or an untracked or ignored file under `azimuth/changes` is a violation of the observation.

## Preconditions

A snapshot is written only when all of the following hold. Each failure is reported and no file is written.

- the assembly is complete: no `--local` repository was requested and no input is missing. A partial project assembly cannot be finalized;
- no selected repository checkout is dirty;
- the assembly loads into a model with no load warnings;
- validation of that model reports zero errors and zero warnings.

`azimuth project finalize --project <file> --workset <file> --out <snapshot.json>` requires `--out`; the snapshot is never written to standard output.

## Accepted change

`azimuth project accept-change --project <file> --before <workset> --after <workset> --change <id> --date <YYYY-MM-DD> --out <snapshot.json>` writes the post-archive snapshot with one extra field:

```json
{
  "accepted_change": {
    "id": "critical-refund",
    "repository": "backend",
    "archive_date": "2026-08-22",
    "archive_digest": "<64-lowercase-hex>",
    "pre_archive_revisions": [
      { "repository": "backend", "revision": "9f1c1a2b3c4d5e6f708192a3b4c5d6e7f8091a2b" }
    ]
  }
}
```

`pre_archive_revisions` lists every repository of the pre-archive account with the revision it was selected at, in that account's sorted repository order. `archive_digest` is the archived change directory's tree digest.

`archive_date` must be ten characters, digits everywhere except a `-` at positions five and eight. Calendar validity is not checked.

Both worksets must assemble into accounts that satisfy every finalization precondition above. On top of that:

- the pre-archive account must declare the change `active` and the post-archive account must not;
- the post-archive account must declare it `archived` at exactly `azimuth/changes/archive/<date>-<id>`;
- the archiving repository must be the same in both accounts;
- the archived directory's digest must equal the active directory's digest: an archive that differs in content from the accepted change is a violation;
- the change must satisfy its completion conditions in the pre-archive account;
- the two accounts must select the same set of repositories;
- every repository other than the archiving one must keep its revision;
- the archiving repository's revision must advance;
- the archiving repository's tracked content outside the change directory must be identical before and after;
- no other change observation may move or change.

## Relationships

A snapshot restates the catalog it was derived from by digest and topology, the workset selection by revision and manifest digest, and the receipts by id and digest. It is the only federation document that carries a derived model identity, and it exists only for an account the catalog declares complete: a partial workset can locate inputs and produce a local result, but it can never produce a finalization record.
