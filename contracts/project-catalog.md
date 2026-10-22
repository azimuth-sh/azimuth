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
| `components` | array of objects | no | runtime component registry |
| `unshipped_areas` | array of strings | no | areas that are code or configuration but no running component; read only when `components` is present |

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

`subjects` must be present and an array; a non-string element is a violation. The key names the repositories a receipt covers; it is not an assurance Verification Target. Repository ids are unique within one required receipt and must each be a declared repository. The required receipt is exact: a supplied receipt whose repository set differs from `subjects` in either direction fails assembly.

## Components

Optional `components` is the registry of runtime units, each of which may serve one execution-context document. A component is not an area: areas are code ownership namespaces, components are what runs. The component-to-areas mapping is declared intent and the core cannot detect a wrong mapping. A catalog without `components` is read exactly as before, and a Requirement or Scenario `Context:` condition then names an unknown component.

```json
"components": [
  { "id": "web-api", "repository": "backend", "areas": ["api"],
    "context": "azimuth/execution-contexts/web-api.json" },
  { "id": "web-app", "repository": "experience", "areas": ["web", "bff"] }
],
"unshipped_areas": ["deployment"]
```

| Field | Type | Required | Meaning |
|---|---|---|---|
| `id` | non-empty string | yes | component id, lowercase kebab-case, unique across the catalog |
| `repository` | non-empty string | yes | owning repository, which must be declared |
| `areas` | non-empty array of strings | yes | areas whose code the component runs; each must be declared and owned by the same repository |
| `shared_areas` | array of strings | no | the subset of `areas` the component deliberately shares with another component |
| `context` | non-empty string | no | normalized repository-relative path of the execution-context declaration |

Structural violations, each reported with its code:

| Code | Violation |
|---|---|
| `component-duplicate-id` | an id is invalid or repeated |
| `component-unknown-area` | `areas` is empty, names an undeclared area, `shared_areas` names an area outside `areas`, or `unshipped_areas` names an undeclared area |
| `component-area-repository-mismatch` | an area belongs to a different repository than the component |
| `component-area-unmapped` | a declared area is in no component and not in `unshipped_areas` |
| `component-area-shared-undeclared` | an area is in several components and not every one lists it in `shared_areas` |
| `component-context-missing` | the declared `context` file cannot be read or parsed in the supplying checkout |

A component whose `repository` is undeclared, or whose `context` is not a normalized repository-relative path, is also a violation. The context file is read when the workset's checkout for the component's repository is available; a component without `context` declares no leaves.

### Execution-context declaration

The `context` file is strict JSON with format `azimuth-component-context`, version `1` and the string `component` equal to the registry id. `leaves` is a flat object from a dotted leaf path to `{ "type": "string" | "boolean" | "integer", "description"?: string }`. A leaf path is dot-separated names of ASCII letters, digits and `_`; hierarchy is naming only. A leaf whose name contains `password`, `secret`, `credential`, `token`, `privatekey` or `private_key` is rejected.

### Conditions

A Requirement or Scenario `Context: <component-id>: <leaf> = <value>` line is validated against this registry: `condition-unknown-component` when the component is not declared, `condition-unknown-leaf` when the component declares no such leaf, `condition-type-mismatch` when the value's type differs from the leaf's declared type, and `condition-conflict` when one component leaf is given two different values among the lines that hold together. Removing a leaf or a component therefore fails validation until its conditions are removed.

## Digest

Assembly fingerprints the catalog file as read: `catalog_digest` is the SHA-256 of the exact file bytes, lowercase hex with no algorithm prefix. It covers the whole file, including formatting and ignored fields. A project snapshot carries it verbatim.

## Relationships

A project reference names this catalog and one of its repositories. A workset supplies checkouts for its repositories and paths to execution receipts. A repository manifest observes the areas, model sources and changes the catalog assigns to one repository. A project snapshot restates the catalog id, its digest and its area topology.

Completeness is declared here and never inferred from what a workset happens to contain. A complete account selects every required repository, the repository owning every required model source, the standards repository, every declared area and every required receipt. Anything else is a partial local account, which cannot be finalized.

## Verification packages

Optional `packages` explicitly enables `azimuth.surface` and `azimuth.network`; see [verification-packages.md](verification-packages.md). Federation propagates this selection to the assembled workspace. Entity declaration and producer contracts fail when their owning package is absent. No package selection derives from source directory names.

## Release repository

Optional `release: {repository, path}` names the declared repository and normalized relative directory owning Git release requests. It supplies coordination location only, not deployment authorization or an inferred implementation dependency. The repository must be declared. The model, implementation and release repository may be the same or different. Deployment automation remains project-owned.
