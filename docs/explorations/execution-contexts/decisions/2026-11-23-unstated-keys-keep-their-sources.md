# Decision: unstated keys keep their sources

Date: 2026-11-23. Decided by: the maintainer.

Asked where the default value of a key should be declared, and whether it could differ from one
suite to another, the maintainer asked: "Shouldn't it just be a default from sources?"

Claude's words, which the maintainer agreed to: a suite declares no default. In a test run
Azimuth applies only the keys that the Scenario's condition states. Every other key keeps the
value that the component's own sources give in the test setup, and the report records it with
its source. A launch groups the requested Checks by the values their conditions state. In a
deployment every declared key is still set explicitly.

This replaces what the note `keys-mirror-settings-and-values-are-typed-present-and-exact` says
about setting every key explicitly in a test run, what the note `one-launch-per-context` says
about grouping by the stated values together with the suite's defaults, and what the note
`a-scenario-states-the-context-and-its-checks-run-under-it` says about the suite's default
giving every other key.

Rejected: a default per suite, because a value that does not decide whether a result applies
needs no declaration. A default per key for the whole project, for the same reason.

The cost: on two machines, an unstated key can take different values from the environment, and
a Check can behave differently on them although its fingerprint is equal. The report shows the
value and its source, so the difference can be seen after the run.

Example: the Scenario `payout-is-sent-at-once` states `payouts.instant=true`. The key
`payouts.limit` is not stated, and the test setup of `api` gives `1000` from its settings file.
The run applies only `payouts.instant`, and the report shows `payouts.limit` as `1000`, from
configuration.
