# Decision: one launch per context

Date: 2026-11-23. Decided by: the maintainer.

Asked how the Checks requested from one suite should run when they state different contexts,
the maintainer asked: "There would be 2 Checks, correct? If so, can we just group them by
context and make 2 runs? One Check would be executed in one run and another Check in another
run?". Claude proposed that a test marked by Checks whose contexts differ is a lint finding,
and the maintainer asked: "Why? Won't this test just be run several times with different
contexts?"

Claude's words, which the maintainer agreed to: the core groups the requested Checks of a suite
by their full context, the stated values together with the suite's defaults, and makes one
launch per group. Each launch passes its own `AZIMUTH_CONTEXT` and the ids of its group's
Checks. Launches of one suite run one after another. A test marked by Checks whose contexts
differ runs in the launch of each of them. A launch's result of a test counts only for the
Checks that the launch was made for, whatever other markers the test carries.

Rejected: one launch that changes the context between tests, because a whole stack takes its
configuration when it starts. A launch per Check, because Checks with the same context would
start the same stack again for nothing. A lint finding for a test marked by Checks whose
contexts differ, because such a test can run once in each context.

The cost: a suite whose requested Checks state several contexts starts once per context, and
for a whole stack each start takes its own minute. A test marked by Checks of several contexts
runs once in each of them.

Example: of thirteen Checks requested from a suite, ten state `Payouts:Instant=true` and three
state `false`. The core makes two launches, one with ten Check ids and one with three. A
journey marked by `payout-is-sent-at-once` and by `payout-waits-for-review` runs in both
launches. Its result in the first counts only for `payout-is-sent-at-once`, and in the second
only for `payout-waits-for-review`.
