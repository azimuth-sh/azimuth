# Decision: a component reports its name

Date: 2026-11-24. Decided by: the maintainer.

About a wrong mark in a deployment's configuration, which would put a report under the wrong
component, the maintainer asked: "Shouldn't a component report its name to help preventing and
diagnosing such problems?"

Claude's words, which the maintainer agreed to: a component names itself when it registers the
library, with the name that `project.toml` declares, and its report carries that name beside the
format version and the context, in a launch and in a deployment alike. Whoever reads a report
compares that name with the component expected where it read: in a launch, the one that the
suite's runner configuration gives at that address; in a deployment, the one that the
environment's mark gives. When they differ, or the name is one that `project.toml` does not
declare, a launch gives "context not applied", with the cause naming the component expected and
the one that answered. In a deployment the observation carries both names, the server shows a
finding of that instance, and no result that depends on a key of either component applies there.

This replaces what the note `a-launch-owns-what-it-reads-and-the-report-names-nothing` says about
the report holding no name of its component, and its reason for rejecting the name. The report
still holds no identity of its instance and no mark of a launch.

Rejected: finding a wrong mark or a wrong address only through a stated key that one report lacks
or another holds unexpectedly, because two components that map the same keys would then be
confused with no sign.

The cost: the name is written twice, in `project.toml` and in the registration, and lint does not
see the registration. A renamed component changes both in one change.

Example: the deployment's configuration marks the website's workload `api` by mistake, and both
components map `payouts.instant`. The reconciler reads the Pod `website-5c8d-q7mn` and finds
`"component": "website"` in its report where the mark says `api`. The server shows the finding
"Pod website-5c8d-q7mn: marked api, reports website".
