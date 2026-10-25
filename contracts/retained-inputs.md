# Retained input packages

A retained package makes the exact relied-on review and execution input closure portable after local scratch files or queue branches disappear. It is not an approval, source-correctness claim or permission to publish. Native record validation and exact content identity remain separate from semantic completeness and evidence applicability.

## Capture

```text
azimuth retained capture --root <change-directory> --input <draft.json> --out <existing-parent/package-directory>
azimuth retained verify --input <package-directory/manifest.json>
```

The strict draft is `azimuth-retained-inputs-draft` version 1:

```json
{
  "format": "azimuth-retained-inputs-draft",
  "version": 1,
  "roots": ["accepted-review"],
  "inputs": [
    {"id":"accepted-review","kind":"verification-review","path":"reviews/verification-reviews.json","dependencies":["review-markdown","review-packet"]},
    {"id":"review-markdown","kind":"file","path":"reviews/verification-reviews.md","dependencies":["review-packet"]},
    {"id":"review-packet","kind":"review-input","path":".azimuth/review-input.json","dependencies":[]}
  ]
}
```

Inputs use unique lower-kebab IDs. Kinds are exactly `file`, `review-input`, `verification-review`, `realization-review` or `run-bundle`. Source paths resolve strictly below the explicit root without symlinks. Normal relative paths have no absolute prefix, empty components, traversal, backslash, fragment or NUL. Roots are nonempty; dependencies are explicit IDs, exist, are acyclic and reach every retained input. Required native proof/source dependencies must be supplied; unexplained unused files are rejected instead of silently retained.

Capture reads and fingerprints the exact opened source bytes, validates the complete package before publication, writes and syncs a staging directory, and publishes by directory rename. An existing identical verified package is an idempotent replay; a different package at the same destination is rejected. It does not overwrite records or edit their content. The output parent must exist. Packages cannot live in `.azimuth`, `.azimuth-ci` or `.git`; required retention is distinct from scratch storage.

## Manifest

The output `manifest.json` is `azimuth-retained-inputs` version 1 with exact fields `format`, `version`, `roots`, `inputs`, `fingerprint`. Each input has exactly `id`, `kind`, `source_path`, `path`, `fingerprint`, `dependencies`. `source_path` preserves the original logical locator; stored `path` is exactly `files/<id>`, relative to the manifest directory. The lowercase `sha256:` input fingerprint hashes the actual bytes. The manifest fingerprint hashes its canonical JSON without its own fingerprint field. Entries and ID collections are emitted deterministically.

Verification rejects unknown/duplicate fields, malformed references, cycles, unused inputs, symlinks, missing/nonregular files, changed bytes and a changed manifest fingerprint. It validates native Run schema/history, native review records and native review-input packet grammar. A sealed review's retained Markdown section must match its native artifact digest; its exact original Input reference must map to a retained input packet with the stated fingerprint. Reassessment predecessors and exact Challenge Run/result/target proof tuples must be present in retained native history, including Runs embedded in prepared packets.

This is a machine-checkable declared/native dependency closure. It does not infer missing semantic source dependencies from arbitrary method prose or authenticate content merely because it was captured.

## Conventional change package

The final relied-on package is `<change>/retained/manifest.json` and its `files/` directory. Finalization of a workflow with a `finalize-change` action requires this package; every sealed JSON review under the change's `reviews/` must be mapped as a native `verification-review` or `realization-review` input of its own kind with matching bytes and complete dependencies. Actual original Markdown/sealed outputs remain at their owning locations as well; the package preserves exact prior packet/proof bytes.

Review compilation resolves an unavailable original Input path through this verified `source_path` map. Thus an immutable Markdown review can retain its original `.azimuth/...` locator without depending on that ignored local file remaining present. The fallback does not rewrite the review or its input fingerprint. Archive relocation preserves the package's internal paths.

Ratification loads native Run proofs from the explicitly named conventional manifests of active and archived changes, including embedded packet proofs. It verifies correction history and passes those actual Runs to review currentness assessment. It never scans ignored runtime directories for evidence and never substitutes an empty proof set for a declared Challenge basis.

The package is retained with the change or in declared delivery storage accessible at the submitted revision. A present local package alone does not prove it was included in Git; CI verifies the submitted state. Scratch artifacts and unrelied-on exploratory Runs need not all be copied. Git, policy and operational storage retention remain explicit responsibilities outside raw hash validation.
