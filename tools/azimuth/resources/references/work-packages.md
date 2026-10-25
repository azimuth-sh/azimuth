# Work-package reference

`work-packages.md` is optional local engineering coordination. Each `## Work package: <id>` declares `Status: pending | in-progress | complete`, `Depends on: none | <id>, ...`, `Owns: <checkout-relative paths>`, one `Objective:` and one `Evidence:` account.

Dependencies must exist and be acyclic. Owned paths remain inside the checkout and do not overlap. Workers edit only declared paths. The coordinator retains shared contracts, integration files, proposal state, current-model application, outcome and archival paths outside worker ownership.

Validate with `azimuth change work-packages <id>` before delegation and use `azimuth change brief <id> --package <package>` for exact handoffs. Every worker reports actual changed files, permitted verification and residuals. Independent review remains separate from authoring; several implementation workers do not automatically constitute review.

These packages are authored locally for approved parallel engineering. CI neither generates nor reports work packages, creates assignments nor launches agents. On CI failure it retains ordinary diagnostics; Dima explicitly requests any correction.
