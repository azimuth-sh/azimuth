# Decision: a context that was not applied gives no result

Date: 2026-11-23. Decided by: the maintainer.

Asked what a Check's result should be when the report of a test run shows a value other than
the one the Check stated, the maintainer said: "Not run with an explicit error? I forgot what
statuses Azimuth has". Asked about the reason, the maintainer asked: "Would we then know that
the component is to blame and what to see at?"

Claude's words, which the maintainer agreed to: after a launch, the core compares the report of
each component of the suite with the full context of the launch. When a value differs, a key is
not reported as applied by Azimuth, or a component gives no report, the Checks of that launch
get no result, whatever their tests' outcomes. Each such Check is owed with the reason "context
not applied". The reason ends the local lane with status 1, as a crash does, because the fault
is in the tree and not on the machine. The report of a component says whether the library
received the context, whether applying was turned on, and, for each key, whether the value it
applied was overridden later and by which source. So the reason names the component, the key,
the stated and the reported value, and one cause: the context never reached the component,
applying is off, the value was overridden by a named source, or no report came from the
declared place.

Rejected: a fail, because the tests did not run under the values the Check claims. A pass whose
report carries a warning, because a pass under other values would let the result apply where
its claim was never shown. "Cannot run here", because it lets the local lane succeed, and a
component that does not apply its context is not a machine's lack.

The cost: one component that does not apply its context leaves every Check of its launch owed,
also Checks that state nothing about that component. The library has to follow its stack's
configuration closely enough to see a later source override its value.

Example: a launch passes `Payouts:Instant=false` to `api`. The orchestrator that starts the
stack does not pass the variable on. The report of `api` says it received no context, and
`true` from its configuration. Both Checks of the launch are owed with "context not applied:
api never received the context; Payouts:Instant stated false, reported true", and the run ends
with status 1.
