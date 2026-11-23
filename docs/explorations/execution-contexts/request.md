# Request: execution contexts

Date: 2026-11-23. Asked by: the maintainer.

The request came up while a project was setting up a test suite that starts its whole stack
once per run, and found that a feature flag must be off for some Checks and on for others.

The maintainer's words, in the order they were said:

- "I think we should discuss execution contexts and how to use them for Checks and how to make
  adapters apply them."
- "I believe we didn't not implemented setting a context in adapters. This is what we
  potentially should do now"
- "I believe the Check states what execution context it should run in. But I may be wrong"
- "A default context. I believe every Check stating its context would be too cumbersome."
- "Can we work through it using the exploration skill before that? I like how it works."
- Asked where the records of this exploration should live: "Outside azimuth/"

This repository is built without Azimuth (`AGENTS.md`), so the exploration keeps its records in
`docs/explorations/execution-contexts/` and not in `azimuth/initiatives/`. Its cut becomes
lists of behaviors for increments, as `AGENTS.md` says, instead of change folders.
