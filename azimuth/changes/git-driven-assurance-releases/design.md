# Design

## Ownership
The canonical change is the sole authority; consumer repositories contain integration only. A release repository explicitly configures allowlisted sources and component bindings. Labels are required and non-unique; fingerprints are identities.

## Trust
PR validation never receives publication credentials. Trusted workflow code reads only bounded declarative request/input JSON from an exact approved PR commit, independently gates source revisions and does not execute release PR scripts. Publication is environment-protected. Generated values and receipts are tied to an input content fingerprint excluding generated outputs, preventing recursive rebuilds and self-reference. Final deployment gates verify rendered values and exact publication identity.

## Offline handoff
Local workflow ends with prepared publication inputs; native records remain immutable, compact and Git tracked. CI retains verified packages as Actions artifacts. No local ASS connection or generated receipt is required.

## Observations
A separate reconciler periodically reads operator-approved namespaces and workload names using read-only RBAC. It checks Argo applied revision, workload readiness and actual Pod image IDs. Desired images never substitute for runtime observations. Partial, unknown and drifted states remain explicit. ASS stores append-only observations with stable observation identities and monotonic publisher sequence, plus server receipt freshness. Retries are idempotent; stale writes cannot replace current state. Only a complete matched rollout can record/select a known release deployment; adverse observations invalidate current freshness without inventing a composition.

## Acceptance
An authenticated owner performs an explicit server-side action with exact release, expected accepted baseline and bounded reason. Existing server evidence gates remain authoritative. No credentials enter the browser; Git finalization remains separate.

## Operations
Terraform enables a separate Deployment only with an actual pinned digest and mounted publisher secret. It has no workload mutation permissions and no automatic repair/deploy/acceptance authority. No infrastructure is applied in this change.
