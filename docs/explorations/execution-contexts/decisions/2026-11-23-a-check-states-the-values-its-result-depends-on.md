# Decision: a Check states the values its result depends on

Date: 2026-11-23. Decided by: the maintainer.

Asked which values belong in a Check's execution context, the maintainer said: "I believe
Azimuth should have execution contexts declared per component and a Check can use any subset
of their values". Asked whether a Check's result should depend only on the values the Check
states, or on every value its component declares, the maintainer said: "On the values the
Check states"

Claude's words, which the maintainer agreed to: each component declares the keys of its
execution context, as the design already says. A Check states values for any subset of those
keys, naming the component of each. A key the Check does not state takes its suite's default.
The result of the Check depends only on the values it states: it applies to a deployment when
each stated value equals the deployment's value, whatever the deployment holds for the other
keys. This replaces the design's "A result depends on every declared value of its component"
and "There is no list per Check". A Check whose claim depends on a value states it, also when
it equals the default. A Check that states a key its suite cannot set is a lint finding.

Rejected: every declared value counts for every result, because one flag turned on in
production would make every result of the component stop applying. And a list of the values a
result depends on, kept apart from the values the Check sets, as the earlier version had,
because a Check would hold two lists to keep in step.

The cost: a Check that depends on a value and does not state it keeps applying when production
changes that value, and only a review catches the missing line. A Check that states more than
it needs applies less often and needs more runs.

Example: the component `api` declares the keys `instant_payouts` and `signup`. The Check
`payout-is-sent-at-once` states `api.instant_payouts=on`, and `signup` takes the suite's
default. When production turns `signup` off, the result still applies. When production turns
`instant_payouts` off, it stops applying, and a Check that states `api.instant_payouts=off` is
what covers that deployment.
