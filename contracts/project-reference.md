# Project reference format

A project reference is the repository-local locator for the singular project catalog and,
optionally, an integration workset. It carries no topology of its own: every field it holds is
either a path or an assertion that must agree with the catalog it names. The file is strict JSON
with format `azimuth-project-reference` and version `1`.

```json
{
  "format": "azimuth-project-reference",
  "version": 1,
  "project": "rides",
  "repository": "experience",
  "catalog": "../rides-backend/project.json",
  "workset": "../integration/workset.json"
}
```

## Reading rules

Unrecognized fields are ignored. Duplicate object keys are accepted and the first occurrence wins.
Every diagnostic names the reference file. `format` must be the string
`azimuth-project-reference` and `version` the number `1`; a different version reports
`unsupported-version`.

Envelope violations are reported together and abort the read before the catalog is opened. Only
when the envelope is clean is the catalog loaded, and catalog violations are then reported against
the catalog file.

## Fields

| Field | Type | Required | Meaning |
|---|---|---|---|
| `format` | string `azimuth-project-reference` | yes | document kind |
| `version` | number `1` | yes | format version |
| `project` | non-empty string | yes | the project id the catalog must declare |
| `repository` | non-empty string | yes | this checkout's repository; the catalog must declare it |
| `catalog` | non-empty string | yes | path to the project catalog |
| `workset` | string | no | path to an integration workset |

`catalog` and `workset` are filesystem paths, not repository-relative locators. An absolute path is
used as given; a relative path is resolved against the directory holding the reference file. A
`workset` key present with a non-string value is a violation; an absent `workset` means the workset
is supplied by integration rather than by this repository.

## Agreement with the catalog

The named catalog must parse as a project catalog. Beyond that:

- the catalog's `project` must equal this document's `project`;
- the catalog's `repositories` must contain this document's `repository`.

Both are violations of the reference, not of the catalog. The resolved catalog path is canonicalized
once the reference is accepted; if canonicalization fails, the resolved path is kept as written.

The reference asserts nothing about the workset beyond its location. It is not validated, opened or
required to exist at resolution time.

## Relationships

A project reference locates exactly one project catalog and identifies exactly one of that
catalog's repositories. It is duplicated per checkout deliberately: it is a locator, not a second
source of project authority, and a repository's areas, model sources, standards and change
obligations are read from the catalog rather than restated here.

A reference never establishes completeness. Selecting inputs, checking revisions and deciding
whether an account is complete belong to the workset and the assembly it drives.
