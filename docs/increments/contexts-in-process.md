# Execution contexts, contexts in process: behaviors

A row is of one of two kinds. A row of behavior is one observable fact, and a test shows it. A row of prose says what a text tells its reader; its section begins with the words "Rows of prose", and no test reads the words of the text for it. The second cell is copied exactly from `docs/design.md`. A plain test fails when a quoted sentence is no longer in that file. A behavior with no sentence to quote is not built. This list is built after the list of launches on the sample (`contexts-launches.md`) and the list of launch records (`contexts-launch-records.md`), because it adds the reads of a host inside the test process to the launches and the collected reads those lists build. The code that reads in a fixture is the team's own code, and no Azimuth helper package for it is listed here.

Status of this list: a draft, not yet read by a person.

## The reads of a host inside the test process

| Behavior | Quote from the design |
| --- | --- |
| A host inside the test process gives a read with the phase `built` and a read with the phase `disposing` | The phases are `start` and `stop` for a component in its own process, and `built` and `disposing` for a host inside the test process. |
| The reads of a host reach the report's `components` under the name of its component | After the runner ends, the adapter puts every read it finds into the report's `components`. |
| A read holds either a `report` or an `error` | Each read has `phase` and either `report`, the document as read, or `error`, the cause it could not be read. |
| Several hosts of one component in one suite give several pairs of `built` and `disposing` reads | Several hosts may give several pairs of `built` and `disposing` reads for one component. |
| Every host built under an expected component contributes two reads | Every host built under an expected component contributes two reads, including several hosts in one suite. |
| Two hosts of `api` in one suite contribute four reads | Every host built under an expected component contributes two reads, including several hosts in one suite. |

## The folder for reads of the xUnit adapter

| Behavior | Quote from the design |
| --- | --- |
| The adapter gives the code of the tests a folder for reads | The xUnit adapter gives the code of the tests a folder for reads. |
| For each launch the adapter creates a folder that is empty | For each launch it creates an empty folder, names it in `AZIMUTH_READS` |
| The folder of a launch is named in `AZIMUTH_READS` | For each launch it creates an empty folder, names it in `AZIMUTH_READS` |
| Two launches of one suite get different folders | For each launch it creates an empty folder, names it in `AZIMUTH_READS` |
| The adapter writes `components.json` in the folder before the tests run | writes there a file `components.json` that lists the request's components with their names and report paths |
| `components.json` lists each component of the request with its name and report path | writes there a file `components.json` that lists the request's components with their names and report paths |
| A read is one JSON file in the folder with `name`, `phase` and `report` | The code writes each read as its own JSON file in that folder, with `name`, `phase` and either `report` or `error`. |
| A read is one JSON file in the folder with `name`, `phase` and `error` | The code writes each read as its own JSON file in that folder, with `name`, `phase` and either `report` or `error`. |
| Every read file the adapter finds after the runner ends is in the report | After the runner ends, the adapter puts every read it finds into the report's `components`. |
| A launch run without the adapter's folder has no `AZIMUTH_READS`, and the code reads nothing | Without `AZIMUTH_READS` the code reads nothing. |

## A separate process that the code of the tests starts

| Behavior | Quote from the design |
| --- | --- |
| The code of the tests that starts a separate process reads its report after it is up and before it stops | right after it is up and right before it stops, at its report path, with the phases `start` and `stop` |
| Its reads have the phases `start` and `stop` | right after it is up and right before it stops, at its report path, with the phases `start` and `stop` |
| Both of its reads go to the adapter with the results | and both reads go to the adapter with the results |
| The `website` that `website-tests` starts as a separate process has a `start` and a `stop` read in the report | and both reads go to the adapter with the results |
| The adapter does not look at the address of such a process before the start | No adapter looks at its address before the start, because no runner configuration names it |
| A port already taken fails the start of that code | a port already taken fails that code's own start |
| The xUnit adapter reports no "something else answers there" for a host inside the test process | A host inside the test process has no address. |

## The comparison with the condition

| Behavior | Quote from the design |
| --- | --- |
| The `built` read of a host must show the stated value applied by Azimuth | This must hold in both reads, for a separate process or a host inside the test process. |
| The `disposing` read of a host must show the stated value applied by Azimuth | This must hold in both reads, for a separate process or a host inside the test process. |
| A launch of `payouts.instant=true` with an `api` host whose `built` and `disposing` reads both show `true` applied by Azimuth runs the Check `payout-is-sent-at-once` | Every such report must meet the stated condition. |
| A launch where only one host of two reports a wrong value leaves every Check of that launch owed | One wrong report leaves every Check of that launch owed, as for a separate process. |
| A launch where the `built` read of a host is wrong leaves every Check of that launch owed | One wrong report leaves every Check of that launch owed, as for a separate process. |
| A launch where the `disposing` read of a host is wrong leaves every Check of that launch owed | One wrong report leaves every Check of that launch owed, as for a separate process. |
| The state of those Checks is `owed` and the reason is `context not applied` | The state is `owed`, the reason is `context not applied` |
| The detail of such a result names the component, key, stated and reported values, and cause | the detail names the component, key, stated and reported values, and cause |
| A host that gives a `built` read and no `disposing` read gives the cause of a report missing or unreadable | a report missing or unreadable |
| A read of a host with an `error` instead of a report gives the cause of a report missing or unreadable, naming its declared place | Missing reports name their declared place. |
| A launch with no condition needs no reads of a host | Only components needed by a nonempty condition are required to have reads. |
| A malformed read of a host is a component read error, not a new outcome of a test | A malformed component report is a component read error, not a new outcome of a test. |
| The collected reads of a host appear in `run-1` and in each Check result of the launch | The same launch's reports appear in each of its Check results. |
