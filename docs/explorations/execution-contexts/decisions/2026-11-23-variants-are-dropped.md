# Decision: variants are dropped

Date: 2026-11-23. Decided by: the maintainer.

Asked what should become of variants in the design, the deployment-named sets of values that
the core would pass to a suite at launch, the maintainer said: "Maybe just drop them?"

Claude's words, which the maintainer agreed to: the design has no variants. A Check runs under
its Scenario's condition, and the values that a deployment holds for other keys never decide
whether its result applies, so a run of the same Check under a deployment's values would add
no evidence. A launch's context comes entirely from `Context:` lines, which are already inputs
of the Checks they touch, so nothing else about the context enters the fingerprint. The
design's example of a flag about to be turned on reads: the Scenarios whose condition holds the
new value begin to apply, those with the old value become not applicable, and nothing runs
again. A run that sets unstated keys to another deployment's values, to find a Check that
depends on a key its Scenario does not state, is left for Challenges.

Rejected: variants that override the values a Scenario states, because a result would then
claim one value while its tests ran under another. Variants that set only unstated keys and
count as evidence, because such keys do not decide whether a result applies.

The cost: a Check that silently depends on a key its Scenario does not state is found only by a
review until Challenges exist.

Example: production is about to turn `payouts.instant` on. `operator-sees-an-instant-payout`,
with `Context: payouts.instant=true`, already has its result. On the day of the switch it
begins to apply, `operator-sees-a-waiting-payout` becomes not applicable, and no Check runs
again.
