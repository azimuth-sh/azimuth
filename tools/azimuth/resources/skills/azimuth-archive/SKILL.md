---
name: azimuth-archive
description: Finalize and archive a completed Azimuth change after current facets, permitted engineering checks, outcomes and rollout-dependent conditions are satisfied. Use only at the explicit acceptance boundary; never manufacture facts or commit implicitly.
---

# Finalize and archive a change

1. Read the target repository's instructions, complete change, current account and implementation diff.
2. Require explicit finalization or archival authorization. In a federated project, require the complete project account rather than local-only output.
3. Confirm every completion condition has an honest disposition, current facets describe what exists, plan work is complete, required permitted checks have results and unresolved rollout conditions are recorded without being promoted to facts.
4. Run `azimuth reference show outcome`, write or revise `outcome.md` with delivered behavior, departures, verification performed, residuals and operational conditions.
5. Run the repository's normal `azimuth validate`, relevant change inspection and `azimuth change finalize <id>`.
6. Inspect the exact finalized account and ask for archival approval if it was not already explicit.
7. Run `azimuth change archive <id> --date <YYYY-MM-DD>` or the applicable project acceptance command.
8. Report the archive destination and remaining external conditions. Do not deploy, push, publish or commit unless separately authorized.

Never archive incomplete work to clear an active directory, treat a passing command as product evidence, or fabricate production exposure.
