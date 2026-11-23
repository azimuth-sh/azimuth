# Decision: a component has one report

Date: 2026-11-23. Decided by: the maintainer.

About the place where a component's report is read, the maintainer asked: "So would we have a
component report that will hold an execution context and can host other data later?". Asked
whether it should be one document per component, with a format version and a part for the
context and nothing else until a decision adds a part, the maintainer said: "Yes, I agree"

Claude's words, which the maintainer agreed to: a component exposes one document, the report of
the component, at the path that `project.toml` gives as its `report`, and its callback gives the
same document. The report holds a format version and one part, `context`. That part gives each
key the component maps, with its value as text or with no value, its source, and when it was
read. Nothing else is in the report until a decision adds a part. A new part is added under the
same path, and a change to an existing part raises the format version.

Rejected: a report of the context alone under a route named for it, because every later part
would then need a route, and a way in for each suite, of its own. Filling the report now with
what test runs record, such as which endpoint served a test, because that exists only in test
runs and stays on the side of the tests. The image digest of the component, because the
instances of a deployment are already known from outside it.

The cost: a route that can grow, held back only by a decision for each part. Every reader of the
report reads its format version.

Example: `api` declares `report = "/internal/azimuth"`. The adapter in a launch, and the server
in a deployment, read from it:

```json
{
  "format": 1,
  "context": {
    "payouts.instant": { "value": "true", "source": "configuration", "read": "2026-11-23T10:15:00Z" },
    "payouts.limit": { "value": null, "source": null, "read": "2026-11-23T10:15:00Z" }
  }
}
```
