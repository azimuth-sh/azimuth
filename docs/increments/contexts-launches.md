# Execution contexts, launches on the sample: behaviors

A row is of one of two kinds. A row of behavior is one observable fact, and a test shows it. A row of prose says what a text tells its reader; its section begins with the words "Rows of prose", and no test reads the words of the text for it. The second cell is copied exactly from `docs/design.md`. A plain test fails when a quoted sentence is no longer in that file. A behavior with no sentence to quote is not built. This list is built after the list of started components in the model (`contexts-components.md`, on its own branch), because a launch expects the components its suite starts.

Status of this list: a draft, not yet read by a person.

## Grouping the requested Checks into launches

| Behavior | Quote from the design |
| --- | --- |
| Requested Checks of a suite with different effective conditions get different launches | The core groups the requested Checks of a suite by their effective condition and makes one launch per group. |
| Requested Checks of a suite with the same effective condition share one launch | The core groups the requested Checks of a suite by their effective condition and makes one launch per group. |
| Only the keys a condition states decide the group | Only the stated keys count. |
| Checks of a suite with no condition form a group of their own, with its own launch | The core groups the requested Checks of a suite by their effective condition and makes one launch per group. |
| Launches of one suite run one after another | Launches of one suite run one after another, each with its own facility proof and time limit. |
| Each launch of a suite has its own facility proof | Launches of one suite run one after another, each with its own facility proof and time limit. |
| A facility proof runs before each launch | run by the core before each launch |
| Each launch of a suite has its own time limit | Launches of one suite run one after another, each with its own facility proof and time limit. |
| A launch that timed out gives no result for its Checks, and the next launch of the suite still runs | A build failure, crash, missing facility or context not applied stops only that launch; the next condition of the same suite still runs. A launch that timed out gives no result for any of its Checks, also for tests that had finished, as for a launch that crashed. |
| `azimuth run` makes one launch per group | The core groups the requested Checks of a suite by their effective condition and makes one launch per group. |
| `azimuth check run` makes one launch per group of the Checks it was asked to run | The core groups the requested Checks of a suite by their effective condition and makes one launch per group. |

## A test marked for Checks of several groups

| Behavior | Quote from the design |
| --- | --- |
| A test marked for Checks of different groups runs in each group's launch | A test marked for Checks in different groups runs in each group's launch. |
| The outcome of such a test counts only for the Check ids requested in that launch | Its outcome counts only for the Check ids requested in that launch. |
| A journey marked for `payout-is-sent-at-once` and `payout-waits-for-review` runs twice and counts for one Check each time | A journey marked for both Checks runs twice and counts for one Check each time. |
| A failure of the test in one launch does not fail the Check of the other launch | Its outcome counts only for the Check ids requested in that launch. |

## `AZIMUTH_CONTEXT`

| Behavior | Quote from the design |
| --- | --- |
| A launch with a condition gets `AZIMUTH_CONTEXT` as one JSON object of product keys to normalized text values | Each launch gets `AZIMUTH_CONTEXT` as one JSON object of product keys to normalized text values. |
| A launch of `payouts.instant=true` gets `AZIMUTH_CONTEXT={"payouts.instant":"true"}` | The first receives `AZIMUTH_CONTEXT={"payouts.instant":"true"}`. |
| A launch with no condition gets `{}` | A launch with no condition gets `{}`, replacing any inherited variable, and expects no component report. |
| A value of `AZIMUTH_CONTEXT` inherited from the environment of the core is replaced by the launch's condition | an adapter gets it with `AZIMUTH_CONTEXT` replaced by its launch's condition. |
| An inherited `AZIMUTH_CONTEXT` is replaced by `{}` in a launch with no condition | A launch with no condition gets `{}`, replacing any inherited variable, and expects no component report. |
| A launch with no condition expects no component report | A launch with no condition gets `{}`, replacing any inherited variable, and expects no component report. |
| The adapter passes the variable to the runner | The adapter passes the variable to the runner, and the runner's setup passes it to the components it starts. |
| The core sets no key that the condition does not state | Azimuth sets no unstated key. |
| A facility proof gets the environment unchanged | A proof gets the environment unchanged; |

## The fields the request gains

| Behavior | Quote from the design |
| --- | --- |
| The request keeps the number `adapter-request-1` | Contexts add fields to `adapter-request-1` and `adapter-report-1` and keep both numbers. |
| The run request carries `condition`, the same object as `AZIMUTH_CONTEXT` | The run request gains `condition`, the same object of normalized text values as `AZIMUTH_CONTEXT` |
| The request of a launch with no condition carries `{}` as its condition | A launch with no condition gets `{}`, replacing any inherited variable, and expects no component report. |
| The run request carries `components`, one entry with `name` and `report` path for each component in the suite's `starts` | and `components`, one entry with `name` and `report` path for each component in `starts`. |
| The request lists the components of `starts` even when the condition needs none of their keys | The request lists components even when this condition needs none of their keys. |
| An adapter that predates the new fields still proves a launch with `{}` as its condition | An adapter that predates these fields can still prove a launch with `{}` as its condition. |

