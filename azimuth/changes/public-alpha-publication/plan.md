# Plan: Publish the first public alpha

- [x] Add deterministic registry-state adapters and provider fixture tests.
- [x] Add tag, rehearsal-account and credential preflight with a strict no-write mode.
- [x] Add the owner-dispatched retained-candidate publication workflow.
- [x] Add post-publication retrieval, GHCR provenance and completion receipt generation.
- [x] Run local gates and a hosted no-write rehearsal from the candidate branch.
- [x] Transfer the frozen distribution repository to `azimuth-sh/azimuth`.
- [x] Record that identity replacement is not a supported projected intent delta in this version.
- [x] Revise npm, source, homepage and GHCR identities; run every local component gate and confirm
  the composed gate stops only at the stale pre-transfer release receipt.
- [ ] Retain a fresh hosted rehearsal from `azimuth-sh/azimuth`.
- [ ] Establish `@azimuth-sh` administration and bounded registry credentials.
- [ ] Create the annotated tag and run the resumable publication operation.
- [ ] Retrieve every public target, judge the refreshed evidence and record the outcome.
- [ ] Finalize the completed change for separate acceptance and archive.
