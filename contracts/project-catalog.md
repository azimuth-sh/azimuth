# Project catalog format

A project catalog declares which repositories, model sources, areas, decision standards and execution receipts make one complete project account. It is the single topology authority for a project: a workset supplies checkouts, and a repository manifest reports observations, but neither may widen or narrow what the catalog declares. The file is strict JSON with format `azimuth-project` and version `1`.

```json
{
  "format": "azimuth-project",
  "version": 1,
  "project": "rides",
  "repositories": [
    { "id": "backend", "required": true },
    { "id": "experience", "required": true }
  ],
  "areas": [
    {
      "id": "payments",
      "repository": "backend",
      "mounts": [
        { "id": "code", "path": "app/services/Payments" },
        { "id": "tests", "path": "app/services/Payments.Tests" }
      ]
    }
  ],
  "model_sources": [
    { "id": "system-intent", "repository": "backend", "path": "azimuth/model", "required": true }
  ],
  "standards": { "repository": "backend", "path": "azimuth/standards/verification.md" },
  "required_receipts": [
    { "id": "integrated", "subjects": ["backend", "experience"] }
  ]
}
```

## Reading rules

The document is parsed by the core JSON reader and then read field by field. Unrecognized fields are ignored. Duplicate object keys are accepted and the first occurrence wins. Every diagnostic names the catalog file and what was expected. All violations in one document are reported together; a catalog with any violation yields no project, and no assembly is attempted.

`format` must be the string `azimuth-project` and `version` the number `1`. A missing, non-string or different `format`, and a missing, non-numeric or different `version`, are both violations; a different version reports `unsupported-version`.

## Envelope

| Field | Type | Required | Meaning |
|---|---|---|---|
| `format` | string `azimuth-project` | yes | document kind |
| `version` | number `1` | yes | format version |
| `project` | non-empty string | yes | project id every other federation document must repeat |
| `repositories` | array of objects | yes | declared repository boundaries |
| `areas` | array of objects | yes | declared area topology |
| `model_sources` | array of objects | yes | declared intent owners |
| `standards` | object | yes | the singular Decision Policies and Challenge Schedule locator |
| `required_receipts` | array of objects | no | composed engineering-check requirements |

`repositories`, `areas` and `model_sources` must each be present and be arrays; an empty array is accepted. `required_receipts` may be omitted, and must be an array when present.

## Repositories

| Field | Type | Required | Meaning |
|---|---|---|---|
| `id` | non-empty string | yes | repository id |
| `required` | boolean | no | whether a complete account must select it; default `true` |

A non-boolean `required` is read as the default `true` rather than reported. Repository ids are unique; a repeated id is a violation.

## Areas

An area is a stable architectural ownership namespace owned by exactly one repository. Its mounts are the locators from which a source file's area is derived.

| Field | Type | Required | Meaning |
|---|---|---|---|
| `id` | non-empty string | yes | area id, unique across the catalog |
| `repository` | non-empty string | yes | owning repository, which must be declared |
| `mounts` | array of objects | yes | normalized repository-relative source locators |
| `mounts[].id` | non-empty string | yes | mount id, unique within the area |
| `mounts[].path` | non-empty string | yes | normalized repository-relative path |

An area whose `repository` is not a declared repository id is a violation. Mount ids are unique within one area only; two areas may both declare a mount `code`.

A normalized repository-relative path is non-empty and, after backslashes are read as `/`, consists only of ordinary path components: no absolute prefix, no root, no `.` and no `..`. A mount path that is not normalized repository-relative is a violation.

A source locator resolves to the mount with the longest matching path among the mounts of the areas owned by the reporting repository, where a match means the locator equals the mount path or continues it after a `/`. Two matching mounts of equal path length are ambiguous and are a violation of the observation, not of the catalog. Identity is `(area, typed address)`; repository, mount and path are locators and never disambiguate an identity.

## Model sources

| Field | Type | Required | Meaning |
|---|---|---|---|
| `id` | non-empty string | yes | model-source id, unique across the catalog |
| `repository` | non-empty string | yes | owning repository, which must be declared |
| `path` | non-empty string | yes | normalized repository-relative root of the intent packages |
| `required` | boolean | no | whether a complete account must include it; default `true` |

A model source whose `repository` is undeclared, or whose `path` is not normalized repository-relative, is a violation. A non-boolean `required` is read as `true`.

## Standards

`standards` is one object with non-empty `repository` and `path`. A missing `standards` object is a violation. The repository must be declared and the path must be normalized repository-relative. A project has exactly one standards locator; there is no per-repository standards declaration.

## Required receipts

| Field | Type | Required | Meaning |
|---|---|---|---|
| `id` | non-empty string | yes | receipt id, unique across the catalog |
| `subjects` | array of strings | yes | the exact repository ids the receipt must cover |

`subjects` must be present and an array; a non-string element is a violation. Subject ids are unique within one requirement and must each be a declared repository. The requirement is exact: a supplied receipt whose subject set differs from `subjects` in either direction fails assembly.

## Digest

Assembly fingerprints the catalog file as read: `catalog_digest` is the SHA-256 of the exact file bytes, lowercase hex with no algorithm prefix. It covers the whole file, including formatting and ignored fields. A project snapshot carries it verbatim.

## Relationships

A project reference names this catalog and one of its repositories. A workset supplies checkouts for its repositories and paths to execution receipts. A repository manifest observes the areas, model sources and changes the catalog assigns to one repository. A project snapshot restates the catalog id, its digest and its area topology.

Completeness is declared here and never inferred from what a workset happens to contain. A complete account selects every required repository, the repository owning every required model source, the standards repository, every declared area and every required receipt. Anything else is a partial local account, which cannot be finalized.