## The fields the report gains

| Behavior | Quote from the design |
| --- | --- |
| The report keeps the number `adapter-report-1` | Contexts add fields to `adapter-request-1` and `adapter-report-1` and keep both numbers. |
| The report gains `components`, a list with the expected `name` and `reads` for each component collected | The report gains `components`, a list with the expected `name` and `reads` for each component collected. |
| Each read has `phase` and either `report`, the document as read, or `error`, the cause it could not be read | Each read has `phase` and either `report`, the document as read, or `error`, the cause it could not be read. |
| The phases `start` and `stop` mark the two reads of a component in its own process | The phases are `start` and `stop` for a component in its own process |
| A component that answered before start has `error: "something else answers there"` and no reads | A component that answered before start instead has `error: "something else answers there"` and no reads, and the report lists the tests as `not_run`. |
| Only components needed by a nonempty condition are required to have reads | Only components needed by a nonempty condition are required to have reads. |
| A malformed component report is a component read error, not a new outcome of a test | A malformed component report is a component read error, not a new outcome of a test. |

## A report with no `components`

| Behavior | Quote from the design |
| --- | --- |
| A report with no `components` leaves Checks with a condition owed with "context not applied: the adapter reports no components" | A report with no `components` leaves Checks with a condition owed with "context not applied: the adapter reports no components". |
| Checks with no condition keep their results when the report has no `components` | Checks with no condition keep their results. |

## What the core compares before the Check rule

| Behavior | Quote from the design |
| --- | --- |
| The core compares the component reports with the launch's condition before it applies the Check rule | Before applying the Check rule, the core compares the component reports with the launch's condition. |
| Only stated keys are compared | For each stated key it expects every component that is both in `starts` and listed for that key. |
| For a stated key the core expects every component that is in `starts` and listed for that key | For each stated key it expects every component that is both in `starts` and listed for that key. |
| A component listed for the key but not in `starts` is not expected | it expects every component that is both in `starts` and listed for that key. |
| A component in `starts` but not listed for the key is not expected | it expects every component that is both in `starts` and listed for that key. |
| Each expected report must give the stated typed value | Each expected report must give the stated typed value, applied by Azimuth. |
| A value reported with source precedence `azimuth` and applying `possible` is accepted | A launch accepts a stated value only with source precedence `azimuth` and applying `possible`. |
| A correct value from configuration alone is not accepted | A correct value from configuration alone is insufficient. |
| The condition must hold in the `start` read | This must hold in both reads, for a separate process or a host inside the test process. |
| The condition must hold in the `stop` read | This must hold in both reads, for a separate process or a host inside the test process. |
| A difference in an unstated key changes nothing | A difference in an unstated key or in a read's time changes nothing. |
| A difference in a read's time changes nothing | A difference in an unstated key or in a read's time changes nothing. |

## "context not applied" and its causes

| Behavior | Quote from the design |
| --- | --- |
| A launch whose context was not applied gives no result for any Check of that launch | A context that was not applied gives no result for any Check of that launch. |
| A wrong report loses the result of a Check that states no key of that component | One component's wrong report loses every result of its launch, even a Check that states no key of that component. |
| The state is `owed` and the reason is `context not applied` | The state is `owed`, the reason is `context not applied`, and the detail names the component, key, stated and reported values, and cause. |
| The detail names the component, key, stated and reported values, and cause | The state is `owed`, the reason is `context not applied`, and the detail names the component, key, stated and reported values, and cause. |
| Cause: applying is off | If applying is `not possible`, the cause is "applying is off". |
| Cause: the context never reached the component | If possible and the source precedence is `before azimuth`, the cause is "the context never reached the component". |
| Cause: an overriding source, which the detail names | If the precedence is `after azimuth`, the cause names the overriding source. |
| Cause: a missing key or value, reported as absent | An absent value or source is reported as absent. |
| Cause: a value that cannot be read by its type | a value that cannot be read by its type |
| Cause: a wrong component name, naming both the expected and the reported component | A wrong name names both the expected and reported component. |
| Cause: an address already answering | an address already answering |
| Cause: a report missing or unreadable, naming its declared place | Missing reports name their declared place. |
| Cause: no started component reading a stated key | and no started component reading a stated key. |
| A condition that no started component can apply gives "context not applied" in both commands, also in a suite with Checks that have no condition | A condition that no started component can apply gives "context not applied" in both `azimuth run` and `azimuth check run`, also in a suite with Checks that have no condition. |

## Where it stands in the result of a Check

