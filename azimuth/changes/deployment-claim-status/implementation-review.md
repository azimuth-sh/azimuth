# Deployment Claim Status implementation review

Reviewer: independent implementation worker, 2026-10-20.
Scope: native deployment assessor, applicability/policy projection and parser, root ASS routing/attribution/acceptance wiring and additive relational initialization. The reviewer authored the separate status-service and UI packages; observations about their SQL integration are explicitly integration self-inspection, not independent assessment of those internals. This document is engineering review evidence, not a Claim Review, release acceptance or archive decision.

## Findings and disposition

### 1. Definitive violation bypassed current review of the counterexample mapping — resolved

The assessor set a definitive violation from `contribution: counterexample|both` and selected `violated` before checking `review_ok`. Thus a critical Claim with missing, rejected or stale Claim Review could obtain a definitive violated Status from a mapping that had not been independently accepted. The agreed contract distinguishes visible adverse observations from a reviewed definitive counterexample.

The root corrected final Status selection to require `definitive && review_ok`; routine Claims retain the explicit review exemption. Adverse observations remain visible in the explanation. The focused regression `missing_review_blocks_support_without_hiding_declared_counterexample` now expects unresolved while retaining the applicable adverse fact. All native assessor tests pass.

### 2. Explicit release acceptance still used the authored exact-Subject evaluator — resolved by inspection

`release::accept` originally invoked `assurance_review::subject_state`, requiring an authored Claim Health record despite the new computed deployment Status endpoint. A deployment could be Confirmed in the dashboard while its explicit owner acceptance remained blocked by the previous assessment requirement.

The root replaced that evaluation with `deployment_status::assess_deployment` while retaining authority, deployment-selection and observation locks, baseline compare-and-swap and owner authorization. No acceptance is automatically performed. A dedicated non-routine owner-acceptance lifecycle is not included in this review's focused execution evidence.

### 3. Service Run selection discarded applicable evidence from another whole-model revision — resolved; integration self-inspection

The SQL candidate set originally required equal model fingerprint, an explicit model association or publication membership. Native applicability deliberately permits an unchanged exact Check definition from another model revision. A fresh directly ingested Run could therefore be omitted before the assessor saw it.

The root changed candidate selection to pair current Check IDs with exact fingerprints in typed Check-selection rows. Full histories of selected Run IDs are retained. The real HTTP/PostgreSQL regression now uses an unrelated `plan.model_fingerprint` with exact current Check/source identities and obtains supported deployment Status through release lineage.

### 4. Periodic Monitor history eventually exceeded the complete-snapshot limit — resolved for current operational scope; integration self-inspection

Loading every historical poll for unchanged Monitor definitions eventually exceeded the 5000-record cap, making Status unavailable although only recent polls can establish current monitoring evidence.

The root now limits Monitor-only candidate Run IDs to the exact deployment Subject and a conservative maximum freshness window, retaining complete correction histories and equal-time candidates. Persistent non-Monitor evidence is not expired by age. A fresh observation must have a Run finish at or after this cutoff, so the filter does not discard usable fresh evidence. Newer incomplete executions still reach the assessor and invalidate fallback to an older green poll.

The PostgreSQL integration includes 5001 expired metadata entries plus one genuine current Monitor Run admitted through HTTP. Expired metadata is deliberately not given a complete hydrated document subtree, proving that filtering occurs before evidence hydration; only the valid fresh Run establishes support. Status remains available and supported. Expired records remain in inspection storage. When all polls expire, the assessor explains missing fresh monitoring evidence rather than hydrating an old poll for a detailed overdue timeline.

## Additional operational boundaries

