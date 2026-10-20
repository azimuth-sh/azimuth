# Runtime observation identity

Status: proposed
Intent delta: none
Because: Remove per-release infrastructure bindings while preserving exact deployment evidence attribution.

Dima approved push-only observer and collector data flow: observers identify what they actually measured, collectors validate and deliver, reconciliation independently reports deployments, and ASS resolves applicability. Ordinary application releases must not require observer/collector configuration or rollout. Existing immutable evidence remains intact; unmatched/ambiguous evidence cannot establish Claim Status. No release acceptance or archival is authorized by this adjustment.
