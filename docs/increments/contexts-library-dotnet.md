# Execution contexts, the context library for .NET: behaviors

A row is of one of two kinds. A row of behavior is one observable fact, and a test shows it. A row of prose says what a text tells its reader; its section begins with the words "Rows of prose", and no test reads the words of the text for it. The second cell is copied exactly from `docs/design.md`. A plain test fails when a quoted sentence is no longer in that file. A behavior with no sentence to quote is not built.

Status of this list: read and accepted by a person on 2026-11-24.

## The package

| Behavior | Quote from the design |
| --- | --- |
| The library is named `Azimuth.Context` for .NET | The libraries are `Azimuth.Context` for .NET and `@azimuth-sh/context` for Node. |
| The library speaks of keys, sources, values, applying and reports, and names no testing | The library speaks of keys, sources, values, applying and reports, and knows no testing. |
| The library opens no port | It opens no port and makes no connection out. |
| The library makes no connection out | It opens no port and makes no connection out. |
| The library exposes the report at no path of its own; the component does, at its declared `report` path | Its callback returns the report; the component exposes it under the rules of a door it already has, at its declared `report` path. |
| The same report is used in a launch and in a deployment | The same report is used in a launch and a deployment. |

## Registration

| Behavior | Quote from the design |
| --- | --- |
| A component registers its name | The component registers its name, a condition under which applying is possible, and an explicit mapping of every key it uses to its own setting. |
| A component registers the condition under which applying is possible | The component registers its name, a condition under which applying is possible, and an explicit mapping of every key it uses to its own setting. |
| A component registers an explicit mapping of every key it uses to its own setting | The component registers its name, a condition under which applying is possible, and an explicit mapping of every key it uses to its own setting. |
| A mapping reads the key's own setting | The mapping reads and writes that setting. |
| A mapping writes the key's own setting | The mapping reads and writes that setting. |
| No rule derives a setting from a key; a key with no mapping has no setting | No rule derives a setting from a key, and the library knows no types and reads no project file. |
| The library knows no types | No rule derives a setting from a key, and the library knows no types and reads no project file. |
| The library reads no project file | No rule derives a setting from a key, and the library knows no types and reads no project file. |
| The library gives text values and interprets none; the core and server interpret them by `execution-context.toml` | The core and server interpret its text values by `execution-context.toml`. |

## The condition and applying

| Behavior | Quote from the design |
| --- | --- |
| The component's own code gives the condition | The component's code gives the condition. |
| The library never guesses applying from an environment name | The library never guesses applying from an environment name. |
| The condition is evaluated once, at start | It is evaluated once, at start, before applying. |
| The condition is evaluated before applying | It is evaluated once, at start, before applying. |
| A later change to the condition leaves `applying` as it was | A later change to the condition changes neither `applying` nor the applied values. |
| A later change to the condition leaves the applied values as they were | A later change to the condition changes neither `applying` nor the applied values. |
| `applying` is `possible` when the condition held at start | `possible` when the registration's condition held at start, otherwise `not possible` |
| `applying` is `not possible` when the condition did not hold at start | `possible` when the registration's condition held at start, otherwise `not possible` |
| The library applies a value only when its condition held at start | The library applies only when its condition held at start. |
| At start, when its condition holds, the library reads `AZIMUTH_CONTEXT` | At start, when its condition holds, it reads `AZIMUTH_CONTEXT` and applies the value of each mapped key the variable names. |
| The library applies the value of each mapped key that `AZIMUTH_CONTEXT` names, and no other key | At start, when its condition holds, it reads `AZIMUTH_CONTEXT` and applies the value of each mapped key the variable names. |
| Without `AZIMUTH_CONTEXT` the library applies nothing | Without the variable it applies nothing. |
| An `AZIMUTH_CONTEXT` that is not one JSON object of text values stops the start with an error that names the variable | A variable that is not one JSON object of text values stops the start with an error that names it. |
| Applying is possible only under the component's own explicit condition | Applying is possible only under each component's own explicit condition. |
| The values to apply come from `AZIMUTH_CONTEXT`, one JSON object of product keys to text values | Each launch gets `AZIMUTH_CONTEXT` as one JSON object of product keys to normalized text values. |
| The launch's value reaches each mapped setting when applying is possible | The launch's value reaches both settings. |

## Sources

| Behavior | Quote from the design |
| --- | --- |
| A source reads a key's current value and where it came from | A source reads a key's current value and where it came from, and applies a value in this process. |
| A source applies a value in this process | A source reads a key's current value and where it came from, and applies a value in this process. |
| The library ships one source over the stack's configuration | It ships one source over the stack's configuration, used for each key with no other registered source. |
| The shipped source is used for each key with no other registered source | It ships one source over the stack's configuration, used for each key with no other registered source. |
| The source that applies Azimuth's values is added last among the configuration's sources | The source that applies Azimuth's values is added last among the configuration's sources, so its value wins. |
| Azimuth's applied value wins over the configuration's other sources | The source that applies Azimuth's values is added last among the configuration's sources, so its value wins. |
| A team registers its own source for the keys that a flag service, a database or another store serves | A team that reads a flag service, a database or another store registers its own source for those keys. |
| A team's own source has a read operation and an apply operation | A key served by a flag service instead uses the team's source, with both read and apply operations. |
| A key with a team's own source is read and applied through it | A key served by a flag service instead uses the team's source, with both read and apply operations. |
| A later source that wins is reported as an override | A later source that wins is reported as an override. |
| The library ships no flag-service integration | Azimuth ships no flag-service integration and requires no OpenFeature integration. |
| The library requires no OpenFeature integration | Azimuth ships no flag-service integration and requires no OpenFeature integration. |

