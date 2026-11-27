# Execution contexts, contexts in process: behaviors

A row is of one of two kinds. A row of behavior is one observable fact, and a test shows it. A row of prose says what a text tells its reader; its section begins with the words "Rows of prose", and no test reads the words of the text for it. The second cell is copied exactly from `docs/design.md`. A plain test fails when a quoted sentence is no longer in that file. A behavior with no sentence to quote is not built. This list is built after the list of launches on the sample (`contexts-launches.md`) and the list of launch records (`contexts-launch-records.md`), because it adds the reads of a host inside the test process to the launches and the collected reads those lists build. The code that reads in a fixture is the team's own code, and no Azimuth helper package for it is listed here.

Status of this list: read and accepted by a person on 2026-11-25.

## The reads of a host inside the test process

| Behavior | Quote from the design |
| --- | --- |
| The core accepts reads with the phases `built` and `disposing` as the reads of a host | The phases are `start` and `stop` for a component in its own process, and `built` and `disposing` for a host inside the test process. |
| The reads of a host reach the report's `components` under the name of its component | After the runner ends, the adapter puts every read it finds into the report's `components`. |
| A read holds either a `report` or an `error` | Each read has `phase` and either `report`, the document as read, or `error`, the cause it could not be read. |
| The report carries several pairs of `built` and `disposing` reads for one component | Several hosts may give several pairs of `built` and `disposing` reads for one component. |

## The folder for reads of the xUnit adapter

| Behavior | Quote from the design |
| --- | --- |
| The adapter gives the code of the tests a folder for reads | The xUnit adapter gives the code of the tests a folder for reads. |
| For each launch the adapter creates a folder that is empty | For each launch it creates an empty folder, names it in `AZIMUTH_READS` |
| The folder of a launch is named in `AZIMUTH_READS` | For each launch it creates an empty folder, names it in `AZIMUTH_READS` |
| Two launches of one suite get different folders | For each launch it creates an empty folder, names it in `AZIMUTH_READS` |
| The adapter writes `components.json` in the folder before the tests run | writes there a file `components.json` that lists the request's components with their names and report paths |
| `components.json` lists every component of the request with its name and report path | writes there a file `components.json` that lists the request's components with their names and report paths |
| The adapter takes a read file with `name`, `phase` and `report` into the report | The code writes each read as its own JSON file in that folder, with `name`, `phase` and either `report` or `error`. |
| The adapter takes a read file with `name`, `phase` and `error` into the report | The code writes each read as its own JSON file in that folder, with `name`, `phase` and either `report` or `error`. |
| Every read file the adapter finds after the runner ends is in the report | After the runner ends, the adapter puts every read it finds into the report's `components`. |
| A read whose `name` is no component of the request is kept in the report | After the runner ends, the adapter puts every read it finds into the report's `components`. |
| A read file of an earlier launch is not read by a later launch | For each launch it creates an empty folder, names it in `AZIMUTH_READS` |
| A file in the folder that is not JSON makes the launch `crashed` | A file in the folder that is not a read, because it is not JSON or has no `name` or `phase`, makes the launch `crashed`, and the adapter names the file. |
| A file in the folder with no `name` makes the launch `crashed` | A file in the folder that is not a read, because it is not JSON or has no `name` or `phase`, makes the launch `crashed`, and the adapter names the file. |
| A file in the folder with no `phase` makes the launch `crashed` | A file in the folder that is not a read, because it is not JSON or has no `name` or `phase`, makes the launch `crashed`, and the adapter names the file. |
| The adapter names the file that is not a read | A file in the folder that is not a read, because it is not JSON or has no `name` or `phase`, makes the launch `crashed`, and the adapter names the file. |

## A separate process that the code of the tests starts

| Behavior | Quote from the design |
| --- | --- |
| A `start` and a `stop` read that the code of the tests writes in the folder for a separate process reach the report under its component's name | After the runner ends, the adapter puts every read it finds into the report's `components`. |
| The adapter does not look at the address of a process that the code of the tests starts | No adapter looks at its address before the start, because no runner configuration names it |
| The xUnit adapter reports no "something else answers there" for a host inside the test process | A host inside the test process has no address. |

