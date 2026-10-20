# Independent design review

Date: 2026-10-20
Disposition: implement after incorporating the bounded decisions below. These are contract decisions, not requests for another framework abstraction.

## Evidence inspected

- `proposal.md`, `design.md`, `plan.md` for this change.
- `tools/azimuth/src/run.rs`: exact Subjects, repository/artifact identities, Run status, attempts and three Observation outcomes.
- `tools/azimuth/src/run_plan.rs` and `run_selection.rs`: existing exact string context maps and launch selection.
- `tools/azimuth/src/assurance_review.rs`: current exact-Subject assessment, correction heads, prose binding contribution and routine-Claim handling.
- `tools/azimuth/src/release.rs`: sources, builds, component artifacts and their structural provenance validation.
- `services/assurance/server/src/run_ledger/deployment_observation.rs`: authenticated publication, stale/future rejection and deployment observation expiry.

No test execution was needed for this review. This report does not assert that the proposed behavior is implemented.

## Required design corrections

### 1. Make contribution semantics machine-readable

Existing binding `contribution` is prose. The design requires a reviewed mapping that distinguishes a counterexample from an operational failure, but does not yet define its representation. Add a bounded structured contribution rule to the fingerprinted Check/Case binding or verification policy, included in Claim Review dependencies. A test adapter must classify execution/setup failures as inconclusive. It cannot infer semantic counterexamples from a native test failure. A protocol `violated` Observation alone must not bypass the reviewed contribution rule.

### 2. Specify temporal selection across distinct Runs

Correction heads select versions of one Run, not the current observation among successive Monitor executions. Choose the latest eligible observation per Monitor/Check/Case/deployment, ordered by observation time with explicit ambiguity handling. An equal-time disagreement must remain visible and block confirmation. A newer explicit inconclusive execution cannot silently disappear behind an older successful execution. Missing/stale review must not hide applicable adverse facts; whether Status is violated or unresolved must follow a single documented precedence rule. Historical counterexamples stay inspectable even when no longer current.

### 3. Resolve component scope through one unambiguous provenance path

Build `sources` contains repository identifiers resolved through release `sources`, not embedded tested revisions. Compare the selected component artifact's complete build input repository set and exact revision/content identities against the Run's declared source scope. Do not compare only one matching repository or require all unrelated release repositories. Existing release validation allows an artifact digest in more than one build; reject conflicting producer input sets for applicability, or require an explicit selected build. Unsupported Subject kinds and missing component scope fail closed with reasons.

### 4. Bound context lifetime and instance coverage

Before/after equality is sufficient only under the proposed startup-fixed configuration restriction. Require captures to identify the same application instance lifetime (not a reusable pod/service name) and context contract revision; reject a restart, absent capture, mixed generation or interval outside the execution bounds. Reloadable configuration is unsupported initially unless an immutable configuration generation can bind the whole execution. Production support requires coverage of every relevant live replica/component; do not sample one replica or merge disagreeing contexts. Unknown/unready instances must have an explicit policy rather than disappear from coverage.

### 5. Specify trusted capture attribution

A self-reported context envelope is not proof of deployment identity or collection time. The authenticated observer/publisher must bind capture to an independently resolved instance and deployment. Reuse project-scoped transport trust without PKI, but define which authorized producer roles may supply source execution, production context and Monitor evidence. Never treat any valid viewer/publisher token as authority to invent all relation types. CI artifact import must bind a report to its expected launch, workflow source and exact execution scope; timestamps and instance strings alone are insufficient.

### 6. Define Monitor freshness completely

Use observed time, bound future clock skew, and reject expired/future evidence for support; replaying uploads never renews evidence. Apply freshness to individual required Check/Case observations, not just the latest Run or deployment observation. Changing a Monitor's interval/freshness/target is a fingerprinted model change. Define whether interval is operational scheduling metadata while freshness determines applicability. An overdue Monitor is uncertainty, not a Claim violation. Mixed-deployment windows cannot support a single deployment without explicit partitioning.

### 7. Preserve baseline policy and identity boundaries

Routine Claims currently require no independent review; retain that explicit rule rather than fabricate accepted reviews or make all 46 framework Claims unresolved. Historical authored health records remain separate from computed Status. Changes to reviewed verification/context/contribution dependencies invalidate support until reviewed under the existing dependency rules. Existing immutable evidence and historical catalogs must remain readable after additive persistence changes.

## Recommended initial evaluation order

1. Resolve current observed deployment and selected release/model; report freshness and composition failures.
2. Resolve current Check definitions, binding requirements and review eligibility.
3. Select correction heads and current Monitor observations without erasing adverse facts.
4. Validate exact scope/provenance, trusted attribution, contexts and per-observation freshness.
5. Expose applicable counterexamples according to the structured reviewed rule.
6. Confirm only when all required bindings have applicable supporting evidence and review policy is satisfied; otherwise explain unresolved requirements.

## Verification targets

Cover conflicting build producers, an extra required source input, unrelated release repositories, heterogeneous test contexts, instance restart, mixed replicas, replayed capture, newer inconclusive Monitor execution, equal-time contradictory observations, correction invalidation, stale review plus adverse evidence, and preserved routine policy. Existing identity/correction mechanisms should be reused rather than duplicated.
