# Decision: a Check's condition is part of its claim

Date: 2026-11-24. Decided by: the maintainer.

Asked whether a Check's effective condition, with its inherited lines, should be part of its
claim digest, the maintainer said: "Yes"

Claude's words, which the maintainer agreed to: the claim part of a Check's fingerprint is the
digest of the Check's claim text followed by its effective condition, when it has one. The
effective condition is the `Context:` lines of its Scenario, or of its Requirement when the
Scenario has none, normalized. A Check with no condition keeps the digest it has today. So a
changed condition makes the Check owed with "claim changed", also once a provider narrows the
Check's files and the spec file is no longer among them.

This replaces what the note `variants-are-dropped` says about `Context:` lines being "already
inputs": they are inputs through the claim part, not through the files part.

Rejected: a fifth part `context`, because the layout of the fingerprint would change and every
Check of every project would be owed once. Keeping the spec file of the Check's Requirement in
the Check's list of files, because any edit of that file would then make the Check owed.

The cost: the owed reason says "claim changed" and not which line of the condition changed. An
approval of the claim becomes stale when the condition changes.

Example: the Requirement `payout-is-sent-at-once` carries `Context: payouts.region=eu`, and its
Check `payout-reaches-the-card` passed. A change makes the line `payouts.region=us`, and no code
changes. The claim digest of the Check moves, so it is owed with "claim changed", and its next
result runs under `payouts.region=us`.
