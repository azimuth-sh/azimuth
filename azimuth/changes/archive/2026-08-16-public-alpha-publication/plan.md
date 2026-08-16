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
- [x] Retain a fresh hosted rehearsal from `azimuth-sh/azimuth`.
- [x] Establish `@azimuth-sh` administration and bounded registry credentials.
- [x] Create the annotated tag, pass the no-write preflight and retain the first write-enabled
  attempt that stopped before publication because two non-mutating probes overclaimed provider
  authorization.
- [x] Replace the unused tag at the repaired revision and retain a fresh exact-tag rehearsal.
- [x] Repeat the no-write preflight and retain the write attempt that published six observable
  targets before npm rejected a prerelease without an explicit distribution tag.
- [x] Wait for NuGet indexing, observe both provider-signed archives and identify raw archive
  equality as an invalid NuGet registry identity rule.
- [x] Establish that a targeted rerun preserves the tagged SHA but cannot bypass the failed publish
  dependency; retain candidate authority and record a separate reviewed publication revision.
- [x] Wait for provider read state to settle, pass hosted no-write run 31905158474 with eight
  preserved targets, and let run 31905266399 publish only the two absent npm tarballs.
- [x] Observe that npm assigned both first versions to `latest` despite the explicit `alpha` tag;
  retain the tarballs and add a separate tag-normalization plan and completion gate.
- [x] Run hosted and WebAuthn-authorized normalization attempts; observe that npm requires
  `latest`, then revise completion to accept the alias only while no stable version exists.
- [x] Retain run 31937065763 as adverse evidence: authenticated GHCR reads accepted two private
  images; require anonymous image reads and remove registry login from completion.
- [x] Retain a fresh completion receipt whose npm state assigns `0.1.0-alpha.1` to `alpha`, records
  no stable versions and accounts for the provider-required `latest` alias.
- [x] Retrieve every public target, judge the refreshed evidence and record the outcome.
- [x] Finalize the completed change for separate acceptance and archive.
