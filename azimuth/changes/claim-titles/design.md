# Design

Add optional `Title:` to behavioural and site-domain Claim headers. Preserve it through intent round trips and catalog publication; exclude display metadata from normative fingerprint projections. Catalogs may omit a title. ASS shows title or stable ID in compact views, with the full statement in details. Add a versioned nullable SQL title column to existing catalog Claim rows and recreate readers transactionally; preserve absence and immutable historical records.
