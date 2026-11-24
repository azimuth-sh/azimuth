# Decision: contexts add fields and keep the numbers of the adapter contract

Date: 2026-11-24. Decided by: the maintainer.

Asked whether contexts should add fields to `adapter-request-1` and `adapter-report-1` and keep
their numbers, the maintainer said: "Yes, they should"

Claude's words, which the maintainer agreed to: the request of a launch gains its condition and,
for each component the suite starts, its name and report path. The report gains `components`,
with each report the adapter read, or the reason it could not read one. Both formats keep their
numbers, as the design's rule on added fields says. A report with no `components` gives every
Check of the launch that has a condition "context not applied", with the adapter as the cause,
and the Checks with no condition keep their results.

This replaces what the map said about "a new number of the adapter contract that every adapter
speaks".

Rejected: raising both numbers, because every adapter would be released again at once, and an
adapter that does not know contexts would lose the results of Checks with no condition too.
Giving the reports to the core in a file beside the report, because the contract is one document
in and one document out.

The cost: an adapter that does not know contexts is told apart only by a missing field, so the
core cannot say "this adapter is too old", only that it reports no components.

Example: a project pins a Playwright adapter released before contexts. A launch of
`payout-is-sent-at-once` states `payouts.instant=true`. The report comes back with no
`components`, so its Checks are owed with "context not applied: the adapter reports no
components". A launch of the same suite with no condition gives its results as before.