## How the core tells a host from a separate process

| Behavior | Quote from the design |
| --- | --- |
| A component whose reads have the phase `built` is read as a host | The core tells a host from a separate process by the phases of its reads. |
| A component whose reads have the phase `disposing` is read as a host | The core tells a host from a separate process by the phases of its reads. |
| A component whose reads have only `start` and `stop` is read as a separate process | The core tells a host from a separate process by the phases of its reads. |
| A host of `api` with one `built` and one `disposing` read is accepted | A component whose reads have `built` or `disposing` needs at least one of each and as many `disposing` reads as `built` reads; any other component needs a `start` and a `stop` read. |
| A host with a `built` read and no `disposing` read leaves the Checks of its launch owed | A component whose reads have `built` or `disposing` needs at least one of each and as many `disposing` reads as `built` reads; any other component needs a `start` and a `stop` read. |
| A host with a `disposing` read and no `built` read leaves the Checks of its launch owed | A component whose reads have `built` or `disposing` needs at least one of each and as many `disposing` reads as `built` reads; any other component needs a `start` and a `stop` read. |
| Two `built` reads of `api` and one `disposing` read leave the Checks of the launch owed | A component whose reads have `built` or `disposing` needs at least one of each and as many `disposing` reads as `built` reads; any other component needs a `start` and a `stop` read. |
| Two `built` reads and two `disposing` reads of `api` are accepted | A component whose reads have `built` or `disposing` needs at least one of each and as many `disposing` reads as `built` reads; any other component needs a `start` and a `stop` read. |
| A separate process with a `start` read and no `stop` read leaves the Checks of its launch owed | A component whose reads have `built` or `disposing` needs at least one of each and as many `disposing` reads as `built` reads; any other component needs a `start` and a `stop` read. |
| A separate process with a `stop` read and no `start` read leaves the Checks of its launch owed | A component whose reads have `built` or `disposing` needs at least one of each and as many `disposing` reads as `built` reads; any other component needs a `start` and a `stop` read. |
| More `disposing` reads of `api` than `built` reads leave the Checks of the launch owed | A component whose reads have `built` or `disposing` needs at least one of each and as many `disposing` reads as `built` reads; any other component needs a `start` and a `stop` read. |
| A component with a `start` read and a `built` read in one launch is a report the core cannot read | A component with reads of both kinds in one launch is a report the core cannot read. |
| A component with a `stop` read and a `disposing` read in one launch is a report the core cannot read | A component with reads of both kinds in one launch is a report the core cannot read. |
| A component with reads of both kinds leaves the Checks of its launch owed | A component with reads of both kinds in one launch is a report the core cannot read. |

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
| A read of a host with an `error` instead of a report gives the cause of a report missing or unreadable, naming its declared place | Missing reports name their declared place. |
| A host with a `built` read and no `disposing` read gives the cause of a report missing or unreadable | a report missing or unreadable |
| A separate process with a `start` read and no `stop` read gives the cause of a report missing or unreadable | a report missing or unreadable |
| A separate process with a `stop` read and no `start` read gives the cause of a report missing or unreadable | a report missing or unreadable |
| An unequal count of `built` and `disposing` reads gives the cause of a report missing or unreadable | a report missing or unreadable |
| More `disposing` reads than `built` reads give the cause of a report missing or unreadable | a report missing or unreadable |
| A component with reads of both kinds gives the cause of a report missing or unreadable | a report missing or unreadable |
| A launch with no condition needs no reads of a host | Only components needed by a nonempty condition are required to have reads. |
| A malformed read of a host is a component read error, not a new outcome of a test | A malformed component report is a component read error, not a new outcome of a test. |
| The collected reads of a host appear in `run-1` and in each Check result of the launch | The same launch's reports appear in each of its Check results. |
