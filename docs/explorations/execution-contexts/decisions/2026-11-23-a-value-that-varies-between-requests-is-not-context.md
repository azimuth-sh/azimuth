# Decision: a value that varies between requests is not context

Date: 2026-11-23. Decided by: the maintainer.

About a setting whose value can differ between two concurrent requests, for example a flag on
for one user in ten, the maintainer said: "I think we should exclude such a settings from an
execution context." Asked how a source would tell that a value varies, the maintainer said:
"I think the change making a setting a per-request value should retire it from the context as
well"

Claude's words, which the maintainer agreed to: a key of an execution context has one value in
a component at a time, for every request it serves. A value that can differ between two
concurrent requests is not a key. A case that depends on it is a Scenario whose setup arranges
it, such as "a user for whom the flag is on", and carries no `Context:` for it. Such a Scenario
is promised everywhere, and where nobody has the flag on, its given does not occur. The change
that makes a key's value differ between requests also retires the key from the declaration, in
the same change, as the note `a-retired-key-leaves-in-one-change` says, and its Scenarios move
the condition into their setup.

Rejected: a key that holds several values at once, because a launch, a comparison with a
deployment and the agreement of components all need one value. Treating a varying key as
promised in both values, because a deployment's evidence would then need every Scenario of
both values, and a rollout to one user in ten would read as if the whole product held both. A
source that reports that a value varies, because only an integration with a flag service could
know it, and many services do not tell.

The cost: a Scenario of a per-user flag is proven only through its setup, and Azimuth cannot
tell in which deployments its given occurs. Azimuth cannot see that a value has begun to differ
between requests, so a change that forgets to retire its key is caught only by its review, and
the report then shows whatever single value the source gives.

Example: `payouts.instant` moves from a plain flag to a rollout for one user in ten. The change
that makes the move also removes the key from the declaration of `api`, and lint holds it until
the Scenarios with `Context: payouts.instant=true` are rewritten as "a user with instant
payouts receives a payout at once", with the user's variant arranged in the test.
