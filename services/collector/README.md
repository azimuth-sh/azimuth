# Assurance collector

Push-only runtime collection with in-process probe and Prometheus adapters. The collector discovers only approved internal Services in approved namespaces, validates observations, atomically checkpoints their durable SQLite outbox and POSTs retained bytes to ASS. It never reads ASS. ASS resolves target identity against independently reconciled deployment intervals; unmatched observations remain retained there.

## Configuration version 2

```json
{
  "format": "azimuth-collector",
  "version": 2,
  "assurance_url": "http://assurance-api.assurance.svc:8081",
  "namespaces": ["application"],
  "interval_seconds": 30,
  "max_pending": 10000,
  "projects": {"example": {"token_file": "/credentials/example"}},
  "sources": [],
  "context_sources": []
}
```

A stable Check source has `id`, `project`, `environment`, `adapter`, `allowed_outcomes: ["violated", "inconclusive"]`, and `definition: {plan, adapters, routes}`. The definition is the reviewed one-Check/one-unit native import launch with `plan.fingerprint` removed. It has no Subject or validity interval. Definitions change only when the Check changes; ordinary releases do not require collector configuration changes. Legacy launch/binding configurations are rejected explicitly.

A `probe-results` source also has `namespace`, `service`, `port`, `path`. Discovery requires an internal ClusterIP Service labeled `azimuth.sh/collect=true`; scrape annotations must match that exact approved route. Unknown Services are ignored. Results use `azimuth-check-results` version **2**, reporting actual `{component,image,instance}` targets and measurement intervals without a Subject. Empty targets preserve inconclusive collection facts. Source history gaps are logged and stored, never converted into satisfaction.

A `prometheus-rules` source also has `url`, `rule`, `rule_labels`, `target_labels`, `identity_labels: {component, image, instance}`, and optional bounded interval/freshness settings. Identity label values name the target labels carrying the actual component, immutable image and runtime process identity. Missing/malformed identity fails closed. A healthy fresh firing rule produces violated only when the firing alert carries actual identity labels matching active targets scraped no later than that evaluation. Its measurement timestamp is the rule evaluation time, not collection time. An old alert cannot acquire a replacement process identity from a later targets scrape. Missing identities, stale or pending alerts and inactive rules remain inconclusive without an attributed target. The adapter cannot establish positive support or infer deployment identity from configured labels. Operators must review rule-to-Case semantics and measurement scope.

Each context source has `id`, `project`, `environment`, `namespace`, `component`, Kubernetes `deployment` and `container`, `port:9001`, `path:"/internal/verification-context"`, `contracts:[{name,fields,fingerprint}]`, and a separately mounted `token_file`. Full reviewed contracts are local allowlists, not ASS GET references. Only configured fields are retained. The collector inventories actual ready owned Pods and immutable images, reads each process twice, and independently repeats the inventory. Ownership/readiness/resource-version changes, process restart, boot-header change or selected context changes abort capture. No publisher or Kubernetes credential reaches the application telemetry endpoint.

All runtime observations use `azimuth-target-observation` version 1 and POST exactly `{observation}` to `/v1/projects/{project}/target-observations`. ACK must equal the canonical submitted observation fingerprint before deletion. Checkpoint and outbox staging share one transaction; failure/full capacity preserves the cursor. Delivery retries preserve exact bytes even across release changes. Redirects, duplicate JSON fields, non-finite values and oversized responses are rejected; ambient HTTP proxies are disabled. Preserve the state PVC and use a single replica/Recreate strategy.

Existing queued native Run and deployment capture rows still drain through their original POST endpoints with unchanged bytes and receipt checks. This preserves already collected historical evidence; no new runtime observation is assigned a configured Subject. The separate pinned native adapter CLI remains available through `register.py` and `adapter.py` for explicit native import workflows.

## Offline configuration preparation

From a checkout with the Python Run package available:

```sh
PYTHONPATH=packages/python python3 services/collector/configure.py \
  --current /private/current-collector.json \
  --reviewed /private/reviewed-collector-definitions.json \
  --out /private/collector-v2.json
```

The reviewed file has exactly `sources` and `contracts`. `sources` maps every current source ID to `{id, environment, definition}`; choose a stable descriptive `id` explicitly. Prometheus entries additionally require `identity_labels`. `definition` is the current exact `{plan,adapters,routes}` with only `plan.fingerprint` removed. `contracts` contains full reviewed `{name,fields,fingerprint}` objects covering every referenced context contract. The tool refuses definition drift, missing contracts and incomplete/duplicate stable mappings. It strips release-specific launch/Subject/times and runtime code bindings; it neither contacts ASS nor modifies a live configuration. Output must be a fresh path. Review it before deployment.
