# Decision: reports are collected outside the library, and the library knows no testing

Date: 2026-11-23. Decided by: the maintainer.

Asked who should read the reports of the components during a launch, the maintainer said: "An
adapter right after setting the context?". About components that would write their reports into
a folder named by the adapter, the maintainer asked: "How would a component know it needs so
write something in a folder? With the library inspecting an env var?", and then said: "I
believe it should not. It is a testing concern moving inside a production library. Or does the
library already know about testing?". Asked whether collecting should stay on the side of the
tests, the maintainer said: "Yes. And we should move the testing vocabulary from the library.
Its client should just specify a condition when applying is possible"

Claude's words, which the maintainer agreed to: the library inside a component reports through
its callback and applies values, and does nothing else for a launch. Its interface speaks of
keys, sources, values, applying and reports, and never of tests. The component registers it
with a condition under which applying is possible, and the library applies only while that
condition holds. In a launch, the reports are collected outside the component. For a host that
runs inside the test process, the code that builds the host calls its callback right after
building it and gives the report to the adapter with the results. For a component in a process
of its own, the adapter reads its report from the place its declaration names, the same place
that the server reads in a deployment, right after the component is up and again just before it
stops. A suite names the components it starts, so the core knows which reports to expect. A
difference between the two reports of one component is the case "context not applied", as a
changed value during the launch.

This replaces what the note `one-library-per-stack-applies-and-reports-the-context` says about
"a test run's context" and about the team choosing a condition "when it registers the library":
the condition is all the library is told.

Rejected: components that write their reports into a folder named by the adapter, because
writing for a launch would be a second concern of testing inside a library that runs in
production. A report read only once, at the start of a launch, because a host built later, or a
value changed during the launch, would be missed.

The cost: in a launch, the place where a component's report is read has to be reachable by the
adapter, and each suite wires the way in. A component that crashes before its stop report
leaves the second read empty, and the reason names that.

Example: `api` registers the library with the condition that its own setting `Demo:Open` is
`true`. A suite of the whole stack starts `api` with that setting and passes
`payouts.instant=false`. The adapter reads `/internal/context` of `api` once `api` is up and
finds `false`, applied by Azimuth, then runs the tests and reads the report again before the
stack stops.