- Typed v3 schema changes are additive: new roots and child tables, retained `_present` semantics and an idempotent title-column addition. No existing record/document rewrite was found.
- The catalog unit regression alone only relabels current schema metadata. This verification gap was closed by an actual previous-version rehearsal: exact v2 source commit `a4e6d167ed07d39d75c7febfca13c11126d5e110` initialized a disposable PostgreSQL 17 database and its real HTTP API seeded 11 native records. V3 context tables were absent. A hash-verified snapshot of current implementation then upgraded that same database to v3 twice; all 11 authority/catalog/change/deployment/release/Review/Run/Subject records remained equal and passed applicable native validators. New tables were present afterward and no JSON columns appeared. Preserve the production backup before rollout.
- A previous v2 binary rejects v3 storage identity. Document forward repair or an explicit retained-database/backup rollback procedure; additive storage does not imply automatic image rollback compatibility.
- The Kubernetes adapter's instance identifier is a container runtime ID. Its supported application process must track that container lifetime; in-container supervisors restarting the app without a container restart are unsupported. The collector checks boot identity within capture, not a universal independent process observer. The public contract now states this boundary.
- Nonsecret field contracts are owner-reviewed allowlists. Field-name validation cannot detect arbitrary credentials or credential hashes disguised as benign values; documentation must not promise a universal secret-value classifier.
- Source/artifact evidence remains subject to a bounded complete snapshot. This change fixes continuous Monitor growth; very large persistent evidence histories may still require narrower indexed applicability selection or explicit retention design later. Bounds fail closed rather than silently truncating evidence.

## Verification executed

- `cargo test --manifest-path tools/azimuth/Cargo.toml --test deployment_status`: 17 passed. Includes exact source/build inputs, conflicting provenance, artifact identity, contexts and every observed process, changed/restarted/untrusted captures, Monitor expiry/inconclusive/conflicting/incomplete results, corrections and review freshness.
- `cargo test --manifest-path services/assurance/Cargo.toml -p azimuth-assurance-server --test deployment_status`: passed on PostgreSQL 17. Includes authenticated context ingestion, immutable replay, cross-principal rejection, source-to-deployment support across model revisions and the 5001-expired-Monitor regression.
- `cargo test --manifest-path services/assurance/Cargo.toml -p azimuth-assurance-server --lib catalog_title_upgrade_preserves_retained_records`: passed. Limited upgrade coverage as noted above.
- Actual v2 → v3 rehearsal: exact old initializer and HTTP seeding, then current initializer against the same disposable PostgreSQL database; 11 retained native records unchanged across eight families, explicit absent/new table checks, repeated initialization and native post-upgrade validation passed. Synthetic proof and source manifest retained under `/private/tmp/assurance-storage-upgrade-proof.json` and `/private/tmp/assurance-v3-upgrade-source-manifest.json`.
- Earlier UI package verification: TypeScript check, production build and actual React rendering passed; complete synthetic Next.js dashboard and Claim pages returned HTTP 200 for monitoring, context mismatch, lineage and historical-record scenarios.

## Conclusion

The four concrete implementation/integration blockers found here have been corrected. No additional false-confirmation path was identified in the inspected native applicability rules. Production rollout still requires the root's complete engineering gates, a retained production backup and reviewed operational configuration. The actual previous-version data-preservation rehearsal is now complete. Release acceptance and archival remain explicit owner boundaries.


## Consumer CI applicability review

This is a read-only review of the consumer integration; no consumer test restore or execution was performed while its instrumentation worker was active. This is not an accepted Claim Review.

### Source scanner execution Subject omits an actual repository input — open blocker

`scripts/ingress_ci_runs.py::prepare` selects only `drim-dev` into `subject.json`. `source_reports` then passes that same Subject to both the application and `drim-dev-argocd` source and credential-delivery scanners. Their actual input inventories include both repositories. Thus the native scanner account can attribute a GitOps finding or successful scan to an app-only Subject. The consumer's component build provenance separately lists only the component's application repository, so adding GitOps to every behavioral Run would also incorrectly describe the build inputs. Split the scanner evidence into its own exact input Subject/plan and explicitly model its deployment applicability; retain the app-only runtime test Subject. Reported to the root before rollout.

### Context scope and retained limitations

The verification account now explicitly explains empty runtime context dependencies: source-byte scans, credential-reference delivery and parameterized client transformations do not establish current API schemes, credentials or networking. Registration and behavioral API Checks require both authentication and feature contexts. The same document says whole-suite semantic accounts currently aggregate multiple fixture lifetimes without bracketed per-Run captures; these Runs must remain inapplicable to deployment. Their green engineering execution must not be presented as sufficient Claim support.

