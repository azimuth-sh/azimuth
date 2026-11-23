# Map: execution contexts

The cut of the exploration of execution contexts. This repository is built without Azimuth, so
each row becomes a list of behaviors in `docs/increments/` or an edit of `docs/design.md`, not a
change folder. The map is edited freely and nobody approves it.

Asked which real work should be the first to use execution contexts, the maintainer said: "The
admin pages change?". About the two libraries it needs, the maintainer said: "And we need to
publish them", and about the order: "We should postpont the current increment and build
execution contexts before it. Right now". So the rows below come before increment 3, whose list
was accepted on 2026-11-21 and waits on the branch `increment-3`.

## Rows

| Row | Takes from the sources | After |
| --- | --- | --- |
| design-of-execution-contexts | `request.md` and every note in `decisions/`. The edit of `docs/design.md` that states execution contexts: keys of the product with their type and listed components, `Context:` on Requirements and Scenarios inherited by Checks, typed and exact values, unstated keys keeping their sources, one library per stack that applies under the component's condition and reports through a callback, the interface of a source, one launch per condition, collecting outside the library, "context not applied" with its causes, retired keys, values that vary between requests, and the agreement of components and instances in a deployment. It replaces the sentences on "A result depends on every declared value of its component", "There is no list per Check", variants and `production-next`, and it says where increment 3 now stands | |
| contexts-in-test-runs | the design as `design-of-execution-contexts` leaves it. A list of behaviors for the pass before increment 3: the reader accepts `Context:`, the project file declares keys and a suite names the components it starts, lint reports an undeclared key, a contradicting `Context:` and a retired key still named, the core makes one launch per condition and passes `AZIMUTH_CONTEXT`, the Playwright adapter passes it to its runner and reads each component's report after start and before stop, the core compares and gives "context not applied". A library for .NET and one for Node, each with the source over configuration, the condition and the callback, published by the release path that publishes the adapters. Done when the first real project that needs it proves, through a whole stack of a website and an API, a Check with a flag off and one with the flag on | design-of-execution-contexts |
| contexts-in-process | the design as `design-of-execution-contexts` leaves it. Collecting the report of a host that runs inside the test process, for xUnit: the code that builds the host calls its callback and gives the report to the adapter with the results | contexts-in-test-runs |
| contexts-in-deployments | the design as `design-of-execution-contexts` leaves it. The server reads the report of each running component from its declared place, applies a result only where its Scenario's condition holds, shows "Not applicable here", reports an absent key, an applied value in a deployment, and components or instances that disagree, with the time first seen. It is built with the server rung, which has no row here | |
| check-that-depends-on-an-unstated-key | note `variants-are-dropped`. A run that sets unstated keys to another deployment's values to find a Check that depends on a key its Scenario does not state. It is built with Challenges, which have no row here | |

