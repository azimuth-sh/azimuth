# Decision: a launch compares the stated keys of the components its suite starts

Date: 2026-11-23. Decided by: the maintainer.

Asked what a suite must show when it starts only some of the components that a key is listed
for, the maintainer said: "The components the suite starts?"

Claude's words, which the maintainer agreed to: in a launch, Azimuth applies a stated key to the
components that the suite starts and that the key is listed for, and expects a report of the key
from each of them. A listed component that the suite does not start, such as one replaced by a
stand-in, is not expected. The comparison covers only the keys that the launch's condition
states: for each, every expected report gives the stated value, applied by Azimuth, in both of
its reads. A difference in an unstated key, or in the time of a read, changes nothing. A launch
whose condition states nothing expects no report. A Check whose Scenario states a key that no
component of its suite is listed for gives no result, "context not applied", with the cause that
no started component reads the key, and lint names the case before the run.

This replaces what the note `a-key-belongs-to-the-product` says about a test run applying a
key's value "to every component that the key is listed for", what the note
`a-context-that-was-not-applied-gives-no-result` says about a key "not reported as applied by
Azimuth", and what the note `reports-are-collected-outside-the-library` says about "a difference
between the two reports".

Rejected: expecting every listed component, because a suite with a stand-in could then never
prove a Scenario with a condition. Counting a Check whose suite starts no component of its key,
because its result would claim a condition that nobody applied.

The cost: a result proven with a stand-in counts in a deployment like one proven through the
whole stack, and how faithful the stand-in is stays the suite's concern, as with any stand-in.

Example: `payouts.instant` is listed for `api` and `website`. The suite `website-tests` starts
`website` with a stand-in for `api`. A launch for `payout-is-sent-at-once`, with
`payouts.instant=true`, expects a report from `website` only. The suite `ledger-tests` starts
only `ledger`, so a Check of that Scenario marked there gives "context not applied", and lint
names it before the run.
