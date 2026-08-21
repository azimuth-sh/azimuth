# Workspace format

`azimuth/workspace.json` declares durable architectural facts used by local extraction and
checking. It is strict JSON because extractors in several language ecosystems must consume the
same input without deriving a second topology.

```json
{
  "format": "azimuth-workspace",
  "version": 1,
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
      "claim": "referral-summary-explains-state",
      "areas": ["trips", "rider-experience"]
    }
  ]
}
```

## Areas

An area is a stable architectural ownership namespace (D33), not a spec domain or evidence scope.
Its mounts are normalized workspace-relative source locators. Local checks derive a relation's area
from its file and the longest containing mount; tags do not repeat the area. The shape deliberately
matches project-catalog areas except that a local workspace has no repository field.

Areas may exist without participating in a surface or obligation. Routine claims do not acquire
area declarations or linkage obligations.

## Surfaces

A surface is a named site domain with one or more independently derived contributions. Each
contribution selects a declared area mount and the enumerator that reads the system's build source.
`next-routes`, for example, reads Next.js's built route manifest. It never reconstructs membership
from `Realizes` declarations.

A site-domain claim's `Over:` value names a surface id. Every contribution must produce a witness;
one successful area cannot conceal a failed contribution from another. An enumerated member without
a discharge produces `invariant-breach`.

## Realization obligations

An optional realization obligation applies to one non-routine case-level Claim. Every
named area must contain at least one `Realizes` site for the claim. The site's area is derived from
its source locator. The declaration has no role vocabulary and does not assign evidentiary meaning.

An area obligation is not an evidence obligation. Evidence follows explicit Evidence Bindings.
One Check may bear on several areas when each Check-to-Claim edge is declared separately. Test-file
location does not establish evidence scope, and ordinary tests emit no Check linkage.
