# Decision: three repairs of earlier words

Date: 2026-11-24. Decided by: the maintainer.

A fresh reader of the notes found three places where earlier words of Claude contradict later
notes or the design. Claude proposed the repairs below.

Claude's words, which the maintainer agreed to:

First, a Scenario with no `Context:` of its own is promised wherever its Requirement is, and its
Checks run under its Requirement's condition. A Requirement with no `Context:`, and a Scenario of
it with none, are promised everywhere. The same holds when a Scenario moves a varying value into
its setup: it keeps any condition of its Requirement.

Second, what lint holds when a key is retired: a `Context:` line that names a key
`execution-context.toml` does not declare, a component listed for a key that `project.toml` does
not declare, and a key listed for no component. A Scenario that claimed the behavior that stays
drops its `Context:` for the key, and the results of its Checks are owed. A Scenario that claimed
the behavior that leaves goes with its Checks. The mapping inside a component is code, so lint
does not see it, and a component that still maps a retired key shows it in its report as a key it
is not listed for.

Third, during the rollout that retires a key, a result whose Scenario no longer names the key
applies to a release only where its provenance and fingerprint apply to that release, as for any
result. Retiring the key widens no result to code it did not run on.

This replaces what the note `a-scenario-states-the-context-and-its-checks-run-under-it` says about
a Scenario with no `Context:` being "promised everywhere" and running "under the default", what
the note `a-value-that-varies-between-requests-is-not-context` says about such a Scenario being
"promised everywhere", and what the note `a-retired-key-leaves-in-one-change` says about a Check
that states a key, a suite's default or mapping, and results that "apply to both releases".

Rejected: none. These repair words, not decisions.

The cost: none beyond those of the notes they repair.

Example: the Requirement `payout-is-sent-at-once` carries `Context: payouts.instant=true`. Its
Scenario `payout-reaches-the-card` carries no `Context:`, so it is promised only where
`payouts.instant` is `true`, and its Checks run with `payouts.instant=true`. Later
`payouts.instant` is retired: the change removes the key from `execution-context.toml` and the
`Context:` line from the Requirement, and `api` stops mapping the key. Lint passes only after
that, and the results of the Checks are owed.
