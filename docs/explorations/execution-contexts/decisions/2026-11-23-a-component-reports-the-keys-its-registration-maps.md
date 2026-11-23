# Decision: a component reports the keys its registration maps

Date: 2026-11-23. Decided by: the maintainer.

Asked how the library inside a component knows which keys to report in a deployment, where no
context arrives, the maintainer said: "All mapped keys?". About a rule in the registration that
gives each key's setting, with an override, the maintainer asked: "Shouldn't we require that each
key must explicitly mapped?", and about a key named with the setting it is read from: "Not only
reads but also writes. So it is a mapping"

Claude's words, which the maintainer agreed to: a component that registers the library maps each
key it uses to one of its own settings. The library reads the key's value from that setting, and
applies a value to that setting. No rule derives a setting from a key. The library reports
exactly the keys it maps, each with its value or with no value, and with its source. The library
knows no types: the core and the server read a reported value by the type that the project file
declares. A component listed for a key whose report lacks it, and a report of a key whose
component is not listed for it, are findings, as the note `a-key-belongs-to-the-product` says.

This replaces what the note `a-key-belongs-to-the-product` says about a library that "maps a key
to the component's own setting by a rule that the component gives when it registers the library,
such as a section or a prefix, and an override in the same registration names the setting of one
key".

Rejected: a rule from a key to its setting, such as a section or a prefix, with an override, as
the maintainer first proposed, because each stack's library would need its own agreed spelling of
a dotted key as a setting, and a reviewer would have to apply the rule to learn which setting a
key uses. A library that reads the project file, because a component would then need the model
repository where it runs. A library that reports every setting under a section or a prefix,
because it would report settings that are not keys and could not say that a key is missing.

The cost: one mapping per key in every component that uses it. The keys of a component are
written twice, in the project file and in its mapping, and a difference between them shows only
in a report. A renamed setting changes the mapping too.

Example: `api` maps `payouts.instant` to `Features:Payouts:Instant` and `payouts.limit` to
`Payouts:DailyLimit`. `website` maps `payouts.instant` to `FEATURE_INSTANT_PAYOUTS`. A launch
with `payouts.instant=true` applies `true` to `Features:Payouts:Instant` and to
`FEATURE_INSTANT_PAYOUTS`. In production the report of `api` gives `payouts.instant` as `true`,
from configuration, and `payouts.limit` with no value, and the server shows the finding that
`api` reports no value for `payouts.limit`.
