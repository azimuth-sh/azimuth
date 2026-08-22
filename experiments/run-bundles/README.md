# Run bundle conformance

This synthetic experiment exercises the provider-neutral Run bundle without invoking an adapter, provider account or Assurance Service. `generate.py` derives every canonical fingerprint from strict local data, binds each Run to explicit synthetic adapter provenance and writes fixtures into a temporary directory.

The gate covers all six Subject variants, historical import provenance, multi-bundle aggregation, retry and shard reduction, a physical activity shared by independent Check and Challenger executions, exact Challenge identity, lane, semantic scope and accountable launch inputs, partial-to-complete correction with one omission diagnostic, exact replay, semantic mismatch and schema failure. It asserts the public `azimuth run verify` and `azimuth run inspect` exit classes and the inspection account's explicitly unresolved repository and Assurance State authority.

These hand-authored synthetic selections establish standalone protocol consistency only. The experiment does not claim current model authority for their Check, Challenger or Qualification.

Run it from any directory:

```sh
experiments/run-bundles/check.sh
```
