# Outcome: Compact verification packages

## Implemented

`tools/azimuth/src/verification_packages.rs` parses explicitly enabled `azimuth.surface` and `azimuth.network` declarations, compact Checks and inline Check–Case Evidence Bindings. Method prose remains authored prose; no Terminal, approved qualification or Claim Judgment is synthesized. Producer descriptors publish source-derived field types and supported values. Validation covers package ownership, references, selectors, source identities, schemas, complete runtime enumerations, expectation records and joins in both directions. Expectation input files participate in producer and execution fingerprints.

Package annotations and extraction integration live in `packages/dotnet`, `packages/python` and `tools/extractors`. Source producers have ecosystem semantic sites and exact qualified identities. Loading preserves supplied qualified identities and derives only missing local identities. Untagged external or unmaterialized generated .NET source is omitted; tagged source outside the admitted root fails closed. The .NET Surface producer describes the designated output contract, serialized enum values and nullable fields. Python network producers and structural Rego Check sites support consumer deployment implementations.

`tools/azimuth/src/assurance_review.rs` supplies strict authority projections, independent review records, freshness and dependency assessment, and exact-Subject state. `services/assurance/server/src/run_ledger.rs` persists project-scoped Runs, corrections, authority selections and reviews in PostgreSQL. Producer, reviewer and owner permissions are distinct. Explicit authority selection uses compare-and-swap preconditions. State joins current authority, independent reviews and latest correction heads without inferring positive judgments. Actual ledger storage also exposed object-key-order sensitivity in mechanism dependencies: matching now compares typed IDs and fingerprints, and review targets are normalized before latest-review lookup. Reordered JSON objects retain the same review meaning; changed dependency fingerprints still produce stale assessment. Stored records do not require rewriting. Credential configuration rejects duplicate JSON keys; correction-history conflicts return HTTP 409; valid current-authority replay returns HTTP 200 without appending history.

`packages/python/azimuth_run/protocol.py` constructs accountable Check-only imports. Actual integration found and fixed two boundary errors: the normalizer ID must be `adapter/<adapter-id>`, and native zero-based attempt ordinals must become the Run protocol's one-based ordinals. Missing native units remain visible, inconclusive work. The helper preserves native execution intervals and correction anchors; normalization does not renew old measurements.

Public contracts, managed references, authoring guidance and framework documentation agree with this implementation. The CLI was rebuilt with the updated embedded resources. Release source producers emit current global Claim records without the retired `spec` field. The experimental Helm root is aligned with the release acceptance declaration and has a real npm build relation in the polyglot gate.

## Engineering verification

Dima explicitly authorized these relevant canonical tests, overriding the repository's default restriction for this task. Tooling regressions use synthetic fixtures and do not depend on consumer vocabulary or checkouts.

| Command | Result | Report |
| --- | --- | --- |
| `cargo test --manifest-path tools/azimuth/Cargo.toml --target-dir /tmp/azimuth-packages-build --test adapter_host --test run --test run_plan --test compact_verification_packages --test compact_assurance_reviews --test qualified_source_identity --no-fail-fast` | 107 passed: host 16, Run 49, planning 31, compact package 4, review 5, qualified identity 2 | `/private/tmp/azimuth-relevant-regressions.log` |
| `npm test`, in `tools/extractors/typescript` | Full build and 62 tests passed | `/private/tmp/azimuth-ts-current-tests.log` |
| `PYTHONPATH=packages/python python3 -m unittest azimuth_run.test_protocol -v` | 9 passed | `/private/tmp/azimuth-python-run-tests.log` |
| `python3 -m unittest -v release.test_qualify release.test_isolate_experiments release.test_orchestrate` | 37 passed | `/private/tmp/azimuth-release-producer-tests.log` |
| `cargo build --manifest-path tools/azimuth/Cargo.toml --target-dir /tmp/azimuth-packages-build` | Passed; updated CLI | `/tmp/azimuth-packages-build/debug/azimuth` |
| `npm run build`, in `tools/extractors/helm` | Passed | Actual emitter invocation below |

The cross-language host regression invokes the real Python builder, parses and verifies its output with core, and passes it through the bounded adapter host. It checks both covered work and missing-unit inconclusion. Service and .NET emitter builds passed. The independent service review and its remediation record are in `/private/tmp/ingress-assurance-service-review.md`; it is independent of service implementation, not of the core helpers authored by that reviewer.

Actual product emission with the current Helm executable and the consumer's chart succeeded, producing `/private/tmp/azimuth-helm-current-product.json`. A genuine local HTTP probe, exact local deployment Subject, native normalizer, immutable adapter registration, `azimuth run import` and standalone `azimuth run verify` succeeded. The retained control is `/private/tmp/deployment-native-wire-control/run.json`, using `/private/tmp/ingress-wire-ordinal-registration/adapters.json`. Its public origin is deliberately unverified; the resulting inconclusive observation establishes protocol interoperability and supplies no hosted-isolation evidence.

## Canonical source-account validation

Current release source manifests were emitted directly by `write_linkage` in `release/qualify.py`, `release/isolate_experiments.py` and `release/orchestrate.py` into `/private/tmp/azimuth-current-release-source`. No package qualification, publication or hosted execution was invoked. The separate `private-deployment-artifact-declarations.json` preserves the three existing Artifact declarations from historical linkage after checking their declared current source files exist. It contains no private-deployment realization or successful qualification, lifecycle or image-build observation.

```sh
/tmp/azimuth-packages-build/debug/azimuth validate \
  --model azimuth/model --standards azimuth/standards/verification.md \
  --manifest /private/tmp/azimuth-current-release-source/linkage.json \
  --manifest /private/tmp/azimuth-current-release-source/experimental-isolation-linkage.json \
  --manifest /private/tmp/azimuth-current-release-source/orchestration-linkage.json \
  --manifest /private/tmp/azimuth-current-release-source/private-deployment-artifact-declarations.json
```

Result: 46 Claims in 11 specs, no findings, zero errors and zero warnings. Report: `/private/tmp/azimuth-current-release-validation.log`. `azimuth change check compact-verification-packages` with the same four supplied manifests also reports zero incomplete plan items and an accepted-state model with zero errors and warnings; without those inputs its nine unresolved bindings remain visible. All three work packages are marked complete. The nine earlier unresolved Artifact bindings therefore resolve with their supplied declarations. This is structural validation, not qualification execution.

## Residuals and acceptance boundary

- Decision Policies and challenge scheduling for the compact review lane remain deferred. No implicit deployment, review approval, automatic expiry or cross-Subject applicability was added.
- Consumer API execution, native import and Claim decisions remain the focused consumer change's responsibility. Actual hosted network-isolation observations remain unavailable until the admitted deployment and origins are exercised.
- Historical retained release linkage still contains the retired `realizes.spec` field. Its immutable bytes were preserved; current source linkage is generated separately. A successful current structural account does not rewrite historical qualification facts.
- Qualified local source manifests do not create versioned Git observations. Uncommitted inputs cannot discharge full federation acceptance.
- Temporary reports identify this implementation session's observed commands. They are not published release artifacts or permanent assurance records.
- Accepted `azimuth/model` was not changed. No intent acceptance, archive, publication or commit was performed.
