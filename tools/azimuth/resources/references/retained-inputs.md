# Retained inputs reference

The canonical schema is `contracts/retained-inputs.md`. Retain the exact relied-on input closure with the change, conventionally `retained/manifest.json` plus `retained/files/`. Ignored runtime files alone are insufficient.

Use `azimuth retained capture --root <change-directory> --input <draft.json> --out <change-directory/retained>` and `azimuth retained verify --input <change-directory/retained/manifest.json>`. The output parent must exist; capture validates first and publishes atomically. A matching existing package is idempotent; conflicting content cannot overwrite it.

The strict draft is `{format:"azimuth-retained-inputs-draft",version:1,roots:[id],inputs:[{id,kind,path,dependencies:[id]}]}`. Input kinds are `file`, `review-input`, `requirement-review` or `run-bundle`; IDs are unique lower-kebab names. Use nonempty roots and an explicit acyclic dependency graph that reaches every supplied input. Source paths resolve below the named root without traversal, symlinks or absolute paths.

The manifest is `{format:"azimuth-retained-inputs",version:1,roots,inputs,fingerprint}`. Each input has exactly `id`, `kind`, `source_path`, `path`, `fingerprint`, `dependencies`; stored `path` is `files/<id>`. Tooling hashes actual bytes and the canonical manifest. Native review packets, sealed records, matching authored Markdown, reassessment predecessors and exact relied-on Run/Challenge proof tuples must be retained and available through each review's dependency closure. Supply semantic source/configuration/expectation dependencies explicitly; tooling cannot infer them from prose.

`source_path` preserves immutable original references. Missing original review inputs may resolve through this verified mapping without rewriting their native history. Archive movement preserves internal package paths. Conventional workflow relocation maps only its pinned owning change from the exact active path to the actual archived path; it does not search for similarly named instances. External bootstrap instances require explicit portable artifact retention and relocation, not automatic discovery.

Retain actual generated records and observations. A successful hash check proves integrity and declared/native closure, not semantic adequacy, approval or applicability to another target. CI verifies the submitted package is available and valid; it does not invent missing evidence or dispatch corrective agents.
