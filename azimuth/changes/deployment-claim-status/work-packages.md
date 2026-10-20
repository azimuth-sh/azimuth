# Work packages

## Work package: design-review
Status: complete
Depends on: none
Owns: azimuth/changes/deployment-claim-status/design-review.md
Objective: Independently review proposal/design against agreed discussion and actual contracts, record blocking semantic ambiguities and bounded corrections without editing implementation.
Evidence: Read-only inspection of request, shared artifacts and native protocol/core.

## Work package: xunit-harness
Status: complete
Depends on: none
Owns: packages/dotnet/Azimuth.Testing.Xunit, packages/dotnet/Azimuth.Testing.Xunit.Tests
Objective: Implement reusable xUnit v2 context capture and typed test reporting against the shared design contract, with real synthetic verification and no ASS publisher embedded.
Evidence: Package build, synthetic integration tests including effective overrides, failures, different contexts, capture errors and lifecycle cleanup.

## Work package: playwright-harness
Status: complete
Depends on: none
Owns: packages/typescript-playwright
Objective: Implement reusable Playwright context fixture/reporter against shared design contract, including parallel/retry-safe typed reports, no ASS credentials and no arbitrary test-to-Claim inference.
Evidence: Typecheck/build and real synthetic Playwright/HTTP fixture verification including failures, overrides and capture errors.

## Work package: deployment-assessor
Status: complete
Depends on: design-review
Owns: tools/azimuth/src/deployment_status.rs, tools/azimuth/tests/deployment_status.rs
Objective: Implement strict context/deployment-policy validation and deterministic deployment Claim Status using native release, review, Run and correction contracts; provide explained source/artifact/deployment applicability, temporal Monitor selection and fail-closed context/producer handling.
Evidence: Synthetic native-core tests for lineage, context, corrections, reviews, monitors, conflicts and adverse outcomes.

## Work package: test-adapter
Status: complete
Depends on: xunit-harness, playwright-harness
Owns: packages/python/azimuth_testing, experiments/test-report-adapter
Objective: Implement strict public test-report/native-report reconciliation and explicit test-to-Check/Case mapping into validated native Run bundles using existing azimuth_run builder, plus separate retry-safe project-scoped publisher. No xUnit/Playwright fixtures or core semantics duplicated.
Evidence: Synthetic real report imports, failed/skipped/missing/duplicate unit handling, context partitions and native CLI Run verification; publisher tested against actual local HTTP receiver.

## Work package: status-service
Status: complete
Depends on: design-review
Owns: services/assurance/server/src/relational/status.rs, services/assurance/server/src/run_ledger/deployment_status.rs, services/assurance/server/tests/deployment_status.rs
Objective: Define explicit relational schemas and authenticated ingestion/read/assessment handlers for context contracts and trusted execution/deployment captures, retaining project/producer scope and immutable facts; assemble native deployment Status requests from stored exact release/Run/review/deployment data.
Evidence: Real PostgreSQL/HTTP lifecycle, strict trust injection, role/identity guards, native assessment and preserved historical records.

## Work package: deployment-context-collection
Status: complete
Depends on: design-review
Owns: services/collector, services/reconciler
Objective: Observe exact ready process-lifetime identities and collect explicitly approved effective verification contexts from every ready replica, reject restart/coverage races, and publish immutable authenticated context captures separately from existing Check collection.
Evidence: Synthetic real HTTP and Kubernetes response integration, all-replica coverage and restart rejection, preserved existing collector/reconciler checks, exact retry/outbox behavior and native validation.

## Work package: status-ui
Status: complete
Depends on: design-review
Owns: services/assurance/web
Objective: Present computed deployment Claim Status with Monitors first, explicit applicability/context/lineage/freshness explanations and accurately labelled historical authored assessments; update synthetic development scenarios.
Evidence: Typecheck, production build and focused rendering verification of monitor ordering, missing evidence and historical-record distinction.

