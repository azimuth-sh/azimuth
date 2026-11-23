# Decision: one library per stack applies and reports the context

Date: 2026-11-23. Decided by: the maintainer.

Asked whether a translator of keys should be a data rule or a command, the maintainer asked:
"Isn't this a small lib for each stack?". About reporting and production, the maintainer said:
"I believe a library wouldn't open a port for reporting, but provide some callback for this?
And a library should somehow disable context setting in production?", that it is "Just
protection from a bug", and asked: "What if a team uses custom environment names? Shouldn't we
make this explicit?". Asked whether applying should be turned on only by an explicit call in
the component's own code, with the report and the server as the catch for a wrong condition,
the maintainer said: "Yes"

Claude's words, which the maintainer agreed to: one library per stack lives inside the
component, and it both applies a test run's context and reports the values in force. The
adapter passes the whole context in one variable, `AZIMUTH_CONTEXT`, and knows no stack. At
start, the library sets each key through the stack's own configuration by its real path, so no
rule of separators is needed. The library applies a context only when the component's own code
turns applying on, with a condition the team chooses, when it registers the library. It never
guesses from the environment's name. The library opens no port. It offers a callback that
returns the report, and the component exposes it through a door it already has, under that
door's rules. The component's declaration says where its report is exposed, for a reader that
cannot call the callback in its own process. The report says, for each key, whether its value
came from the component's configuration or was applied by Azimuth. The server lets no result
apply on a deployment whose report shows an applied value, and reports it as a finding.

Rejected: a rule of separators or a command outside the component, because the component's own
configuration already knows its paths. Guessing production from the environment's name,
because teams name environments their own way, and a production build of a website runs with
`NODE_ENV=production` in its tests too. Applying code that exists only in a test build, because
a suite of the whole stack should run the build that is deployed. A second variable that turns
applying on, because a leftover setup leaves both behind.

The cost: every component under a context carries the library, also in production, and a line
in its start-up code. A component whose settings are read before the library starts needs an
earlier hook of its stack. A wrong condition of applying is caught only by the report, after
the fact.

Example: the component `api` registers the library in `Program.cs` with applying turned on when
its own setting `Testing:Enabled` is `true`, and maps the report to its internal route
`/internal/context`. A test run passes `AZIMUTH_CONTEXT` with `Payouts:Instant=false`. The
library sets it, and the report shows `false`, applied by Azimuth. In production
`Testing:Enabled` is absent, the variable is ignored, and the report shows the configured value.
