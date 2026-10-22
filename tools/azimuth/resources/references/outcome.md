# Outcome reference

`outcome.md` records what the completed change actually established. State delivered behavior, accepted departures from proposal or design, engineering verification performed, unperformed checks, residual risks and rollout-dependent conditions.

Do not manufacture production exposure, observations or evidence. Finalization requires current facets to describe what exists and every completion condition to have an honest disposition. Archival preserves the complete decision record.

When a change adds non-routine Requirements and no locally authored Requirement Status exists, a human may record the acceptance decision in `acceptance.json` beside `outcome.md`: `format` `azimuth-change-acceptance`, `version` 1, `accepted_by`, `accepted_on` (YYYY-MM-DD), the accepted `requirements` this change adds, and a non-empty `basis`. The record clears only `missing-requirement-assurance` for those Requirements. An agent never writes it on its own initiative; it records a decision the accountable human has stated.
