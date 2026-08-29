# Work-package reference

`work-packages.md` is optional. Each `## Work package: <id>` declares `Status: pending | in-progress | complete`, `Depends on: none | <id>, ...`, `Owns: <checkout-relative paths>`, one `Objective:` and one `Evidence:` account.

Dependencies must exist and be acyclic. Owned paths must remain inside the checkout and must not overlap between packages. Workers edit only their declared paths and never own proposal state, current-facet integration, outcome or archival unless the coordinator explicitly retained those paths outside every package.
