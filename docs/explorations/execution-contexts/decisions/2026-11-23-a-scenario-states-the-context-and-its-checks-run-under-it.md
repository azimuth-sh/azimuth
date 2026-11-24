# Decision: a Scenario states the context, and its Checks run under it

Date: 2026-11-23. Decided by: the maintainer.

Asked what should become of the `Context:` condition on a Requirement now that a Check states
its own context, the maintainer said: "I believe we should drop it. But I may be wrong. Let's
discuss possible problems". Claude named the problems and, on the maintainer's request, wrote a
sample with a line `Runs with:` on Checks. The maintainer said: "It seems that we should keep
it?", and about a Scenario whose Checks state different values: "Shouldn't a Scenario specify
what context its Check needs? If so, we could drop "Runs with:"". About `Context:` meaning two
things on a Scenario, the maintainer asked: "If we apply the rule of a Scenario per context
would it stop meaning two things at once?", and about a Requirement that carries one: "What
happens when a Requirement carries a Context? All Checks run in this context? What if a
Requirement and its Scenario Contexts disagree?"

Claude's words, which the maintainer agreed to: a Requirement or a Scenario may carry a
`Context:` line that names a key and a value. On a Requirement it says where the Requirement is
promised. On a Scenario it is the condition of that case, like its other givens: the case is
promised in that condition and proven in it. Where the condition is false, the Requirement or
the Scenario is "Not applicable here", as the design already says. A Scenario takes the
condition of its Requirement and may add its own; a Scenario whose `Context:` gives a key of its
Requirement's condition another value is a lint finding. A Scenario that repeats its
Requirement's condition with the same value changes nothing. A condition names one value per
key, so a behavior promised for some values of a key is written with one Scenario per value. A
Check states no values of its own. It runs under its Scenario's condition, and the suite's
default gives every other key. So in a deployment, a Scenario either applies with its Checks'
results, or is not applicable. A behavior promised in several conditions is written as one
Scenario per condition, and a condition with no Scenario is a missing case, like any other. A
Scenario with no `Context:` is promised everywhere, and its Checks run under the default.

This replaces what the note `a-check-states-the-values-its-result-depends-on` says about a
Check stating values: the values that a result depends on are those of its Scenario's
condition.

Rejected: dropping `Context:`, because a deployment with a feature switched off would show
every Requirement of that feature as lacking evidence. Inferring where a Requirement applies
from its Checks' values, because it would read "nobody tested this configuration" as "not
promised here". A line `Runs with:` on Checks, because two Checks of one Scenario could then
state different values, and the Scenario would be half proven in a deployment.

The cost: a behavior promised in both states of a flag is written twice. A Scenario with no
`Context:` is promised everywhere and proven only under the suite's default.

Example: the Requirement `operator-sees-every-payout` holds whatever `payouts.instant` says. It
has the Scenarios `operator-sees-an-instant-payout`, with `Context: payouts.instant=true`, and
`operator-sees-a-waiting-payout`, with `Context: payouts.instant=false`. Their Checks state
nothing. In a deployment with the flag on, the first Scenario needs evidence and the second is
not applicable.
