# Map: execution contexts

The cut of the exploration of execution contexts. This repository is built without Azimuth, so
each row becomes a list of behaviors in `docs/increments/` or an edit of `docs/design.md`, not a
change folder. The map is edited freely and nobody approves it.

Asked which real work should be the first to use execution contexts, the maintainer said: "The
admin pages change?". About the two libraries it needs, the maintainer said: "And we need to
publish them", and about the order: "We should postpont the current increment and build
execution contexts before it. Right now". So the rows up to `first-real-use` come before
increment 3, whose list was accepted on 2026-11-21 and waits on the branch `increment-3`. The
rows `contexts-in-deployments` and `check-that-depends-on-an-unstated-key` do not.

## Rows

| Row | Takes from the sources | After |
| --- | --- | --- |
| design-of-execution-contexts | `request.md` and every note in `decisions/`, read in date order with each "This replaces" applied. The edit of `docs/design.md` that states execution contexts. It replaces the sentences on "A result depends on every declared value of its component", "There is no list per Check", "All marked tests of one Check" reporting the same values, variants and `production-next`, "The adapter sees tests only" and "The environment is passed on unchanged". It states the exact form of `Context:`, the tables of `execution-context.toml` and the new keys of `project.toml`, which findings are reading errors and which are lint, where "context not applied" stands among the failed and incomplete results and in the order of the Check rule, how the output of `run` shows two launches of one suite, the report of a component as a new contract in "Stack support", the two libraries in the release path, and the run that is left for Challenges. It says where increment 3 now stands | |
| contexts-in-the-model | the design as `design-of-execution-contexts` leaves it. A list of behaviors: the reader accepts `Context:` on Requirements and Scenarios, `execution-context.toml` declares keys, `project.toml` declares components with their areas and report path and each suite's `starts`, lint reports the findings of the design, and `run` gives no result for a Check whose condition it cannot yet apply | design-of-execution-contexts |
| context-library-for-dotnet | the design as `design-of-execution-contexts` leaves it. A list of behaviors for the library of .NET: registration with the component's name, the condition under which applying is possible and the mapping of each key, the source over configuration and the sources of a team, and the callback that gives the report | design-of-execution-contexts |
| context-library-for-node | the design as `design-of-execution-contexts` leaves it. The same list for the library of Node | design-of-execution-contexts |
| libraries-in-the-release | the design as `design-of-execution-contexts` leaves it. Both libraries packed with the release's version and sent to NuGet and npm by the release path that sends the adapters, with its rules for tags, staging and retries | context-library-for-dotnet, context-library-for-node |
| launches-per-condition | the design as `design-of-execution-contexts` leaves it. A list of behaviors: the core groups the requested Checks by condition and makes one launch per group, passes `AZIMUTH_CONTEXT`, and looks before the start at each address known then; a new number of the adapter contract, with every adapter updated; the Playwright adapter takes each started component's address from the runner's configuration and reads its report after the start and before the stop; the core compares and gives "context not applied" with its causes. Proven on Azimuth's own sample suite | contexts-in-the-model |
| first-real-use | the design as `design-of-execution-contexts` leaves it. The first real project that needs it proves, through a whole stack of a website and an API, a Check with a flag off and one with the flag on, with the published libraries and adapter | launches-per-condition, libraries-in-the-release |
| contexts-in-process | the design as `design-of-execution-contexts` leaves it. For xUnit: the code that builds a host inside the test process calls its callback and gives the report to the adapter with the results, naming the tests it served | launches-per-condition, context-library-for-dotnet |
| contexts-in-deployments | the design as `design-of-execution-contexts` leaves it. The reconciler for Kubernetes and the observation it sends, and on the server: a result applies only where its Scenario's condition holds, "Not applicable here", and the findings of an absent key, an applied value, a wrong name, an instance that cannot be read, and components or instances that disagree. It is built with the server rung, which has no row here | design-of-execution-contexts |
| check-that-depends-on-an-unstated-key | the design as `design-of-execution-contexts` leaves it, and the note `variants-are-dropped`. A run that sets unstated keys to another deployment's values to find a Check that depends on a key its Scenario does not state. It is built with Challenges, which have no row here | design-of-execution-contexts |
