# Decision: the library reads and applies through a source

Date: 2026-11-23. Decided by: the maintainer.

The maintainer asked: "Would this whole design work for systems with centralized settings
management in production? With settings stored in DB, etcd, or something", then "Shouldn't
Azimuth go with OpenFeature as well?", and about reading every key through OpenFeature: "Isn't
this too big a constraint?". The maintainer then said: "I think we should have a library and
any team that wants to support OpenFeature would need to intergate it with the library inside
their codebase." Asked whether the library should offer an interface for a source of values,
with the stack's configuration as the one implementation it ships, the maintainer said: "Yes"

Claude's words, which the maintainer agreed to: the library of a stack reads and applies values
through a source. A source is an interface with two operations. Read a key gives its current
value and where the value came from. Apply a value to a key sets it in this process, and the
library calls it only when the component's code has turned applying on. The library ships one
source, over the stack's own configuration, and uses it for every key that no other source
serves. A team that reads values from a flag service, such as one behind OpenFeature, from a
table of its database, or from a store such as etcd writes its own source in its codebase and
registers it for the keys it serves. Azimuth ships no integration with a flag service. The
source that the library ships is added last among the configuration's sources, so a value it
applies wins, and a later source that still wins shows in the report as an override. A value
that can change while the component runs is read again each time the report is asked for, and
the report says when it was read.

Rejected: OpenFeature as the only way to read a key, because every place that reads a flag
would have to change before a team's first Check with a context. A source for OpenFeature
shipped with the library, because Azimuth would then follow a standard's libraries in every
language for teams that may never use them.

The cost: a team's own source can report a value that the code does not get, and only the
review of that code guards against it. A key that the code reads under another name is caught
by a team's source only if the team writes it to report that.

Example: `api` reads `payouts.instant` from a flag service. Its team writes a source whose read
asks the service's client for the flag, and whose apply puts the client into its test mode with
the given value, and registers it for `payouts.instant`. The key `labs` stays on the source
over the configuration.
