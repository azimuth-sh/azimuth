# Deployment reconciler

Standalone standard-library Python process, separate from Check collection. See `contracts/deployment-observation.md` for strict protocol, ordering and authority semantics. Image workflow `publish-reconciler.yml` publishes multi-architecture `ghcr.io/azimuth-sh/azimuth-reconciler` images using GitHub package credentials; it does not deploy.

Mount `/config/reconciler.json`, publisher files under `/credentials`, and a persistent writable `/state`. Kubernetes service account token/CA are used only against `https://kubernetes.default.svc`; ASS publisher credentials are never forwarded to Kubernetes. Redirects/proxies are disabled and responses are bounded to 4 MiB. Configure `assurance_url`, integer `interval_seconds` (5..60) and explicit `bindings`. Each binding contains project, environment, namespace, argocd_namespace, application, release_config_map, workloads (component→{deployment,container}), token_file and freshness_seconds (at least two intervals, at most 300). Example workload IDs for drim.dev are web-api and web-app; add cli-gateway only while its Deployment is enabled. The component set must match the generated release ConfigMap exactly.

No health success, release acceptance or workload mutation is performed. Unknown/partial/drifted observations remain adverse visibility facts. Deployment observations do not replace Check Runs or Claim Health. Runtime rollout, admission defaults, digest formats, network policy and authenticated API behavior require verification before activation; no tests or production deployment were performed by this change.


## Process lifetime observations

Component records now include `instances`, the sorted exact containerIDs of ready replicas.
A complete observation requires a unique supported runtime containerID for every approved
container, unambiguous Pod -> ReplicaSet -> Deployment controller ownership, matching image
identities and the full desired ready replica count. Final Pod UID/resourceVersion inventory
must equal the initial inventory; a container restart/readiness/ownership change during discovery
rejects that observation. Missing IDs never become an inferred process lifetime.

Add namespace-scoped ReplicaSet `get/list` RBAC alongside existing Deployment/Pod permissions.
Deploy the ASS protocol/schema expansion before this reconciler image. Context collection remains
separate in the collector; the reconciler does not scrape settings or create Check evidence.
