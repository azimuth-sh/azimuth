# Execution contexts, started components in the model: behaviors

A row is of one of two kinds. A row of behavior is one observable fact, and a test shows it. A row of prose says what a text tells its reader; its section begins with the words "Rows of prose", and no test reads the words of the text for it. The second cell is copied exactly from `docs/design.md`. A plain test fails when a quoted sentence is no longer in that file. A behavior with no sentence to quote is not built. This list is built after `contexts-conditions.md`, because a Check's condition is held against the components its suite starts.

Status of this list: read and accepted by a person on 2026-11-24.

## Components in `azimuth/project.toml`

| Behavior | Quote from the design |
| --- | --- |
| The reader reads each `[[component]]` table of the project file | **Execution contexts add components to the project file and `starts` to each suite.** |
| A component has a `name`, unique in the project | `name`, unique in the project |
| A component's `areas` names the areas it is built from | `areas`, the names of the areas it is built from |
| A component is made from one area | Something that runs, made from one or more areas |
| A component is made from several areas | Something that runs, made from one or more areas |
| A component's `report` is the absolute path where its report is exposed | `report`, the absolute path under its existing door where its report is exposed |
| A project file with no component declares none | **A project with no contexts needs neither this file nor a component declaration.** |
| `azimuth init` writes no component | `azimuth init` writes no `execution-context.toml` and no components, and leaves `starts` omitted. |

## `starts` on a suite

| Behavior | Quote from the design |
| --- | --- |
| A suite's `starts` lists the names of the components it starts | the names of the real components it starts, including hosts inside the test process |
| A component in its own process counts as started | A component in its own process and a host built inside the test process both count. |
| A host built inside the test process counts as started | A component in its own process and a host built inside the test process both count. |
| A suite with no `starts` starts no component | Omitted means `[]` |
| A suite with `starts = []` starts no component | Omitted means `[]` |
| `azimuth init` leaves `starts` omitted | `azimuth init` writes no `execution-context.toml` and no components, and leaves `starts` omitted. |

## Reading errors in components and `starts`

| Behavior | Quote from the design |
| --- | --- |
| A component with no `name` is a reading error | a missing component `name`, `areas` or `report`; an undeclared area; a report path that does not start with `/`; a field of the wrong TOML type, including `starts` other than a list of names |
| A component with no `areas` is a reading error | a missing component `name`, `areas` or `report`; an undeclared area; a report path that does not start with `/`; a field of the wrong TOML type, including `starts` other than a list of names |
| A component with no `report` is a reading error | a missing component `name`, `areas` or `report`; an undeclared area; a report path that does not start with `/`; a field of the wrong TOML type, including `starts` other than a list of names |
| A component whose `areas` names an undeclared area is a reading error | a missing component `name`, `areas` or `report`; an undeclared area; a report path that does not start with `/`; a field of the wrong TOML type, including `starts` other than a list of names |
| A `report` path that does not start with `/` is a reading error | a missing component `name`, `areas` or `report`; an undeclared area; a report path that does not start with `/`; a field of the wrong TOML type, including `starts` other than a list of names |
| A component field of the wrong TOML type is a reading error | a missing component `name`, `areas` or `report`; an undeclared area; a report path that does not start with `/`; a field of the wrong TOML type, including `starts` other than a list of names |
| A `starts` that is not a list of names is a reading error | a missing component `name`, `areas` or `report`; an undeclared area; a report path that does not start with `/`; a field of the wrong TOML type, including `starts` other than a list of names |
| Two components with the same name are a reading error | Two entries of one table with the same name. |
| A field of a component table that the CLI does not know is a reading error | a key or a table that the CLI does not know is a reading error |
| A component reading error names the line of the key in the project file | for the project file it is the line of the key |

## What commands do on a reading error

| Behavior | Quote from the design |
| --- | --- |
| A reading error of components or `starts` has the same words in every command | Every command reads the model through the same reader, so a reading error has the same words everywhere. |
| On such a reading error, `azimuth run`, `azimuth check run` and the reading commands do nothing else | print the same message, do nothing else, end with failure |
| On such a reading error, commands end with status 2 | a reading error, a wrong argument, `azimuth run` under a CLI version other than the pinned one, a missing common ancestor |

## Lint findings

| Behavior | Quote from the design |
| --- | --- |
| Lint reports an undeclared component in a key's `components` | An undeclared component in a key's `components` or a suite's `starts` |
| Lint reports an undeclared component in a suite's `starts` | An undeclared component in a key's `components` or a suite's `starts` |
| Lint reports a Check whose condition states a key that no component its suite starts is listed for | Lint names a Check whose condition states a key that no component its suite starts is listed for. |
| Lint reports such a Check when its suite has no `starts` | A suite that starts neither cannot prove that condition. |
| Lint reports such a Check when its suite starts only components the key is not listed for | Lint names a Check whose condition states a key that no component its suite starts is listed for. |
| Lint reports nothing for a Check whose suite starts one component the key is listed for | Lint names a Check whose condition states a key that no component its suite starts is listed for. |
| Lint reports nothing for a Check whose suite starts every component the key is listed for | Lint names a Check whose condition states a key that no component its suite starts is listed for. |
| A component that exists but is not started by the suite does not satisfy the finding | Lint names a Check whose condition states a key that no component its suite starts is listed for. |
| A component finding fails lint | A finding fails only lint and the commit lane. |
| A component finding fails the commit lane | A finding fails only lint and the commit lane. |
| A component finding does not stop the reading commands | work. The affected Check is shown with the state it has |

## `azimuth change diff`

| Behavior | Quote from the design |
| --- | --- |
| A change to a suite's `starts` is class D | **Every change to `execution-context.toml` is class D, as is a change to `starts` in the project file.** |
| The reason for a change to `starts` names the project file | `azimuth change diff` names the file as the reason. |
| A change to a component declaration is class D | The project file is class D. |
