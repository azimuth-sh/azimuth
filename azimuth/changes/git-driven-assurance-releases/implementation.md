# Implementation handoff

## Implemented paths

- Canonical: contracts/project-catalog.md, contracts/release-publication.md, contracts/deployment-observation.md, contracts/run-ledger.md; tools/azimuth/src/{federation,release,main,workflow_state}.rs; standard workflow/reference and shipped propose/apply skills.
- Canonical ASS: additive migration 0008; run_ledger/deployment_observation.rs and release acceptance preconditions; owner credential configuration, release acceptance Server Action/component and exact observed-deployment freshness presentation. Pre-existing migration 0007 and UI/domain changes were preserved.
- Canonical runtime: services/reconciler/{reconciler.py,Dockerfile,README.md}; .github/workflows/publish-reconciler.yml.
- Consumer: explicit project release owner, azimuth/releases/{policy.json,README.md}, scripts/git_release.py, generalized source/component preparation in release_pipeline.py and .github/workflows/git-release.yaml.
- GitOps: request validation/trusted coordination workflows; release_request.py, select_release.py and extended check_release.py; auxiliary release ConfigMap template; generated environment values locations; release history/activation documentation.
- Infrastructure: deployment-reconciler module, application publisher Secret/module, app-production variable wiring/default-null activation, Argo generated values inputs, explicit optional per-user ASS owner mapping.

## Permitted checks

Passed native CLI cargo build and server cargo check, Web typecheck, Python compilation, workflow YAML parsing, Helm lint, existing rendered production release gate, Terraform fmt and app-production validate after local terraform get. Native current change validation reports 46 current/prospective Claims, no Findings and structural validity valid. Package declarations parse; standard workflow definition exports with the offline final action. git diff --check passed during integration. No tests or test-suite inspection, Terraform plan/apply, Kubernetes writes, image build/publication, ASS submission, acceptance, commit or push.

## Activation prerequisites

Commit/review the code first. Pin policy.json to the actual new committed canonical tooling revision; its existing HEAD cannot identify this uncommitted code. Publish compatible ASS API/Web and actual reconciler image via existing/new image workflows. Deploy additive SQL migrations without inventing baseline/selection data. Install trusted GitOps coordinator scripts/workflows before authoring a request. Configure protected release environments and least-privilege cross-repository coordinator/source-read/PR-update credentials described in GitOps RELEASES.md; retain existing ASS/Tailscale publisher credentials only in protected publication jobs. GitOps generated values files must be merged before applying the Argo Application valueFiles change.

Configure per-user explicit owner identities and a distinct reconciler publisher, real immutable image digest, exact enabled runtime workload/container names, Argo Application/ConfigMap location, and API-server/DNS/internal-ASS egress. assurance_reconciler remains null. The existing production chart disables gateway; include only rendered enabled components in reconciler bindings. Do not use a fabricated digest or enable infrastructure until prerequisites exist.

## Runtime limitations and remaining evidence

No authenticated API/database migration, CI dispatch/token permission, actual image build, live Argo rollout, CNI policy, CRI imageID format or injected template/default behavior was exercised. Unsupported image identity/admission forms fail closed as partial/drifted. Polling is not complete event capture. The reconciler records an exact new deployment Subject; production Check launches/collector bindings and independently authored Health must target that Subject before acceptance can succeed. No Health, acceptance or archive completion is inferred.

The generic core supports explicit source/build lists and release owner location. The consumer is configured only for its actual application and GitOps repositories and production deployment. Adding a real repository requires topology, native extraction and Check coverage, allowlist/build bindings and credentials. Additional environment files can be prepared from retained releases without rebuilding, but deployment adapters/Argo applications and gates must be configured per actual environment. Receipt artifact retention and application/database compatibility can block rollback; no data recovery is automated. Existing pinned workflow instances are not silently migrated; start a fresh instance to use the revised standard handoff.

## Coordinator readiness review

Production frontend build and server compilation passed. Production Terraform
configuration validation passed; a no-refresh plan contains only an Argo
Application update and Web-auth Secret update, with no resource deletion. No plan
was applied. The Argo values-file update must follow GitOps file installation.

Corrections restore the private-API boundary gate against generated values using
trusted base code, strengthen reconciler completion to require updated/ready/
available replicas and stable observed Deployment versions, include environment
in deployment identity, and align the minimum observation freshness with the API.
The workspace refreshes visible observations every 30 seconds. No tests were run.

Production has API, Web and collector; no reconciler. Existing runtime owner is
mitro, but Web has no owner mapping. GitOps release-preparation was created with
a main-only deployment branch policy. Coordinator and source-read PATs remain
owner-supplied activation prerequisites. Existing publisher/Tailscale secrets
are present; OIDC workflow restriction still requires validation in publication.
