# Decision: a report says whether applying is possible

Date: 2026-11-24. Decided by: the maintainer.

Asked whether the report of a component should carry, beside the name and the context, whether
applying is possible in that component, the maintainer said: "Yes"

Claude's words, which the maintainer agreed to: the report of a component carries `applying`,
`possible` when the condition its registration gives holds and `not possible` when it does not,
in a launch and in a deployment alike. When a launch gives "context not applied" for a stated
key, the cause comes from the report: `not possible` gives "applying is off"; `possible`, with
the key's value from a source that Azimuth's value would win over, gives "the context never
reached the component"; `possible`, with the value from a source that wins over Azimuth's, gives
"overridden by" that source, named. How the report names a source so that these two can be told
apart is left to the design.

This replaces what the note `a-context-that-was-not-applied-gives-no-result` says about the
report saying "whether the library received the context": the report says whether applying is
possible, and a context that never arrived is read from that and the source. It adds `applying`
to what the note `a-component-has-one-report` says the report holds.

Rejected: a report with the context alone, whose output lists "never reached" and "applying is
off" as both possible, because the two are fixed in different places, the runner configuration
and the component's own setting. A field saying whether the library received `AZIMUTH_CONTEXT`,
because it names a launch's variable in a report that production gives too.

The cost: one more field that every library reports and every reader of the report reads. Every
production instance reports `applying: not possible`.

Example: `api` registers the library with applying possible when `ASPNETCORE_ENVIRONMENT` is
`Development`, and its configuration holds `Payouts:Instant=false`. A launch states
`payouts.instant=true`, but the runner configuration starts `api` with `Staging`. The report
shows `"applying": "not possible"` and `false` from configuration, and the Checks are owed with
"context not applied: api, applying is off; payouts.instant stated true, reported false".
