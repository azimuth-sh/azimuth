#!/usr/bin/env sh
set -eu

assurance_url="${ASSURANCE_URL:-http://127.0.0.1:8080}"
definition_id="expected-load"
observed_at=1786442400

curl --fail --silent --show-error \
  --header 'content-type: application/json' \
  --data '{"id":"checkout","name":"Checkout assurance","createdAt":1786442300}' \
  "${assurance_url}/v1/projects" >/dev/null

snapshot_payload="$(cargo run --quiet --manifest-path Cargo.toml \
  --package azimuth-assurance-domain --example seed_snapshot)"
snapshot_id="$(printf '%s' "$snapshot_payload" | \
  sed -n 's/.*"id":"\([^"]*\)","project".*/\1/p')"
contract_fingerprint="$(printf '%s' "$snapshot_payload" | \
  sed -n 's/.*"contractFingerprint":"\([^"]*\)".*/\1/p')"

if [ -z "$snapshot_id" ] || [ -z "$contract_fingerprint" ]; then
  printf '%s\n' 'The generated model snapshot did not contain its identities.' >&2
  exit 1
fi

curl --fail --silent --show-error \
  --header 'content-type: application/json' \
  --data "$snapshot_payload" \
  "${assurance_url}/v1/projects/checkout/model-snapshots" >/dev/null

definition_response="$(curl --fail --silent --show-error \
  --header 'content-type: application/json' \
  --data "{\"id\":\"expected-load\",\"claim\":{\"spec\":\"checkout/performance\",\"claim\":\"latency-objective\",\"contractFingerprint\":\"${contract_fingerprint}\"},\"assertion\":\"p95 latency is below 300 milliseconds\",\"scope\":\"e2e\",\"quantification\":\"example\",\"oracle\":\"direct\",\"stage\":\"merge\",\"inputs\":[\"tests/load.js@sha256:definition\"],\"requiredContext\":{\"capacity-profile\":\"production-like\"},\"declaredAt\":1786442200}" \
  "${assurance_url}/v1/projects/checkout/definitions")"
definition_fingerprint="$(printf '%s' "$definition_response" | sed -n 's/.*"definitionFingerprint":"\([^"]*\)".*/\1/p')"

if [ -z "$definition_fingerprint" ]; then
  printf '%s\n' 'The definition response did not contain a fingerprint.' >&2
  exit 1
fi

curl --fail --silent --show-error \
  --header 'content-type: application/json' \
  --data "{\"id\":\"qualification-1\",\"definitionId\":\"${definition_id}\",\"definitionFingerprint\":\"${definition_fingerprint}\",\"verdict\":\"qualified\",\"qualifiedAt\":1786442250,\"rationale\":\"The threshold and execution context directly test the claim.\"}" \
  "${assurance_url}/v1/projects/checkout/qualifications" >/dev/null

curl --fail --silent --show-error \
  --header 'content-type: application/json' \
  --data "{\"id\":\"ci-revision-a\",\"definitionId\":\"${definition_id}\",\"definitionFingerprint\":\"${definition_fingerprint}\",\"stage\":\"merge\",\"subject\":{\"projectSnapshot\":\"${snapshot_id}\",\"revision\":\"revision-a\",\"artifactDigest\":null,\"deploymentId\":null,\"environment\":\"ci\",\"cohort\":null},\"context\":{\"capacity-profile\":\"production-like\"},\"observedAt\":${observed_at},\"expiresAt\":null,\"outcome\":\"satisfied\",\"report\":\"reports/load-revision-a.json\"}" \
  "${assurance_url}/v1/projects/checkout/observations" >/dev/null

curl --fail --silent --show-error \
  --header 'content-type: application/json' \
  --data "{\"definitionId\":\"${definition_id}\",\"stage\":\"merge\",\"subject\":{\"projectSnapshot\":\"${snapshot_id}\",\"revision\":\"revision-a\",\"artifactDigest\":null,\"deploymentId\":null,\"environment\":\"ci\",\"cohort\":null},\"at\":${observed_at}}" \
  "${assurance_url}/v1/projects/checkout/gates/evaluate"
printf '\nOpen http://127.0.0.1:3000/projects/checkout\n'
