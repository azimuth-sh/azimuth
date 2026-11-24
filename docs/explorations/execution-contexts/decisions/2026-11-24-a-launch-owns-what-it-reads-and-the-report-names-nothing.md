# Decision: a launch owns what it reads, and the report names nothing

Date: 2026-11-24. Decided by: the maintainer.

About an id in the report of a component, the maintainer asked: "Does a component need to report
its id? Shouldn't it be just an evironment id like k8s pod name?". About an id of the launch that
the report would return, the maintainer said: "I don't think we should have a value present for
testing and absent in production. Maybe we should track launches from somewhere outside?". About
an adapter that looks at a component's address before the stack starts, the maintainer asked:
"How would it know this address if no component is yet started? A component can take any port it
wants". Asked whether an address known before the start must not answer before it, an address
learned after the start needs no look, and a wrong address in a suite's configuration stays the
suite's own bug, the maintainer said: "Yes"

Claude's words, which the maintainer agreed to: the report of a component holds no id and no
name: no name of its component, no identity of its instance, and no mark of a launch. It is the
same document in a launch and in a deployment. Whoever reads a report knows which component and
which instance it comes from by where it read. In a launch, the suite's runner configuration
gives the address of each component in `starts`. In a deployment, the environment gives the
instances of each component, such as the name and the UID of a Pod. A launch owns the processes
it reads. An address that the suite's configuration gives before the stack starts must not answer
before the launch starts the stack. If it answers, the launch gives "context not applied", with
the cause that something else answers there, and runs no test. An address that the suite learns
only after the start, such as a free port chosen then, needs no look, because no other process
could hold it. A host built inside the test process is owned by the test code that builds it.

Rejected: the name of the component in the report, because the reader already knows which
component it reads, and a mix-up of two addresses in one launch shows as a stated key that one
report lacks or another holds unexpectedly. An id of the launch returned in the report, because
the report would carry a value that only tests fill. An id that each process makes at its start,
because the environment's identity of an instance already changes with a restart of its
container, and it would be a second identity to keep.

The cost: a wrong address in a suite's configuration can make a launch read the report of a
process it did not start, with values that may match, and that is the suite's own bug. A process
that a supervisor restarts inside one container keeps the container's identity, so a restart
between two reads is not seen. A stack started outside the launch cannot serve Checks with a
condition.

Example: the suite `full-stack` gives `website` at `http://localhost:4000` in its runner
configuration, and `api` at a free port that its host chooses at the start. A launch in another
worktree already serves `localhost:4000`. Before the start, the adapter finds an answer there, so
the launch gives "context not applied" with that cause and runs no test. In production the
reconciler reads the Pod `api-7d9f-xk2p` and records its report under `api` and that Pod.
