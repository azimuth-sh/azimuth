# Map: execution contexts

The cut of the exploration of execution contexts. This repository is built without Azimuth, so
each row becomes a list of behaviors in `docs/increments/` or an edit of `docs/design.md`, not a
change folder. The map is edited freely and nobody approves it.

Asked which real work should be the first to use execution contexts, the maintainer said: "The
admin pages change?". About the two libraries it needs, the maintainer said: "And we need to
publish them", and about the order: "We should postpont the current increment and build
execution contexts before it. Right now". So the rows up to `first-real-use` come before
increment 3, whose list was accepted on 2026-11-21 and waits on the branch `increment-3`. The
rows `kubernetes-observation` and `status-from-observations` are built with the server rung.

## Rows

| Row | Takes from the sources | After |
| --- | --- | --- |
| design-of-execution-contexts | `request.md` and every note in `decisions/`, read in date order with each "This replaces" applied. The edit of `docs/design.md` that states execution contexts. It replaces the sentences on "A result depends on every declared value of its component", "There is no list per Check", "All marked tests of one Check" reporting the same values, variants and `production-next`, "The adapter sees tests only" and "The environment is passed on unchanged". It states the exact form of `Context:`, the tables of `execution-context.toml` and the new keys of `project.toml`, which findings are reading errors and which are lint, where "context not applied" stands among the failed and incomplete results and in the order of the Check rule, how the output of `run` shows two launches of one suite, the report of a component as a new contract in "Stack support", and the two libraries in the release path. What no note decides for deployments is written as an open question of the design. It says where increment 3 now stands | |
| conditions-in-the-model | the design as `design-of-execution-contexts` leaves it. A list of behaviors: the reader accepts `Context:` on Requirements and Scenarios, `execution-context.toml` declares keys, the reading errors and lint findings of the design, a change to `execution-context.toml` is class D in `azimuth change diff`, and both commands that run Checks leave a Check with a condition owed until a launch can apply it, also in a suite that holds Checks without one | design-of-execution-contexts |
| started-components-in-the-model | the design as `design-of-execution-contexts` leaves it. A list of behaviors: `project.toml` declares components with their areas and report path, and each suite's `starts`, and lint names a Check whose Scenario states a key that no component its suite starts is listed for | conditions-in-the-model |
| context-library-for-dotnet | the design as `design-of-execution-contexts` leaves it. A list of behaviors for the library of .NET: registration with the component's name, the condition under which applying is possible and the mapping of each key, the source over configuration and the sources of a team, and the callback that gives the report | design-of-execution-contexts |
| context-library-for-node | the design as `design-of-execution-contexts` leaves it. The same list for the library of Node | design-of-execution-contexts |
| libraries-in-the-release | the design as `design-of-execution-contexts` leaves it. Both libraries packed with the release's version under their public names and sent to NuGet and npm by the release path that sends the adapters, with the first publication at npm, retries for each package, and the rules for tags and staging | context-library-for-dotnet, context-library-for-node |
| launches-on-the-sample | the design as `design-of-execution-contexts` leaves it. A list of behaviors: the core groups the requested Checks by condition and makes one launch per group, each with its own facility proof and time limit, and passes `AZIMUTH_CONTEXT`; a new number of the adapter contract that every adapter speaks; the core compares the reports and gives "context not applied" with its causes in the output of `run`. Proven on Azimuth's own sample suite with two groups and a test marked for Checks of both | started-components-in-the-model |
| launches-through-playwright | the design as `design-of-execution-contexts` leaves it. A list of behaviors: the Playwright adapter takes each started component's address from the runner's configuration, looks before the start at each address known then, and reads each report after the start and before the stop | launches-on-the-sample |
| first-real-use | the design as `design-of-execution-contexts` leaves it. The first real project that needs it proves, through its real website and its real API, one Check with a flag off and one with the flag on, each result with the reports of both components showing the value applied by Azimuth, using the published libraries and adapter | launches-through-playwright, libraries-in-the-release |
| contexts-in-process | the design as `design-of-execution-contexts` leaves it. For xUnit: the code that builds a host inside the test process gives its report to the adapter with the results | launches-on-the-sample, context-library-for-dotnet |
| kubernetes-observation | the design as `design-of-execution-contexts` leaves it. The reconciler for Kubernetes: the mark on a workload, the address of each ready instance, the report path, a set of ready instances that stayed the same, and the observation it sends. It is built with the server rung, which has no row here | design-of-execution-contexts |
| status-from-observations | the design as `design-of-execution-contexts` leaves it. On the server, from an observation: a result applies only where its Scenario's condition holds, "Not applicable here", and the findings of an absent key, an applied value, a wrong name, an instance that cannot be read, and components or instances that disagree. It is built with the server rung, which has no row here | design-of-execution-contexts |

## Not cut yet

- A run that sets unstated keys to another deployment's values, to find a Check that depends on a key its Scenario does not state, left for Challenges by the note `variants-are-dropped`. Which observation it takes, what counts as a finding, and when it is owed again are not decided.
