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

A stable Check source has `id`, `project`, `environment`, `adapter`, `allowed_outcomes: ["violated", "inconclusive"]`, and `definition: {plan, adapters, routes}`. The definition is the reviewed one-Check/one-unit native import launch with `plan.fingerprint` removed, so its plan has exactly `model_fingerprint`, `required_execution_context`, `checks` and `challenges`, and its Check selects `scenarios`. It has no Verification Target or validity interval. A definition generated before the Requirement vocabulary (`required_context`, `cases`) is rejected at startup rather than queued for a permanent ASS rejection. Definitions change only when the Check changes; ordinary releases do not require collector configuration changes. Legacy launch/binding configurations are rejected explicitly.

A `probe-results` source also has `namespace`, `service`, `port`, `path`. Discovery requires an internal ClusterIP Service labeled `azimuth.sh/collect=true`; scrape annotations must match that exact approved route. Unknown Services are ignored. Results use `azimuth-check-results` version **2**, reporting actual `{component,image,instance}` targets and measurement intervals without a Verification Target. A result carrying a Verification Target, release, deployment or binding key (including the earlier `subject`/`subject_fingerprint`) is rejected. Empty targets preserve inconclusive collection facts. Source history gaps are logged and stored, never converted into satisfaction.

A `prometheus-rules` source also has `url`, `rule`, `rule_labels`, `target_labels`, `identity_labels: {component, image, instance}`, and optional bounded interval/freshness settings. Identity label values name the target labels carrying the actual component, immutable image and runtime process identity. Missing/malformed identity fails closed. A healthy fresh firing rule produces violated only when the firing alert carries actual identity labels matching active targets scraped no later than that evaluation. Its measurement timestamp is the rule evaluation time, not collection time. An old alert cannot acquire a replacement process identity from a later targets scrape. Missing identities, stale or pending alerts and inactive rules remain inconclusive without an attributed target. The adapter cannot establish positive support or infer deployment identity from configured labels. Operators must review rule-to-Scenario semantics and measurement scope.

Each context source serves one component and has `id`, `project`, `environment`, `namespace`, `component`, Kubernetes `deployment` and `container`, `port:9001`, `path:"/internal/execution-context"` (the Context endpoint), `contracts` and a separately mounted `token_file`. `contracts` holds exactly one full reviewed context contract `{name,fields,fingerprint}`: the contract core derives from the component's declaration (`azimuth/execution-contexts/<component>.json`), so `name` is the component id and `fields` maps each dotted leaf (`features.labs_enabled`) to `string | boolean | integer`. Leaf names must be dot-separated ASCII letters, digits and `_` and may not contain `password`, `secret`, `credential`, `token`, `privatekey` or `private_key`; a contract with several entries, another name or such a leaf aborts startup. It is a local allowlist, not an ASS GET reference.

The endpoint must return an `azimuth-execution-context` version 2 report holding exactly one context, named by the component id, whose `contract_revision` equals the configured fingerprint and whose `values` keys equal the contract's leaves with the declared types; other formats, versions, extra or differently named contexts abort capture. The retained snapshot keeps that single context unchanged, and ASS pins it per leaf as `context/<component>/<leaf>`. The collector inventories actual ready owned Pods and immutable images, reads each process twice, and independently repeats the inventory. Ownership/readiness/resource-version changes, process restart, boot-header change (`X-Azimuth-Instance` must be identical across the before and after snapshots of each instance) or context changes abort capture. No publisher or Kubernetes credential reaches the application telemetry endpoint.

```json
{
  "id": "web-api-context", "project": "example", "environment": "production",
  "namespace": "application", "component": "web-api", "deployment": "web-api", "container": "web-api",
  "port": 9001, "path": "/internal/execution-context",
  "contracts": [{"name": "web-api", "fields": {"features.labs_enabled": "boolean", "auth.authenticate_scheme": "string"}, "fingerprint": "sha256:<canonical {name,fields}>"}],
  "token_file": "/credentials/context-publisher"
}
```

All runtime observations use `azimuth-target-observation` version 2 and POST exactly `{observation}` to `/v1/projects/{project}/target-observations`. ACK must equal the canonical submitted observation fingerprint before deletion. Checkpoint and outbox staging share one transaction; failure/full capacity preserves the cursor. Delivery retries preserve exact bytes even across release changes. Redirects, duplicate JSON fields, non-finite values and oversized responses are rejected; ambient HTTP proxies are disabled. A rejected delivery is never deleted: it is retried with its exact bytes and logged as `target-publication-failed` with the HTTP `status` (422 for a semantic rejection). Preserve the state PVC and use a single replica/Recreate strategy.

The `outbox` (native Runs) table is drained through its original POST endpoint with unchanged bytes and receipt checks; deployment captures travel as target observations through `target_outbox`. No new runtime observation is assigned a configured Verification Target. The separate pinned native adapter CLI remains available through `register.py` and `adapter.py` for explicit native import workflows.

## Vocabulary upgrade: clear collector state

The ASS accepts only `azimuth-target-observation` version 2 and rejects version 1. Rows staged by a collector from the earlier vocabulary (version 1 target observations, native Runs or deployment captures in the earlier wire shape) are therefore rejected forever: they are retried every cycle, count toward `max_pending`, and once the outbox is full they block all new capture while checkpoints stay put. There is no state migration. When upgrading, stop the collector, empty the contents of the state volume (`/state/collector.db` with its outboxes, checkpoints and gaps), regenerate source definitions from the renamed tooling, point context sources at `/internal/execution-context`, and start the new image together with the renamed ASS. The SQLite table and column names themselves are unchanged.

## Offline configuration preparation

From a checkout with the Python Run package available:

```sh
PYTHONPATH=packages/python python3 services/collector/configure.py \
  --current /private/current-collector.json \
  --reviewed /private/reviewed-collector-definitions.json \
  --out /private/collector-v2.json
```

The reviewed file has exactly `sources` and `contracts`. `sources` maps every current source ID to `{id, environment, definition}`; choose a stable descriptive `id` explicitly. Prometheus entries additionally require `identity_labels`. `definition` is the current exact `{plan,adapters,routes}` with only `plan.fingerprint` removed. `contracts` contains full reviewed `{name,fields,fingerprint}` objects covering every referenced context contract; each context source references exactly one, named by its component id. The tool refuses definition drift, missing contracts and incomplete/duplicate stable mappings. It strips release-specific launch/Verification Target/times and runtime code bindings; it neither contacts ASS nor modifies a live configuration. Output must be a fresh path. Review it before deployment.