The public xUnit mapping currently covers one exhaustive registration test, with independently fixed service/caller expectation inventories and exact native identity. Its declared `failed_outcome: inconclusive` is conservative because arbitrary assertion/setup failures cannot establish the specific semantic counterexample. Other native semantic providers retain inventories, controls and gaps rather than synthesizing pass from suite exit. Mapping only this test is a bounded integration, not full behavioral context instrumentation. The absence of generic mappings for other tests is therefore an evidence gap, not permission to infer them.

### Runtime context extraction and publication

`VerificationContext` reads the application's resolved cached startup `IOptions` for authentication defaults/scheme inventory and feature gates. The fixture captures this same instance before and after its mapped test. The production listener uses its own telemetry-only routing table, supplies a fresh application boot identity and no-store response, and does not expose product API endpoints. This supports the stated startup-fixed configuration boundary; it does not support hot-reloaded options.

The release pipeline packs the exact checked-out public xUnit harness version, restores that version explicitly, imports actual native reports and retains failed execution artifacts. Release publication precedes execution-context upload, permitting exact stored Run ownership checks; immutable retry semantics are required if upload is interrupted. Runtime endpoint, mapping and CI code were inspected, but this review does not replace the root's actual consumer execution and fresh evidence validation.


## Independent activity-scoped applicability refinement review

Inspected current `tools/azimuth/src/deployment_status.rs`, its native Run validation relationship and focused regression cases after fixture partitioning requirements were added. No additional false-support path was identified in these inspected rules.

- Authored required unit IDs must equal both planned and executed unit inventories. The underlying Run validator independently requires each attempt's outcome keys to equal the selected Case set, rejects missing activity references and reduces incomplete units to inconclusive; unit identity alone is not evidence.
- Execution context selection is restricted to the exact Run/bundle/Subject/component and either the contributing activity or an explicitly whole-Run capture. Activity-specific captures must enclose that activity, while whole-Run captures enclose the Run. Both forms require matching exact fingerprint pins in actual selection. Unrelated baseline activities do not satisfy a target's capture requirement.
- Every contributing attempt requires capture, including controls. Explicit control exemptions are limited to authored units and named required contracts; they exempt production-context equality, not capture authenticity, stability or identity. At least one noncontrol attempt must support the Case when the aggregate asserts positive support. Controls alone cannot create positive Case evidence. Ordinary attempts must all match every independently observed live production instance.
- `configuration_sources` is restricted to source-scoped policies and adds exact released repository commit/content identities to the component build input set. It does not rewrite artifact build provenance or tolerate arbitrary extra repositories. This provides the necessary separate scope for actual app+GitOps scanner Runs.

Inspected regressions cover distinct fixture lifetimes, irrelevant baseline context, incomplete required units, differing control contexts, control-only support and wrong configuration-source versions. This inspection does not certify the consumer's not-yet-final fixture partition emitter or author accepted Claim Reviews; those require exact complete review packets.


### Consumer feature-context relevance decision

Independent source inspection supports authentication-only context dependencies for structural registration equality and shared service/caller authorization invariants: Program maps endpoint types and the Lab hub regardless of toggle values, validates completed registrations, and executes ServiceAdmission → Authentication → Authorization before endpoint filters. `RequireFeature` only returns HTTP404 or invokes the same endpoint continuation; it does not mutate admission metadata, identities or policies. The authentication/admission implementations do not read `FeatureFlagsOptions`. Feature captures should remain retained as actual execution facts, but unrelated flag differences are not control exemptions and need not invalidate those bounded auth propositions.

This is not blanket permission to remove feature dependencies. A Check asserting a feature-gated successful protocol outcome or protected delivery may have a changed oracle when production disables that feature. Such a Check must retain the relevant feature dependency or demonstrate that its asserted boundary is independent of the downstream filter. Explicitly attributed service/caller admission followed by a downstream feature404 is different from claiming the protocol operation succeeded. This decision was sent to the coordinator and consumer author before final review packets.
