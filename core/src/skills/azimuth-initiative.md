---
name: azimuth-initiative
description: "The form of an initiative: which files it leaves, where they stand, and what a row of the map and a decision note hold."
---

# The form of an initiative

The word "initiative" names the work, not the talk.
Exploring is how a person and an agent work an initiative out, and it leaves nothing in the tree of its own.
Only work of several changes needs an initiative.
A change belongs to an initiative only when its map lists the change's name.
The proposal does not have to say anything about that membership.
An initiative can be born late.
Its first row is then a change that is already finished.
Initiatives live in the repository that holds the model.

## The folder

An initiative is the folder `azimuth/initiatives/<name>/`, with `map.md` and shared `sources/`.
A shared source or a decision note is first written in the initiative's `sources/`.
The initiative is the pool from which a change copies what its row names.
Use `azimuth-change` for writing a proposal and copying its sources into the change folder.
A source that only one change needs is written in that change's folder and nowhere else.

An initiative ends when every row is finished or deleted.
The next change moves its folder to `azimuth/initiatives/finished/<date>-<name>/`.
The date is the day the initiative folder arrived on the target branch.
Use `azimuth-change` for that move and for reading the date from git.

## Sources

A source says what was asked for.
One snapshot is one file in `sources/`, with its address, version and date at the top.
A snapshot is never edited.
A newer version of the page comes in as a new snapshot.
A source written directly in the repository goes in the same folder.
A person, or an agent with a connector, takes the snapshot.
Azimuth downloads nothing.

## Decision notes

A decision that holds for several changes is a source, written as a decision note.
Neither the map, a proposal nor the model holds it.
A note holds the date, who decided, and the words.
Notes stand together in the initiative's `sources/decisions/`, one note to a file.
A note's file name is the day of the decision and a short name.
For example, `2026-11-21-catalog-uses-short-titles.md` is a decision note's file name.
A note is never edited.
A later decision is a new note that names the one it replaces.
A note binds nobody by itself.
It becomes binding when a person approves Scenarios that follow it.

## The map

The map is in `map.md`.
Every row has the columns "Change", "Takes from the sources" and "After".
A row names what its change takes from the sources and which rows must be finished first.
A row carries the name its change will get, as a plain change name.
Only work under another model gets a prefixed row.

For a sample catalog, the table shows the form only.
It is not a split of any real work.

| Change | Takes from the sources | After |
| --- | --- | --- |
| catalog-titles | catalog-request, "Titles" | |
| catalog-search | catalog-request, "Search" | catalog-titles |

Nobody types the state of a row.
Until a command computes it, the agent reads the state from the target branch.
A row is finished when the folder of its change stands there, in `azimuth/changes/` or in
`azimuth/changes/finished/`.
A deleted row is gone from the map.
"After" names the rows that must be finished first.
It is the only ordering.

The map is edited freely, in ordinary commits.
An initiative is never approved.
A person approves each proposal when it is written.
A row may exist with no change behind it for as long as needed.

A row cites a decision note by its short name alone.
Whoever writes a decision note adds its short name to every row it bears on, in the same commit.
After its names, a row may say in one line which part of a source its change takes, or what the
work is when no source says it.
A decision is never written only in a row.
A row is deleted only in a commit that adds a decision note saying why.

## Leftovers

Every leftover of a change is a row in the map.
The model never holds a promise.
Work not started gets a row.
A release toggle to remove gets a row.
The contract half of an expand and contract migration gets a row.
A Scenario that is wanted and not built goes into the sources and a row, not into the model.
Evidence that is weak or owed gets no row, because status already shows it.
