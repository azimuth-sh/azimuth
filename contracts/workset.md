# Workset format

A workset supplies the concrete inputs for one assembly: which repository checkouts are selected,
at which revisions, with which pinned repository manifests and execution receipts. It never
declares what a project contains — a workset may be partial, and a partial workset produces a local
account rather than a complete one. The file is strict JSON with format `azimuth-workset` and
version `1`.

```json
{
  "format": "azimuth-workset",
  "version": 1,
  "project": "rides",
  "repositories": [
    {
      "id": "backend",
      "root": "../rides-backend",
      "revision": "9f1c1a2b3c4d5e6f708192a3b4c5d6e7f8091a2b",
      "manifest": "artifacts/backend.json",
      "manifest_digest": "<64-lowercase-hex>"
    }
  ],
  "receipts": [
    { "path": "artifacts/integrated.json", "digest": "<64-lowercase-hex>" }
  ]
}
```

## Reading rules

Unrecognized fields are ignored. Duplicate object keys are accepted and the first occurrence wins.
Every diagnostic names the workset file, the manifest, the receipt or the checkout it concerns.
`format` must be the string `azimuth-workset` and `version` the number `1`; a different version
reports `unsupported-version`.

Parse violations are reported together and abort before any checkout is inspected.

## Fields

| Field | Type | Required | Meaning |
|---|---|---|---|
| `format` | string `azimuth-workset` | yes | document kind |
| `version` | number `1` | yes | format version |
| `project` | non-empty string | yes | the catalog project id this workset is for |
| `repositories` | array of objects | yes | selected checkouts |
| `receipts` | array of objects | no | supplied execution receipts |

`repositories` must be present and an array; an empty array is accepted by the parser and then
fails assembly on missing inputs. `receipts` may be omitted and must be an array when present.

### Repositories

| Field | Type | Required | Meaning |
|---|---|---|---|
| `id` | non-empty string | yes | catalog repository id |
| `root` | non-empty string | yes | path to the checkout |
| `revision` | non-empty string | yes | the exact revision the checkout must be at |
| `manifest` | non-empty string | yes | path to that repository's manifest |
| `manifest_digest` | non-empty string | yes | SHA-256 of the manifest file, lowercase hex |

Repository ids are unique within a workset; a repeated id is a violation.

### Receipts

| Field | Type | Required | Meaning |
|---|---|---|---|
| `path` | non-empty string | yes | path to an execution receipt |
| `digest` | non-empty string | yes | SHA-256 of the receipt file, lowercase hex |

A receipt is identified by the `id` inside the file it names, not by its path. Two entries
resolving to the same receipt id are a violation.

## Paths and ordering

`root`, `manifest` and `path` are filesystem paths, not repository-relative locators. An absolute
path is used as given; a relative path is resolved against the directory holding the workset file.

Array order carries no meaning. Repository snapshots are sorted by id and change observations are
sorted before they reach a derived account.

## Digests

`manifest_digest` and `digest` are the SHA-256 of the exact file bytes, lowercase hex with no
algorithm prefix. They cover the whole file, including formatting and any field the reader ignores.
A mismatch is `manifest-digest-mismatch` or `receipt-digest-mismatch` and fails assembly; nothing
else is re-derived from a mismatching file.

## Assembly

Assembly takes one catalog, one workset and an optional `--local <repository>`, and either derives
a complete or local account or reports every violation it found without deriving a model.

The workset's `project` must equal the catalog's `project`. Every workset repository must be
declared by the catalog. A requested `--local` repository must be both declared by the catalog and
present in the workset. With `--local`, only that one repository is selected.

For each selected repository:

- `git rev-parse HEAD` in `root` must equal `revision`, otherwise `revision-mismatch`; a checkout
  that cannot be inspected is a violation;
- the checkout is recorded as dirty when `git status --porcelain --untracked-files=all` is
  non-empty, and also when that status cannot be read. Dirtiness does not fail assembly; it
  blocks finalization;
- the manifest file's SHA-256 must equal `manifest_digest`;
- the manifest's `project` must equal the catalog project, its `repository` must equal the entry's
  `id`, and its `revision` must equal the entry's `revision`.

### Model sources and standards

For every catalog model source owned by a selected repository, the declared path must resolve
inside the repository root, must contain no untracked and no ignored file, and the SHA-256 tree
digest of the resolved directory must equal the digest the manifest observed for that id, otherwise
`model-source-mismatch`. A required model source the manifest does not observe is
`missing-input: model-source:<id>`.

The tree digest of a directory is the SHA-256 over its files sorted by normalized relative path,
each contributing its path, a zero byte, its exact content and a `0xff` byte. A `.git` directory is
excluded. A symbolic link anywhere in the tree is a violation.

When the standards repository is selected, the declared standards path must resolve inside the
repository, must be a tracked file at that revision, and its SHA-256 must equal the manifest's
`standards_digest`, otherwise `standards-mismatch`. A manifest without `standards_digest` is
`missing-input: standards digest`.

### Areas, changes and source identities

Every area a manifest claims must be declared by the catalog and owned by the reporting repository;
a claim by another repository, or the same area claimed twice, is `area-ownership-conflict`. The
same applies to model-source observations, as `model-source-ownership-conflict`, and a repeated
observation of one model-source id is a violation.

Every catalog area — restricted to the local repository under `--local` — must be claimed by some
selected manifest, otherwise `missing-input: area:<id>`.

A manifest's change observations must equal the change directories observed in its checkout, in
both directions, otherwise `change-observation-mismatch`; a duplicate observation and a change path
that is not normalized repository-relative are violations. Across the whole assembly one change id
may be declared by exactly one repository, active or archived, otherwise
`change-authority-conflict`.

Every linkage record in a manifest must carry a typed source identity, must name an area the
manifest claims, and must resolve, from the catalog's mounts for that repository, to the same area,
mount, address kind and address it claims. Its file must resolve inside the repository and be
tracked at that revision. One identity key resolving to two different files or fingerprints
anywhere in the assembly is `source-identity-conflict`.

### Manifest inputs

Assembly reads each pinned repository manifest as strict JSON with format
`azimuth-repository-manifest` and version `1`. It requires non-empty strings `project`,
`repository`, `revision` and `producer`, an array `changes` whose entries carry non-empty `id`,
`path` and `digest` and a `state` of exactly `active` or `archived`, and a `linkage` object. It
accepts optional arrays `areas` of strings and `model_sources` of `{ id, digest }`, and an optional
string `standards_digest`. Its `linkage` object is read by the same reader that reads a bare
linkage manifest, and carries the same six collections. A repository manifest is produced by
`azimuth project observe --project <file> --repository <id> --root <dir> --producer <name/version>
--manifest <file>... --out <repository.json>`.

## Completeness

`missing_inputs` accumulates `repository:<id>`, `model-source:<id>`, `standards`, `area:<id>` and
`receipt:<id>`. An account is complete when no `--local` repository was requested and
`missing_inputs` is empty.

Without `--local`, every missing required input is also a violation and assembly fails. With
`--local`, missing inputs outside the selected repository are recorded but not reported as
violations, and model-source, standards and receipt requirements are not enforced. That is what
makes a local result a local result: it never establishes project completeness, and it can never be
finalized.

## Relationships

A workset names one catalog project, supplies checkouts for that catalog's repositories, pins one
repository manifest per selected repository and locates the execution receipts the catalog
requires. A project snapshot records the resulting revisions and manifest digests.