| Behavior | Quote from the design |
| --- | --- |
| A launch whose context was not applied gives no result even when a test of a Check failed | This comes before a failed test in the Check rule. |
| A Check whose condition was not applied has no result, whatever its tests did | If it did not, the Check has no result, with the reason "context not applied", whatever its tests did. |
| A launch whose context was applied has its Checks decided by the Check rule | Only then does the core apply this rule to the marked tests of a Check. |
| A Check whose launch gave no result for this cause is `owed` in `run-1`, with the reason `context not applied` | in `run-1` the state is `owed`, the reason is `context not applied` for that case and `no result` for other launch faults, and `detail` names the cause |
| A launch with no result stores no result record | A launch with no result still stores no result record. |
| A later `azimuth status` shows the reason the Check had before the run | A launch with no result stores nothing, so a later `azimuth status` shows such a Check with the reason it had before the run. |
| The next plan runs the Check again | Run after repairing how the condition reaches the components |
| The local lane ends with status 1 | The local lane ends with status 1, as for a crash. Context not applied adds one more case. |
| The run ends with failure while a Check is owed because its launch gave no result | any Check is failed on the current content, whether it ran now or is a known fail, or is still owed because its launch gave no result |
| Example: `api` reports `payouts.instant=true` where the launch states `false`, and the Checks are owed with "context not applied: api, the context never reached the component; payouts.instant stated false, reported true" | The Checks are owed with "context not applied: api, the context never reached the component; payouts.instant stated false, reported true". |

## Every launch of a suite runs

| Behavior | Quote from the design |
| --- | --- |
| A run does not stop at the first failure | A run never stops at the first failure. |
| A launch with a failed Check does not stop the next launch of its suite | It runs everything the lane planned and then reports every Check with its state. |
| A launch whose context was not applied does not stop the next launch of its suite | A later launch of the same suite still runs. |
| A launch that crashes gives no results and does not stop the next launch of its suite | A launch that crashes gives no results. A build failure, crash, missing facility or context not applied stops only that launch; the next condition of the same suite still runs. |
| A launch whose build failed does not stop the next launch of its suite | A build failure, crash, missing facility or context not applied stops only that launch; the next condition of the same suite still runs. |

## `run-1` and the text of a run

| Behavior | Quote from the design |
| --- | --- |
| `suites` in `run-1` has one entry per launch, so a suite's name can occur more than once | In `run-1`, `suites` has one entry per launch, so a suite's name can occur more than once. |
| Each `suites` entry gains `condition` | Each entry gains `condition`, `checks`, the ids requested for that launch, and `components`, the collected reads. |
| Each `suites` entry gains `checks`, the ids requested for that launch | Each entry gains `condition`, `checks`, the ids requested for that launch, and `components`, the collected reads. |
| Each `suites` entry gains `components`, the collected reads | Each entry gains `condition`, `checks`, the ids requested for that launch, and `components`, the collected reads. |
| `launch` and `duration_ms` of an entry describe that launch | Its `launch` and `duration_ms` describe that launch. |
| The top-level `checks` holds each Check's state once | The top-level `checks` still holds each Check's state once. |
| `run-1` keeps its number | These are added fields, and the format keeps its number. |
| A crash of the first launch does not remove the second from the plan | A crash of the first does not remove the second from the plan. |
| `run-1` exposes the reads, so a launch that gives no result can be inspected in the output of the command | `run-1` exposes the reads too, so a launch that gives no result can still be inspected in that command's output. |
| The text shows the suite, condition, duration and requested Checks for each launch | Text shows the suite, condition, duration and requested Checks for each launch, including a launch that failed before the next one ran. |
| The text shows a launch that failed before the next one ran | including a launch that failed before the next one ran. |
| The start and end lines of a launch name its condition | The start and end lines name the condition too. |
| `suites` lists only the suites whose adapter the core started or tried to start | only the suites whose adapter the core started or tried to start. |

## What a result keeps of its launch

| Behavior | Quote from the design |
| --- | --- |
| `result-1` gains `condition` and `components` in its body | `result-1` gains `condition` and `components` in its body, with the request's condition and the adapter report's collected reads. |
| The `condition` of a result is the request's condition | with the request's condition and the adapter report's collected reads. |
| The `components` of a result are the adapter report's collected reads | with the request's condition and the adapter report's collected reads. |
| `azimuth check show` exposes the same added fields of `check-1` | `azimuth check show` exposes them as the same added fields of `check-1` |
| The text of `azimuth check show` lists the mapped values, their sources and read times | its text lists the mapped values, their sources and read times. |
| The kept reads include unstated keys | This includes unstated keys. |
| The same launch's reports appear in each of its Check results | The same launch's reports appear in each of its Check results. |
| A result of a Check on a machine whose unstated key differs has the same fingerprint | An unstated key can differ between two machines without changing a fingerprint; the reports keep its value and source visible after the run. |
| The collected reports are not an input of the fingerprint | No extra fingerprint part holds context; reported values of unstated keys are not fingerprint inputs of their own. |

## Proof on the sample

| Behavior | Quote from the design |
| --- | --- |
| Azimuth's own tests prove launches with a small script as an adapter | Azimuth's own tests use a small script as an adapter, so testing the core needs no test framework of another stack. |