## Work package: consumer-context-evidence
Status: pending
Depends on: deployment-assessor, test-adapter, xunit-harness
Owns: backend/src/DrimDev.Api.Tests/Features/Guides/GetGuideTests.cs, backend/src/DrimDev.Api.Tests/Ingress/behavior-selection.json, backend/src/DrimDev.Api.Tests/Harnesses/IngressContextObservation.cs, backend/src/DrimDev.Api.Tests/Harnesses/HttpClientHarness.cs, backend/src/DrimDev.Api.Tests/Harnesses/IngressFixtureIdentity.cs, backend/src/DrimDev.Api.Tests/Features/Ingress/IngressObservationCapture.cs, scripts/normalize_ingress_api.py, scripts/normalize_ingress_deployment.py, scripts/normalize_ingress_clients.py, scripts/release_linkage.py, scripts/normalize_ingress_reports.py, scripts/tests/test_ingress_context_units.py, scripts/tests/test_ingress_deployment.py, backend/src/DrimDev.Api.Tests/Features/Ingress/IngressAdmissionTests.cs, backend/src/DrimDev.Api.Tests/Features/CliAuth/CliAuthTestFixture.cs, backend/src/DrimDev.Api/Program.cs, backend/src/DrimDev.Api/Common/Assurance, backend/src/DrimDev.Api.Tests/Harnesses/VerificationContextCapture.cs, backend/src/DrimDev.Api.Tests/Features/Ingress/VerificationContextTests.cs, backend/src/DrimDev.Api.Tests/Fixtures/TestFixture.cs, backend/src/DrimDev.Api.Tests/DrimDev.Api.Tests.csproj, scripts/check_application_credential_delivery.py, scripts/check_ingress_source.py, drim-dev-argocd/scripts/scan_service_credentials.py, drim-dev-argocd/scripts/check-credential-delivery.rb, scripts/release_pipeline.py, scripts/ingress_ci_runs.py, azimuth/changes/private-api-service-ingress/deltas/api/ingress/verification.md, azimuth/verification-contexts
Objective: Owning repository drim-dev/drim-dev; owned paths are relative to that consumer repository. Adopt effective startup contexts through a telemetry-only API listener and actual application test harness capture; configure fresh native CI Run conversion and authored Check applicability without weakening private API admission.
Evidence: Real HTTP component tests, public harness reports and exact native import, strict context contract validation and CI pipeline failure retention.

## Work package: implementation-review
Status: complete
Depends on: design-review
Owns: azimuth/changes/deployment-claim-status/implementation-review.md
Objective: Independently review deployment applicability, deterministic assessor, root service wiring, schema upgrades and parser contracts; report concrete correctness/security gaps without editing implementation.
Evidence: Read-only contract/implementation inspection and focused synthetic verification where needed.

## Work package: harness-release-qualification
Status: complete
Depends on: xunit-harness, playwright-harness
Owns: release/acceptance.py, release/candidates.py, release/README.md, release/artifacts.json, scripts/check.sh
Objective: Align the public harness artifact cohort, pinned release contract and independent protocol versions, verify actual packed metadata/file sets and real local-feed/tarball consumer execution, and include context integration checks in canonical verification.
Evidence: Real NuGet/npm archive qualification and executable consumer smokes, pure release catalog validation, no new mocked verification or registry publication.

## Work package: deployment-status-workflow
Status: complete
Depends on: design-review
Owns: tools/azimuth/src/workflow_state.rs, tools/azimuth/resources/workflows/change-development.json, tools/azimuth/resources/references/workflow.md, tools/azimuth/resources/references/verification.md, tools/azimuth/resources/references/assurance-reviews.md, tools/azimuth/resources/skills/azimuth-apply/SKILL.md, contracts/workflow.md
Objective: Remove mandatory authored exact-Subject health from the standard workflow, derive local execution targets from the pinned template, defer deployment-only Checks to hosted monitoring, and retain historical task kinds and immutable pinned workflow semantics.
Evidence: Actual workflow expansion tests cover hosted-only publication, missing local execution Subjects and historical scope rules; all library tests pass. Review/approval gates and explicit publication dependencies remain; packaged guidance explains computed deployment Status and owner acceptance.


## Work package: hosted-client-destinations
Status: complete
Depends on: deployment-assessor
Owns: drim-dev-argocd/policies/private_exposure.rego, drim-dev-argocd/scripts/capture_private_api_resources.py, drim-dev-argocd/scripts/hosted_destination.py, drim-dev-argocd/scripts/tests/test_hosted_destination.py
Objective: Owning repository drim-dev/drim-dev-argocd; derive safe typed actual client destination bindings before redaction, verify selected Service and real EndpointSlice topology, and preserve a complete observed deployment Subject only after exact raw template and ready image verification.
Evidence: Pure resource JSON mutation tests and actual Conftest evaluations for wrong destination, duplicate/override inputs and endpoint mismatch; no live connectivity inferred.

## Work package: python-check-source-closure
Status: complete
Depends on: design-review
Owns: tools/extractors/python/azimuth_emit.py, tools/extractors/python/test_azimuth_emit.py, tools/extractors/README.md
Objective: Bind Python Check implementation support to complete module bytes so imports and module inputs participate in review freshness, while retaining logical site identities and other marker fingerprint boundaries.
Evidence: Python emitter suite 12/12 passed; import-only and constant-only module mutations change function/class Check support while site identities and Realizes/Mechanism fingerprints remain stable. Actual native authority compilation produced three distinct execution Check fingerprints for baseline/import-only/constant-only variants in temporary azimuth-python-check-closure-9_0qi5nr.
