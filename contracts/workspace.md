# Workspace format

`azimuth/workspace.json` declares durable architectural facts used by local extraction and checking. It is strict JSON because extractors in several language ecosystems must consume the same input without deriving a second topology.

```json
{
  "format": "azimuth-workspace",
  "version": 2,
  "areas": [
    {
      "id": "rider-experience",
      "mounts": [{ "id": "code", "path": "app/web/rider" }]
    }
  ],
  "surfaces": [
    {
      "id": "trips/rider-view",
      "contributions": [
        {
          "area": "rider-experience",
          "mount": "code",
          "enumerator": "next-routes"
        }
      ]
    }
  ],
  "realization_obligations": [
    {
      "spec": "referrals/rewards",
      "requirement": "referral-summary-explains-state",
      "areas": ["trips", "rider-experience"]
    }
  ]
}
```

The tool accepts only version 2 and reports any other version as `unsupported-version`. Version 1 named a realization obligation's Requirement `claim`; `azimuth migrate` rewrites it to version 2 on the `alpha-requirement-scenario` migration line.

## Areas

An area is a stable architectural ownership namespace, not a spec domain or evidence scope. Its mounts are normalized workspace-relative source locators. Local checks derive a relation's area from its file and the longest containing mount; tags do not repeat the area. The shape deliberately matches project-catalog areas except that a local workspace has no repository field.

Areas may exist without participating in a surface or obligation. Routine Requirements do not acquire area declarations or linkage obligations.

## Surfaces

A surface is a named site domain with one or more independently derived contributions. Each contribution selects a declared area mount and the enumerator that reads the system's build source. `next-routes`, for example, reads Next.js's built route manifest. It never reconstructs membership from `Realizes` declarations.

A site-domain Requirement's `Over:` value names a surface id. Every contribution must produce a witness; one successful area cannot conceal a failed contribution from another. An enumerated member without a discharge produces `invariant-breach`.

## Realization obligations

For an adopted Requirement-first design account, `Areas:` under a Requirement design is the authoritative realization obligation. The workspace still declares Areas and mounts, but must not repeat that Requirement's obligation in `realization_obligations`; duplicate authoring is an error. The derived obligation appears in model export and participates in missing-realization validation and Requirement Judgment freshness.

The workspace `realization_obligations` list remains available for modules that have not migrated to Requirement-first design. An optional realization obligation applies to one non-routine parent Requirement. Every named area must
contain at least one `Realizes` site for the Requirement. The site's area is derived from its source
locator. The declaration has no role vocabulary and does not assign Scenario or evidentiary meaning.

An area obligation is not an evidence obligation. Evidence follows explicit Evidence Bindings. One Check may bear on several areas when each Check-to-Requirement edge is declared separately. Test-file location does not establish evidence scope, and ordinary tests emit no Check linkage.

## Package enablement

Optional `packages` enables the verification package contracts described in [verification-packages.md](verification-packages.md). Package Surfaces publish member fields and source producer linkage; they are distinct from the existing workspace quantification `surfaces` collection. No authored enumerator ID or duplicate endpoint annotation is required for package Surfaces.
