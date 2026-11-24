# Execution contexts, what a run and a result record of a launch: behaviors

A row is of one of two kinds. A row of behavior is one observable fact, and a test shows it. A row of prose says what a text tells its reader; its section begins with the words "Rows of prose", and no test reads the words of the text for it. The second cell is copied exactly from `docs/design.md`. A plain test fails when a quoted sentence is no longer in that file. A behavior with no sentence to quote is not built. This list is built after the list of started components in the model (`contexts-components.md`, on its own branch), because a launch expects the components its suite starts. It is built after `contexts-launches.md`, because it records the launches that list builds.

Status of this list: a draft, not yet read by a person.

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

