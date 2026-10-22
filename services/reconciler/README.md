# Deployment reconciler

Standalone standard-library Python process, separate from Check collection. See `contracts/deployment-observation.md` for strict protocol, ordering and authority semantics. Image workflow `publish-reconciler.yml` publishes multi-architecture `ghcr.io/azimuth-sh/azimuth-reconciler` images using GitHub package credentials; it does not deploy.

Mount `/config/reconciler.json`, publisher files under `/credentials`, and a persistent writable `/state`. Kubernetes service account token/CA are used only against `https://kubernetes.default.svc`; AAS publisher credentials are never forwarded to Kubernetes. Redirects/proxies are disabled and responses are bounded to 4 MiB. Configure `assurance_url`, integer `interval_seconds` (5..60) and explicit `bindings`. Each binding contains project, environment, namespace, argocd_namespace, application, release_config_map, workloads (component→{deployment,container}), token_file and freshness_seconds (at least two intervals, at most 300). Example workload IDs for drim.dev are web-api and web-app; add cli-gateway only while its Deployment is enabled. The component set must match the generated release ConfigMap exactly.

No Requirement Status, release acceptance or workload mutation is produced. Unknown/partial/drifted observations remain adverse visibility facts. Deployment observations do not replace Check Runs or Requirement Status. Runtime rollout, admission defaults, digest formats, network policy and authenticated API behavior require verification before activation; no tests or production deployment were performed by this change.


## Requirement vocabulary

`azimuth-deployment-observation` version 1 is a runtime fact and keeps its name, version and shape under the Requirement vocabulary; the reconciler sends and reads no Verification Target fields. Its single pending request per environment in `/state/reconciler.sqlite` is cleared together with the AAS reset: its `expected_revision` belongs to the previous database. A request AAS rejects with 400/409/422 is discarded with a warning and the environment is observed again, so the reconciler cannot block on a leftover request.

## Process lifetime observations

Component records now include `instances`, the sorted exact containerIDs of ready replicas.
A complete observation requires a unique supported runtime containerID for every approved
container, unambiguous Pod -> ReplicaSet -> Deployment controller ownership, matching image
identities and the full desired ready replica count. Final Pod UID/resourceVersion inventory
must equal the initial inventory; a container restart/readiness/ownership change during discovery
rejects that observation. Missing IDs never become an inferred process lifetime.

Add namespace-scoped ReplicaSet `get/list` RBAC alongside existing Deployment/Pod permissions.
Deploy the AAS protocol/schema expansion before this reconciler image. Context collection remains
separate in the collector; the reconciler does not scrape settings or create Check evidence.

## Applied revision

The applied revision is the exact 40-character Git commit recorded by a successful Argo operation in `status.operationState.syncResult.revision`. The comparison head in `status.sync.revision` may advance after an observer-only Git commit without a sync; it does not create a different applied deployment. A dry-run operation, missing successful-operation revision, or missing/invalid completion timestamp keeps the observation incomplete.