## The shipped source over .NET configuration

| Behavior | Quote from the design |
| --- | --- |
| The shipped source reads the mapped configuration path through a configuration provider | The .NET source reads and writes the mapped configuration path through a configuration provider. |
| The shipped source writes the mapped configuration path through a configuration provider | The .NET source reads and writes the mapped configuration path through a configuration provider. |
| `api` mapped to `Features:Payouts:Instant` reads and applies `payouts.instant` at that configuration path | `api` maps `payouts.instant` to `Features:Payouts:Instant`; `website` maps it to `FEATURE_INSTANT_PAYOUTS`. |
| Another configuration store of the application needs a registered source of its own | An application's other configuration store needs a registered source of its own. |

## The callback and the report

| Behavior | Quote from the design |
| --- | --- |
| The callback returns the report | Its callback returns the report |
| The library reads every mapped key again at each call of the callback | The library reads every mapped key again whenever its callback is called, including a value that can change while it runs. |
| A value that changed while the component ran is read again at the next call | The library reads every mapped key again whenever its callback is called, including a value that can change while it runs. |
| The report has format `1` | ormat` | `1` | |
| The report reports exactly its mapped keys | The report of a component has format `1` and reports exactly its mapped keys. |
| The report carries `component`, `applying` and `context` | It carries `component`, `applying` and `context`. |
| The report carries no identity of its instance and no mark of a launch | It carries no identity of its instance and no mark of a launch. |
| `component` is the name from registration | the name from registration, which the reader compares with the expected component |
| `context` is an object of mapped product keys | an object of mapped product keys |
| A key's `value` is text, or `null` when absent | `value`, text or `null` when absent; `source`, or `null` when absent; `read`, the UTC time it was read |
| A key's `source` is `null` when absent | `value`, text or `null` when absent; `source`, or `null` when absent; `read`, the UTC time it was read |
| A key's `read` is the UTC time it was read | `value`, text or `null` when absent; `source`, or `null` when absent; `read`, the UTC time it was read |
| A source has a `name` and a `precedence` | `name`, and `precedence`: `before azimuth`, `azimuth`, or `after azimuth`. |
| `before azimuth` says that Azimuth's value would win | These say that Azimuth's value would win, that this is its applied value, or that this source wins over it |
| `azimuth` says that this is Azimuth's applied value | These say that Azimuth's value would win, that this is its applied value, or that this source wins over it |
| `after azimuth` says that this source wins over Azimuth's value | These say that Azimuth's value would win, that this is its applied value, or that this source wins over it |
| An absent value or source is reported as absent | An absent value or source is reported as absent. |
| The library reports its mappings, not what it thinks the declaration ought to hold | The library reports its mappings, not what it thinks the declaration ought to hold. |

## Cases of the conformance kit for the report of a component

| Behavior | Quote from the design |
| --- | --- |
| A mapping reads a value through the callback, for read | It proves explicit mappings for read and apply, applying possible and not possible, an absent value, an unstated key keeping its source, a later overriding source, and a changed value read again. |
| A mapping applies a value, for apply | It proves explicit mappings for read and apply, applying possible and not possible, an absent value, an unstated key keeping its source, a later overriding source, and a changed value read again. |
| The callback gives a report when applying is possible | It proves explicit mappings for read and apply, applying possible and not possible, an absent value, an unstated key keeping its source, a later overriding source, and a changed value read again. |
| The callback gives a report when applying is not possible | It proves explicit mappings for read and apply, applying possible and not possible, an absent value, an unstated key keeping its source, a later overriding source, and a changed value read again. |
| An absent value is reported as absent | It proves explicit mappings for read and apply, applying possible and not possible, an absent value, an unstated key keeping its source, a later overriding source, and a changed value read again. |
| A key the launch does not state keeps its source | It proves explicit mappings for read and apply, applying possible and not possible, an absent value, an unstated key keeping its source, a later overriding source, and a changed value read again. |
| A source added later that gives another value is reported with precedence `after azimuth` | The report must give `false` from that named source with precedence `after azimuth`. |
| A value that changed is read again at the next call | It proves explicit mappings for read and apply, applying possible and not possible, an absent value, an unstated key keeping its source, a later overriding source, and a changed value read again. |
| The component name and the report format are checked | It checks the component name and the report format, then exercises the same document through the fixture's declared path. |
| The document of the callback is the same document through the fixture's declared path | It checks the component name and the report format, then exercises the same document through the fixture's declared path. |
| A team's own source passes these cases too | A team's custom source runs these cases too. |
