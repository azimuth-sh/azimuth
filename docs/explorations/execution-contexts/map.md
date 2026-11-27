# Map: execution contexts

The cut of the exploration of execution contexts. This repository is built without Azimuth, so
each row becomes a list of behaviors in `docs/increments/` or an edit of `docs/design.md`, not a
change folder. The map is edited freely and nobody approves it.

Asked which real work should be the first to use execution contexts, the maintainer said: "The
admin pages change?". About the two libraries it needs, the maintainer said: "And we need to
publish them", and about the order: "We should postpont the current increment and build
execution contexts before it. Right now". So the rows up to `first-real-use` come before
increment 3, whose list was accepted on 2026-11-21 and waits on the branch `increment-3`. Asked
where `contexts-in-process` stands, the maintainer said: "Before increment 3, built beside
first-real-use", so it comes before increment 3 too and is not a row `first-real-use` waits
for. This changed on 2026-11-25. The first real project's browser Checks run in an xUnit fixture
that hosts the API inside the test process and starts the website as a separate process. A spike
showed that both work, and the maintainer agreed to cut the map to it: "Yes, after the spike". So
`first-real-use` waits for `contexts-in-process`, and `launches-through-playwright` is on hold.
This changed again on 2026-11-27. The first real project's page Checks run a website served alone,
with a stand-in where the API would be, and their launches need the Playwright adapter's reads. So
`launches-through-playwright` comes next, before increment 3.
The rows `kubernetes-observation` and `status-from-observations` are built with the server
rung.

## Rows

| Row | Takes from the sources | After |
| --- | --- | --- |
| design-of-execution-contexts | `request.md` and every note in `decisions/`, read in date order with each "This replaces" applied. The edit of `docs/design.md` that states execution contexts. It replaces the sentences on "A result depends on every declared value of its component", "There is no list per Check", "All marked tests of one Check" reporting the same values, variants and `production-next`, "The adapter sees tests only" and "The environment is passed on unchanged". It states the exact form of `Context:`, the tables of `execution-context.toml` and the new keys of `project.toml`, which findings are reading errors and which are lint, where "context not applied" stands among the failed and incomplete results and in the order of the Check rule, how the output of `run` shows two launches of one suite, the report of a component as a new contract in "Stack support", and the two libraries in the release path. What no note decides for deployments is written as an open question of the design. It says where increment 3 now stands | |
| keys-in-the-model | the design as `design-of-execution-contexts` leaves it. The list `docs/increments/contexts-keys.md`: `execution-context.toml` declares keys with their types, values and components, its reading errors and the lint findings about keys alone, `azimuth init` writes none, and a change to it is class D in `azimuth change diff` | design-of-execution-contexts |
| conditions-in-the-model | the design as `design-of-execution-contexts` leaves it. The list `docs/increments/contexts-conditions.md`: the reader accepts `Context:` on Requirements and Scenarios, the effective condition, typed values and their normalization, the reading errors and lint findings of conditions, the condition in the claim digest, and both commands that run Checks leave a Check with a condition owed until a launch can apply it, also in a suite that holds Checks without one. Its build ends the interim sentence of the design that a `Context:` line is a reading error, and the row of `docs/increments/1.md` that quotes it | keys-in-the-model |
| started-components-in-the-model | the design as `design-of-execution-contexts` leaves it. A list of behaviors: `project.toml` declares components with their areas and report path, and each suite's `starts`, and lint names a Check whose Scenario states a key that no component its suite starts is listed for | conditions-in-the-model |
| context-library-for-dotnet | the design as `design-of-execution-contexts` leaves it. A list of behaviors for the library of .NET: registration with the component's name, the condition under which applying is possible and the mapping of each key, the source over configuration and the sources of a team, and the callback that gives the report | design-of-execution-contexts |
| context-library-for-node | the design as `design-of-execution-contexts` leaves it. The same list for the library of Node | design-of-execution-contexts |
| libraries-in-the-release | the design as `design-of-execution-contexts` leaves it. Both libraries packed with the release's version under their public names and sent to NuGet and npm by the release path that sends the adapters, with the first publication at npm, retries for each package, and the rules for tags and staging | context-library-for-dotnet, context-library-for-node |
| launches-on-the-sample | the design as `design-of-execution-contexts` leaves it. The list `docs/increments/contexts-launches.md`: the core groups the requested Checks by condition and makes one launch per group, each with its own facility proof and time limit, and passes `AZIMUTH_CONTEXT`; the fields that the request and the report of the adapter contract gain, with their numbers kept; the core compares the reports and gives "context not applied" with its causes; every launch of a suite runs. Proven on Azimuth's own sample suite with two groups and a test marked for Checks of both | started-components-in-the-model |
| launch-records | the design as `design-of-execution-contexts` leaves it. The list `docs/increments/contexts-launch-records.md`: `run-1` shows one entry per launch with its condition, Checks and components, the text of a run has one line per launch, and each result keeps its launch's condition and the reports read in `result-1`, `check-1` and `azimuth check show`, never in the fingerprint | launches-on-the-sample |
| launches-through-playwright | On hold from 2026-11-25 to 2026-11-27; it comes next, before increment 3. The design as `design-of-execution-contexts` leaves it. A list of behaviors: the Playwright adapter takes each started component's address from the runner's configuration, looks before the start at each address known then, and reads each report after the start and before the stop | launches-on-the-sample |
| first-real-use | the design as `design-of-execution-contexts` leaves it. The first real project that needs it proves, through its real website and its real API, in a fixture of its tests that hosts the API and starts the website, one Check with a flag off and one with the flag on, each result with the reports of both components showing the value applied by Azimuth, using the published libraries and adapter | contexts-in-process, libraries-in-the-release, launch-records |
| contexts-in-process | the design as `design-of-execution-contexts` leaves it, with a sentence on who reads a separate process that a fixture of the tests starts. A list of behaviors for xUnit: the code that builds a host inside the test process gives its report to the adapter with the results, and the code that starts a separate process for the tests, such as a website, reads its report after the start and before the stop and gives both to the adapter | launches-on-the-sample, context-library-for-dotnet |
| kubernetes-observation | the design as `design-of-execution-contexts` leaves it. The reconciler for Kubernetes: the mark on a workload, the address of each ready instance, the report path, a set of ready instances that stayed the same, and the observation it sends. It is built with the server rung, which has no row here | design-of-execution-contexts |
| status-from-observations | the design as `design-of-execution-contexts` leaves it. On the server, from an observation: a result applies only where its Scenario's condition holds, "Not applicable here", and the findings of an absent key, an applied value, a wrong name, an instance that cannot be read, and components or instances that disagree. It is built with the server rung, which has no row here | design-of-execution-contexts |

## Not cut yet

- A run that sets unstated keys to another deployment's values, to find a Check that depends on a key its Scenario does not state, left for Challenges by the note `variants-are-dropped`. Which observation it takes, what counts as a finding, and when it is owed again are not decided.
