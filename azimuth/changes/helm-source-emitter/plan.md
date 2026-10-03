# Plan: Helm source emitter

- [x] Separate generic Helm source extraction from the consumer's chart policy checks.
- [x] Add an experimental TypeScript emitter and document its marker and identity contract.
- [x] Add `helm-resource` assembly identity and update the manifest contract.
- [x] Build the emitter and CLI; inspect an emitted consumer manifest and the federated project observation.
- [ ] Close the canonical repository's experimental-source gate and record the outcome. Current repository instructions prohibit authoring or running tests without a direct request, so this remains open.
