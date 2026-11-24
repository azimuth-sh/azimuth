# Decision: a suite names the components it starts

Date: 2026-11-24. Decided by: the maintainer.

About `starts = ["api", "website"]` on a suite, the maintainer asked: "What is this for?". Shown
the whole project file, and asked whether the field should be named for what it means, such as
`runs_against`, the maintainer said: "Keep starts"

Claude's words, which the maintainer agreed to: each suite in `project.toml` names, under
`starts`, the real components its tests run against, whether a component runs in a process of
its own or in a host built inside the test process. A component replaced by a stand-in is not
named. The core expects a report from each named component that the launch's condition needs,
whatever the runner's own configuration gives. So a component that the runner's configuration
forgets gives "context not applied", and is not left out unseen. Lint reads `starts` to name,
before any run, a Check whose Scenario states a key that no component of its suite is listed
for. A change to `starts` is a change to `project.toml`, so it is class D, and a person sees when
a suite begins to run against a stand-in.

Rejected: learning the components of a suite from the adapter's report, because a component that
the runner's configuration forgets would then be invisible. A name such as `runs_against`,
because `starts` is plain enough once the design says that a host built inside the test process
counts.

The cost: the names are written twice, in `starts` and in the runner's configuration, and a
difference shows only in a launch, as a report missing or a report nobody expected.

Example: the suite `website-tests` declares `starts = ["website"]` and runs against a stand-in
for `api`. A change replaces the stand-in with the real API and adds `api` to `starts`. A person
reviews the change, and from then on its launches expect the report of `api` as well.
