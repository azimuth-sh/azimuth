# Run bundle conformance

This synthetic experiment exercises the provider-neutral Run bundle without an adapter, provider
account or Assurance Service. `generate.py` derives every canonical fingerprint from strict local
data and writes fixtures into a temporary directory.

The gate covers all six Subject variants, historical import provenance, multi-bundle aggregation,
retry and shard reduction, a physical activity shared by independent Check and Challenger
executions, partial-to-complete correction, exact replay, semantic mismatch and schema failure. It
asserts the public `azimuth run verify` and `azimuth run inspect` exit classes and the inspection
account's explicitly unresolved repository and Assurance State authority.

Run it from any directory:

```sh
experiments/run-bundles/check.sh
```
