# Assurance collector

Standalone standard-library Python service with two in-process adapters: `probe-results` and `prometheus-rules`. It discovers labeled Services only in explicitly allowed namespaces and joins them to operator-approved `(namespace, service)` bindings. Unknown labeled Services are ignored; annotations cannot supply credentials, ASS projects, model identities or arbitrary scrape URLs. Redirects are rejected. Only the separate ASS publisher reads producer tokens, never the scrape adapters.

The collector writes normalized native Runs and source checkpoints in one SQLite transaction. Publication retries replay the exact retained bytes; ACK fingerprint must match before deletion. Discovery failure does not stop delivery. Full outbox pauses collection, which can eventually cause explicit source history gaps. Collection gaps are logged and retained in SQLite `collection_gaps`; they are not manufactured as successful observations. Preserve and back up the state PVC.

## Configuration

Mount `/config/collector.json` with:

```json
{
  "assurance_url": "http://assurance-api.assurance.svc:8081",
  "namespaces": ["example-application"],
  "interval_seconds": 30,
  "max_pending": 10000,
  "projects": {"example": {"token_file": "/credentials/example"}},
  "sources": []
}
```

Each source requires `id`, `adapter`, `project`, `source_system`, `valid_from_ms`, `valid_until_ms`, `allowed_outcomes`, and `launch`. `launch` is the complete frozen native Check-only import launch for the exact deployed configuration and model revision, with exactly one Check and one unit. It is not inferred from labels, filenames or a mutable latest deployment. Use a distinct binding ID for every relevant deployment or definition change; retain a binding long enough to drain its backlog. The source report must carry that binding ID, exact Subject fingerprint and execution interval within its validity interval. Native core Run verification gates staging. Historical records are never rebound to the current deployment.

The launch adapter ID is `probe-results` or `prometheus-rules`. Its adapter fingerprint must be SHA-256 of the exact deployed `collector.py` bytes. The launch must retain the corresponding descriptor, configuration, capability routes and model/Check fingerprints. Obtain these through the project-owned explicit adapter/launch integration; there is no fake default identity or automatic launch generation in this service. Version and normalizer identity come from that frozen adapter description. Configuration of an approved launch is required before initial publication.

A `probe-results` source additionally requires `namespace`, `service`, `port`, `path` (normally `/results`), and `allowed_outcomes: ["violated", "inconclusive"]`. Its Service must be internal ClusterIP, labeled `azimuth.sh/collect=true`, annotated with `azimuth.sh/results-port` and `azimuth.sh/results-path` exactly matching the approved binding. Each page is limited to 100 results and 4 MiB. One page per source is fetched each collector cycle; choose retention for the collection/delivery outage budget. No source authentication token is forwarded.

A `prometheus-rules` source additionally requires an operator-approved `url`, exact `rule`, nonempty `rule_labels`, nonempty `target_labels`, `max_age_seconds`, and optional `allow_healthy_clear` (default false). `rule_labels.azimuth_deployment_fingerprint` must equal the exact deployment fingerprint in its launch. The configured rule must be unique; its evaluation must be healthy and fresh, and matching scrape targets must exist, be healthy and fresh. Only then does firing produce violated; inactive may produce satisfied only with explicit healthy-clear policy. Pending, missing, stale and unhealthy states remain inconclusive. The author must establish that this rule and scrape-target scope cover the selected Cases. This adapter does not infer coverage or create Claim Health. Monitoring windows crossing a deployment must be split by operator-owned immutable bindings. Static authenticated Prometheus endpoints require a future explicitly scoped authentication extension; no ASS token is reused.

## Protocol and operations

Probe pages use `azimuth-check-results` version 1 with `history_gap`, `items`, `next_cursor`, `has_more`. Every item contains stable `execution_id`, `check`, `binding`, `subject_fingerprint`, integer execution timestamps, `outcome` and native measurements. Native report data is retained as the activity's string attribute and hashed import input; test logs are not copied. Translation creates Check observations only; no reviews, authority acceptance or Health is fabricated.

The collector posts protocol-verified bundles to existing `POST /v1/projects/{project}/runs`. Give its existing Secret a project-scoped **producer** token, not an owner/reviewer token. ASS Run ingestion requires producer authorization; release publisher authorization alone is insufficient. Tokens never appear in source config or Git. Success and exact replay receipts are both ACKs. Invalid bundles and rejected authentication remain pending and visible in logs.

The Docker image includes the native `azimuth` verifier and Python Run builder from the same source revision. `publish-collector.yml` is manual on main and uses GitHub's package credential. It does not deploy. Kubernetes resources are consumer infrastructure-owned, separate from this generic source tree. A single replica and Recreate strategy prevent simultaneous SQLite writers on independent Pod mounts. State size is bounded by pending count and payload cap, with compact collection-gap records; operators must monitor capacity and retained gap growth. There is no bulk log storage or automatic review engine.
