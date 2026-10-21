# Design

## Authority and projections

Immutable evidence and the native deployment assessor remain authoritative. Materialized assessment rows are derived, rebuildable state, never authored assurance or acceptance. Relational storage remains typed; neither document blobs nor serialized JSON text replace relational fields.

## Read boundary

Dashboard and detail reads select a committed assessment revision through indexed relational projections. They do not resolve target observations, reconstruct complete evidence histories or invoke the native assessor. Compact summaries omit evidence detail. Reads expose whether the committed projection matches current inputs and is still before its expiry boundary. Pending, failed or expired assessment never appears as current Confirmed status.

## Update boundary

Input changes durably invalidate dependent projections in the same transaction as the fact. The dependency graph covers Check definitions/fingerprints, Case bindings, Claim definitions/reviews, execution/capture history, contracts, deployment reconciliation and release composition. Selective work uses affected Claims while retaining complete native validation closure; changing a deployment's composition or validity can affect every Claim. Replays do not manufacture new evidence.

Workers recover pending work after restarts, claim it exclusively and compute against one coherent input generation. They atomically publish Case/Claim rows and aggregate metadata only when that generation remains current. Concurrent writes cannot allow a stale assessment to overwrite fresh inputs. Work remains queued after failure. Expiry schedules refresh even without incoming facts. Full rebuild remains available for recovery and parity verification.

## Release boundary

Release publication prepares dependencies; deployment-specific status requires an actual reconciled deployment. Publication does not select authority or accept the release. Runtime observations remain attributed by exact observed image/process history.

## Review evidence closure

Deployment and definition-review projections must agree on relevant Challenger and retained basis history. Their snapshots include complete correction chains for exact current-review targets and relied-on logical Run identities, including a corrected head that removes or changes a target. Newly arriving relevant Challenge-only Runs invalidate the dependent Claim; unrelated target histories do not. Selecting only Check-bearing Runs is insufficient to preserve current review credibility.
