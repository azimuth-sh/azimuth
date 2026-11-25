# Execution contexts, launches through Playwright: behaviors

A row is of one of two kinds. A row of behavior is one observable fact, and a test shows it. A row of prose says what a text tells its reader; its section begins with the words "Rows of prose", and no test reads the words of the text for it. This list has no rows of prose. The second cell is copied exactly from `docs/design.md`. A plain test fails when a quoted sentence is no longer in that file. A behavior with no sentence to quote is not built. This list is built after the list of launches on the sample (`contexts-launches.md`), because it makes the Playwright adapter do for a launch what that list requires of every adapter, and it is the second runner that tests those fields. It is built after the list of launch records (`contexts-launch-records.md`), so that a launch through Playwright ends in the result that list records.

Status of this list: a draft, not yet read by a person.

## Where the adapter takes what it needs

| Behavior | Quote from the design |
| --- | --- |
| The address of a component comes from the runner configuration | It gets component addresses from the runner configuration and names and report paths from the request. |
| The name of a component comes from the request | It gets component addresses from the runner configuration and names and report paths from the request. |
| The report path of a component comes from the request | It gets component addresses from the runner configuration and names and report paths from the request. |
| The adapter finds the addresses of components in the `webServer` entries of the config | Playwright uses its `webServer` entries. |
| The entry of a component is the `webServer` entry whose `name` is the component's name | A component's entry is the one whose `name` is the component's name. |
| The address of a component is the origin of the `url` of its entry | Its address is the origin of the entry's `url` |
| An entry that gives only `port` has the address `http://localhost:<port>` | or `http://localhost:<port>` when the entry gives only `port`. |
| A config in which two entries have the name of one component is refused, and the adapter runs nothing | A config in which two entries have the name of one component is refused: the adapter runs nothing, says why, and the Checks get no result. |
| The adapter says why it refused such a config | A config in which two entries have the name of one component is refused: the adapter runs nothing, says why, and the Checks get no result. |
| The Checks of a launch with such a config get no result | A config in which two entries have the name of one component is refused: the adapter runs nothing, says why, and the Checks get no result. |
| An address that the runner configuration does not name is not looked at | Each adapter finds only the addresses its own runner's configuration names |

## The look before the start

| Behavior | Quote from the design |
| --- | --- |
| The adapter looks at each address of a component in `starts` before it starts anything | Before it starts anything, an adapter looks at each address of a component in `starts` that is known before the start. |
| The look happens in a launch with no condition too | It is a rule of the contract, for every adapter, in every launch, also one with no condition. |
| An address answers when a TCP connection to its host and port is accepted | An address answers when a TCP connection to its host and port is accepted. |
| When an address answers, no test runs | If one answers, it runs no test |
| When an address answers, the report lists the tests as `not_run` | reports the tests as `not_run` |
| When an address answers, the report gives "something else answers there" for that component | reports "something else answers there" for that component |
| The component that answered has `error: "something else answers there"` and no reads | A component that answered before start instead has `error: "something else answers there"` and no reads |
| The look covers every address before the adapter decides | The look covers every address before the adapter decides, so each component whose address answers has that error. |
| Each component whose address answers has the error "something else answers there" | The look covers every address before the adapter decides, so each component whose address answers has that error. |
| When an address answers, the adapter starts nothing | The adapter then starts nothing, and every other component is listed with no reads. |
| Every component whose address does not answer is listed with no reads | The adapter then starts nothing, and every other component is listed with no reads. |
| For `localhost` an answer on `127.0.0.1` is found | For `localhost` the look covers both `127.0.0.1` and `::1`. |
| For `localhost` an answer on `::1` is found | For `localhost` the look covers both `127.0.0.1` and `::1`. |
| An address learned only after the start, such as a free port chosen then, is not looked at | An address learned only after the start, such as a free port chosen then, needs no look. |

## The two reads

| Behavior | Quote from the design |
| --- | --- |
| The adapter reads a component's report after the servers start and before the tests | it reads after the servers start and before the tests |
| The adapter reads a component's report after the tests and before the servers stop | and after the tests before the servers stop |
| A component in its own process gets a read with the phase `start` | The phases are `start` and `stop` for a component in its own process |
| A component in its own process gets a read with the phase `stop` | The phases are `start` and `stop` for a component in its own process |
| A read is one HTTP GET of the component's `report` path at its address | A read is one HTTP GET of the component's `report` path at its address |
| The body of a 200 response is the document as read | the body of a 200 response is the document as read |
| A read that succeeded has `report`, the document as read | Each read has `phase` and either `report`, the document as read, or `error`, the cause it could not be read. |
| A response with a status other than 200 is a failed read whose `error` says which | Any other status, a failed connection, a body that is not JSON or no response within 10 seconds is a failed read, and its `error` says which. |
| A failed connection is a failed read whose `error` says which | Any other status, a failed connection, a body that is not JSON or no response within 10 seconds is a failed read, and its `error` says which. |
| A body that is not JSON is a failed read whose `error` says which | Any other status, a failed connection, a body that is not JSON or no response within 10 seconds is a failed read, and its `error` says which. |
| No response within 10 seconds is a failed read whose `error` says which | Any other status, a failed connection, a body that is not JSON or no response within 10 seconds is a failed read, and its `error` says which. |
| A read that failed has `error`, the cause it could not be read | Each read has `phase` and either `report`, the document as read, or `error`, the cause it could not be read. |
| The adapter reads every component that the request lists | The adapter reads every component that the request lists, in every launch, also one with no condition. |
| It reads them in a launch with no condition too | The adapter reads every component that the request lists, in every launch, also one with no condition. |
| The report lists each component collected under the name that the request gave | The report gains `components`, a list with the expected `name` and `reads` for each component collected. |
| A component report that is malformed is a read error, not a new outcome of a test | A malformed component report is a component read error, not a new outcome of a test. |

## Wrapping the suite's setup and teardown

| Behavior | Quote from the design |
| --- | --- |
| The adapter wraps the runner's global setup and teardown | Playwright's adapter wraps the runner's global setup and teardown |
| The suite's own global setup still runs | while preserving the suite's own setup and teardown |
| The suite's own global teardown still runs | while preserving the suite's own setup and teardown |
| The start read comes before the suite's own global setup | the start read comes before the suite's own global setup |
| The stop read comes after the suite's own global teardown | and the stop read after its own global teardown |
| A failing global setup of the suite still gets the stop read | A failing global setup of the suite still gets the stop read. |

## A missing address, a failed read, a missing second read

| Behavior | Quote from the design |
| --- | --- |
| A component in `starts` with no `webServer` entry of its name has a missing address | A component in `starts` with no such entry has a missing address. |
| A component with no address has `error: "no address in the runner configuration"` and no reads | A component with no address has `error: "no address in the runner configuration"` and no reads. |
| A stop read that was never made is a read with phase `stop` and `error: "the stop read was not made"` | is a read with phase `stop` and `error: "the stop read was not made"`. |
| A stop read is not made when the runner ended before its teardown | A stop read that was never made, because the runner ended before its teardown |
| A missing address, a failed read and a missing second read are not left out of the report | A missing address, failed read or missing second read is carried as a cause, not left out. |

## `AZIMUTH_CONTEXT` through the adapter

| Behavior | Quote from the design |
| --- | --- |
| The runner gets the `AZIMUTH_CONTEXT` that the adapter got | The adapter passes the variable to the runner |
| The adapter sets no key that the condition does not state | Azimuth sets no unstated key. |
