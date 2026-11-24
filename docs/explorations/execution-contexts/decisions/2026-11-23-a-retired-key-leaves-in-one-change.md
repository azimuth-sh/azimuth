# Decision: a retired key leaves in one change

Date: 2026-11-23. Decided by: the maintainer.

Asked what happens when a feature flag is retired, Claude described what names the flag's key:
Checks that state it, the default and mapping of a suite, and `Context:` conditions. Asked
whether retiring a flag should be one change that lint holds until nothing names the key any
more, or whether some of it may wait for a later change, the maintainer said: "One change that
lint holds"

Claude's words, which the maintainer agreed to: a key leaves a component's declaration in one
change, together with everything that names it. Lint reports each of these as a finding until
it is gone: a Check that states a key its component does not declare, a suite's default or
mapping for such a key, and a `Context:` condition that names it. A Check that claimed the
behavior that stays drops its statement, and its result is owed. A Check that claimed the
behavior that leaves goes with its Scenario. During the rollout, results that no longer state
the key apply to both releases, and results that still state it apply only where it is still
reported.

Rejected: cleaning up over several changes, with lint only warning meanwhile, because a Check
would then state a key that no release reports, and its result would never apply again without
anyone noticing. And leaving it to a review, because a review reads the change, not every Check
that names the key.

The cost: retiring a flag that many Checks name is one large change.

Example: payouts at once become permanent. One change removes `instant_payouts` from the
declaration of `api`. It drops `api.instant_payouts=on` from `payout-is-sent-at-once`, whose
result becomes owed and runs again. It removes the Scenario of the delay that the flag used to
allow, with its Check, and the suite's default and mapping for the key. Lint passes only then.
