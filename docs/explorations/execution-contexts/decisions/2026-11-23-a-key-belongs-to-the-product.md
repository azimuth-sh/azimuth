# Decision: a key belongs to the product

Date: 2026-11-23. Decided by: the maintainer.

About a key written as `api Payouts:Instant`, the maintainer asked: "Shouldn't this be
"Context: api.payouts.instant" to be stack-agnostic? Where the first word is an area. Though it
should be a component but not an area. Should we have a single shared context tree?". About
keys of components beside keys of the product, the maintainer asked: "Wouldn't such duality
bring chaos? Shouldn't Azimuth have one strict rule?". About components that must report the
same value, the maintainer said: "I think it is a valuable correctness property." About keys
named after components, the maintainer said: "Technically ".api" and ".website" shouldn't be
component names. Azimuth should not impose this". About names of settings in the project file,
the maintainer asked: "Wouldn't both Next.js and .NET libraries just get "labs: true" and map it
to "FEATURE_LABS=true" and "Features:Labs=true" accordingly?", and then: "Do we need mapping?".
Asked whether the project file should keep, for each key, the list of components that must read
it, the maintainer said: "Yes, keep the list."

Claude's words, which the maintainer agreed to: a key of an execution context is a key of the
product, written in one notation for every stack: lowercase segments joined by dots, such as
`labs` or `payouts.instant`. The project file declares each key with its type and the
components that must read it, and names no setting. `AZIMUTH_CONTEXT` carries keys of the
product. The library of each component maps a key to the component's own setting by a rule
that the component gives when it registers the library, such as a section or a prefix, and an
override in the same registration names the setting of one key. The report of a component gives
its values by the keys of the product. A listed component that reports no value for a key is
the case "context not applied", and a component that reports a key it is not listed for is a
finding. A test run applies a key's value to every component that the key is listed for. In a
deployment, every component listed for a key, and every instance of each, must report the same
value. When they differ, the server shows a finding of the deployment that names the key, the
components, the instances, their values and when it was first seen, and no result that depends
on the key applies there. A feature that is switched on component by component on purpose is
declared as several keys, each listed for the components that are switched together. Their
names are the team's choice, and Azimuth reads no component in a key's name.

This replaces what the note `a-check-states-the-values-its-result-depends-on` says about each
component declaring the keys of its context, and what the note
`keys-mirror-settings-and-values-are-typed-present-and-exact` says about a key being the path
of a setting in its stack's own notation.

Rejected: keys that belong to one component, because a `Context:` would then name the parts
inside the system, which a Scenario does not speak of, and every split or rename of a component
would touch every `Context:` that names it. Keys of both kinds side by side, because the same
flag could then be stated in two ways. Names of settings in the project file, because how a
component names its settings is the component's own knowledge. Learning from the reports alone
which components read a key, because a component that was forgotten would then be invisible.

The cost: a flag that two components read independently cannot be stated differently without
two keys. The mapping from a key to a component's setting is code in the component, so lint
does not see it, and a wrong mapping shows only in the report. During a rolling deploy, the
finding about instances is true for minutes.

Example: the product's flag `labs` is read by two components under their own names. The
project file declares `labs`, a boolean, listed for `api` and `website`. The API registers its
library with the section `Features`, and the website with the prefix `FEATURE_`. A Scenario
with `Context: labs=true` runs with `Features:Labs=true` in the API and `FEATURE_LABS=true` in
the website. In production, somebody turns labs on in the API's configuration and forgets the
website. The API reports `true` for `labs` and the website `false`, so the product is half on.
The server shows the finding "labs: api reports true, website reports false", and no result
that depends on `labs` applies there, whichever value its Scenario states. A team that opens
labs for browsing before their actions declares `labs.browsing`, listed for `website`, and
`labs.actions`, listed for `api`.
