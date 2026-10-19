# Change: Relational Assurance State storage

Status: proposed
Intent delta: none
Because: This implements existing immutable-record, authority and exact-evidence obligations using relational storage; native wire contracts and accepted identities remain unchanged.

## Authorization
Dima explicitly requested removing JSON from the ASS data model, authorized redeployment, and subsequently approved a fresh database instead of migration on 2026-10-19. Keep the previous database and a backup for recovery. Restore the previously approved baseline and republish the current release. Do not accept a release or silently transfer evidence across Subjects.

## Outcome and scope
All persisted ASS domain data uses named relational tables and typed columns, including the earlier assurance API. No JSON/JSONB, serialized JSON text/blob payloads or generic JSON-tree/EAV store. JSON is the exchange/reconstruction format only. Schema-bound codecs may share plumbing but every accepted property has a named schema and typed column or domain relation. Preserve array order, absent/null distinction, historical variants and fingerprints.

Use a fresh schema and database; never run a destructive schema transition against the previous production database. Validate APIs, immutable replay/correction rules and lossless reconstruction with real PostgreSQL. Query compact summaries and move expensive assessment work away from request-serving workers.

## Affected claims
Existing framework assurance-deployment, release orchestration and Run protocol Claims retain their accepted intent. No new non-routine Claim or automatic acceptance policy is introduced.

## Completion conditions
No JSON storage in initialized ASS schema. Existing public API behavior and fingerprint contracts remain valid. Native publication round-trips against relational storage. Tests and CI pass, new images are published, Terraform switches to a separately retained fresh database, baseline/current release are republished, reconciler records a fresh complete deployment, and UI read latency is measured. Historic database remains recoverable.
